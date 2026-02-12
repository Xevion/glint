use std::io::{Cursor, Read};

use super::ExtractionError;
use super::limits;

/// Result of scanning a shader pack zip archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipScanResult {
    /// All file paths in the archive.
    pub file_paths: Vec<String>,
    /// Detected dimension folders: "overworld", "nether", "end".
    pub dimension_support: Vec<String>,
    /// Whether the pack contains custom textures.
    pub has_custom_textures: bool,
}

/// Scan a zip archive's structure without extracting files.
pub fn scan_zip(data: &[u8]) -> Result<ZipScanResult, ExtractionError> {
    if data.len() > limits::MAX_ARCHIVE_SIZE {
        return Err(ExtractionError::ArchiveTooLarge {
            size: data.len(),
            max: limits::MAX_ARCHIVE_SIZE,
        });
    }

    let cursor = Cursor::new(data);
    let archive = zip::ZipArchive::new(cursor)?;

    if archive.len() > limits::MAX_ENTRY_COUNT {
        return Err(ExtractionError::TooManyEntries {
            count: archive.len(),
            max: limits::MAX_ENTRY_COUNT,
        });
    }

    let mut file_paths = Vec::with_capacity(archive.len());
    let mut has_overworld = false;
    let mut has_nether = false;
    let mut has_end = false;
    let mut has_custom_textures = false;

    for i in 0..archive.len() {
        let Some(entry) = archive.name_for_index(i) else {
            continue;
        };
        let name = entry.to_string();

        // Dimension detection via path segments
        if name.contains("world0/") {
            has_overworld = true;
        }
        if name.contains("world-1/") {
            has_nether = true;
        }
        if name.contains("world1/") {
            has_end = true;
        }

        // Custom texture detection
        if name.contains("/image/") || name.starts_with("shaders/image/") {
            has_custom_textures = true;
        }

        file_paths.push(name);
    }

    let mut dimension_support = Vec::new();
    if has_overworld {
        dimension_support.push("overworld".to_string());
    }
    if has_nether {
        dimension_support.push("nether".to_string());
    }
    if has_end {
        dimension_support.push("end".to_string());
    }

    Ok(ZipScanResult {
        file_paths,
        dimension_support,
        has_custom_textures,
    })
}

/// Read a specific entry from a zip archive by path, with size limit enforcement.
pub fn read_zip_entry(data: &[u8], path: &str, max_size: usize) -> Result<String, ExtractionError> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    let entry = match archive.by_name(path) {
        Ok(entry) => entry,
        Err(zip::result::ZipError::FileNotFound) => {
            return Err(ExtractionError::EntryNotFound(path.to_string()));
        }
        Err(e) => return Err(ExtractionError::InvalidZip(e)),
    };

    let declared_size = entry.size() as usize;
    if declared_size > max_size {
        return Err(ExtractionError::EntryTooLarge {
            name: path.to_string(),
            size: declared_size,
            max: max_size,
        });
    }

    // Defense-in-depth: don't trust declared size, cap actual I/O
    let mut limited = entry.take((max_size as u64) + 1);
    let mut contents = String::new();
    limited.read_to_string(&mut contents)?;
    if contents.len() > max_size {
        return Err(ExtractionError::EntryTooLarge {
            name: path.to_string(),
            size: contents.len(),
            max: max_size,
        });
    }
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn make_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let buf = Vec::new();
        let cursor = Cursor::new(buf);
        let mut writer = zip::ZipWriter::new(cursor);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn test_scan_basic_structure() {
        let zip = make_zip(&[
            ("shaders/shaders.properties", b"clouds=fancy"),
            ("shaders/lang/en_US.lang", b"profile.LOW=Low"),
            ("shaders/gbuffers_terrain.vsh", b""),
        ]);
        let result = scan_zip(&zip).unwrap();
        assert_eq!(result.file_paths.len(), 3);
        assert!(
            result
                .file_paths
                .contains(&"shaders/shaders.properties".into())
        );
    }

    #[test]
    fn test_dimension_detection() {
        let zip = make_zip(&[
            ("shaders/world0/gbuffers_terrain.vsh", b""),
            ("shaders/world-1/gbuffers_terrain.vsh", b""),
        ]);
        let result = scan_zip(&zip).unwrap();
        assert!(result.dimension_support.contains(&"overworld".into()));
        assert!(result.dimension_support.contains(&"nether".into()));
        assert!(!result.dimension_support.contains(&"end".into()));
    }

    #[test]
    fn test_custom_texture_detection() {
        let zip = make_zip(&[("shaders/image/noise.png", b"fake-png")]);
        let result = scan_zip(&zip).unwrap();
        assert!(result.has_custom_textures);
    }

    #[test]
    fn test_no_custom_textures() {
        let zip = make_zip(&[("shaders/gbuffers_terrain.vsh", b"")]);
        let result = scan_zip(&zip).unwrap();
        assert!(!result.has_custom_textures);
    }

    #[test]
    fn test_read_entry() {
        let content = "key=value\nother=thing";
        let zip = make_zip(&[("shaders/shaders.properties", content.as_bytes())]);
        let result = read_zip_entry(
            &zip,
            "shaders/shaders.properties",
            limits::MAX_PROPERTIES_SIZE,
        )
        .unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_read_entry_not_found() {
        let zip = make_zip(&[("shaders/other.txt", b"")]);
        let result = read_zip_entry(
            &zip,
            "shaders/shaders.properties",
            limits::MAX_PROPERTIES_SIZE,
        );
        assert!(matches!(result, Err(ExtractionError::EntryNotFound(_))));
    }

    #[test]
    fn test_read_entry_too_large() {
        let big = vec![b'x'; 100];
        let zip = make_zip(&[("big.txt", &big)]);
        let result = read_zip_entry(&zip, "big.txt", 50);
        assert!(matches!(result, Err(ExtractionError::EntryTooLarge { .. })));
    }

    #[test]
    fn test_too_many_entries_accepted() {
        // Verify that a small archive with few entries succeeds.
        let zip = make_zip(&[("a.txt", b""), ("b.txt", b"")]);
        let result = scan_zip(&zip);
        assert!(result.is_ok());
    }
}
