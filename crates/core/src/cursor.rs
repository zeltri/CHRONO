/// Posición del cursor en la pantalla
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            visible: true,
        }
    }

    pub fn move_to(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }

    pub fn move_down(&mut self, rows: usize, max_row: usize) {
        self.row = (self.row + rows).min(max_row);
    }

    pub fn move_up(&mut self, rows: usize) {
        self.row = self.row.saturating_sub(rows);
    }

    pub fn move_right(&mut self, cols: usize, max_col: usize) {
        self.col = (self.col + cols).min(max_col);
    }

    pub fn move_left(&mut self, cols: usize) {
        self.col = self.col.saturating_sub(cols);
    }

    pub fn carriage_return(&mut self) {
        self.col = 0;
    }

    pub fn line_feed(&mut self, max_row: usize) {
        if self.row < max_row {
            self.row += 1;
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
