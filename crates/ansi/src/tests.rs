#[cfg(test)]
mod tests {
    use crate::AnsiParser;
    use terminal_core::{Color, Screen};

    #[test]
    fn test_simple_text() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"Hello", &mut screen);

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'H');
        assert_eq!(grid[0][1].character, 'e');
        assert_eq!(grid[0][2].character, 'l');
        assert_eq!(grid[0][3].character, 'l');
        assert_eq!(grid[0][4].character, 'o');
    }

    #[test]
    fn test_line_feed_and_carriage_return() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"Line1\r\nLine2", &mut screen);

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'L');
        assert_eq!(grid[1][0].character, 'L');
    }

    #[test]
    fn test_sgr_bold() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[1mBold\x1b[0m", &mut screen);

        let grid = screen.get_visible();
        assert!(grid[0][0].attrs.bold);
        assert!(grid[0][3].attrs.bold);
    }

    #[test]
    fn test_sgr_color() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[31mRed\x1b[0m", &mut screen);

        let grid = screen.get_visible();
        if let Color::Indexed(color) = grid[0][0].attrs.fg_color {
            assert_eq!(color, 1); // Red
        } else {
            panic!("Expected indexed color");
        }
    }

    #[test]
    fn test_cursor_movement() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[5;10HX", &mut screen);

        let grid = screen.get_visible();
        assert_eq!(grid[4][9].character, 'X');
    }

    #[test]
    fn test_clear_screen() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"Text", &mut screen);
        parser.process(b"\x1b[2J", &mut screen);

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, ' ');
    }

    #[test]
    fn test_clear_line() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"Hello World", &mut screen);
        parser.process(b"\x1b[1G", &mut screen); // Move to start of line
        parser.process(b"\x1b[2K", &mut screen); // Clear entire line

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, ' ');
        assert_eq!(grid[0][5].character, ' ');
    }

    #[test]
    fn test_multiple_sgr_params() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[1;4;31mText\x1b[0m", &mut screen);

        let grid = screen.get_visible();
        assert!(grid[0][0].attrs.bold);
        assert!(grid[0][0].attrs.underline);
        if let Color::Indexed(color) = grid[0][0].attrs.fg_color {
            assert_eq!(color, 1); // Red
        }
    }

    #[test]
    fn test_rgb_color() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[38;2;255;128;64mRGB\x1b[0m", &mut screen);

        let grid = screen.get_visible();
        if let Color::Rgb(r, g, b) = grid[0][0].attrs.fg_color {
            assert_eq!(r, 255);
            assert_eq!(g, 128);
            assert_eq!(b, 64);
        } else {
            panic!("Expected RGB color");
        }
    }

    #[test]
    fn test_cursor_up_down() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        // Escribir en la primera línea
        parser.process(b"Line1", &mut screen);
        // Nueva línea
        parser.process(b"\r\n", &mut screen);
        // Escribir en la segunda línea
        parser.process(b"Line2", &mut screen);

        // Verificar que estamos en la línea 1 (segunda línea, 0-indexed)
        assert_eq!(screen.cursor.row, 1);

        // Mover cursor hacia arriba explícitamente
        parser.process(b"\x1b[1A", &mut screen);

        // Ahora debemos estar en la línea 0
        assert_eq!(screen.cursor.row, 0);
    }
}
