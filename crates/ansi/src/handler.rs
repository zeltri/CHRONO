use terminal_core::{CellAttributes, Color, Screen};
use vte::{Params, Perform};

/// Handler que implementa el trait Perform de vte para procesar secuencias ANSI
pub struct AnsiHandler<'a> {
    screen: &'a mut Screen,
}

impl<'a> AnsiHandler<'a> {
    pub fn new(screen: &'a mut Screen) -> Self {
        Self { screen }
    }
}

impl<'a> Perform for AnsiHandler<'a> {
    fn print(&mut self, c: char) {
        self.screen.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                // Backspace
                if self.screen.cursor.col > 0 {
                    self.screen.cursor.col -= 1;
                    self.screen.handle_backspace();
                }
            }
            0x09 => {
                // Tab (8 espacios)
                let next_tab = ((self.screen.cursor.col / 8) + 1) * 8;
                self.screen.cursor.col = next_tab.min(self.screen.cols - 1);
            }
            0x0A => {
                // Line feed
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
        log::trace!("Hook no implementado");
    }

    fn put(&mut self, _byte: u8) {
        log::trace!("Put no implementado");
    }

    fn unhook(&mut self) {
        log::trace!("Unhook no implementado");
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        log::trace!("OSC dispatch no implementado");
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'H' | 'f' => {
                // CUP - Cursor Position
                let row = params
                    .iter()
                    .next()
                    .and_then(|p| p.first().copied())
                    .unwrap_or(1)
                    .saturating_sub(1) as usize;
                let col = params
                    .iter()
                    .nth(1)
                    .and_then(|p| p.first().copied())
                    .unwrap_or(1)
                    .saturating_sub(1) as usize;
                self.screen.move_cursor_to(row, col);
            }
            'A' => {
                // CUU - Cursor Up
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first().copied())
                    .unwrap_or(1) as usize;
                self.screen.cursor.move_up(n);
            }
            'B' => {
                // CUD - Cursor Down
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first().copied())
                    .unwrap_or(1) as usize;
                self.screen.cursor.move_down(n, self.screen.rows - 1);
            }
            'C' => {
                // CUF - Cursor Forward
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first().copied())
                    .unwrap_or(1) as usize;
                self.screen.cursor.move_right(n, self.screen.cols - 1);
            }
            'D' => {
                // CUB - Cursor Back
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first().copied())
                    .unwrap_or(1) as usize;
                self.screen.cursor.move_left(n);
            }
            'J' => {
                // ED - Erase in Display
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first().copied())
                    .unwrap_or(0);
                match n {
                    0 => { /* Clear from cursor to end */ }
                    1 => { /* Clear from cursor to beginning */ }
                    2 => self.screen.clear(),
                    _ => {}
                }
            }
            'K' => {
                // EL - Erase in Line
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first().copied())
                    .unwrap_or(0);
                match n {
                    0 => self.screen.clear_line_right(),
                    1 => { /* Clear from start to cursor */ }
                    2 => self.screen.clear_line(),
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

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        log::trace!("ESC dispatch no implementado");
    }
}

impl<'a> AnsiHandler<'a> {
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
