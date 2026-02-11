/// Convert a human-readable name into a URL-safe slug.
///
/// Rules:
/// - Lowercase
/// - Replace non-alphanumeric chars with hyphens
/// - Collapse consecutive hyphens
/// - Trim leading/trailing hyphens
///
/// Examples: "Complementary Reimagined" → "complementary-reimagined"
///           "BSL Shaders v8.2.1" → "bsl-shaders-v8-2-1"
pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_hyphen = true; // suppress leading hyphens

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_hyphen = false;
        } else if c == '\'' {
            // Strip apostrophes entirely (possessives: "Sildur's" → "sildurs")
        } else if !prev_hyphen {
            slug.push('-');
            prev_hyphen = true;
        }
    }

    // Trim trailing hyphen
    if slug.ends_with('-') {
        slug.pop();
    }

    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slugify() {
        assert_eq!(
            slugify("Complementary Reimagined"),
            "complementary-reimagined"
        );
    }

    #[test]
    fn version_numbers() {
        assert_eq!(slugify("BSL Shaders v8.2.1"), "bsl-shaders-v8-2-1");
    }

    #[test]
    fn special_chars() {
        assert_eq!(
            slugify("Sildur's Vibrant Shaders"),
            "sildurs-vibrant-shaders"
        );
    }

    #[test]
    fn consecutive_spaces() {
        assert_eq!(slugify("a   b"), "a-b");
    }

    #[test]
    fn leading_trailing() {
        assert_eq!(slugify("  hello  "), "hello");
    }

    #[test]
    fn empty_string() {
        assert_eq!(slugify(""), "");
    }
}
