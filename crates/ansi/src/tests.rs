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

    #[test]
    fn test_carriage_return_overwrite_preserves_rest() {
        // Las barras de progreso dependen de que \r NO borre la línea
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"[####    ] 50%\r[#####", &mut screen);

        let grid = screen.get_visible();
        assert_eq!(grid[0][5].character, '#'); // Sobrescrito
        assert_eq!(grid[0][11].character, '5'); // "50%" sigue visible
    }

    #[test]
    fn test_erase_to_end_of_display() {
        let mut screen = Screen::new(4, 10);
        let mut parser = AnsiParser::new();

        parser.process(b"AAAA\r\nBBBB\r\nCCCC", &mut screen);
        parser.process(b"\x1b[2;2H\x1b[0J", &mut screen); // cursor a (2,2), ED 0

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'A'); // Primera línea intacta
        assert_eq!(grid[1][0].character, 'B'); // Antes del cursor intacto
        assert_eq!(grid[1][1].character, ' '); // Desde el cursor, borrado
        assert_eq!(grid[2][0].character, ' '); // Líneas siguientes borradas
    }

    #[test]
    fn test_erase_line_left() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"Hello World\x1b[1;6H\x1b[1K", &mut screen); // EL 1 en col 6

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, ' ');
        assert_eq!(grid[0][5].character, ' '); // Incluye el cursor
        assert_eq!(grid[0][6].character, 'W'); // Resto intacto
    }

    #[test]
    fn test_scroll_region() {
        let mut screen = Screen::new(5, 10);
        let mut parser = AnsiParser::new();

        // Región de scroll: filas 2-4 (1-indexed). Llenar y forzar scroll.
        parser.process(b"TOP\x1b[2;4r", &mut screen);
        parser.process(b"\x1b[4;1HX\x1b[4;1H\nY", &mut screen);

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'T'); // Fuera de la región, intacto
        assert_eq!(grid[2][0].character, 'X'); // X subió una fila dentro de la región
        assert_eq!(grid[3][0].character, 'Y');
    }

    #[test]
    fn test_alt_screen() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"main content", &mut screen);
        parser.process(b"\x1b[?1049h", &mut screen); // Entrar a alt screen
        assert!(screen.is_alt_screen());
        assert_eq!(screen.get_visible()[0][0].character, ' '); // Alt screen vacía

        parser.process(b"vim!", &mut screen);
        parser.process(b"\x1b[?1049l", &mut screen); // Salir

        assert!(!screen.is_alt_screen());
        assert_eq!(screen.get_visible()[0][0].character, 'm'); // Contenido restaurado
    }

    #[test]
    fn test_insert_delete_lines() {
        let mut screen = Screen::new(4, 10);
        let mut parser = AnsiParser::new();

        parser.process(b"AA\r\nBB\r\nCC", &mut screen);
        parser.process(b"\x1b[2;1H\x1b[1L", &mut screen); // Insertar línea en fila 2

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'A');
        assert_eq!(grid[1][0].character, ' '); // Línea insertada
        assert_eq!(grid[2][0].character, 'B'); // BB bajó

        parser.process(b"\x1b[1M", &mut screen); // Borrar la línea insertada
        let grid = screen.get_visible();
        assert_eq!(grid[1][0].character, 'B'); // BB volvió a subir
    }

    #[test]
    fn test_insert_delete_chars() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"ABCDEF\x1b[1;3H\x1b[2@", &mut screen); // ICH 2 en col 3

        let grid = screen.get_visible();
        assert_eq!(grid[0][1].character, 'B');
        assert_eq!(grid[0][2].character, ' ');
        assert_eq!(grid[0][3].character, ' ');
        assert_eq!(grid[0][4].character, 'C'); // Desplazado a la derecha

        parser.process(b"\x1b[2P", &mut screen); // DCH 2: deshace la inserción
        let grid = screen.get_visible();
        assert_eq!(grid[0][2].character, 'C');
    }

    #[test]
    fn test_cursor_visibility() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[?25l", &mut screen);
        assert!(!screen.cursor.visible);
        parser.process(b"\x1b[?25h", &mut screen);
        assert!(screen.cursor.visible);
    }

    #[test]
    fn test_bracketed_paste_mode() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        assert!(!screen.bracketed_paste);
        parser.process(b"\x1b[?2004h", &mut screen);
        assert!(screen.bracketed_paste);
        parser.process(b"\x1b[?2004l", &mut screen);
        assert!(!screen.bracketed_paste);
    }

    #[test]
    fn test_device_status_report() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        // CPR: pedir posición del cursor tras moverlo a (5,10)
        let responses = parser.process(b"\x1b[5;10H\x1b[6n", &mut screen);
        assert_eq!(responses, b"\x1b[5;10R");
    }

    #[test]
    fn test_save_restore_cursor() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[5;10H\x1b7\x1b[1;1H\x1b8", &mut screen);
        assert_eq!(screen.cursor.row, 4);
        assert_eq!(screen.cursor.col, 9);
    }

    #[test]
    fn test_window_title() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b]2;Mi Titulo\x07", &mut screen);
        assert_eq!(screen.take_title().as_deref(), Some("Mi Titulo"));
        assert_eq!(screen.take_title(), None); // Consumido
    }

    #[test]
    fn test_sgr_attrs_persist_across_lines() {
        // Los atributos NO deben resetearse en \r\n (antes había un hack que lo hacía)
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[31mrojo\r\ntodavia rojo", &mut screen);

        let grid = screen.get_visible();
        assert_eq!(grid[1][0].attrs.fg_color, Color::Indexed(1));
    }

    #[test]
    fn test_hyperlink_osc8() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(
            b"\x1b]8;;https://example.com\x07link\x1b]8;;\x07 normal",
            &mut screen,
        );

        assert_eq!(
            screen.display_hyperlink_at(0, 0),
            Some("https://example.com")
        );
        assert_eq!(
            screen.display_hyperlink_at(0, 3),
            Some("https://example.com")
        );
        assert_eq!(screen.display_hyperlink_at(0, 5), None); // Después del cierre
    }

    #[test]
    fn test_mouse_modes() {
        use terminal_core::MouseMode;
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[?1000h", &mut screen);
        assert_eq!(screen.mouse_mode, MouseMode::Normal);
        parser.process(b"\x1b[?1002h\x1b[?1006h", &mut screen);
        assert_eq!(screen.mouse_mode, MouseMode::ButtonEvent);
        assert!(screen.mouse_sgr);
        parser.process(b"\x1b[?1002l", &mut screen);
        assert_eq!(screen.mouse_mode, MouseMode::None);
    }

    #[test]
    fn test_sgr_dim_strikethrough() {
        let mut screen = Screen::new(24, 80);
        let mut parser = AnsiParser::new();

        parser.process(b"\x1b[2;9mX\x1b[22;29mY", &mut screen);

        let grid = screen.get_visible();
        assert!(grid[0][0].attrs.dim);
        assert!(grid[0][0].attrs.strikethrough);
        assert!(!grid[0][1].attrs.dim);
        assert!(!grid[0][1].attrs.strikethrough);
    }

    #[test]
    fn test_prompt_marks_navigation() {
        let mut screen = Screen::new(3, 20);
        let mut parser = AnsiParser::new();

        // Dos prompts separados por output que provoca scroll
        parser.process(
            b"\x1b]133;A\x07$ uno\r\nout1\r\nout2\r\nout3\r\n",
            &mut screen,
        );
        parser.process(b"\x1b]133;A\x07$ dos", &mut screen);

        assert!(screen.scrollback_len() > 0);
        screen.view_to_prev_prompt();
        assert!(screen.view_offset() > 0); // Saltó al primer prompt
        screen.view_to_next_prompt();
        // El siguiente prompt está en pantalla → vuelve en vivo o más abajo
        assert!(screen.view_offset() < screen.scrollback_len());
    }

    #[test]
    fn test_view_scrollback() {
        let mut screen = Screen::new(3, 10);
        let mut parser = AnsiParser::new();

        parser.process(b"uno\r\ndos\r\ntres\r\ncuatro\r\ncinco", &mut screen);
        assert_eq!(screen.scrollback_len(), 2);

        // En vivo: la primera fila visible es "tres"
        assert_eq!(screen.display_line(0)[0].character, 't');

        screen.scroll_view_up(2);
        assert_eq!(screen.view_offset(), 2);
        assert_eq!(screen.display_line(0)[0].character, 'u'); // "uno"

        screen.reset_view();
        assert_eq!(screen.display_line(0)[0].character, 't');
    }

    #[test]
    fn test_reflow_on_resize() {
        let mut screen = Screen::new(5, 10);
        let mut parser = AnsiParser::new();

        // 14 caracteres en 10 cols → envuelve en 2 filas
        parser.process(b"abcdefghijklmn", &mut screen);
        assert_eq!(screen.get_visible()[0][9].character, 'j');
        assert_eq!(screen.get_visible()[1][0].character, 'k');

        // Ensanchar a 20: la línea lógica vuelve a caber en una fila
        screen.resize(5, 20);
        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'a');
        assert_eq!(grid[0][13].character, 'n');
        assert_eq!(grid[1][0].character, ' ');
        // El cursor queda al final del texto
        assert_eq!(screen.cursor.row, 0);
        assert_eq!(screen.cursor.col, 14);

        // Estrechar a 7: se envuelve en dos filas
        screen.resize(5, 7);
        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, 'a');
        assert_eq!(grid[1][0].character, 'h');
    }

    #[test]
    fn test_reverse_index_scrolls_down() {
        let mut screen = Screen::new(3, 10);
        let mut parser = AnsiParser::new();

        parser.process(b"AA\x1b[1;1H\x1bM", &mut screen); // RI en la primera fila

        let grid = screen.get_visible();
        assert_eq!(grid[0][0].character, ' '); // Línea nueva arriba
        assert_eq!(grid[1][0].character, 'A'); // AA bajó
    }
}
