pub mod attributes;
pub mod cell;
pub mod context;
pub mod cursor;
pub mod screen;

#[cfg(test)]
mod tests;

pub use attributes::{CellAttributes, Color};
pub use cell::Cell;
pub use context::{
    analyze_line_context, detect_file_type_by_extension, extract_file_references,
    has_file_reference, is_file_listing, parse_file_entry, FileEntry, FileReference, FileType,
    LineContext,
};
pub use cursor::Cursor;
pub use screen::Screen;
