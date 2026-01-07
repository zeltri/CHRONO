pub mod attributes;
pub mod cell;
pub mod context;
pub mod cursor;
pub mod screen;

#[cfg(test)]
mod tests;

pub use attributes::{CellAttributes, Color};
pub use cell::Cell;
pub use context::{analyze_line_context, extract_file_references, FileReference, LineContext};
pub use cursor::Cursor;
pub use screen::Screen;
