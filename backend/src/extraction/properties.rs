use super::ExtractionError;
use super::limits;

/// Parse Java .properties format text into ordered key-value pairs.
///
/// Supports: `key=value`, `key:value`, `key<space>value` separators.
/// Lines starting with `#` or `!` are comments. Blank lines are skipped.
/// Backslash at end of line joins the next line (continuation).
///
/// Returns pairs in file order (important for profile ordering).
pub fn parse_properties(input: &str) -> Result<Vec<(String, String)>, ExtractionError> {
    let lines: Vec<&str> = input.lines().collect();
    if lines.len() > limits::MAX_LINE_COUNT {
        return Err(ExtractionError::TooManyLines {
            count: lines.len(),
            max: limits::MAX_LINE_COUNT,
        });
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim_start();

        // Skip blank lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            i += 1;
            continue;
        }

        // Handle continuation lines: join lines ending with `\`
        let mut logical_line = String::new();
        let mut current = lines[i].to_string();
        while current.ends_with('\\') {
            // Remove trailing backslash
            current.truncate(current.len() - 1);
            logical_line.push_str(&current);
            i += 1;
            if i < lines.len() {
                // Strip leading whitespace from continuation line
                current = lines[i].trim_start().to_string();
            } else {
                break;
            }
        }
        logical_line.push_str(&current);
        i += 1;

        // Strip leading whitespace from the logical line for key extraction
        let trimmed = logical_line.trim_start();
        if trimmed.is_empty() {
            continue;
        }

        // Find separator: first `=`, `:`, or whitespace
        let (key, value) = split_key_value(trimmed);

        let key = key.trim().to_string();
        if key.is_empty() {
            continue;
        }

        let line_num = i; // approximate line number (1-indexed from end of logical line)
        if key.len() > limits::MAX_KEY_LENGTH {
            return Err(ExtractionError::KeyTooLong {
                line: line_num,
                length: key.len(),
                max: limits::MAX_KEY_LENGTH,
            });
        }
        if value.len() > limits::MAX_VALUE_LENGTH {
            return Err(ExtractionError::ValueTooLong {
                line: line_num,
                length: value.len(),
                max: limits::MAX_VALUE_LENGTH,
            });
        }

        result.push((key, value));
    }

    Ok(result)
}

/// Split a trimmed line into (key, value) at the first separator.
///
/// Separators are `=`, `:`, or whitespace. Leading whitespace after the
/// separator is stripped from the value; trailing whitespace is preserved.
fn split_key_value(line: &str) -> (String, String) {
    // Find the first unescaped separator
    let bytes = line.as_bytes();
    let mut sep_pos = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'=' | b':' => {
                sep_pos = Some(i);
                break;
            }
            b' ' | b'\t' => {
                // Whitespace separator — but only if we haven't seen key chars yet
                // that are themselves whitespace (key is everything before first ws)
                sep_pos = Some(i);
                // Check if whitespace is followed by `=` or `:` (e.g., "key = value")
                let rest = &line[i..].trim_start();
                if rest.starts_with('=') || rest.starts_with(':') {
                    // The real separator is the `=` or `:` after whitespace
                    let offset = line[i..].find(['=', ':']).unwrap();
                    sep_pos = Some(i + offset);
                }
                break;
            }
            _ => {}
        }
    }

    match sep_pos {
        Some(pos) => {
            let key = line[..pos].to_string();
            let after_sep = &line[pos + 1..];
            let value = after_sep.trim_start().to_string();
            (key, value)
        }
        None => {
            // No separator found — entire line is the key with empty value
            (line.to_string(), String::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_equals_separator() {
        let input = "key1=value1\nkey2=value2\n";
        let result = parse_properties(input).unwrap();
        assert_eq!(
            result,
            vec![
                ("key1".into(), "value1".into()),
                ("key2".into(), "value2".into()),
            ]
        );
    }

    #[test]
    fn test_colon_separator() {
        let input = "key1:value1\nkey2 : value2\n";
        let result = parse_properties(input).unwrap();
        assert_eq!(
            result,
            vec![
                ("key1".into(), "value1".into()),
                ("key2".into(), "value2".into()),
            ]
        );
    }

    #[test]
    fn test_space_separator() {
        let input = "key1 value1\nkey2  value2\n";
        let result = parse_properties(input).unwrap();
        assert_eq!(
            result,
            vec![
                ("key1".into(), "value1".into()),
                ("key2".into(), "value2".into()),
            ]
        );
    }

    #[test]
    fn test_comments_and_blank_lines() {
        let input = "# comment\n! another comment\n\nkey=value\n";
        let result = parse_properties(input).unwrap();
        assert_eq!(result, vec![("key".into(), "value".into())]);
    }

    #[test]
    fn test_empty_value() {
        let input = "key=\n";
        let result = parse_properties(input).unwrap();
        assert_eq!(result, vec![("key".into(), "".into())]);
    }

    #[test]
    fn test_leading_whitespace_stripped() {
        let input = "  key = value  \n";
        let result = parse_properties(input).unwrap();
        // Leading whitespace on key is stripped, trailing whitespace on value is preserved
        assert_eq!(result[0].0, "key");
        assert_eq!(result[0].1, "value  ");
    }

    #[test]
    fn test_continuation_lines() {
        let input = "key=value1 \\\n  value2 \\\n  value3\n";
        let result = parse_properties(input).unwrap();
        assert_eq!(result, vec![("key".into(), "value1 value2 value3".into())]);
    }

    #[test]
    fn test_preserves_order() {
        let input = "z=1\na=2\nm=3\n";
        let result = parse_properties(input).unwrap();
        assert_eq!(
            result,
            vec![
                ("z".into(), "1".into()),
                ("a".into(), "2".into()),
                ("m".into(), "3".into()),
            ]
        );
    }

    #[test]
    fn test_too_many_lines_rejected() {
        let input = "k=v\n".repeat(limits::MAX_LINE_COUNT + 1);
        let result = parse_properties(&input);
        assert!(matches!(result, Err(ExtractionError::TooManyLines { .. })));
    }

    #[test]
    fn test_key_too_long_rejected() {
        let long_key = "k".repeat(limits::MAX_KEY_LENGTH + 1);
        let input = format!("{long_key}=value\n");
        let result = parse_properties(&input);
        assert!(matches!(result, Err(ExtractionError::KeyTooLong { .. })));
    }

    #[test]
    fn test_value_too_long_rejected() {
        let long_value = "v".repeat(limits::MAX_VALUE_LENGTH + 1);
        let input = format!("key={long_value}\n");
        let result = parse_properties(&input);
        assert!(matches!(result, Err(ExtractionError::ValueTooLong { .. })));
    }
}
