#[cfg(test)]
mod tests {
    use crate::{Cell, CellAttributes, Color, Screen};

    #[test]
    fn test_screen_creation() {
        let screen = Screen::new(24, 80);
        assert_eq!(screen.rows, 24);
        assert_eq!(screen.cols, 80);
    }

    #[test]
    fn test_write_char() {
        let mut screen = Screen::new(24, 80);
        screen.write_char('H');
        screen.write_char('i');

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'H');
        assert_eq!(grid[0][1].character, 'i');
    }

    #[test]
    fn test_line_feed() {
        let mut screen = Screen::new(24, 80);
        screen.write_char('A');
        assert_eq!(screen.cursor.row, 0);
        assert_eq!(screen.cursor.col, 1); // Después de 'A', columna 1

        screen.line_feed();
        assert_eq!(screen.cursor.row, 1);
        // El cursor no se mueve en columna con line_feed
        assert_eq!(screen.cursor.col, 1);

        // Volver al inicio de la línea
        screen.carriage_return();
        assert_eq!(screen.cursor.col, 0);

        screen.write_char('B');

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'A');
        assert_eq!(grid[1][0].character, 'B');
    }

    #[test]
    fn test_carriage_return() {
        let mut screen = Screen::new(24, 80);
        screen.write_char('A');
        screen.write_char('B');
        screen.carriage_return();
        screen.write_char('C');

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'C');
        assert_eq!(grid[0][1].character, 'B');
    }

    #[test]
    fn test_cursor_movement() {
        let mut screen = Screen::new(24, 80);
        assert_eq!(screen.cursor.row, 0);
        assert_eq!(screen.cursor.col, 0);

        screen.move_cursor_to(10, 20);
        assert_eq!(screen.cursor.row, 10);
        assert_eq!(screen.cursor.col, 20);
    }

    #[test]
    fn test_clear_screen() {
        let mut screen = Screen::new(24, 80);
        screen.write_char('X');
        screen.clear();

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, ' ');
    }

    #[test]
    fn test_resize() {
        let mut screen = Screen::new(24, 80);
        screen.write_char('A');

        screen.resize(30, 100);
        assert_eq!(screen.rows, 30);
        assert_eq!(screen.cols, 100);

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'A');
    }

    #[test]
    fn test_wide_characters() {
        let mut screen = Screen::new(24, 80);
        screen.write_char('你'); // Carácter CJK de ancho 2

        assert_eq!(screen.cursor.col, 2);

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, '你');
        assert_eq!(grid[0][0].width, 2);
    }

    #[test]
    fn test_cell_attributes() {
        let attrs = CellAttributes {
            fg_color: Color::Indexed(1),
            bg_color: Color::Indexed(0),
            bold: true,
            italic: false,
            underline: true,
            reverse: false,
        };

        let cell = Cell::with_attrs('A', attrs);
        assert_eq!(cell.character, 'A');
        assert!(cell.attrs.bold);
        assert!(cell.attrs.underline);
        assert!(!cell.attrs.italic);
    }

    #[test]
    fn test_scrollback() {
        let mut screen = Screen::new(3, 10);

        // Escribir en la primera línea y bajar
        screen.write_char('1');
        screen.carriage_return();
        screen.line_feed();

        // Escribir en la segunda línea y bajar
        screen.write_char('2');
        screen.carriage_return();
        screen.line_feed();

        // Escribir en la tercera línea y bajar
        screen.write_char('3');
        screen.carriage_return();
        screen.line_feed(); // Ahora debería hacer scroll

        // Escribir en la nueva línea
        screen.write_char('4');

        let grid = screen.get_visible();
        // Después del scroll, '1' va al scrollback
        assert_eq!(grid[0][0].character, '2');
        assert_eq!(grid[1][0].character, '3');
        assert_eq!(grid[2][0].character, '4');
    }
}
