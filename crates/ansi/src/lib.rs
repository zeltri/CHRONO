use terminal_core::Screen;
use vte::Parser;

mod handler;
use handler::AnsiHandler;

#[cfg(test)]
mod tests;

/// Parser ANSI/VT que procesa bytes y actualiza la pantalla
pub struct AnsiParser {
    parser: Parser,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    /// Procesa bytes de entrada y actualiza la pantalla
    pub fn process(&mut self, data: &[u8], screen: &mut Screen) {
        let mut handler = AnsiHandler::new(screen);

        for byte in data {
            self.parser.advance(&mut handler, *byte);
        }
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}
