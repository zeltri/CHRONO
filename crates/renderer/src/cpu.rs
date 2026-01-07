use crate::ModernTheme;
use fontdue::{Font, FontSettings};
use terminal_core::{extract_file_references, LineContext, Screen};

/// Renderer CPU moderno con tema oscuro innovador
pub struct CpuRenderer {
    /// Ancho de la ventana en píxeles
    width: u32,
    /// Alto de la ventana en píxeles
    height: u32,
    /// Tamaño de fuente en píxeles
    font_size: f32,
    /// Ancho de carácter en píxeles
    char_width: u32,
    /// Alto de carácter en píxeles
    char_height: u32,
    /// Baseline desde la parte superior de la celda
    baseline: i32,
    /// Fuente cargada
    font: Font,
    /// Archivo sobre el que está el cursor (para enlaces)
    pub hovered_file: Option<(usize, String, Option<usize>)>,
    /// Tema de colores moderno (Tokyo Night inspired)
    theme: ModernTheme,
    /// Frame counter para animaciones sutiles
    frame_count: u32,
}

impl CpuRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let font_size = 16.0;

        // Cargar Cascadia Code
        let font_data = Self::load_system_font();
        let font =
            Font::from_bytes(font_data, FontSettings::default()).expect("Failed to load font");

        // Obtener métricas de línea
        let line_metrics = font.horizontal_line_metrics(font_size).unwrap();

        // Calcular dimensiones de celda
        let char_height =
            (line_metrics.ascent - line_metrics.descent + line_metrics.line_gap).ceil() as u32;
        let baseline = line_metrics.ascent.ceil() as i32;

        // Usar el advance width de 'M' como ancho estándar
        let metrics = font.metrics('M', font_size);
        let char_width = metrics.advance_width.ceil() as u32;

        Self {
            width,
            height,
            font_size,
            char_width,
            char_height,
            baseline,
            font,
            hovered_file: None,
            theme: ModernTheme::default(),
            frame_count: 0,
        }
    }

    fn load_system_font() -> &'static [u8] {
        include_bytes!("../../../fonts/CascadiaCode.ttf")
    }

    /// Calcula el número de filas y columnas que caben en la ventana
    pub fn calculate_grid_size(&self) -> (usize, usize) {
        let cols = (self.width / self.char_width) as usize;
        let rows = (self.height / self.char_height) as usize;
        (rows.max(1), cols.max(1))
    }

    pub fn render(&mut self, screen: &Screen, buffer: &mut [u32]) {
        self.frame_count = self.frame_count.wrapping_add(1);

        // Limpiar buffer con fondo moderno
        let bg_color = self.theme.bg_primary_u32();
        buffer.fill(bg_color);

        let grid = screen.get_visible();

        for (row_idx, row) in grid.iter().enumerate() {
            // Obtener contexto de la línea
            let line_context = screen.get_line_context(row_idx);

            // Extraer referencias a archivos si es stack trace
            let file_refs = if line_context == LineContext::StackTrace {
                let line_text: String = row.iter().map(|c| c.character).collect();
                extract_file_references(&line_text)
            } else {
                Vec::new()
            };

            for (col_idx, cell) in row.iter().enumerate() {
                let x = col_idx as u32 * self.char_width;
                let y = row_idx as u32 * self.char_height;

                // Determinar si esta celda es parte de un enlace
                let is_link = !file_refs.is_empty()
                    && file_refs
                        .iter()
                        .any(|f| col_idx >= f.start_col && col_idx < f.end_col);

                // Renderizar fondo de celda
                let bg = self.color_to_u32(cell.attrs.bg_color);
                self.fill_rect(buffer, x, y, self.char_width, self.char_height, bg);

                // Renderizar el carácter con color contextual
                if cell.character != ' ' {
                    let fg = if is_link {
                        self.theme.accent_blue_u32() // Enlaces en azul moderno
                    } else {
                        self.get_context_color(cell, line_context)
                    };
                    self.render_char(buffer, cell.character, x, y, fg);
                }

                // Subrayar enlaces con efecto moderno
                if is_link {
                    let link_color = self.theme.accent_blue_u32();
                    self.draw_underline(
                        buffer,
                        x,
                        y,
                        self.char_width,
                        self.char_height,
                        link_color,
                    );
                }
            }
        }

        // Renderizar cursor animado con pulse
        if screen.cursor.visible {
            let cursor_x = screen.cursor.col as u32 * self.char_width;
            let cursor_y = screen.cursor.row as u32 * self.char_height;

            // Animación de pulse sutil
            let pulse = ((self.frame_count as f32 * 0.05).sin() + 1.0) * 0.5; // 0.0 - 1.0
            let opacity = 0.7 + pulse * 0.3; // 0.7 - 1.0

            let cursor_color = self.theme.fg_primary_u32();
            let final_color = self.apply_opacity(cursor_color, opacity);

            // Cursor más grueso y moderno (3px)
            self.fill_rect(
                buffer,
                cursor_x,
                cursor_y + self.char_height - 3,
                self.char_width,
                3,
                final_color,
            );
        }
    }

    fn fill_rect(&self, buffer: &mut [u32], x: u32, y: u32, w: u32, h: u32, color: u32) {
        for dy in 0..h {
            let py = y + dy;
            if py >= self.height {
                break;
            }
            for dx in 0..w {
                let px = x + dx;
                if px >= self.width {
                    break;
                }
                let idx = (py * self.width + px) as usize;
                if idx < buffer.len() {
                    buffer[idx] = color;
                }
            }
        }
    }

    fn color_to_u32(&self, color: terminal_core::Color) -> u32 {
        match color {
            terminal_core::Color::Indexed(idx) => self.theme.get_ansi_color(idx),
            terminal_core::Color::Rgb(r, g, b) => ModernTheme::rgb_to_u32(r, g, b),
            terminal_core::Color::Default => self.theme.fg_primary_u32(),
        }
    }

    fn render_char(&self, buffer: &mut [u32], ch: char, x: u32, y: u32, color: u32) {
        let (metrics, bitmap) = self.font.rasterize(ch, self.font_size);

        // Usar baseline consistente
        let glyph_x = x as i32 + metrics.xmin;
        let glyph_y = y as i32 + self.baseline - metrics.height as i32 - metrics.ymin;

        for gy in 0..metrics.height {
            for gx in 0..metrics.width {
                let px = glyph_x + gx as i32;
                let py = glyph_y + gy as i32;

                if px >= 0 && py >= 0 && (px as u32) < self.width && (py as u32) < self.height {
                    let alpha = bitmap[gy * metrics.width + gx];
                    if alpha > 0 {
                        let idx = (py as u32 * self.width + px as u32) as usize;
                        if idx < buffer.len() {
                            // Blend alpha
                            let alpha_f = alpha as f32 / 255.0;
                            let bg = buffer[idx];
                            let bg_r = ((bg >> 16) & 0xFF) as f32;
                            let bg_g = ((bg >> 8) & 0xFF) as f32;
                            let bg_b = (bg & 0xFF) as f32;

                            let fg_r = ((color >> 16) & 0xFF) as f32;
                            let fg_g = ((color >> 8) & 0xFF) as f32;
                            let fg_b = (color & 0xFF) as f32;

                            let r = (fg_r * alpha_f + bg_r * (1.0 - alpha_f)) as u32;
                            let g = (fg_g * alpha_f + bg_g * (1.0 - alpha_f)) as u32;
                            let b = (fg_b * alpha_f + bg_b * (1.0 - alpha_f)) as u32;

                            buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                        }
                    }
                }
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Obtiene color contextual con tema moderno
    fn get_context_color(&self, cell: &terminal_core::Cell, context: LineContext) -> u32 {
        match context {
            LineContext::Error => self.theme.accent_red_u32(),
            LineContext::Warning => self.theme.accent_yellow_u32(),
            LineContext::StackTrace => self.theme.accent_cyan_u32(),
            LineContext::Normal => self.color_to_u32(cell.attrs.fg_color),
        }
    }

    /// Dibuja subrayado moderno con glow sutil
    fn draw_underline(&self, buffer: &mut [u32], x: u32, y: u32, w: u32, h: u32, color: u32) {
        let underline_y = (y + h - 2) as usize;

        // Línea principal
        for dx in 0..w as usize {
            let px = x as usize + dx;
            let idx = underline_y * self.width as usize + px;
            if idx < buffer.len() && px < self.width as usize {
                buffer[idx] = color;
            }

            // Glow sutil arriba (efecto moderno)
            if underline_y > 0 {
                let glow_idx = (underline_y - 1) * self.width as usize + px;
                if glow_idx < buffer.len() && px < self.width as usize {
                    let bg_r = ((buffer[glow_idx] >> 16) & 0xFF) as f32;
                    let bg_g = ((buffer[glow_idx] >> 8) & 0xFF) as f32;
                    let bg_b = (buffer[glow_idx] & 0xFF) as f32;

                    let fg_r = ((color >> 16) & 0xFF) as f32;
                    let fg_g = ((color >> 8) & 0xFF) as f32;
                    let fg_b = (color & 0xFF) as f32;

                    let alpha = 0.3;
                    let r = (fg_r * alpha + bg_r * (1.0 - alpha)) as u32;
                    let g = (fg_g * alpha + bg_g * (1.0 - alpha)) as u32;
                    let b = (fg_b * alpha + bg_b * (1.0 - alpha)) as u32;

                    buffer[glow_idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
    }

    /// Aplica opacidad a un color
    fn apply_opacity(&self, color: u32, opacity: f32) -> u32 {
        let r = ((color >> 16) & 0xFF) as f32;
        let g = ((color >> 8) & 0xFF) as f32;
        let b = (color & 0xFF) as f32;

        let final_r = (r * opacity) as u32;
        let final_g = (g * opacity) as u32;
        let final_b = (b * opacity) as u32;

        0xFF000000 | (final_r << 16) | (final_g << 8) | final_b
    }

    /// Detecta si el cursor está sobre un enlace de archivo
    pub fn check_file_hover(&mut self, screen: &Screen, mouse_x: f64, mouse_y: f64) {
        let col = (mouse_x / self.char_width as f64) as usize;
        let row = (mouse_y / self.char_height as f64) as usize;

        if row >= screen.rows {
            self.hovered_file = None;
            return;
        }

        let line_text: String = screen.grid[row].iter().map(|c| c.character).collect();
        let file_refs = extract_file_references(&line_text);

        for file_ref in file_refs {
            if col >= file_ref.start_col && col < file_ref.end_col {
                self.hovered_file = Some((row, file_ref.path.clone(), file_ref.line));
                return;
            }
        }

        self.hovered_file = None;
    }
}
