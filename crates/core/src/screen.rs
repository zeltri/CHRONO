use crate::{
    attributes::CellAttributes,
    cell::Cell,
    context::{analyze_line_context, LineContext},
    cursor::Cursor,
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

            let cell = Cell::with_attrs(ch, self.current_attrs);
            let width = cell.width as usize;

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
        // Analizar contexto de la línea actual antes de avanzar
        self.update_line_context(self.cursor.row);

        if self.cursor.row < self.rows - 1 {
            self.cursor.row += 1;
        } else {
            self.scroll_up(1);
        }
    }

    /// Carriage return - vuelve al inicio de la línea
    pub fn carriage_return(&mut self) {
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

        // Ajustar cursor
        self.cursor.row = self.cursor.row.min(new_rows.saturating_sub(1));
        self.cursor.col = self.cursor.col.min(new_cols.saturating_sub(1));
    }

    /// Mueve el cursor a una posición específica
    pub fn move_cursor_to(&mut self, row: usize, col: usize) {
        self.cursor.row = row.min(self.rows.saturating_sub(1));
        self.cursor.col = col.min(self.cols.saturating_sub(1));
    }
}
