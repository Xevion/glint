/// Maximum total archive size in bytes (200 MB).
pub const MAX_ARCHIVE_SIZE: usize = 200 * 1024 * 1024;

/// Maximum number of entries in a zip archive.
pub const MAX_ENTRY_COUNT: usize = 10_000;

/// Maximum decompressed size of a shaders.properties file (1 MB).
pub const MAX_PROPERTIES_SIZE: usize = 1024 * 1024;

/// Maximum decompressed size of a .lang file (512 KB).
pub const MAX_LANG_SIZE: usize = 512 * 1024;

/// Maximum number of lines in a single parsed file.
pub const MAX_LINE_COUNT: usize = 50_000;

/// Maximum length of a property key in bytes.
pub const MAX_KEY_LENGTH: usize = 512;
