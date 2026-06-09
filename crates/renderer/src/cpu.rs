use crate::ModernTheme;
use fontdue::{Font, FontSettings, Metrics};
use std::collections::HashMap;
use terminal_core::{
    extract_file_references, parse_file_entry, ContentDetector, ContentType, JsonTokenType,
    LineContext, LogLevel, Screen,
};

/// Glifo rasterizado y cacheado
struct CachedGlyph {
    metrics: Metrics,
    bitmap: Vec<u8>,
}

/// Renderer CPU moderno con tema oscuro, cache de glifos y detección inteligente de contenido
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
    /// Fuente principal + fuentes de fallback del sistema (CJK, símbolos)
    fonts: Vec<Font>,
    /// Cache de glifos rasterizados (evita rasterizar por celda en cada frame)
    glyph_cache: HashMap<char, CachedGlyph>,
    /// Archivo sobre el que está el cursor (para enlaces de stack trace)
    pub hovered_file: Option<(usize, String, Option<usize>)>,
    /// Hyperlink OSC 8 sobre el que está el cursor
    pub hovered_link: Option<String>,
    /// Tema de colores moderno
    theme: ModernTheme,
    /// Frame counter para animaciones sutiles
    frame_count: u32,
    /// Detector de contenido inteligente
    content_detector: ContentDetector,
}

/// Rutas candidatas de fuentes de fallback (CJK, símbolos, emoji monocromo)
const FALLBACK_FONT_PATHS: &[&str] = &[
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansMonoCJKjp-Regular.otf",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
    "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/System/Library/Fonts/PingFang.ttc",
    "/System/Library/Fonts/Apple Symbols.ttf",
];

impl CpuRenderer {
    pub fn new(width: u32, height: u32, font_size: f32) -> Self {
        let font_size = if font_size > 4.0 { font_size } else { 16.0 };

        // Fuente principal: Cascadia Code embebida
        let font_data = Self::load_system_font();
        let font =
            Font::from_bytes(font_data, FontSettings::default()).expect("Failed to load font");

        // Fuentes de fallback del sistema (CJK, símbolos)
        let mut fonts = vec![font];
        for path in FALLBACK_FONT_PATHS {
            if fonts.len() >= 4 {
                break;
            }
            if let Ok(data) = std::fs::read(path) {
                if let Ok(fallback) = Font::from_bytes(data, FontSettings::default()) {
                    log::info!("Fuente de fallback cargada: {}", path);
                    fonts.push(fallback);
                }
            }
        }

        // Métricas desde la fuente principal
        let line_metrics = fonts[0].horizontal_line_metrics(font_size).unwrap();
        let char_height =
            (line_metrics.ascent - line_metrics.descent + line_metrics.line_gap).ceil() as u32;
        let baseline = line_metrics.ascent.ceil() as i32;
        let metrics = fonts[0].metrics('M', font_size);
        let char_width = metrics.advance_width.ceil() as u32;

        Self {
            width,
            height,
            font_size,
            char_width,
            char_height,
            baseline,
            fonts,
            glyph_cache: HashMap::new(),
            hovered_file: None,
            hovered_link: None,
            theme: ModernTheme::default(),
            frame_count: 0,
            content_detector: ContentDetector::new(),
        }
    }

    fn load_system_font() -> &'static [u8] {
        include_bytes!("../../../fonts/CascadiaCode.ttf")
    }

    /// Altura de la barra de tabs en píxeles
    pub fn tab_bar_height(&self) -> u32 {
        self.char_height + 8
    }

    /// Calcula el número de filas y columnas que caben en la ventana (bajo la tab bar)
    pub fn calculate_grid_size(&self) -> (usize, usize) {
        let usable_height = self.height.saturating_sub(self.tab_bar_height());
        let cols = (self.width / self.char_width) as usize;
        let rows = (usable_height / self.char_height) as usize;
        (rows.max(1), cols.max(1))
    }

    /// Renderiza la pantalla activa y la barra de tabs
    pub fn render(
        &mut self,
        screen: &mut Screen,
        buffer: &mut [u32],
        tab_titles: &[String],
        active_tab: usize,
    ) {
        self.frame_count = self.frame_count.wrapping_add(1);

        let bg_color = self.theme.bg_primary_u32();
        buffer.fill(bg_color);

        self.render_tab_bar(buffer, tab_titles, active_tab);

        // Pre-pasada: detectar y cachear content types de las líneas visibles
        for row_idx in 0..screen.rows {
            if screen.content_type_cache[row_idx].is_none() {
                let line_text: String = screen
                    .display_line(row_idx)
                    .iter()
                    .map(|c| c.character)
                    .collect();
                let detected = self.content_detector.detect_line(&line_text);
                screen.content_type_cache[row_idx] = Some(detected);
            }
        }

        let screen = &*screen;
        let y_offset = self.tab_bar_height();

        for row_idx in 0..screen.rows {
            let line = screen.display_line(row_idx);
            if line.is_empty() {
                continue;
            }
            let line_text: String = line.iter().map(|c| c.character).collect();
            let line_context = screen.get_line_context(row_idx);
            let content_type = screen.content_type_cache[row_idx].unwrap_or(ContentType::Normal);

            let file_refs = if line_context == LineContext::StackTrace {
                extract_file_references(&line_text)
            } else {
                Vec::new()
            };

            let file_entry = if line_context == LineContext::FileList {
                parse_file_entry(&line_text)
            } else {
                None
            };

            let json_fragments = if content_type == ContentType::Json {
                self.content_detector.parse_json_fragments(&line_text)
            } else {
                Vec::new()
            };

            for (col_idx, cell) in line.iter().enumerate() {
                let x = col_idx as u32 * self.char_width;
                let y = row_idx as u32 * self.char_height + y_offset;

                let is_link = !file_refs.is_empty()
                    && file_refs
                        .iter()
                        .any(|f| col_idx >= f.start_col && col_idx < f.end_col);
                let is_hyperlink = cell.hyperlink.is_some();

                let is_file_entry = file_entry
                    .as_ref()
                    .is_some_and(|entry| col_idx >= entry.start_col && col_idx < entry.end_col);

                let json_fragment = json_fragments
                    .iter()
                    .find(|f| col_idx >= f.start_col && col_idx < f.end_col);

                let is_selected = screen.is_selected(row_idx, col_idx);

                // Fondo de celda (con soporte de video inverso, SGR 7)
                let bg = if is_selected {
                    self.theme.selection_bg_u32()
                } else if cell.attrs.reverse {
                    self.color_to_u32(cell.attrs.fg_color)
                } else {
                    self.color_to_u32(cell.attrs.bg_color)
                };
                self.fill_rect(buffer, x, y, self.char_width, self.char_height, bg);

                // Color del carácter
                let mut fg = if cell.attrs.reverse {
                    self.bg_color_to_u32(cell.attrs.bg_color)
                } else if cell.is_suggestion {
                    self.theme.fg_suggestion_u32()
                } else if is_link || is_hyperlink {
                    self.theme.accent_blue_u32()
                } else if is_file_entry {
                    self.get_file_color(file_entry.as_ref().unwrap())
                } else if let Some(fragment) = json_fragment {
                    self.get_json_color(fragment.token_type)
                } else {
                    self.get_content_color(cell, line_context, content_type)
                };

                if cell.attrs.dim {
                    fg = self.apply_opacity(fg, 0.55);
                }

                if cell.character != ' ' {
                    self.render_char(
                        buffer,
                        cell.character,
                        x,
                        y,
                        fg,
                        cell.attrs.bold,
                        cell.attrs.italic,
                    );
                }

                // Decoraciones
                if cell.attrs.underline {
                    self.draw_hline(buffer, x, y + self.char_height - 2, self.char_width, fg);
                }
                if cell.attrs.strikethrough {
                    self.draw_hline(buffer, x, y + self.char_height / 2, self.char_width, fg);
                }
                if is_link || is_hyperlink {
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

        // Indicador de scrollback cuando la vista no está en vivo
        if screen.view_offset() > 0 {
            self.render_scroll_indicator(buffer, screen.view_offset(), y_offset);
        }

        // Cursor (solo en vivo, no mientras se navega el scrollback)
        if screen.cursor.visible && screen.view_offset() == 0 {
            let cursor_x = screen.cursor.col as u32 * self.char_width;
            let cursor_y = screen.cursor.row as u32 * self.char_height + y_offset;

            let pulse = ((self.frame_count as f32 * 0.05).sin() + 1.0) * 0.5;
            let opacity = 0.7 + pulse * 0.3;

            let cursor_color = self.theme.fg_primary_u32();
            let final_color = self.apply_opacity(cursor_color, opacity);

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

    /// Dibuja la barra de tabs en la parte superior
    fn render_tab_bar(&mut self, buffer: &mut [u32], titles: &[String], active: usize) {
        let bar_h = self.tab_bar_height();
        let bar_bg = ModernTheme::rgb_to_u32(33, 37, 43); // bg_secondary
        self.fill_rect(buffer, 0, 0, self.width, bar_h, bar_bg);

        let tab_w = self.tab_width();
        for (i, title) in titles.iter().enumerate() {
            let x0 = i as u32 * tab_w;
            if x0 >= self.width {
                break;
            }
            let is_active = i == active;
            let tab_bg = if is_active {
                self.theme.bg_primary_u32()
            } else {
                bar_bg
            };
            let fg = if is_active {
                self.theme.fg_primary_u32()
            } else {
                ModernTheme::rgb_to_u32(92, 99, 112) // fg_muted
            };

            self.fill_rect(buffer, x0, 0, tab_w.saturating_sub(1), bar_h, tab_bg);

            // Acento superior en la tab activa
            if is_active {
                let accent = self.theme.accent_blue_u32();
                self.fill_rect(buffer, x0, 0, tab_w.saturating_sub(1), 2, accent);
            }

            // Título truncado al ancho de la tab
            let max_chars = ((tab_w / self.char_width) as usize).saturating_sub(2);
            let label: String = format!("{} {}", i + 1, title)
                .chars()
                .take(max_chars)
                .collect();
            let text_y = 4u32;
            for (ci, ch) in label.chars().enumerate() {
                let cx = x0 + self.char_width / 2 + ci as u32 * self.char_width;
                if cx + self.char_width > x0 + tab_w {
                    break;
                }
                self.render_char(buffer, ch, cx, text_y, fg, is_active, false);
            }
        }
    }

    /// Ancho en píxeles de cada tab
    fn tab_width(&self) -> u32 {
        (self.char_width * 22).min(self.width.max(1))
    }

    /// Devuelve el índice de tab bajo un punto, si el punto está en la barra
    pub fn tab_hit(&self, x: f64, y: f64, n_tabs: usize) -> Option<usize> {
        if y < 0.0 || y >= self.tab_bar_height() as f64 || x < 0.0 {
            return None;
        }
        let idx = (x as u32 / self.tab_width()) as usize;
        if idx < n_tabs {
            Some(idx)
        } else {
            None
        }
    }

    /// Pequeño indicador "↑ N" cuando se navega el scrollback
    fn render_scroll_indicator(&mut self, buffer: &mut [u32], offset: usize, y_offset: u32) {
        let label = format!(" {} ", offset);
        let w = (label.chars().count() as u32 + 1) * self.char_width;
        let x0 = self.width.saturating_sub(w + 4);
        let y0 = y_offset + 4;
        let bg = ModernTheme::rgb_to_u32(57, 63, 74);
        let fg = self.theme.accent_yellow_u32();
        self.fill_rect(buffer, x0, y0, w, self.char_height, bg);
        let mut x = x0;
        self.render_char(buffer, '↑', x, y0, fg, false, false);
        x += self.char_width;
        for ch in label.chars() {
            self.render_char(buffer, ch, x, y0, fg, false, false);
            x += self.char_width;
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

    /// Línea horizontal de 1px (underline / strikethrough)
    fn draw_hline(&self, buffer: &mut [u32], x: u32, y: u32, w: u32, color: u32) {
        if y >= self.height {
            return;
        }
        for dx in 0..w {
            let px = x + dx;
            if px >= self.width {
                break;
            }
            let idx = (y * self.width + px) as usize;
            if idx < buffer.len() {
                buffer[idx] = color;
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

    /// Como color_to_u32, pero Default mapea al fondo del tema (para video inverso)
    fn bg_color_to_u32(&self, color: terminal_core::Color) -> u32 {
        match color {
            terminal_core::Color::Default => self.theme.bg_primary_u32(),
            other => self.color_to_u32(other),
        }
    }

    /// Obtiene (o rasteriza y cachea) el glifo de un carácter, con fallback de fuentes
    fn glyph(&mut self, ch: char) -> &CachedGlyph {
        let font_size = self.font_size;
        let fonts = &self.fonts;
        self.glyph_cache.entry(ch).or_insert_with(|| {
            // Buscar la primera fuente que tenga el glifo; si ninguna, usar la principal
            let font = fonts
                .iter()
                .find(|f| f.lookup_glyph_index(ch) != 0)
                .unwrap_or(&fonts[0]);
            let (metrics, bitmap) = font.rasterize(ch, font_size);
            CachedGlyph { metrics, bitmap }
        })
    }

    /// Renderiza un carácter con estilos sintetizados (bold = doble pase, italic = shear)
    #[allow(clippy::too_many_arguments)]
    fn render_char(
        &mut self,
        buffer: &mut [u32],
        ch: char,
        x: u32,
        y: u32,
        color: u32,
        bold: bool,
        italic: bool,
    ) {
        let (metrics, bitmap) = {
            let g = self.glyph(ch);
            (g.metrics, g.bitmap.clone())
        };

        self.blit_glyph(buffer, &metrics, &bitmap, x, y, color, italic);
        if bold {
            // Bold sintetizado: segundo pase desplazado 1px
            self.blit_glyph(buffer, &metrics, &bitmap, x + 1, y, color, italic);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn blit_glyph(
        &self,
        buffer: &mut [u32],
        metrics: &Metrics,
        bitmap: &[u8],
        x: u32,
        y: u32,
        color: u32,
        italic: bool,
    ) {
        let glyph_x = x as i32 + metrics.xmin;
        let glyph_y = y as i32 + self.baseline - metrics.height as i32 - metrics.ymin;

        for gy in 0..metrics.height {
            // Italic sintetizado: shear horizontal proporcional a la altura
            let shear = if italic {
                ((metrics.height.saturating_sub(gy)) as f32 * 0.25) as i32
            } else {
                0
            };
            for gx in 0..metrics.width {
                let px = glyph_x + gx as i32 + shear;
                let py = glyph_y + gy as i32;

                if px >= 0 && py >= 0 && (px as u32) < self.width && (py as u32) < self.height {
                    let alpha = bitmap[gy * metrics.width + gx];
                    if alpha > 0 {
                        let idx = (py as u32 * self.width + px as u32) as usize;
                        if idx < buffer.len() {
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
            LineContext::FileList => self.theme.fg_primary_u32(),
            LineContext::Normal => self.color_to_u32(cell.attrs.fg_color),
        }
    }

    /// Obtiene color basado en el tipo de contenido detectado
    fn get_content_color(
        &self,
        cell: &terminal_core::Cell,
        context: LineContext,
        content_type: ContentType,
    ) -> u32 {
        match content_type {
            ContentType::Log(level) => self.get_log_color(level),
            ContentType::Error => self.theme.log_error_u32(),
            ContentType::Warning => self.theme.log_warn_u32(),
            ContentType::Success => self.theme.success_u32(),
            ContentType::StackTrace => self.theme.accent_cyan_u32(),
            ContentType::Table => self.theme.fg_primary_u32(),
            ContentType::Json => self.theme.fg_primary_u32(),
            ContentType::Normal => self.get_context_color(cell, context),
        }
    }

    /// Obtiene color según el nivel de log
    fn get_log_color(&self, level: LogLevel) -> u32 {
        match level {
            LogLevel::Trace => self.theme.log_trace_u32(),
            LogLevel::Debug => self.theme.log_debug_u32(),
            LogLevel::Info => self.theme.log_info_u32(),
            LogLevel::Warn => self.theme.log_warn_u32(),
            LogLevel::Error => self.theme.log_error_u32(),
            LogLevel::Fatal => self.theme.log_fatal_u32(),
        }
    }

    /// Obtiene color según el tipo de token JSON
    fn get_json_color(&self, token_type: JsonTokenType) -> u32 {
        match token_type {
            JsonTokenType::BraceOpen
            | JsonTokenType::BraceClose
            | JsonTokenType::BracketOpen
            | JsonTokenType::BracketClose => self.theme.json_bracket_u32(),
            JsonTokenType::Key => self.theme.json_key_u32(),
            JsonTokenType::String => self.theme.json_string_u32(),
            JsonTokenType::Number => self.theme.json_number_u32(),
            JsonTokenType::Boolean => self.theme.json_boolean_u32(),
            JsonTokenType::Null => self.theme.json_null_u32(),
            JsonTokenType::Colon | JsonTokenType::Comma => self.theme.fg_primary_u32(),
        }
    }

    /// Obtiene color según tipo de archivo y permisos
    pub fn get_file_color(&self, entry: &terminal_core::FileEntry) -> u32 {
        use terminal_core::FileType;

        match entry.file_type {
            FileType::Directory => self.theme.accent_blue_u32(),
            FileType::Executable if entry.is_executable => self.theme.accent_green_u32(),
            FileType::SymbolicLink => self.theme.accent_cyan_u32(),
            FileType::Archive => self.theme.accent_red_u32(),
            FileType::Image => self.theme.accent_magenta_u32(),
            FileType::Video => self.theme.accent_magenta_u32(),
            FileType::Audio => self.theme.accent_cyan_u32(),
            FileType::Document => self.theme.accent_yellow_u32(),
            FileType::Code => self.theme.accent_green_u32(),
            FileType::RegularFile if entry.is_executable => self.theme.accent_green_u32(),
            FileType::RegularFile => self.theme.fg_primary_u32(),
            _ => self.theme.fg_primary_u32(),
        }
    }

    /// Dibuja subrayado moderno con glow sutil
    fn draw_underline(&self, buffer: &mut [u32], x: u32, y: u32, w: u32, h: u32, color: u32) {
        let underline_y = (y + h - 2) as usize;
        if underline_y >= self.height as usize {
            return;
        }

        for dx in 0..w as usize {
            let px = x as usize + dx;
            let idx = underline_y * self.width as usize + px;
            if idx < buffer.len() && px < self.width as usize {
                buffer[idx] = color;
            }

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

    /// Detecta si el cursor está sobre un enlace (stack trace u OSC 8)
    pub fn check_file_hover(&mut self, screen: &Screen, mouse_x: f64, mouse_y: f64) {
        self.hovered_file = None;
        self.hovered_link = None;

        let (row, col) = self.screen_to_grid(mouse_x, mouse_y);
        if row >= screen.rows {
            return;
        }

        // Hyperlinks OSC 8 tienen prioridad
        if let Some(uri) = screen.display_hyperlink_at(row, col) {
            self.hovered_link = Some(uri.to_string());
            return;
        }

        let line_text: String = screen
            .display_line(row)
            .iter()
            .map(|c| c.character)
            .collect();
        let file_refs = extract_file_references(&line_text);

        for file_ref in file_refs {
            if col >= file_ref.start_col && col < file_ref.end_col {
                self.hovered_file = Some((row, file_ref.path.clone(), file_ref.line));
                return;
            }
        }
    }

    /// Convierte coordenadas de pantalla a posición de grid (fila, columna)
    pub fn screen_to_grid(&self, mouse_x: f64, mouse_y: f64) -> (usize, usize) {
        let col = (mouse_x / self.char_width as f64) as usize;
        let y = (mouse_y - self.tab_bar_height() as f64).max(0.0);
        let row = (y / self.char_height as f64) as usize;
        (row, col)
    }
}
