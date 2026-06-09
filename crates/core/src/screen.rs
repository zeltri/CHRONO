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

/// Buffer de pantalla de terminal
pub struct Screen {
    /// Dimensiones
    pub rows: usize,
    pub cols: usize,

    /// Grid principal
    pub grid: Vec<Vec<Cell>>,

    /// Scrollback buffer (líneas pasadas)
    scrollback: Vec<Vec<Cell>>,

    /// Cursor
    pub cursor: Cursor,

    /// Atributos actuales (para próximas escrituras)
    pub current_attrs: CellAttributes,

    /// Máximo de líneas en scrollback
    max_scrollback: usize,

    /// Contexto semántico por línea
    pub line_contexts: Vec<LineContext>,

    /// Cache de ContentType detectado por línea (para evitar re-análisis costosos)
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

    /// Título de ventana pendiente (OSC 0/2), consumido por la app
    pending_title: Option<String>,

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

impl Screen {
    pub fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![vec![Cell::empty(); cols]; rows];

        Self {
            rows,
            cols,
            grid,
            scrollback: Vec::new(),
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
            pending_title: None,
            suggestion_mode: false,
            suggestion_start_col: 0,
            command_history: CommandHistory::from_shell_history(1000),
            current_command: String::new(),
            command_start_col: 0,
            active_suggestion: None,
            dirty: true, // Inicialmente marcado como dirty
            selection: None,
        }
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
                self.cursor.col = 0;
                self.index();
            } else {
                self.cursor.col = self.cols - 1;
            }
        }

        if self.cursor.row < self.rows && self.cursor.col < self.cols {
            // Si no estamos en modo sugerencia pero hay sugerencias visibles,
            // limpiarlas antes de escribir texto normal
            if !self.suggestion_mode && self.has_suggestions() {
                self.clear_suggestions();
            }

            let mut cell = Cell::with_attrs(ch, self.current_attrs);
            let width = (cell.width as usize).max(1);

            // Si estamos en modo sugerencia, marcar la celda
            if self.suggestion_mode {
                cell.is_suggestion = true;
            }

            // Escribir la celda
            self.grid[self.cursor.row][self.cursor.col] = cell;

            // Si el carácter es ancho, marcar las celdas siguientes como continuación
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
        // Limpiar sugerencia activa
        self.active_suggestion = None;
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

            // Solo guardar en scrollback si es pantalla principal y región completa
            if full_screen && self.alt_screen.is_none() {
                self.scrollback.push(line);
                if self.scrollback.len() > self.max_scrollback {
                    self.scrollback.remove(0);
                }
            }

            self.grid.insert(self.scroll_bottom, self.blank_line());
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
        // Per spec, el cursor vuelve al origen
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
        let main_grid = std::mem::replace(&mut self.grid, vec![vec![Cell::empty(); self.cols]; self.rows]);
        self.alt_screen = Some(MainScreenState {
            grid: main_grid,
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

            self.grid = state.grid;
            self.cursor.row = state.cursor.row.min(self.rows.saturating_sub(1));
            self.cursor.col = state.cursor.col.min(self.cols.saturating_sub(1));
            self.cursor.visible = true;
            self.current_attrs = state.attrs;
            self.scroll_top = 0;
            self.scroll_bottom = self.rows.saturating_sub(1);
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
        self.last_char = None;
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

    /// Redimensiona la pantalla
    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        self.mark_dirty();
        // Si hay menos filas, mover las eliminadas al scrollback
        while self.grid.len() > new_rows {
            let line = self.grid.remove(0);
            if self.alt_screen.is_none() {
                self.scrollback.push(line);
            }
        }

        // Si hay más filas, agregar vacías
        while self.grid.len() < new_rows {
            self.grid.push(vec![Cell::empty(); new_cols]);
        }

        // Ajustar columnas
        for row in &mut self.grid {
            row.resize(new_cols, Cell::empty());
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

        // Limitar cursor
        if self.cursor.row >= new_rows {
            self.cursor.row = new_rows.saturating_sub(1);
        }
        if self.cursor.col >= new_cols {
            self.cursor.col = new_cols.saturating_sub(1);
        }
    }

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
                    // Si encontramos una celda que no es sugerencia, detenernos
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
        log::trace!(
            "update_auto_suggestion, current_command: '{}'",
            self.current_command
        );

        // Limpiar sugerencia anterior
        if self.active_suggestion.is_some() {
            self.clear_auto_suggestion();
        }

        // No mostrar sugerencias en la pantalla alternativa (vim, htop, etc.)
        if self.alt_screen.is_some() {
            return;
        }

        // Buscar sugerencia en historial
        if let Some(suggestion) = self.command_history.find_suggestion(&self.current_command) {
            log::trace!(
                "Mostrando sugerencia: '{}' después de col {} + len {}",
                suggestion,
                self.command_start_col,
                self.current_command.len()
            );

            // Guardar sugerencia activa
            self.active_suggestion = Some(suggestion.clone());

            // Mostrar sugerencia después del comando actual
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
            // Agregar la sugerencia al comando actual
            self.current_command.push_str(suggestion);

            // Convertir las celdas de sugerencia a texto normal
            let start_col = self.cursor.col;
            for (i, ch) in suggestion.chars().enumerate() {
                let col = start_col + i;
                if col < self.cols {
                    let mut cell = Cell::with_attrs(ch, self.current_attrs);
                    cell.is_suggestion = false;
                    self.grid[self.cursor.row][col] = cell;
                }
            }

            // Mover cursor al final de la sugerencia
            self.cursor.col += suggestion.len();

            // Limpiar sugerencia activa
            self.active_suggestion = None;
            self.mark_dirty();
        }
    }

    /// Acepta la sugerencia sin renderizar (el PTY hará el eco)
    /// Solo actualiza el current_command y limpia la sugerencia visual
    pub fn accept_suggestion_for_pty(&mut self) {
        if self.active_suggestion.is_some() {
            // Agregar la sugerencia al comando actual
            let suggestion = self.active_suggestion.clone().unwrap_or_default();
            // Limpiar la sugerencia visual (el eco del PTY la escribirá)
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
        // Aceptar cualquier carácter que no sea de control (incluyendo UTF-8)
        if !ch.is_control() {
            // Si es el primer carácter del comando, guardar posición inicial
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
        // Limpiar sugerencia visual antes de resetear
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

            // Normalizar la selección para que start sea siempre antes que end
            let (start_row, start_col, end_row, end_col) =
                if (start_row, start_col) <= (end_row, end_col) {
                    (start_row, start_col, end_row, end_col)
                } else {
                    (end_row, end_col, start_row, start_col)
                };

            // Verificar si la posición está dentro del rango
            if row < start_row || row > end_row {
                return false;
            }

            if row == start_row && row == end_row {
                // Misma fila
                col >= start_col && col <= end_col
            } else if row == start_row {
                // Primera fila de la selección
                col >= start_col
            } else if row == end_row {
                // Última fila de la selección
                col <= end_col
            } else {
                // Filas intermedias
                true
            }
        } else {
            false
        }
    }

    /// Obtiene el texto seleccionado
    pub fn get_selected_text(&self) -> Option<String> {
        let selection = self.selection?;
        let (start_row, start_col) = selection.start;
        let (end_row, end_col) = selection.end;

        // Normalizar la selección
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

            let start_col_in_row = if row == start_row { start_col } else { 0 };
            let end_col_in_row = if row == end_row {
                end_col
            } else {
                self.cols - 1
            };

            for col in start_col_in_row..=end_col_in_row.min(self.cols - 1) {
                let cell = &self.grid[row][col];
                if cell.character != '\0' && cell.character != ' ' {
                    text.push(cell.character);
                } else if col < end_col_in_row {
                    // Mantener espacios intermedios, pero no al final de la línea
                    text.push(' ');
                }
            }

            // Agregar salto de línea si no es la última fila
            if row < end_row {
                text.push('\n');
            }
        }

        // Limpiar espacios al final
        let trimmed = text.trim_end().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }
}
