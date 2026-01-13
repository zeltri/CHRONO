//! Terminal core library providing screen buffer, cell management, and context analysis.
//!
//! This crate contains the fundamental data structures for terminal emulation:
//! - Screen buffer with grid-based cell storage
//! - Cell attributes (colors, styles)
//! - Cursor positioning
//! - Semantic context detection (errors, warnings, file listings)
//! - Smart content detection (logs, JSON, tables, stack traces)

pub mod attributes;
pub mod cell;
pub mod context;
pub mod cursor;
pub mod detector;
pub mod history;
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
pub use detector::{
    ContentDetector, ContentType, JsonFragment, JsonTokenType, LogLevel, TableInfo, TableRowType,
};
pub use history::CommandHistory;
pub use screen::{Screen, Selection};
