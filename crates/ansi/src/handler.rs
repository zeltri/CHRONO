use terminal_core::{CellAttributes, Color, Screen};
use vte::{Params, Perform};

/// Handler que implementa el trait Perform de vte para procesar secuencias ANSI
pub struct AnsiHandler<'a> {
    screen: &'a mut Screen,
    /// Respuestas que deben enviarse de vuelta al PTY (DA, DSR, CPR, etc.)
    responses: &'a mut Vec<u8>,
}

impl<'a> AnsiHandler<'a> {
    pub fn new(screen: &'a mut Screen, responses: &'a mut Vec<u8>) -> Self {
        Self { screen, responses }
    }
}

/// Extrae el primer valor de un parámetro CSI con un valor por defecto
fn param(params: &Params, index: usize, default: u16) -> u16 {
    params
        .iter()
        .nth(index)
        .and_then(|p| p.first().copied())
        .map(|v| if v == 0 { default } else { v })
        .unwrap_or(default)
}

/// Como `param` pero sin tratar 0 como "usar default" (para modos de borrado)
fn param_raw(params: &Params, index: usize, default: u16) -> u16 {
    params
        .iter()
        .nth(index)
        .and_then(|p| p.first().copied())
        .unwrap_or(default)
}

impl<'a> Perform for AnsiHandler<'a> {
    fn print(&mut self, c: char) {
        self.screen.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x07 => {
                // Bell - ignorado por ahora
            }
            0x08 => {
                // Backspace: solo mueve el cursor a la izquierda
                self.screen.cursor.move_left(1);
            }
            0x09 => {
                // Tab (paradas cada 8 columnas)
                let next_tab = ((self.screen.cursor.col / 8) + 1) * 8;
                self.screen.cursor.col = next_tab.min(self.screen.cols - 1);
            }
            0x0A..=0x0C => {
                // Line feed, vertical tab, form feed
                self.screen.line_feed();
            }
            0x0D => {
                // Carriage return
                self.screen.carriage_return();
            }
            _ => {
                log::trace!("Execute no implementado: 0x{:02X}", byte);
            }
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {
        log::trace!("Hook (DCS) no implementado");
    }

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // Procesar secuencias OSC (Operating System Command)
        if params.is_empty() {
            return;
        }

        if let Ok(command) = std::str::from_utf8(params[0]) {
            match command {
                "0" | "1" | "2" => {
                    // OSC 0/1/2: cambiar título de ventana/ícono
                    if params.len() > 1 {
                        if let Ok(title) = std::str::from_utf8(params[1]) {
                            self.screen.set_title(title.to_string());
                        }
                    }
                }
                "7" => {
                    // Notificación de directorio actual (usado por algunos shells)
                    if params.len() > 1 {
                        if let Ok(cwd) = std::str::from_utf8(params[1]) {
                            log::debug!("Current directory notification: {}", cwd);
                        }
                    }
                }
                "9" => {
                    log::trace!("Progress notification received");
                }
                "133" => {
                    // Shell integration sequences (VSCode, iTerm2)
                    log::trace!("Shell integration sequence received");
                }
                _ => {
                    log::trace!("OSC command not implemented: {}", command);
                }
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, c: char) {
        // Modos privados DEC (CSI ? ... h/l)
        if intermediates.first() == Some(&b'?') {
            match c {
                'h' | 'l' => {
                    let enable = c == 'h';
                    for p in params.iter() {
                        let mode = p.first().copied().unwrap_or(0);
                        self.handle_private_mode(mode, enable);
                    }
                }
                _ => {
                    log::trace!("CSI privado no implementado: ?{}", c);
                }
            }
            return;
        }

        match c {
            'H' | 'f' => {
                // CUP - Cursor Position
                let row = param(params, 0, 1).saturating_sub(1) as usize;
                let col = param(params, 1, 1).saturating_sub(1) as usize;
                self.screen.move_cursor_to(row, col);
            }
            'A' => {
                // CUU - Cursor Up
                let n = param(params, 0, 1) as usize;
                self.screen.cursor.move_up(n);
            }
            'B' | 'e' => {
                // CUD - Cursor Down / VPR
                let n = param(params, 0, 1) as usize;
                let max_row = self.screen.rows - 1;
                self.screen.cursor.move_down(n, max_row);
            }
            'C' | 'a' => {
                // CUF - Cursor Forward / HPR
                let n = param(params, 0, 1) as usize;
                let max_col = self.screen.cols - 1;
                self.screen.cursor.move_right(n, max_col);
            }
            'D' => {
                // CUB - Cursor Back
                let n = param(params, 0, 1) as usize;
                self.screen.cursor.move_left(n);
            }
            'E' => {
                // CNL - Cursor Next Line
                let n = param(params, 0, 1) as usize;
                let max_row = self.screen.rows - 1;
                self.screen.cursor.move_down(n, max_row);
                self.screen.carriage_return();
            }
            'F' => {
                // CPL - Cursor Previous Line
                let n = param(params, 0, 1) as usize;
                self.screen.cursor.move_up(n);
                self.screen.carriage_return();
            }
            'G' | '`' => {
                // CHA - Cursor Horizontal Absolute
                let col = param(params, 0, 1).saturating_sub(1) as usize;
                let row = self.screen.cursor.row;
                self.screen.move_cursor_to(row, col);
            }
            'd' => {
                // VPA - Vertical Position Absolute
                let row = param(params, 0, 1).saturating_sub(1) as usize;
                let col = self.screen.cursor.col;
                self.screen.move_cursor_to(row, col);
            }
            'J' => {
                // ED - Erase in Display
                self.screen.erase_in_display(param_raw(params, 0, 0));
            }
            'K' => {
                // EL - Erase in Line
                self.screen.erase_in_line(param_raw(params, 0, 0));
            }
            'L' => {
                // IL - Insert Lines
                self.screen.insert_lines(param(params, 0, 1) as usize);
            }
            'M' => {
                // DL - Delete Lines
                self.screen.delete_lines(param(params, 0, 1) as usize);
            }
            '@' => {
                // ICH - Insert Characters
                self.screen.insert_chars(param(params, 0, 1) as usize);
            }
            'P' => {
                // DCH - Delete Characters
                self.screen.delete_chars(param(params, 0, 1) as usize);
            }
            'X' => {
                // ECH - Erase Characters
                self.screen.erase_chars(param(params, 0, 1) as usize);
            }
            'S' => {
                // SU - Scroll Up
                self.screen.scroll_up(param(params, 0, 1) as usize);
            }
            'T' => {
                // SD - Scroll Down
                self.screen.scroll_down(param(params, 0, 1) as usize);
            }
            'b' => {
                // REP - Repeat preceding character
                self.screen.repeat_last_char(param(params, 0, 1) as usize);
            }
            'r' => {
                // DECSTBM - Set Scrolling Region
                let top = param(params, 0, 1).saturating_sub(1) as usize;
                let bottom = param(params, 1, self.screen.rows as u16).saturating_sub(1) as usize;
                self.screen.set_scroll_region(top, bottom);
            }
            's' => {
                // SCOSC - Save Cursor
                self.screen.save_cursor();
            }
            'u' => {
                // SCORC - Restore Cursor
                self.screen.restore_cursor();
            }
            'c' => {
                // DA - Device Attributes: respondemos como VT100 con Advanced Video Option
                self.responses.extend_from_slice(b"\x1b[?1;2c");
            }
            'n' => {
                // DSR - Device Status Report
                match param_raw(params, 0, 0) {
                    5 => self.responses.extend_from_slice(b"\x1b[0n"), // OK
                    6 => {
                        // CPR - Cursor Position Report (1-indexed)
                        let report = format!(
                            "\x1b[{};{}R",
                            self.screen.cursor.row + 1,
                            self.screen.cursor.col + 1
                        );
                        self.responses.extend_from_slice(report.as_bytes());
                    }
                    _ => {}
                }
            }
            'm' => {
                // SGR - Select Graphic Rendition
                self.handle_sgr(params);
            }
            _ => {
                log::trace!("CSI no implementado: {}", c);
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        if !intermediates.is_empty() {
            // Secuencias de charset (ESC ( B, etc.) - ignoradas
            log::trace!("ESC con intermedios no implementado: {:?} {}", intermediates, byte);
            return;
        }
        match byte {
            b'7' => self.screen.save_cursor(),    // DECSC
            b'8' => self.screen.restore_cursor(), // DECRC
            b'D' => self.screen.index(),          // IND
            b'E' => self.screen.next_line(),      // NEL
            b'M' => self.screen.reverse_index(),  // RI
            b'c' => self.screen.reset(),          // RIS
            _ => {
                log::trace!("ESC dispatch no implementado: {}", byte as char);
            }
        }
    }
}

impl<'a> AnsiHandler<'a> {
    fn handle_private_mode(&mut self, mode: u16, enable: bool) {
        match mode {
            7 => self.screen.autowrap = enable, // DECAWM
            25 => self.screen.set_cursor_visible(enable), // DECTCEM
            47 | 1047 => {
                // Pantalla alternativa (sin guardar cursor)
                if enable {
                    self.screen.enter_alt_screen();
                } else {
                    self.screen.exit_alt_screen();
                }
            }
            1048 => {
                // Guardar/restaurar cursor
                if enable {
                    self.screen.save_cursor();
                } else {
                    self.screen.restore_cursor();
                }
            }
            1049 => {
                // Pantalla alternativa + guardar/restaurar cursor (vim, less, htop)
                if enable {
                    self.screen.save_cursor();
                    self.screen.enter_alt_screen();
                } else {
                    self.screen.exit_alt_screen();
                    self.screen.restore_cursor();
                }
            }
            2004 => self.screen.bracketed_paste = enable,
            12 => {
                // Cursor blinking - ignorado (el renderer ya anima el cursor)
            }
            1000..=1006 | 1015 => {
                // Mouse tracking - aún no soportado
                log::trace!("Mouse tracking mode {} no soportado", mode);
            }
            _ => {
                log::trace!("Modo privado no implementado: {} ({})", mode, enable);
            }
        }
    }

    fn handle_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            // Reset
            self.screen.current_attrs = CellAttributes::default();
            return;
        }

        let mut iter = params.iter();
        while let Some(param) = iter.next() {
            let n = param.first().copied().unwrap_or(0);

            match n {
                0 => {
                    // Reset
                    self.screen.current_attrs = CellAttributes::default();
                }
                1 => self.screen.current_attrs.bold = true,
                3 => self.screen.current_attrs.italic = true,
                4 => self.screen.current_attrs.underline = true,
                7 => self.screen.current_attrs.reverse = true,
                22 => self.screen.current_attrs.bold = false,
                23 => self.screen.current_attrs.italic = false,
                24 => self.screen.current_attrs.underline = false,
                27 => self.screen.current_attrs.reverse = false,
                // Foreground colors
                30..=37 => {
                    self.screen.current_attrs.fg_color = Color::Indexed((n - 30) as u8);
                }
                38 => {
                    // Extended foreground color
                    if let Some(next) = iter.next() {
                        let mode = next.first().copied().unwrap_or(0);
                        match mode {
                            5 => {
                                // 256 color
                                if let Some(color_param) = iter.next() {
                                    let color = color_param.first().copied().unwrap_or(0) as u8;
                                    self.screen.current_attrs.fg_color = Color::Indexed(color);
                                }
                            }
                            2 => {
                                // RGB
                                let r =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let g =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let b =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                self.screen.current_attrs.fg_color = Color::Rgb(r, g, b);
                            }
                            _ => {}
                        }
                    }
                }
                39 => {
                    // Default foreground
                    self.screen.current_attrs.fg_color = Color::default_fg();
                }
                // Background colors
                40..=47 => {
                    self.screen.current_attrs.bg_color = Color::Indexed((n - 40) as u8);
                }
                48 => {
                    // Extended background color
                    if let Some(next) = iter.next() {
                        let mode = next.first().copied().unwrap_or(0);
                        match mode {
                            5 => {
                                // 256 color
                                if let Some(color_param) = iter.next() {
                                    let color = color_param.first().copied().unwrap_or(0) as u8;
                                    self.screen.current_attrs.bg_color = Color::Indexed(color);
                                }
                            }
                            2 => {
                                // RGB
                                let r =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let g =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                let b =
                                    iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                                self.screen.current_attrs.bg_color = Color::Rgb(r, g, b);
                            }
                            _ => {}
                        }
                    }
                }
                49 => {
                    // Default background
                    self.screen.current_attrs.bg_color = Color::default_bg();
                }
                // Bright foreground colors
                90..=97 => {
                    self.screen.current_attrs.fg_color = Color::Indexed((n - 90 + 8) as u8);
                }
                // Bright background colors
                100..=107 => {
                    self.screen.current_attrs.bg_color = Color::Indexed((n - 100 + 8) as u8);
                }
                // Custom: Suggestion mode control
                53 => {
                    // Start suggestion mode (custom extension)
                    self.screen.start_suggestion();
                }
                54 => {
                    // End suggestion mode (custom extension)
                    self.screen.end_suggestion();
                }
                _ => {
                    log::trace!("SGR no implementado: {}", n);
                }
            }
        }
    }
}
