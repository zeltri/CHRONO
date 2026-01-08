use crate::{
    attributes::CellAttributes,
    cell::Cell,
    context::{analyze_line_context, LineContext},
    cursor::Cursor,
    history::CommandHistory,
};

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

    /// Marca si la línea necesita limpieza después de carriage return
    line_needs_clear: Vec<bool>,

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
            line_needs_clear: vec![false; rows],
            suggestion_mode: false,
            suggestion_start_col: 0,
            command_history: CommandHistory::from_shell_history(1000),
            current_command: String::new(),
            command_start_col: 0,
            active_suggestion: None,
        }
    }

    /// Escribe un carácter en la posición del cursor
    pub fn write_char(&mut self, ch: char) {
        if self.cursor.col >= self.cols {
            self.cursor.carriage_return();
            self.cursor.line_feed(self.rows - 1);
            if self.cursor.row >= self.rows {
                self.scroll_up(1);
                self.cursor.row = self.rows - 1;
            }
        }

        if self.cursor.row < self.rows && self.cursor.col < self.cols {
            // Si no estamos en modo sugerencia pero hay sugerencias visibles,
            // limpiarlas antes de escribir texto normal
            if !self.suggestion_mode && self.has_suggestions() {
                self.clear_suggestions();
            }

            // Si la línea está marcada para limpieza y estamos al principio, limpiarla
            if self.line_needs_clear[self.cursor.row] && self.cursor.col == 0 {
                // Limpiar esta línea y todas las siguientes que estén marcadas
                self.clear_line();
                self.line_needs_clear[self.cursor.row] = false;

                // Limpiar líneas consecutivas marcadas
                for row in (self.cursor.row + 1)..self.rows {
                    if self.line_needs_clear[row] {
                        for col in 0..self.cols {
                            self.grid[row][col] = Cell::empty();
                        }
                        self.line_needs_clear[row] = false;
                    } else {
                        break; // Dejar de limpiar si encontramos una línea no marcada
                    }
                }
            }

            let mut cell = Cell::with_attrs(ch, self.current_attrs);
            let width = cell.width as usize;

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

    /// Line feed - avanza una línea
    pub fn line_feed(&mut self) {
        // Limpiar sugerencia activa
        self.active_suggestion = None;

        // NO analizar contexto de la línea actual aquí
        // porque los comandos del usuario NO deben ser detectados como stack traces
        // El contexto se actualizará cuando el output del comando se escriba

        if self.cursor.row < self.rows - 1 {
            self.cursor.row += 1;
        } else {
            self.scroll_up(1);
        }

        // Restablecer atributos después del line feed para evitar que
        // los colores del output anterior afecten el nuevo prompt
        self.current_attrs = CellAttributes::default();
    }

    /// Carriage return - vuelve al inicio de la línea
    pub fn carriage_return(&mut self) {
        // Resetear comando actual
        self.current_command.clear();
        self.active_suggestion = None;

        // Restablecer atributos de color para evitar que los colores del prompt
        // persistan en el texto del usuario
        self.current_attrs = CellAttributes::default();

        // Si no estamos en la última línea y volvemos al inicio,
        // probablemente es porque se va a escribir nuevo contenido que
        // reemplaza al anterior (como después de un tab completion)
        if self.cursor.row < self.rows && self.cursor.col > 0 {
            // Marcar esta línea y todas las siguientes para limpieza
            for row in self.cursor.row..self.rows {
                if row < self.line_needs_clear.len() {
                    self.line_needs_clear[row] = true;
                }
            }
        }
        self.cursor.col = 0;
    }

    /// Scroll hacia arriba n líneas
    pub fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            if !self.grid.is_empty() {
                let line = self.grid.remove(0);
                self.scrollback.push(line);

                // Limitar scrollback
                if self.scrollback.len() > self.max_scrollback {
                    self.scrollback.remove(0);
                }

                // Agregar línea vacía al final
                self.grid.push(vec![Cell::empty(); self.cols]);
            }
        }
    }

    /// Limpia la pantalla
    pub fn clear(&mut self) {
        for row in &mut self.grid {
            for cell in row {
                *cell = Cell::empty();
            }
        }
        // Restablecer los atributos actuales al limpiar la pantalla
        self.current_attrs = CellAttributes::default();
    }

    /// Limpia desde el cursor hasta el final de la línea
    pub fn clear_line_right(&mut self) {
        if self.cursor.row < self.rows {
            for col in self.cursor.col..self.cols {
                self.grid[self.cursor.row][col] = Cell::empty();
            }
        }
    }

    /// Limpia toda la línea actual
    pub fn clear_line(&mut self) {
        if self.cursor.row < self.rows {
            for col in 0..self.cols {
                self.grid[self.cursor.row][col] = Cell::empty();
            }
        }
    }

    /// Limpia desde la línea actual hasta el final de la pantalla
    pub fn clear_to_end_of_screen(&mut self) {
        // Limpiar desde cursor hasta final de la línea actual
        self.clear_line_right();

        // Limpiar todas las líneas siguientes
        for row in (self.cursor.row + 1)..self.rows {
            for col in 0..self.cols {
                self.grid[row][col] = Cell::empty();
            }
            if row < self.line_needs_clear.len() {
                self.line_needs_clear[row] = false;
            }
        }
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
        // Si hay menos filas, mover las eliminadas al scrollback
        while self.grid.len() > new_rows {
            let line = self.grid.remove(0);
            self.scrollback.push(line);
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

        // Ajustar contextos de línea y flags
        self.line_contexts.resize(new_rows, LineContext::Normal);
        self.line_needs_clear.resize(new_rows, false);

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
        eprintln!(
            "[SCREEN] update_auto_suggestion llamado, current_command: '{}'",
            self.current_command
        );

        // Limpiar sugerencia anterior
        if self.active_suggestion.is_some() {
            self.clear_auto_suggestion();
        }

        // Buscar sugerencia en historial
        if let Some(suggestion) = self.command_history.find_suggestion(&self.current_command) {
            eprintln!(
                "[SCREEN] Mostrando sugerencia: '{}' después de col {} + len {}",
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
                    eprintln!(
                        "[SCREEN] Escribiendo '{}' en col {} con is_suggestion=true",
                        ch, col
                    );
                } else {
                    break;
                }
            }
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
        }
    }

    /// Acepta la sugerencia sin renderizar (el PTY hará el eco)
    /// Solo actualiza el current_command y limpia la sugerencia visual
    pub fn accept_suggestion_for_pty(&mut self) {
        if let Some(suggestion) = &self.active_suggestion {
            // Agregar la sugerencia al comando actual
            self.current_command.push_str(suggestion);

            // Limpiar la sugerencia visual (el eco del PTY la escribirá)
            self.clear_auto_suggestion();
        }
    }

    /// Obtiene la sugerencia actual activa
    pub fn get_active_suggestion(&self) -> Option<&str> {
        self.active_suggestion.as_deref()
    }

    /// Agrega un carácter al comando actual (desde input del usuario)
    pub fn add_user_input(&mut self, ch: char) {
        if ch.is_ascii() && !ch.is_control() {
            // Si es el primer carácter del comando, guardar posición inicial
            if self.current_command.is_empty() {
                self.command_start_col = self.cursor.col;
                eprintln!(
                    "[SCREEN] Iniciando comando en col {}",
                    self.command_start_col
                );
            }

            self.current_command.push(ch);
            eprintln!(
                "[SCREEN] add_user_input: '{}' -> current_command: '{}'",
                ch, self.current_command
            );
            self.update_auto_suggestion();
        }
    }

    /// Elimina el último carácter del comando actual (backspace desde usuario)
    pub fn remove_user_input(&mut self) {
        if !self.current_command.is_empty() {
            self.current_command.pop();
            eprintln!(
                "[SCREEN] remove_user_input -> current_command: '{}'",
                self.current_command
            );
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
            eprintln!(
                "[SCREEN] Guardando comando en historial: '{}'",
                self.current_command
            );
            self.command_history
                .add_command(self.current_command.clone());
            self.current_command.clear();
        }
        self.command_start_col = 0;
        self.active_suggestion = None;
    }

    /// Maneja el backspace eliminando el último carácter del comando actual
    pub fn handle_backspace(&mut self) {
        // Este método ahora está vacío, la lógica está en remove_user_input
    }

    /// Mueve el cursor a una posición específica
    pub fn move_cursor_to(&mut self, row: usize, col: usize) {
        self.cursor.row = row.min(self.rows.saturating_sub(1));
        self.cursor.col = col.min(self.cols.saturating_sub(1));
    }
}
