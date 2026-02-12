#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("archive too large: {size} bytes (max {max})")]
    ArchiveTooLarge { size: usize, max: usize },

    #[error("too many zip entries: {count} (max {max})")]
    TooManyEntries { count: usize, max: usize },

    #[error("entry too large: '{name}' is {size} bytes (max {max})")]
    EntryTooLarge {
        name: String,
        size: usize,
        max: usize,
    },

    #[error("entry not found: '{0}'")]
    EntryNotFound(String),

    #[error("invalid zip archive")]
    InvalidZip(#[from] zip::result::ZipError),

    #[error("parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("line too long at line {line}: {length} bytes (max {max})")]
    LineTooLong {
        line: usize,
        length: usize,
        max: usize,
    },

    #[error("too many lines: {count} (max {max})")]
    TooManyLines { count: usize, max: usize },

    #[error("key too long at line {line}: {length} bytes (max {max})")]
    KeyTooLong {
        line: usize,
        length: usize,
        max: usize,
    },

    #[error("I/O error reading zip entry")]
    Io(#[from] std::io::Error),
}
