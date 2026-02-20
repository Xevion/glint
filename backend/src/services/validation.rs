use crate::error::{AppError, AppResult};

/// Image formats detectable from file headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    WebP,
    Png,
}

/// Detect image format from the first bytes of a file.
pub fn detect_format(bytes: &[u8]) -> AppResult<ImageFormat> {
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(ImageFormat::WebP);
    }
    if bytes.len() >= 8 && bytes[0..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return Ok(ImageFormat::Png);
    }
    Err(AppError::BadRequest(
        "Unrecognized image format (expected WebP or PNG)".to_string(),
    ))
}

/// Parse image dimensions from a header byte slice.
///
/// WebP supports three sub-formats:
/// - VP8 (lossy): width at bytes 26-27, height at bytes 28-29 (LE16, masked 0x3FFF)
/// - VP8L (lossless): packed LE32 at bytes 21-24 with two 14-bit fields (+1 each)
/// - VP8X (extended): LE24 at bytes 24-26 (width-1) and 27-29 (height-1)
///
/// PNG: IHDR chunk at bytes 16-23, width and height as BE32.
pub fn parse_dimensions(bytes: &[u8], format: ImageFormat) -> AppResult<(u32, u32)> {
    match format {
        ImageFormat::WebP => parse_webp_dimensions(bytes),
        ImageFormat::Png => parse_png_dimensions(bytes),
    }
}

fn parse_webp_dimensions(bytes: &[u8]) -> AppResult<(u32, u32)> {
    // Need at least 16 bytes to read the VP8 sub-format tag
    if bytes.len() < 16 {
        return Err(AppError::BadRequest(
            "WebP header too short to determine sub-format".to_string(),
        ));
    }

    let fourcc = &bytes[12..16];

    if fourcc == b"VP8 " {
        // Lossy VP8: dimensions at bytes 26-29
        if bytes.len() < 30 {
            return Err(AppError::BadRequest(
                "WebP VP8 header too short for dimensions".to_string(),
            ));
        }
        let width = u16::from_le_bytes([bytes[26], bytes[27]]) & 0x3FFF;
        let height = u16::from_le_bytes([bytes[28], bytes[29]]) & 0x3FFF;
        Ok((width as u32, height as u32))
    } else if fourcc == b"VP8L" {
        // Lossless VP8L: packed LE32 at bytes 21-24
        if bytes.len() < 25 {
            return Err(AppError::BadRequest(
                "WebP VP8L header too short for dimensions".to_string(),
            ));
        }
        let packed = u32::from_le_bytes([bytes[21], bytes[22], bytes[23], bytes[24]]);
        let width = (packed & 0x3FFF) + 1;
        let height = ((packed >> 14) & 0x3FFF) + 1;
        Ok((width, height))
    } else if fourcc == b"VP8X" {
        // Extended VP8X: LE24 at bytes 24-26 (width-1) and 27-29 (height-1)
        if bytes.len() < 30 {
            return Err(AppError::BadRequest(
                "WebP VP8X header too short for dimensions".to_string(),
            ));
        }
        let width = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]) + 1;
        let height = u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]) + 1;
        Ok((width, height))
    } else {
        Err(AppError::BadRequest(format!(
            "Unknown WebP sub-format: {:?}",
            String::from_utf8_lossy(fourcc)
        )))
    }
}

fn parse_png_dimensions(bytes: &[u8]) -> AppResult<(u32, u32)> {
    // PNG IHDR: width at bytes 16-19, height at bytes 20-23 (big-endian u32)
    if bytes.len() < 24 {
        return Err(AppError::BadRequest(
            "PNG header too short for dimensions".to_string(),
        ));
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Ok((width, height))
}

/// Validate a capture's content against expected format and dimensions.
pub fn validate_capture(
    bytes: &[u8],
    expected_format: &str,
    reported_width: i32,
    reported_height: i32,
) -> AppResult<()> {
    let detected = detect_format(bytes)?;

    let expected = match expected_format {
        "webp" => ImageFormat::WebP,
        "png" => ImageFormat::Png,
        other => {
            return Err(AppError::BadRequest(format!(
                "Unknown expected format: {other}"
            )));
        }
    };

    if detected != expected {
        return Err(AppError::BadRequest(format!(
            "Format mismatch: expected {expected_format}, detected {detected:?}"
        )));
    }

    let (width, height) = parse_dimensions(bytes, detected)?;

    if width != reported_width as u32 || height != reported_height as u32 {
        return Err(AppError::BadRequest(format!(
            "Dimension mismatch: expected {}x{}, detected {}x{}",
            reported_width, reported_height, width, height
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helper: build minimal WebP VP8 (lossy) header --
    fn make_vp8_header(width: u16, height: u16) -> Vec<u8> {
        let mut buf = vec![0u8; 30];
        buf[0..4].copy_from_slice(b"RIFF");
        buf[8..12].copy_from_slice(b"WEBP");
        buf[12..16].copy_from_slice(b"VP8 ");
        // VP8 bitstream frame tag at 23-25 (3 bytes), then dims at 26-29
        buf[26] = (width & 0xFF) as u8;
        buf[27] = ((width >> 8) & 0x3F) as u8;
        buf[28] = (height & 0xFF) as u8;
        buf[29] = ((height >> 8) & 0x3F) as u8;
        buf
    }

    // -- Helper: build minimal WebP VP8L (lossless) header --
    fn make_vp8l_header(width: u32, height: u32) -> Vec<u8> {
        let mut buf = vec![0u8; 25];
        buf[0..4].copy_from_slice(b"RIFF");
        buf[8..12].copy_from_slice(b"WEBP");
        buf[12..16].copy_from_slice(b"VP8L");
        // Signature byte at 20
        buf[20] = 0x2F;
        // Packed LE32 at bytes 21-24: (width-1) | ((height-1) << 14)
        let packed = (width - 1) | ((height - 1) << 14);
        let packed_bytes = packed.to_le_bytes();
        buf[21..25].copy_from_slice(&packed_bytes);
        buf
    }

    // -- Helper: build minimal WebP VP8X (extended) header --
    fn make_vp8x_header(width: u32, height: u32) -> Vec<u8> {
        let mut buf = vec![0u8; 30];
        buf[0..4].copy_from_slice(b"RIFF");
        buf[8..12].copy_from_slice(b"WEBP");
        buf[12..16].copy_from_slice(b"VP8X");
        // Canvas width-1 as LE24 at bytes 24-26
        let w = (width - 1).to_le_bytes();
        buf[24] = w[0];
        buf[25] = w[1];
        buf[26] = w[2];
        // Canvas height-1 as LE24 at bytes 27-29
        let h = (height - 1).to_le_bytes();
        buf[27] = h[0];
        buf[28] = h[1];
        buf[29] = h[2];
        buf
    }

    // -- Helper: build minimal PNG header with IHDR --
    fn make_png_header(width: u32, height: u32) -> Vec<u8> {
        let mut buf = vec![0u8; 24];
        // PNG signature
        buf[0..8].copy_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        // IHDR chunk: length (4 bytes), type (4 bytes), then width + height (BE32 each)
        // bytes 8-11: chunk length (13 for IHDR data)
        buf[8..12].copy_from_slice(&13u32.to_be_bytes());
        // bytes 12-15: chunk type "IHDR"
        buf[12..16].copy_from_slice(b"IHDR");
        // bytes 16-19: width (BE32)
        buf[16..20].copy_from_slice(&width.to_be_bytes());
        // bytes 20-23: height (BE32)
        buf[20..24].copy_from_slice(&height.to_be_bytes());
        buf
    }

    // ---- detect_format tests ----

    #[test]
    fn test_detect_format_webp() {
        let header = make_vp8_header(100, 100);
        assert_eq!(detect_format(&header).unwrap(), ImageFormat::WebP);
    }

    #[test]
    fn test_detect_format_png() {
        let header = make_png_header(100, 100);
        assert_eq!(detect_format(&header).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn test_detect_format_too_short() {
        assert!(detect_format(&[0x89, b'P', b'N']).is_err());
    }

    #[test]
    fn test_detect_format_unknown() {
        let garbage = vec![0u8; 30];
        assert!(detect_format(&garbage).is_err());
    }

    // ---- parse_dimensions: WebP VP8 (lossy) ----

    #[test]
    fn test_parse_dimensions_vp8() {
        let header = make_vp8_header(3840, 2160);
        let (w, h) = parse_dimensions(&header, ImageFormat::WebP).unwrap();
        assert_eq!((w, h), (3840, 2160));
    }

    #[test]
    fn test_parse_dimensions_vp8_small() {
        let header = make_vp8_header(1, 1);
        let (w, h) = parse_dimensions(&header, ImageFormat::WebP).unwrap();
        assert_eq!((w, h), (1, 1));
    }

    #[test]
    fn test_parse_dimensions_vp8_too_short() {
        let header = make_vp8_header(100, 100);
        assert!(parse_dimensions(&header[..20], ImageFormat::WebP).is_err());
    }

    // ---- parse_dimensions: WebP VP8L (lossless) ----

    #[test]
    fn test_parse_dimensions_vp8l() {
        let header = make_vp8l_header(1920, 1080);
        let (w, h) = parse_dimensions(&header, ImageFormat::WebP).unwrap();
        assert_eq!((w, h), (1920, 1080));
    }

    #[test]
    fn test_parse_dimensions_vp8l_max_14bit() {
        // Maximum for 14-bit field: 16383 + 1 = 16384
        let header = make_vp8l_header(16384, 16384);
        let (w, h) = parse_dimensions(&header, ImageFormat::WebP).unwrap();
        assert_eq!((w, h), (16384, 16384));
    }

    #[test]
    fn test_parse_dimensions_vp8l_too_short() {
        let header = make_vp8l_header(100, 100);
        assert!(parse_dimensions(&header[..20], ImageFormat::WebP).is_err());
    }

    // ---- parse_dimensions: WebP VP8X (extended) ----

    #[test]
    fn test_parse_dimensions_vp8x() {
        let header = make_vp8x_header(3840, 2160);
        let (w, h) = parse_dimensions(&header, ImageFormat::WebP).unwrap();
        assert_eq!((w, h), (3840, 2160));
    }

    #[test]
    fn test_parse_dimensions_vp8x_one() {
        let header = make_vp8x_header(1, 1);
        let (w, h) = parse_dimensions(&header, ImageFormat::WebP).unwrap();
        assert_eq!((w, h), (1, 1));
    }

    #[test]
    fn test_parse_dimensions_vp8x_too_short() {
        let header = make_vp8x_header(100, 100);
        assert!(parse_dimensions(&header[..20], ImageFormat::WebP).is_err());
    }

    // ---- parse_dimensions: PNG ----

    #[test]
    fn test_parse_dimensions_png() {
        let header = make_png_header(3840, 2160);
        let (w, h) = parse_dimensions(&header, ImageFormat::Png).unwrap();
        assert_eq!((w, h), (3840, 2160));
    }

    #[test]
    fn test_parse_dimensions_png_too_short() {
        let header = make_png_header(100, 100);
        assert!(parse_dimensions(&header[..16], ImageFormat::Png).is_err());
    }

    // ---- validate_capture integration ----

    #[test]
    fn test_validate_capture_webp_vp8_ok() {
        let header = make_vp8_header(3840, 2160);
        assert!(validate_capture(&header, "webp", 3840, 2160).is_ok());
    }

    #[test]
    fn test_validate_capture_webp_vp8l_ok() {
        let header = make_vp8l_header(1920, 1080);
        assert!(validate_capture(&header, "webp", 1920, 1080).is_ok());
    }

    #[test]
    fn test_validate_capture_webp_vp8x_ok() {
        let header = make_vp8x_header(3840, 2160);
        assert!(validate_capture(&header, "webp", 3840, 2160).is_ok());
    }

    #[test]
    fn test_validate_capture_png_ok() {
        let header = make_png_header(1920, 1080);
        assert!(validate_capture(&header, "png", 1920, 1080).is_ok());
    }

    #[test]
    fn test_validate_capture_format_mismatch() {
        let header = make_png_header(1920, 1080);
        let err = validate_capture(&header, "webp", 1920, 1080).unwrap_err();
        assert!(
            err.to_string().contains("Format mismatch"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_validate_capture_dimension_mismatch() {
        let header = make_vp8_header(1920, 1080);
        let err = validate_capture(&header, "webp", 3840, 2160).unwrap_err();
        assert!(
            err.to_string().contains("Dimension mismatch"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_validate_capture_unknown_format_string() {
        let header = make_vp8_header(100, 100);
        let err = validate_capture(&header, "jpeg", 100, 100).unwrap_err();
        assert!(
            err.to_string().contains("Unknown expected format"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_validate_capture_empty_bytes() {
        let err = validate_capture(&[], "webp", 100, 100).unwrap_err();
        assert!(
            err.to_string().contains("Unrecognized image format"),
            "unexpected error: {}",
            err
        );
    }
}
