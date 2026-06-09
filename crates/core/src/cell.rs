use crate::attributes::CellAttributes;
use unicode_width::UnicodeWidthChar;

/// Una celda individual de la terminal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub character: char,
    pub attrs: CellAttributes,
    pub width: u8,           // Ancho en celdas (1 para ASCII, 2 para CJK, etc.)
    pub is_suggestion: bool, // Indica si es parte de una sugerencia de autocompletado
    /// Id de hyperlink OSC 8 en el registro del Screen, si la celda es parte de uno
    pub hyperlink: Option<u16>,
}

impl Cell {
    pub fn new(character: char) -> Self {
        let width = character.width().unwrap_or(1) as u8;
        Self {
            character,
            attrs: CellAttributes::default(),
            width,
            is_suggestion: false,
            hyperlink: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            character: ' ',
            attrs: CellAttributes::default(),
            width: 1,
            is_suggestion: false,
            hyperlink: None,
        }
    }

    pub fn with_attrs(character: char, attrs: CellAttributes) -> Self {
        let width = character.width().unwrap_or(1) as u8;
        Self {
            character,
            attrs,
            width,
            is_suggestion: false,
            hyperlink: None,
        }
    }

    pub fn as_suggestion(character: char) -> Self {
        let width = character.width().unwrap_or(1) as u8;
        Self {
            character,
            attrs: CellAttributes::default(),
            width,
            is_suggestion: true,
            hyperlink: None,
        }
    }

    pub fn is_wide(&self) -> bool {
        self.width > 1
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::empty()
    }
}
