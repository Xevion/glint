pub mod archive;
pub mod error;
pub mod lang;
pub mod limits;
pub mod normalize;
pub mod properties;
pub mod shader_props;

pub use error::ExtractionError;

use std::io::{Read, Seek, SeekFrom};

use self::archive::ZipScanResult;
use self::lang::LangData;
use self::shader_props::ShaderPropertiesData;

/// Complete extraction result from a shader pack zip archive.
#[derive(Debug, Clone)]
pub struct ShaderPackData {
    pub properties: Option<ShaderPropertiesData>,
    pub lang: Option<LangData>,
    pub scan: ZipScanResult,
}

/// Extract all metadata from a shader pack zip archive.
///
/// Accepts any seekable reader (e.g. `File`, `Cursor<Vec<u8>>`).
/// The archive is re-opened for each entry read via seek-to-start,
/// so the reader must support seeking.
///
/// 1. Scans the zip structure (file paths, dimensions, textures)
/// 2. Reads and parses shaders.properties (if present)
/// 3. Reads and parses en_US.lang (if present)
///
/// Missing files are not errors — many shader packs omit shaders.properties
/// or lang files entirely.
pub fn extract_shader_pack<R: Read + Seek>(
    mut reader: R,
) -> Result<ShaderPackData, ExtractionError> {
    let scan = archive::scan_zip(&mut reader)?;

    reader.seek(SeekFrom::Start(0))?;
    let properties = match archive::read_zip_entry(
        &mut reader,
        "shaders/shaders.properties",
        limits::MAX_PROPERTIES_SIZE,
    ) {
        Ok(content) => {
            let pairs = properties::parse_properties(&content)?;
            Some(shader_props::interpret_shader_properties(&pairs)?)
        }
        Err(ExtractionError::EntryNotFound(_)) => None,
        Err(e) => return Err(e),
    };

    reader.seek(SeekFrom::Start(0))?;
    let lang = match archive::read_zip_entry(
        &mut reader,
        "shaders/lang/en_US.lang",
        limits::MAX_LANG_SIZE,
    ) {
        Ok(content) => Some(lang::parse_lang(&content)?),
        Err(ExtractionError::EntryNotFound(_)) => None,
        Err(e) => return Err(e),
    };

    Ok(ShaderPackData {
        properties,
        lang,
        scan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
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
    fn test_full_extraction_with_properties_and_lang() {
        let props = "\
            profile.LOW=SHADOWS AO_QUALITY=1 !DOF\n\
            profile.HIGH=profile.LOW BLOOM shadowMapResolution=2048\n\
            iris.features.required=SSBO\n\
            separateAo=true\n\
            screen=shadowMapResolution [POST]\n\
            screen.POST=BLOOM DOF\n";
        let lang_content = "\
            profile.LOW=Low Quality\n\
            profile.HIGH=High Quality\n\
            option.SHADOWS=Shadow Quality\n\
            screen.POST=Post Processing\n";
        let zip = make_zip(&[
            ("shaders/shaders.properties", props.as_bytes()),
            ("shaders/lang/en_US.lang", lang_content.as_bytes()),
            ("shaders/gbuffers_terrain.vsh", b""),
            ("shaders/world0/gbuffers_terrain.vsh", b""),
            ("shaders/world-1/gbuffers_terrain.vsh", b""),
        ]);

        let result = extract_shader_pack(Cursor::new(zip.as_slice())).unwrap();

        // Scan
        assert_eq!(result.scan.file_paths.len(), 5);
        assert!(result.scan.dimension_support.contains(&"overworld".into()));
        assert!(result.scan.dimension_support.contains(&"nether".into()));
        assert!(!result.scan.has_custom_textures);

        // Properties
        let props = result.properties.as_ref().unwrap();
        assert_eq!(props.profiles.len(), 2);
        assert_eq!(props.profiles[0].name, "LOW");
        assert_eq!(props.profiles[1].inherits_from, Some("LOW".into()));
        assert_eq!(props.iris_features_required, vec!["SSBO"]);
        assert_eq!(props.pipeline.booleans.get("separateAo"), Some(&true));

        // Lang
        let lang = result.lang.as_ref().unwrap();
        assert_eq!(lang.profile_labels.get("LOW"), Some(&"Low Quality".into()));
        assert_eq!(
            lang.option_labels.get("SHADOWS"),
            Some(&"Shadow Quality".into())
        );
    }

    #[test]
    fn test_extraction_without_properties() {
        let zip = make_zip(&[("shaders/gbuffers_terrain.vsh", b"")]);
        let result = extract_shader_pack(Cursor::new(zip.as_slice())).unwrap();
        assert!(result.properties.is_none());
        assert!(result.lang.is_none());
        assert_eq!(result.scan.file_paths.len(), 1);
    }

    #[test]
    fn test_extraction_empty_archive() {
        let zip = make_zip(&[]);
        let result = extract_shader_pack(Cursor::new(zip.as_slice())).unwrap();
        assert!(result.properties.is_none());
        assert!(result.lang.is_none());
        assert!(result.scan.file_paths.is_empty());
    }

    #[test]
    fn test_archive_too_large_rejected() {
        // Size check happens before zip parsing, so data needn't be valid zip
        let oversized = vec![0u8; limits::MAX_ARCHIVE_SIZE + 1];
        let result = extract_shader_pack(Cursor::new(oversized.as_slice()));
        assert!(matches!(
            result,
            Err(ExtractionError::ArchiveTooLarge { .. })
        ));
    }
}
