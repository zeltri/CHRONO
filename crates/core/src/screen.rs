use crate::{
    attributes::CellAttributes,
    cell::Cell,
    context::{analyze_line_context, LineContext},
    cursor::Cursor,
    detector::ContentType,
    history::CommandHistory,
};

/// Estado guardado de la pantalla principal mientras la pantalla alternativa está activa
struct MainScreenState {
    grid: Vec<Vec<Cell>>,
    grid_wrapped: Vec<bool>,
    cursor: Cursor,
    attrs: CellAttributes,
}

/// Cursor guardado por DECSC / CSI s
#[derive(Clone, Copy)]
struct SavedCursor {
    row: usize,
    col: usize,
    attrs: CellAttributes,
}

/// Modo de tracking de mouse solicitado por la aplicación (DECSET 9/1000/1002/1003)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseMode {
    #[default]
    None,
    /// Modo 9: solo press
    X10,
    /// Modo 1000: press + release
    Normal,
    /// Modo 1002: press + release + drag
    ButtonEvent,
    /// Modo 1003: todo el movimiento
    AnyEvent,
}

/// Buffer de pantalla de terminal
pub struct Screen {
    /// Dimensiones
    pub rows: usize,
    pub cols: usize,

    /// Grid principal
    pub grid: Vec<Vec<Cell>>,

    /// Flag por fila del grid: true si la fila continúa en la siguiente (soft wrap)
    grid_wrapped: Vec<bool>,

    /// Scrollback buffer (líneas pasadas) con sus flags de wrap
    scrollback: Vec<Vec<Cell>>,
    scrollback_wrapped: Vec<bool>,

    /// Líneas descartadas del scrollback desde el inicio (para índices absolutos estables)
    lines_scrolled_off: usize,

    /// Desplazamiento de la vista sobre el scrollback (0 = pegado abajo, en vivo)
    view_offset: usize,

    /// Cursor
    pub cursor: Cursor,

    /// Atributos actuales (para próximas escrituras)
    pub current_attrs: CellAttributes,

    /// Máximo de líneas en scrollback
    max_scrollback: usize,

    /// Contexto semántico por línea
    pub line_contexts: Vec<LineContext>,

    /// Cache de ContentType detectado por línea visible (para evitar re-análisis costosos)
    pub content_type_cache: Vec<Option<ContentType>>,

    /// Región de scroll (DECSTBM): primera y última fila, inclusive
    scroll_top: usize,
    scroll_bottom: usize,

    /// Estado de la pantalla principal mientras la alternativa está activa
    alt_screen: Option<MainScreenState>,

    /// Cursor guardado (DECSC / CSI s)
    saved_cursor: Option<SavedCursor>,

    /// Último carácter imprimible escrito (para REP, CSI b)
    last_char: Option<char>,

    /// Modo de bracketed paste (DECSET 2004)
    pub bracketed_paste: bool,

    /// Auto-wrap (DECAWM, modo 7)
    pub autowrap: bool,

    /// Modo de mouse tracking activo
    pub mouse_mode: MouseMode,

    /// Codificación SGR para mouse (modo 1006)
    pub mouse_sgr: bool,

    /// Título de ventana pendiente (OSC 0/2), consumido por la app
    pending_title: Option<String>,

    /// Registro de URIs de hyperlinks OSC 8
    hyperlinks: Vec<String>,

    /// Hyperlink activo para próximas escrituras
    current_hyperlink: Option<u16>,

    /// Marcas de prompt (OSC 133;A) como índices de línea absolutos
    prompt_marks: Vec<usize>,

    /// Modo de sugerencia activo (para autocompletado)
    suggestion_mode: bool,

    /// Columna donde inició la sugerencia (para limpieza)
    suggestion_start_col: usize,

    /// Historial de comandos ejecutados
    command_history: CommandHistory,

    /// Buffer del comando actual siendo escrito
    current_command: String,

    /// Columna donde inició el comando actual
    command_start_col: usize,

    /// Sugerencia actual activa (el sufijo que se muestra en gris)
    active_suggestion: Option<String>,

    /// Flag para indicar si el screen ha sido modificado desde el último render
    dirty: bool,

    /// Selección activa (inicio y fin)
    selection: Option<Selection>,
}

/// Estructura para manejar la selección de texto
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection {
    /// Posición inicial de la selección (fila, columna)
    pub start: (usize, usize),
    /// Posición final de la selección (fila, columna)
    pub end: (usize, usize),
}

/// Caracteres que forman parte de una "palabra" para la selección por doble click
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '~' | ':' | '@')
}

impl Screen {
    pub fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![vec![Cell::empty(); cols]; rows];

        Self {
            rows,
            cols,
            grid,
            grid_wrapped: vec![false; rows],
            scrollback: Vec::new(),
            scrollback_wrapped: Vec::new(),
            lines_scrolled_off: 0,
            view_offset: 0,
            cursor: Cursor::new(),
            current_attrs: CellAttributes::default(),
            max_scrollback: 10_000,
            line_contexts: vec![LineContext::Normal; rows],
            content_type_cache: vec![None; rows],
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            alt_screen: None,
            saved_cursor: None,
            last_char: None,
            bracketed_paste: false,
            autowrap: true,
            mouse_mode: MouseMode::None,
            mouse_sgr: false,
            pending_title: None,
            hyperlinks: Vec::new(),
            current_hyperlink: None,
            prompt_marks: Vec::new(),
            suggestion_mode: false,
            suggestion_start_col: 0,
            command_history: CommandHistory::from_shell_history(1000),
            current_command: String::new(),
            command_start_col: 0,
            active_suggestion: None,
            dirty: true,
            selection: None,
        }
    }

    /// Ajusta el máximo de líneas de scrollback (desde la config)
    pub fn set_max_scrollback(&mut self, max: usize) {
        self.max_scrollback = max;
    }

    /// Verifica si el screen necesita ser re-renderizado
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marca el screen como limpio (después de renderizar)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Marca el screen como dirty (necesita re-renderizado)
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    // ==================== Vista de scrollback ====================

    /// Desplazamiento actual de la vista (0 = en vivo)
    pub fn view_offset(&self) -> usize {
        self.view_offset
    }

    /// Cantidad de líneas disponibles en el scrollback
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    /// Desplaza la vista hacia arriba (hacia el pasado)
    pub fn scroll_view_up(&mut self, n: usize) {
        if self.alt_screen.is_some() {
            return;
        }
        let new_offset = (self.view_offset + n).min(self.scrollback.len());
        if new_offset != self.view_offset {
            self.view_offset = new_offset;
            self.invalidate_all_lines();
            self.mark_dirty();
        }
    }

    /// Desplaza la vista hacia abajo (hacia el presente)
    pub fn scroll_view_down(&mut self, n: usize) {
        let new_offset = self.view_offset.saturating_sub(n);
        if new_offset != self.view_offset {
            self.view_offset = new_offset;
            self.invalidate_all_lines();
            self.mark_dirty();
        }
    }

    /// Vuelve la vista al presente (pegada abajo)
    pub fn reset_view(&mut self) {
        if self.view_offset != 0 {
            self.view_offset = 0;
            self.invalidate_all_lines();
            self.mark_dirty();
        }
    }

    /// Línea visible en la fila `row` de la pantalla, considerando el scroll de vista.
    /// Devuelve un slice vacío si la fila no existe.
    pub fn display_line(&self, row: usize) -> &[Cell] {
        let sb = self.scrollback.len();
        let offset = self.view_offset.min(sb);
        let idx = sb - offset + row;
        if idx < sb {
            &self.scrollback[idx]
        } else {
            let g = idx - sb;
            if g < self.grid.len() {
                &self.grid[g]
            } else {
                &[]
            }
        }
    }

    // ==================== Marcas de prompt (OSC 133) ====================

    /// Índice absoluto de la línea donde está el cursor
    fn cursor_abs_line(&self) -> usize {
        self.lines_scrolled_off + self.scrollback.len() + self.cursor.row
    }

    /// Registra el inicio de un prompt (OSC 133;A)
    pub fn mark_prompt(&mut self) {
        let abs = self.cursor_abs_line();
        if self.prompt_marks.last() != Some(&abs) {
            self.prompt_marks.push(abs);
        }
    }

    /// Índice absoluto de la primera fila visible
    fn view_top_abs(&self) -> usize {
        self.lines_scrolled_off + self.scrollback.len()
            - self.view_offset.min(self.scrollback.len())
    }

    /// Convierte un índice absoluto a un view_offset y lo aplica
    fn view_to_abs(&mut self, abs: usize) {
        let sb_start = self.lines_scrolled_off;
        let sb_len = self.scrollback.len();
        if abs < sb_start + sb_len {
            self.view_offset = sb_len - (abs - sb_start);
        } else {
            self.view_offset = 0;
        }
        self.invalidate_all_lines();
        self.mark_dirty();
    }

    /// Salta la vista al prompt anterior
    pub fn view_to_prev_prompt(&mut self) {
        let top = self.view_top_abs();
        if let Some(&mark) = self.prompt_marks.iter().rev().find(|&&m| m < top) {
            self.view_to_abs(mark);
        }
    }

    /// Salta la vista al prompt siguiente
    pub fn view_to_next_prompt(&mut self) {
        let top = self.view_top_abs();
        if let Some(&mark) = self.prompt_marks.iter().find(|&&m| m > top) {
            self.view_to_abs(mark);
        } else {
            self.reset_view();
        }
    }

    // ==================== Hyperlinks (OSC 8) ====================

    /// Define el hyperlink activo para próximas escrituras (None lo termina)
    pub fn set_hyperlink(&mut self, uri: Option<String>) {
        self.current_hyperlink = uri.and_then(|u| {
            if u.is_empty() {
                return None;
            }
            if let Some(pos) = self.hyperlinks.iter().position(|h| h == &u) {
                Some(pos as u16)
            } else if self.hyperlinks.len() < u16::MAX as usize {
                self.hyperlinks.push(u);
                Some((self.hyperlinks.len() - 1) as u16)
            } else {
                None
            }
        });
    }

    /// URI de un hyperlink registrado
    pub fn hyperlink_uri(&self, id: u16) -> Option<&str> {
        self.hyperlinks.get(id as usize).map(|s| s.as_str())
    }

    /// Hyperlink bajo una posición visible (fila/columna de pantalla)
    pub fn display_hyperlink_at(&self, row: usize, col: usize) -> Option<&str> {
        let line = self.display_line(row);
        line.get(col)
            .and_then(|cell| cell.hyperlink)
            .and_then(|id| self.hyperlink_uri(id))
    }

    // ==================== Escritura ====================

    /// Celda en blanco con el color de fondo actual (Background Color Erase)
    fn blank_cell(&self) -> Cell {
        let attrs = CellAttributes {
            bg_color: self.current_attrs.bg_color,
            ..CellAttributes::default()
        };
        Cell::with_attrs(' ', attrs)
    }

    /// Línea en blanco con el color de fondo actual
    fn blank_line(&self) -> Vec<Cell> {
        vec![self.blank_cell(); self.cols]
    }

    /// Invalida el cache de detección de contenido de una línea
    fn invalidate_line(&mut self, row: usize) {
        if row < self.content_type_cache.len() {
            self.content_type_cache[row] = None;
        }
    }

    /// Invalida el cache de todas las líneas (tras scroll u operaciones estructurales)
    fn invalidate_all_lines(&mut self) {
        for entry in &mut self.content_type_cache {
            *entry = None;
        }
    }

    /// Escribe un carácter en la posición del cursor
    pub fn write_char(&mut self, ch: char) {
        self.mark_dirty();
        self.last_char = Some(ch);
        self.invalidate_line(self.cursor.row);

        if self.cursor.col >= self.cols {
            if self.autowrap {
                // Soft wrap: la fila actual continúa en la siguiente
                if self.cursor.row < self.grid_wrapped.len() {
                    self.grid_wrapped[self.cursor.row] = true;
                }
                self.cursor.col = 0;
                self.index();
            } else {
                self.cursor.col = self.cols - 1;
            }
        }

        if self.cursor.row < self.rows && self.cursor.col < self.cols {
            if !self.suggestion_mode && self.has_suggestions() {
                self.clear_suggestions();
            }

            let mut cell = Cell::with_attrs(ch, self.current_attrs);
            let width = (cell.width as usize).max(1);
            cell.hyperlink = self.current_hyperlink;

            if self.suggestion_mode {
                cell.is_suggestion = true;
            }

            self.grid[self.cursor.row][self.cursor.col] = cell;

            if width > 1 {
                for i in 1..width {
                    if self.cursor.col + i < self.cols {
                        self.grid[self.cursor.row][self.cursor.col + i] = Cell::empty();
                    }
                }
            }

            self.cursor.col += width;
        }
    }

    /// Repite el último carácter impreso n veces (REP, CSI b)
    pub fn repeat_last_char(&mut self, n: usize) {
        if let Some(ch) = self.last_char {
            for _ in 0..n {
                self.write_char(ch);
            }
        }
    }

    /// Index (IND): baja una línea, haciendo scroll si está al final de la región
    pub fn index(&mut self) {
        self.mark_dirty();
        if self.cursor.row == self.scroll_bottom {
            self.scroll_up(1);
        } else if self.cursor.row < self.rows - 1 {
            self.cursor.row += 1;
        }
    }

    /// Reverse Index (RI): sube una línea, haciendo scroll inverso si está al inicio de la región
    pub fn reverse_index(&mut self) {
        self.mark_dirty();
        if self.cursor.row == self.scroll_top {
            self.scroll_down(1);
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
        }
    }

    /// Next Line (NEL): carriage return + index
    pub fn next_line(&mut self) {
        self.carriage_return();
        self.index();
    }

    /// Line feed - avanza una línea (respetando la región de scroll)
    pub fn line_feed(&mut self) {
        self.active_suggestion = None;
        // Un line feed explícito es un corte duro: la línea no continúa
        if self.cursor.row < self.grid_wrapped.len() {
            self.grid_wrapped[self.cursor.row] = false;
        }
        self.index();
    }

    /// Carriage return - vuelve al inicio de la línea
    pub fn carriage_return(&mut self) {
        if self.cursor.col != 0 {
            self.mark_dirty();
        }
        self.cursor.col = 0;
    }

    /// Scroll hacia arriba n líneas dentro de la región de scroll
    pub fn scroll_up(&mut self, n: usize) {
        self.mark_dirty();
        self.invalidate_all_lines();
        let full_screen = self.scroll_top == 0 && self.scroll_bottom == self.rows - 1;

        for _ in 0..n {
            if self.scroll_top > self.scroll_bottom || self.scroll_bottom >= self.grid.len() {
                break;
            }
            let line = self.grid.remove(self.scroll_top);
            let wrapped = if self.scroll_top < self.grid_wrapped.len() {
                self.grid_wrapped.remove(self.scroll_top)
            } else {
                false
            };

            // Solo guardar en scrollback si es pantalla principal y región completa
            if full_screen && self.alt_screen.is_none() {
                self.scrollback.push(line);
                self.scrollback_wrapped.push(wrapped);
                if self.scrollback.len() > self.max_scrollback {
                    self.scrollback.remove(0);
                    self.scrollback_wrapped.remove(0);
                    self.lines_scrolled_off += 1;
                    let off = self.lines_scrolled_off;
                    self.prompt_marks.retain(|&m| m >= off);
                }
            }

            self.grid.insert(self.scroll_bottom, self.blank_line());
            self.grid_wrapped
                .insert(self.scroll_bottom.min(self.grid_wrapped.len()), false);
        }
    }

    /// Scroll hacia abajo n líneas dentro de la región de scroll
    pub fn scroll_down(&mut self, n: usize) {
        self.mark_dirty();
        self.invalidate_all_lines();

        for _ in 0..n {
            if self.scroll_top > self.scroll_bottom || self.scroll_bottom >= self.grid.len() {
                break;
            }
            self.grid.remove(self.scroll_bottom);
            self.grid.insert(self.scroll_top, self.blank_line());
            if self.scroll_bottom < self.grid_wrapped.len() {
                self.grid_wrapped.remove(self.scroll_bottom);
                self.grid_wrapped.insert(self.scroll_top, false);
            }
        }
    }

    /// Define la región de scroll (DECSTBM). Filas en base 0, inclusive.
    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        if top < bottom && bottom < self.rows {
            self.scroll_top = top;
            self.scroll_bottom = bottom;
        } else {
            self.scroll_top = 0;
            self.scroll_bottom = self.rows.saturating_sub(1);
        }
        self.cursor.row = 0;
        self.cursor.col = 0;
        self.mark_dirty();
    }

    /// Inserta n líneas en blanco en la posición del cursor (IL)
    pub fn insert_lines(&mut self, n: usize) {
        if self.cursor.row < self.scroll_top || self.cursor.row > self.scroll_bottom {
            return;
        }
        self.mark_dirty();
        self.invalidate_all_lines();
        for _ in 0..n.min(self.scroll_bottom - self.cursor.row + 1) {
            self.grid.remove(self.scroll_bottom);
            self.grid.insert(self.cursor.row, self.blank_line());
            if self.scroll_bottom < self.grid_wrapped.len() {
                self.grid_wrapped.remove(self.scroll_bottom);
                self.grid_wrapped.insert(self.cursor.row, false);
            }
        }
    }

    /// Elimina n líneas en la posición del cursor (DL)
    pub fn delete_lines(&mut self, n: usize) {
        if self.cursor.row < self.scroll_top || self.cursor.row > self.scroll_bottom {
            return;
        }
        self.mark_dirty();
        self.invalidate_all_lines();
        for _ in 0..n.min(self.scroll_bottom - self.cursor.row + 1) {
            self.grid.remove(self.cursor.row);
            self.grid.insert(self.scroll_bottom, self.blank_line());
            if self.cursor.row < self.grid_wrapped.len() {
                self.grid_wrapped.remove(self.cursor.row);
                self.grid_wrapped
                    .insert(self.scroll_bottom.min(self.grid_wrapped.len()), false);
            }
        }
    }

    /// Inserta n celdas en blanco en el cursor, desplazando el resto a la derecha (ICH)
    pub fn insert_chars(&mut self, n: usize) {
        if self.cursor.row >= self.rows {
            return;
        }
        self.mark_dirty();
        self.invalidate_line(self.cursor.row);
        let row = &mut self.grid[self.cursor.row];
        for _ in 0..n.min(self.cols - self.cursor.col) {
            row.pop();
            row.insert(self.cursor.col, Cell::empty());
        }
    }

    /// Elimina n celdas en el cursor, desplazando el resto a la izquierda (DCH)
    pub fn delete_chars(&mut self, n: usize) {
        if self.cursor.row >= self.rows {
            return;
        }
        self.mark_dirty();
        self.invalidate_line(self.cursor.row);
        let blank = self.blank_cell();
        let row = &mut self.grid[self.cursor.row];
        for _ in 0..n.min(self.cols - self.cursor.col) {
            row.remove(self.cursor.col);
            row.push(blank.clone());
        }
    }

    /// Borra n celdas desde el cursor sin desplazar (ECH)
    pub fn erase_chars(&mut self, n: usize) {
        if self.cursor.row >= self.rows {
            return;
        }
        self.mark_dirty();
        self.invalidate_line(self.cursor.row);
        let blank = self.blank_cell();
        let end = (self.cursor.col + n).min(self.cols);
        for col in self.cursor.col..end {
            self.grid[self.cursor.row][col] = blank.clone();
        }
    }

    /// Borrado en pantalla (ED). Modos: 0 = cursor→final, 1 = inicio→cursor, 2 = todo, 3 = todo + scrollback
    pub fn erase_in_display(&mut self, mode: u16) {
        self.mark_dirty();
        self.invalidate_all_lines();
        match mode {
            0 => {
                self.clear_line_right();
                let blank = self.blank_cell();
                for row in (self.cursor.row + 1)..self.rows {
                    for col in 0..self.cols {
                        self.grid[row][col] = blank.clone();
                    }
                }
            }
            1 => {
                let blank = self.blank_cell();
                for row in 0..self.cursor.row {
                    for col in 0..self.cols {
                        self.grid[row][col] = blank.clone();
                    }
                }
                let end = (self.cursor.col + 1).min(self.cols);
                for col in 0..end {
                    self.grid[self.cursor.row][col] = blank.clone();
                }
            }
            2 => self.clear(),
            3 => {
                self.clear();
                self.scrollback.clear();
                self.scrollback_wrapped.clear();
                self.view_offset = 0;
                self.prompt_marks.clear();
            }
            _ => {}
        }
    }

    /// Borrado en línea (EL). Modos: 0 = cursor→final, 1 = inicio→cursor, 2 = línea completa
    pub fn erase_in_line(&mut self, mode: u16) {
        match mode {
            0 => self.clear_line_right(),
            1 => {
                self.mark_dirty();
                self.invalidate_line(self.cursor.row);
                if self.cursor.row < self.rows {
                    let blank = self.blank_cell();
                    let end = (self.cursor.col + 1).min(self.cols);
                    for col in 0..end {
                        self.grid[self.cursor.row][col] = blank.clone();
                    }
                }
            }
            2 => self.clear_line(),
            _ => {}
        }
    }

    /// Limpia la pantalla
    pub fn clear(&mut self) {
        self.mark_dirty();
        self.invalidate_all_lines();
        let blank = self.blank_cell();
        for row in &mut self.grid {
            for cell in row {
                *cell = blank.clone();
            }
        }
        for flag in &mut self.grid_wrapped {
            *flag = false;
        }
    }

    /// Limpia desde el cursor hasta el final de la línea
    pub fn clear_line_right(&mut self) {
        self.mark_dirty();
        self.invalidate_line(self.cursor.row);
        if self.cursor.row < self.rows {
            let blank = self.blank_cell();
            for col in self.cursor.col..self.cols {
                self.grid[self.cursor.row][col] = blank.clone();
            }
        }
    }

    /// Limpia toda la línea actual
    pub fn clear_line(&mut self) {
        self.mark_dirty();
        self.invalidate_line(self.cursor.row);
        if self.cursor.row < self.rows {
            let blank = self.blank_cell();
            for col in 0..self.cols {
                self.grid[self.cursor.row][col] = blank.clone();
            }
        }
    }

    /// Limpia desde la línea actual hasta el final de la pantalla
    pub fn clear_to_end_of_screen(&mut self) {
        self.erase_in_display(0);
    }

    /// Guarda posición del cursor y atributos (DECSC)
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(SavedCursor {
            row: self.cursor.row,
            col: self.cursor.col,
            attrs: self.current_attrs,
        });
    }

    /// Restaura posición del cursor y atributos (DECRC)
    pub fn restore_cursor(&mut self) {
        if let Some(saved) = self.saved_cursor {
            self.cursor.row = saved.row.min(self.rows.saturating_sub(1));
            self.cursor.col = saved.col.min(self.cols.saturating_sub(1));
            self.current_attrs = saved.attrs;
            self.mark_dirty();
        }
    }

    /// Activa la pantalla alternativa (usada por vim, less, htop, etc.)
    pub fn enter_alt_screen(&mut self) {
        if self.alt_screen.is_some() {
            return;
        }
        self.mark_dirty();
        self.invalidate_all_lines();
        self.reset_view();
        let main_grid = std::mem::replace(
            &mut self.grid,
            vec![vec![Cell::empty(); self.cols]; self.rows],
        );
        let main_wrapped = std::mem::replace(&mut self.grid_wrapped, vec![false; self.rows]);
        self.alt_screen = Some(MainScreenState {
            grid: main_grid,
            grid_wrapped: main_wrapped,
            cursor: self.cursor,
            attrs: self.current_attrs,
        });
        self.cursor.row = 0;
        self.cursor.col = 0;
        self.scroll_top = 0;
        self.scroll_bottom = self.rows.saturating_sub(1);
    }

    /// Vuelve a la pantalla principal, restaurando su contenido
    pub fn exit_alt_screen(&mut self) {
        if let Some(mut state) = self.alt_screen.take() {
            self.mark_dirty();
            self.invalidate_all_lines();

            // Ajustar el grid guardado a las dimensiones actuales (por si hubo resize)
            state.grid.resize(self.rows, vec![Cell::empty(); self.cols]);
            for row in &mut state.grid {
                row.resize(self.cols, Cell::empty());
            }
            state.grid_wrapped.resize(self.rows, false);

            self.grid = state.grid;
            self.grid_wrapped = state.grid_wrapped;
            self.cursor.row = state.cursor.row.min(self.rows.saturating_sub(1));
            self.cursor.col = state.cursor.col.min(self.cols.saturating_sub(1));
            self.cursor.visible = true;
            self.current_attrs = state.attrs;
            self.scroll_top = 0;
            self.scroll_bottom = self.rows.saturating_sub(1);
            self.mouse_mode = MouseMode::None;
        }
    }

    /// Indica si la pantalla alternativa está activa
    pub fn is_alt_screen(&self) -> bool {
        self.alt_screen.is_some()
    }

    /// Reset completo del terminal (RIS)
    pub fn reset(&mut self) {
        self.exit_alt_screen();
        self.cursor = Cursor::new();
        self.current_attrs = CellAttributes::default();
        self.scroll_top = 0;
        self.scroll_bottom = self.rows.saturating_sub(1);
        self.saved_cursor = None;
        self.bracketed_paste = false;
        self.autowrap = true;
        self.mouse_mode = MouseMode::None;
        self.mouse_sgr = false;
        self.last_char = None;
        self.current_hyperlink = None;
        self.view_offset = 0;
        self.clear();
    }

    /// Cambia la visibilidad del cursor (DECTCEM)
    pub fn set_cursor_visible(&mut self, visible: bool) {
        if self.cursor.visible != visible {
            self.cursor.visible = visible;
            self.mark_dirty();
        }
    }

    /// Define el título de ventana solicitado (OSC 0/2)
    pub fn set_title(&mut self, title: String) {
        self.pending_title = Some(title);
    }

    /// Consume el título pendiente, si lo hay (lo lee la app)
    pub fn take_title(&mut self) -> Option<String> {
        self.pending_title.take()
    }

    /// Obtiene el contenido visible
    pub fn get_visible(&self) -> &Vec<Vec<Cell>> {
        &self.grid
    }

    /// Actualiza el contexto de una línea analizando su contenido
    pub fn update_line_context(&mut self, row: usize) {
        if row >= self.rows {
            return;
        }

        let line_text: String = self.grid[row].iter().map(|c| c.character).collect();

        self.line_contexts[row] = analyze_line_context(&line_text);
    }

    /// Obtiene el contexto de una línea
    pub fn get_line_context(&self, row: usize) -> LineContext {
        self.line_contexts
            .get(row)
            .copied()
            .unwrap_or(LineContext::Normal)
    }

    // ==================== Resize y reflow ====================

    /// Redimensiona la pantalla. Si cambia el ancho y estamos en la pantalla
    /// principal, re-envuelve las líneas lógicas al nuevo ancho (reflow).
    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        if new_rows == 0 || new_cols == 0 || (new_rows == self.rows && new_cols == self.cols) {
            return;
        }
        self.mark_dirty();

        if new_cols != self.cols && self.alt_screen.is_none() {
            self.reflow(new_rows, new_cols);
        } else {
            self.resize_simple(new_rows, new_cols);
        }

        self.rows = new_rows;
        self.cols = new_cols;

        // La región de scroll vuelve a ocupar toda la pantalla
        self.scroll_top = 0;
        self.scroll_bottom = new_rows.saturating_sub(1);

        // Ajustar contextos de línea y caches
        self.line_contexts.resize(new_rows, LineContext::Normal);
        self.content_type_cache.resize(new_rows, None);
        self.invalidate_all_lines();

        // Limitar cursor y vista
        if self.cursor.row >= new_rows {
            self.cursor.row = new_rows.saturating_sub(1);
        }
        if self.cursor.col >= new_cols {
            self.cursor.col = new_cols.saturating_sub(1);
        }
        self.view_offset = self.view_offset.min(self.scrollback.len());
    }

    /// Resize sin reflow (alt screen o cambio solo de filas)
    fn resize_simple(&mut self, new_rows: usize, new_cols: usize) {
        while self.grid.len() > new_rows {
            let line = self.grid.remove(0);
            let wrapped = if !self.grid_wrapped.is_empty() {
                self.grid_wrapped.remove(0)
            } else {
                false
            };
            if self.alt_screen.is_none() {
                self.scrollback.push(line);
                self.scrollback_wrapped.push(wrapped);
            }
        }

        while self.grid.len() < new_rows {
            self.grid.push(vec![Cell::empty(); new_cols]);
            self.grid_wrapped.push(false);
        }
        self.grid_wrapped.resize(new_rows, false);

        for row in &mut self.grid {
            row.resize(new_cols, Cell::empty());
        }
    }

    /// Re-envuelve todo el contenido (scrollback + pantalla) al nuevo ancho
    fn reflow(&mut self, new_rows: usize, new_cols: usize) {
        // 1. Posición absoluta del cursor dentro del stream combinado
        let cursor_stream_row = self.scrollback.len() + self.cursor.row;
        let cursor_col = self.cursor.col;

        // 2. Combinar scrollback + grid con sus flags
        let mut rows: Vec<Vec<Cell>> = std::mem::take(&mut self.scrollback);
        let mut wrapped: Vec<bool> = std::mem::take(&mut self.scrollback_wrapped);
        wrapped.resize(rows.len(), false);
        rows.append(&mut self.grid);
        let mut grid_wrapped = std::mem::take(&mut self.grid_wrapped);
        grid_wrapped.resize(rows.len() - wrapped.len(), false);
        wrapped.append(&mut grid_wrapped);

        // 3. Construir líneas lógicas, registrando la posición lógica del cursor
        let old_cols = self.cols;
        let mut logical: Vec<Vec<Cell>> = Vec::new();
        let mut cursor_logical: Option<(usize, usize)> = None; // (línea lógica, offset)
        let mut current: Vec<Cell> = Vec::new();
        let mut rows_in_current = 0usize;

        for (i, row) in rows.into_iter().enumerate() {
            if i == cursor_stream_row {
                cursor_logical = Some((logical.len(), rows_in_current * old_cols + cursor_col));
            }
            let is_wrapped = wrapped.get(i).copied().unwrap_or(false);
            current.extend(row);
            rows_in_current += 1;
            if !is_wrapped {
                // Corte duro: recortar blancos finales y cerrar la línea lógica
                while current
                    .last()
                    .map(|c| c.character == ' ' && c.hyperlink.is_none())
                    == Some(true)
                {
                    current.pop();
                }
                logical.push(std::mem::take(&mut current));
                rows_in_current = 0;
            }
        }
        if !current.is_empty() || rows_in_current > 0 {
            logical.push(current);
        }

        // 4. Re-envolver cada línea lógica al nuevo ancho
        let mut new_lines: Vec<Vec<Cell>> = Vec::new();
        let mut new_wrapped: Vec<bool> = Vec::new();
        let mut new_cursor_stream: Option<(usize, usize)> = None;

        for (li, line) in logical.into_iter().enumerate() {
            let first_new_row = new_lines.len();
            if line.is_empty() {
                new_lines.push(vec![Cell::empty(); new_cols]);
                new_wrapped.push(false);
            } else {
                let mut chunks = line.chunks(new_cols).peekable();
                while let Some(chunk) = chunks.next() {
                    let mut row: Vec<Cell> = chunk.to_vec();
                    row.resize(new_cols, Cell::empty());
                    new_lines.push(row);
                    new_wrapped.push(chunks.peek().is_some());
                }
            }
            if let Some((cl, offset)) = cursor_logical {
                if cl == li {
                    let r = first_new_row + offset / new_cols;
                    let c = offset % new_cols;
                    new_cursor_stream = Some((r.min(new_lines.len().saturating_sub(1)), c));
                }
            }
        }

        // Las filas en blanco al final (debajo del cursor) no son contenido:
        // recortarlas para que no empujen líneas reales al scrollback
        let keep_min = new_cursor_stream.map(|(r, _)| r + 1).unwrap_or(1);
        while new_lines.len() > keep_min
            && new_lines.last().is_some_and(|row| {
                row.iter()
                    .all(|c| c.character == ' ' && c.hyperlink.is_none())
            })
        {
            new_lines.pop();
            new_wrapped.pop();
        }

        // 5. Distribuir: las últimas new_rows van a pantalla, el resto a scrollback
        let total = new_lines.len();
        let grid_start = total.saturating_sub(new_rows);

        // El cursor debe quedar en pantalla: si cae antes, anclar el grid ahí
        let grid_start = match new_cursor_stream {
            Some((r, _)) if r < grid_start => r,
            _ => grid_start,
        };

        self.scrollback = new_lines[..grid_start].to_vec();
        self.scrollback_wrapped = new_wrapped[..grid_start].to_vec();
        self.grid = new_lines[grid_start..].to_vec();
        self.grid_wrapped = new_wrapped[grid_start..].to_vec();

        while self.grid.len() < new_rows {
            self.grid.push(vec![Cell::empty(); new_cols]);
            self.grid_wrapped.push(false);
        }
        // Si el grid quedó más grande que new_rows (cursor anclado arriba), recortar abajo
        while self.grid.len() > new_rows {
            self.grid.pop();
            self.grid_wrapped.pop();
        }

        // Limitar scrollback
        while self.scrollback.len() > self.max_scrollback {
            self.scrollback.remove(0);
            self.scrollback_wrapped.remove(0);
            self.lines_scrolled_off += 1;
        }

        // 6. Recolocar el cursor
        match new_cursor_stream {
            Some((r, c)) => {
                self.cursor.row = r.saturating_sub(grid_start).min(new_rows.saturating_sub(1));
                self.cursor.col = c.min(new_cols.saturating_sub(1));
            }
            None => {
                self.cursor.row = self.cursor.row.min(new_rows.saturating_sub(1));
                self.cursor.col = self.cursor.col.min(new_cols.saturating_sub(1));
            }
        }

        // Las marcas de prompt dejan de ser válidas tras un reflow
        self.prompt_marks.clear();
        self.view_offset = 0;
    }

    // ==================== Sugerencias ====================

    /// Activa el modo de sugerencia (texto aparecerá en gris)
    pub fn start_suggestion(&mut self) {
        self.suggestion_mode = true;
        self.suggestion_start_col = self.cursor.col;
    }

    /// Desactiva el modo de sugerencia
    pub fn end_suggestion(&mut self) {
        self.suggestion_mode = false;
    }

    /// Limpia las sugerencias de la línea actual desde donde empezaron
    pub fn clear_suggestions(&mut self) {
        if self.cursor.row < self.rows {
            for col in self.suggestion_start_col..self.cols {
                if self.grid[self.cursor.row][col].is_suggestion {
                    self.grid[self.cursor.row][col] = Cell::empty();
                } else {
                    break;
                }
            }
        }
        self.suggestion_mode = false;
    }

    /// Verifica si hay sugerencias activas en la línea del cursor
    pub fn has_suggestions(&self) -> bool {
        if self.cursor.row < self.rows {
            self.grid[self.cursor.row]
                .iter()
                .any(|cell| cell.is_suggestion)
        } else {
            false
        }
    }

    /// Actualiza la sugerencia automática basada en el historial
    fn update_auto_suggestion(&mut self) {
        if self.active_suggestion.is_some() {
            self.clear_auto_suggestion();
        }

        // No mostrar sugerencias en la pantalla alternativa (vim, htop, etc.)
        if self.alt_screen.is_some() {
            return;
        }

        if let Some(suggestion) = self.command_history.find_suggestion(&self.current_command) {
            self.active_suggestion = Some(suggestion.clone());

            let start_col = self.command_start_col + self.current_command.len();
            for (i, ch) in suggestion.chars().enumerate() {
                let col = start_col + i;
                if col < self.cols {
                    let mut cell = Cell::as_suggestion(ch);
                    cell.attrs = self.current_attrs;
                    self.grid[self.cursor.row][col] = cell;
                } else {
                    break;
                }
            }
            self.mark_dirty();
        }
    }

    /// Limpia la sugerencia automática actual
    fn clear_auto_suggestion(&mut self) {
        if let Some(suggestion) = &self.active_suggestion {
            let start_col = self.command_start_col + self.current_command.len();
            for i in 0..suggestion.len() {
                let col = start_col + i;
                if col < self.cols && self.grid[self.cursor.row][col].is_suggestion {
                    self.grid[self.cursor.row][col] = Cell::empty();
                }
            }
            self.active_suggestion = None;
            self.mark_dirty();
        }
    }

    /// Acepta la sugerencia actual (llamar cuando el usuario presiona Tab o →)
    pub fn accept_suggestion(&mut self) {
        if let Some(suggestion) = &self.active_suggestion {
            self.current_command.push_str(suggestion);

            let start_col = self.cursor.col;
            for (i, ch) in suggestion.chars().enumerate() {
                let col = start_col + i;
                if col < self.cols {
                    let mut cell = Cell::with_attrs(ch, self.current_attrs);
                    cell.is_suggestion = false;
                    self.grid[self.cursor.row][col] = cell;
                }
            }

            self.cursor.col += suggestion.len();
            self.active_suggestion = None;
            self.mark_dirty();
        }
    }

    /// Acepta la sugerencia sin renderizar (el PTY hará el eco)
    pub fn accept_suggestion_for_pty(&mut self) {
        if self.active_suggestion.is_some() {
            let suggestion = self.active_suggestion.clone().unwrap_or_default();
            self.clear_auto_suggestion();
            self.current_command.push_str(&suggestion);
        }
    }

    /// Obtiene la sugerencia actual activa
    pub fn get_active_suggestion(&self) -> Option<&str> {
        self.active_suggestion.as_deref()
    }

    /// Agrega un carácter al comando actual (desde input del usuario)
    pub fn add_user_input(&mut self, ch: char) {
        if !ch.is_control() {
            if self.current_command.is_empty() {
                self.command_start_col = self.cursor.col;
            }
            self.current_command.push(ch);
            self.update_auto_suggestion();
        }
    }

    /// Elimina el último carácter del comando actual (backspace desde usuario)
    pub fn remove_user_input(&mut self) {
        if !self.current_command.is_empty() {
            self.current_command.pop();
            self.update_auto_suggestion();
        }
    }

    /// Resetea el comando actual (Enter desde usuario)
    pub fn reset_user_input(&mut self) {
        if self.active_suggestion.is_some() {
            self.clear_auto_suggestion();
        }

        if !self.current_command.is_empty() {
            self.command_history
                .add_command(self.current_command.clone());
            self.current_command.clear();
        }
        self.command_start_col = 0;
        self.active_suggestion = None;
    }

    /// Mueve el cursor a una posición específica
    pub fn move_cursor_to(&mut self, row: usize, col: usize) {
        self.cursor.row = row.min(self.rows.saturating_sub(1));
        self.cursor.col = col.min(self.cols.saturating_sub(1));
        self.mark_dirty();
    }

    // ==================== Selección ====================

    /// Inicia una selección en la posición especificada
    pub fn start_selection(&mut self, row: usize, col: usize) {
        let row = row.min(self.rows.saturating_sub(1));
        let col = col.min(self.cols.saturating_sub(1));
        self.selection = Some(Selection {
            start: (row, col),
            end: (row, col),
        });
        self.mark_dirty();
    }

    /// Actualiza el final de la selección
    pub fn update_selection(&mut self, row: usize, col: usize) {
        if let Some(selection) = &mut self.selection {
            let row = row.min(self.rows.saturating_sub(1));
            let col = col.min(self.cols.saturating_sub(1));
            selection.end = (row, col);
            self.mark_dirty();
        }
    }

    /// Selecciona la palabra bajo la posición dada (doble click)
    pub fn select_word(&mut self, row: usize, col: usize) {
        let line = self.display_line(row);
        if col >= line.len() || !is_word_char(line[col].character) {
            return;
        }
        let mut start = col;
        while start > 0 && is_word_char(line[start - 1].character) {
            start -= 1;
        }
        let mut end = col;
        while end + 1 < line.len() && is_word_char(line[end + 1].character) {
            end += 1;
        }
        self.selection = Some(Selection {
            start: (row, start),
            end: (row, end),
        });
        self.mark_dirty();
    }

    /// Selecciona la línea completa (triple click)
    pub fn select_line(&mut self, row: usize) {
        self.selection = Some(Selection {
            start: (row, 0),
            end: (row, self.cols.saturating_sub(1)),
        });
        self.mark_dirty();
    }

    /// Limpia la selección actual
    pub fn clear_selection(&mut self) {
        if self.selection.is_some() {
            self.selection = None;
            self.mark_dirty();
        }
    }

    /// Obtiene la selección actual
    pub fn get_selection(&self) -> Option<Selection> {
        self.selection
    }

    /// Verifica si una celda está dentro de la selección
    pub fn is_selected(&self, row: usize, col: usize) -> bool {
        if let Some(selection) = self.selection {
            let (start_row, start_col) = selection.start;
            let (end_row, end_col) = selection.end;

            let (start_row, start_col, end_row, end_col) =
                if (start_row, start_col) <= (end_row, end_col) {
                    (start_row, start_col, end_row, end_col)
                } else {
                    (end_row, end_col, start_row, start_col)
                };

            if row < start_row || row > end_row {
                return false;
            }

            if row == start_row && row == end_row {
                col >= start_col && col <= end_col
            } else if row == start_row {
                col >= start_col
            } else if row == end_row {
                col <= end_col
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Obtiene el texto seleccionado (sobre las líneas visibles)
    pub fn get_selected_text(&self) -> Option<String> {
        let selection = self.selection?;
        let (start_row, start_col) = selection.start;
        let (end_row, end_col) = selection.end;

        let (start_row, start_col, end_row, end_col) =
            if (start_row, start_col) <= (end_row, end_col) {
                (start_row, start_col, end_row, end_col)
            } else {
                (end_row, end_col, start_row, start_col)
            };

        let mut text = String::new();

        for row in start_row..=end_row {
            if row >= self.rows {
                break;
            }
            let line = self.display_line(row);
            if line.is_empty() {
                continue;
            }

            let start_col_in_row = if row == start_row { start_col } else { 0 };
            let end_col_in_row = if row == end_row {
                end_col
            } else {
                line.len().saturating_sub(1)
            };

            let last = end_col_in_row.min(line.len().saturating_sub(1));
            for (col, cell) in line
                .iter()
                .enumerate()
                .take(last + 1)
                .skip(start_col_in_row)
            {
                if cell.character != '\0' && cell.character != ' ' {
                    text.push(cell.character);
                } else if col < end_col_in_row {
                    text.push(' ');
                }
            }

            if row < end_row {
                text.push('\n');
            }
        }

        let trimmed = text.trim_end().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }
}
