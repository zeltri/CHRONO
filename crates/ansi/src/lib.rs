//! ANSI/VT escape sequence parser.
//!
//! This crate provides ANSI/VT100/VT220 escape sequence parsing functionality.
//! It uses the `vte` crate for low-level parsing and translates sequences into
//! terminal state modifications.

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

    /// Procesa bytes de entrada y actualiza la pantalla.
    ///
    /// Devuelve las respuestas que el terminal debe enviar de vuelta al PTY
    /// (por ejemplo, Device Attributes o Cursor Position Report).
    pub fn process(&mut self, data: &[u8], screen: &mut Screen) -> Vec<u8> {
        let mut responses = Vec::new();
        let mut handler = AnsiHandler::new(screen, &mut responses);

        for byte in data {
            self.parser.advance(&mut handler, *byte);
        }

        responses
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}
