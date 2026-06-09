/// Atributos de celda del terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellAttributes {
    pub fg_color: Color,
    pub bg_color: Color,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self {
            fg_color: Color::default_fg(),
            bg_color: Color::default_bg(),
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            strikethrough: false,
            reverse: false,
        }
    }
}

/// Representación de color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Color por defecto
    Default,
    /// Índice de paleta (0-255)
    Indexed(u8),
    /// Color RGB verdadero
    Rgb(u8, u8, u8),
}

impl Color {
    pub fn default_fg() -> Self {
        Color::Indexed(7) // Blanco
    }

    pub fn default_bg() -> Self {
        Color::Indexed(0) // Negro
    }

    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Default => (255, 255, 255),
            Color::Indexed(idx) => Self::indexed_to_rgb(*idx),
            Color::Rgb(r, g, b) => (*r, *g, *b),
        }
    }

    fn indexed_to_rgb(idx: u8) -> (u8, u8, u8) {
        match idx {
            // Paleta básica de 16 colores
            0 => (0, 0, 0),        // Negro
            1 => (205, 0, 0),      // Rojo
            2 => (0, 205, 0),      // Verde
            3 => (205, 205, 0),    // Amarillo
            4 => (0, 0, 238),      // Azul
            5 => (205, 0, 205),    // Magenta
            6 => (0, 205, 205),    // Cyan
            7 => (229, 229, 229),  // Blanco
            8 => (127, 127, 127),  // Negro brillante
            9 => (255, 0, 0),      // Rojo brillante
            10 => (0, 255, 0),     // Verde brillante
            11 => (255, 255, 0),   // Amarillo brillante
            12 => (92, 92, 255),   // Azul brillante
            13 => (255, 0, 255),   // Magenta brillante
            14 => (0, 255, 255),   // Cyan brillante
            15 => (255, 255, 255), // Blanco brillante
            // 16-231: cubo de color 6x6x6 (estándar xterm)
            16..=231 => {
                let i = idx - 16;
                let ch = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
                (ch(i / 36), ch((i % 36) / 6), ch(i % 6))
            }
            // 232-255: rampa de grises
            _ => {
                let gray = 8 + (idx - 232) * 10;
                (gray, gray, gray)
            }
        }
    }
}
