use fontdue::{Font, FontSettings};
use terminal_core::{extract_file_references, LineContext, Screen};

/// Renderer CPU básico que convierte la pantalla a un buffer de píxeles
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
}

impl CpuRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let font_size = 16.0;

        // Cargar fuente monoespaciada del sistema
        let font_data = Self::load_system_font();
        let font =
            Font::from_bytes(font_data, FontSettings::default()).expect("Failed to load font");

        // Obtener métricas de línea
        let line_metrics = font.horizontal_line_metrics(font_size).unwrap();

        // Calcular dimensiones de celda basadas en las métricas de la fuente
        let char_height =
            (line_metrics.ascent - line_metrics.descent + line_metrics.line_gap).ceil() as u32;
        let baseline = line_metrics.ascent.ceil() as i32;

        // Usar el advance width de 'M' como ancho estándar de celda
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
        }
    }

    fn load_system_font() -> &'static [u8] {
        // Usar Cascadia Code monoespaciada
        include_bytes!("../../../fonts/CascadiaCode.ttf")
    }

    /// Calcula el número de filas y columnas que caben en la ventana
    pub fn calculate_grid_size(&self) -> (usize, usize) {
        let cols = (self.width / self.char_width) as usize;
        let rows = (self.height / self.char_height) as usize;
        (rows.max(1), cols.max(1))
    }

    /// Renderiza la pantalla a un buffer RGBA
    pub fn render(&self, screen: &Screen, buffer: &mut [u32]) {
        // Limpiar buffer con color de fondo
        let bg_color = 0xFF000000; // Negro opaco
        buffer.fill(bg_color);

        let grid = screen.get_visible();

        for (row_idx, row) in grid.iter().enumerate() {
            // Obtener contexto de la línea
            let line_context = screen.get_line_context(row_idx);

            // Extraer referencias a archivos si es un stack trace
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
                        self.rgb_to_u32(100, 180, 255) // Azul para enlaces
                    } else {
                        self.get_context_color(cell, line_context)
                    };
                    self.render_char(buffer, cell.character, x, y, fg);
                }

                // Subrayar enlaces
                if is_link {
                    let fg = self.rgb_to_u32(100, 180, 255);
                    self.draw_underline(buffer, x, y, self.char_width, self.char_height, fg);
                }
            }
        }

        // Renderizar cursor
        if screen.cursor.visible {
            let cursor_x = screen.cursor.col as u32 * self.char_width;
            let cursor_y = screen.cursor.row as u32 * self.char_height;
            let cursor_color = 0xFFFFFFFF; // Blanco
            self.fill_rect(
                buffer,
                cursor_x,
                cursor_y + self.char_height - 2,
                self.char_width,
                2,
                cursor_color,
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
        let (r, g, b) = color.to_rgb();
        0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    fn render_char(&self, buffer: &mut [u32], ch: char, x: u32, y: u32, color: u32) {
        let (metrics, bitmap) = self.font.rasterize(ch, self.font_size);

        // Usar baseline consistente para todos los caracteres
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

    /// Obtiene color basado en el contexto de la línea
    fn get_context_color(&self, cell: &terminal_core::Cell, context: LineContext) -> u32 {
        match context {
            LineContext::Error => self.rgb_to_u32(255, 100, 100), // Rojo
            LineContext::Warning => self.rgb_to_u32(255, 200, 100), // Amarillo/naranja
            LineContext::StackTrace => self.rgb_to_u32(180, 180, 180), // Gris claro
            LineContext::Normal => self.color_to_u32(cell.attrs.fg_color),
        }
    }

    /// Dibuja un subrayado
    fn draw_underline(&self, buffer: &mut [u32], x: u32, y: u32, w: u32, h: u32, color: u32) {
        let underline_y = (y + h - 2) as usize;
        for dx in 0..w as usize {
            let px = x as usize + dx;
            let idx = underline_y * self.width as usize + px;
            if idx < buffer.len() {
                buffer[idx] = color;
            }
        }
    }

    /// Convierte RGB a u32
    fn rgb_to_u32(&self, r: u8, g: u8, b: u8) -> u32 {
        0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
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
