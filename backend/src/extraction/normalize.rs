//! Profile display name normalization.
//!
//! Computes a clean, human-readable display name from a shader profile's
//! raw `name` (from shaders.properties) and optional `label` (from en_US.lang).
//! Strips Minecraft formatting codes, decorative Unicode, metadata suffixes,
//! and falls back to humanizing the raw name when no label is available.

/// Known-bad profile names that should never appear as display names.
const BOGUS_NAMES: &[&str] = &["SHADER_VERSION_LABEL"];

/// Known abbreviations that should stay uppercase when title-casing.
const ABBREVIATIONS: &[&str] = &[
    "TV", "BW", "NVG", "RTX", "PVP", "VHS", "IBM", "DOF", "FPS", "HD", "HQ", "LQ", "RT", "GI",
    "SSR", "SSAO", "TAA", "SSPT", "HDR",
];

/// Known words for dictionary-based splitting of all-caps names without separators.
/// All entries are lowercase. The code finds the longest match at each position
/// by scanning all entries, so ordering is cosmetic only.
const DICTIONARY: &[&str] = &[
    "shadowless",
    "screenshot",
    "cinematic",
    "unplayable",
    "timelapse",
    "realistic",
    "preserve",
    "standard",
    "fabulous",
    "balanced",
    "disabled",
    "nostalgic",
    "optifine",
    "platinum",
    "minecraft",
    "borderland",
    "default",
    "effects",
    "shadows",
    "extreme",
    "quality",
    "minimum",
    "maximum",
    "vanilla",
    "diamond",
    "toaster",
    "thermal",
    "imager",
    "potato",
    "render",
    "normal",
    "medium",
    "insane",
    "strong",
    "lowest",
    "higher",
    "lower",
    "ultra",
    "fancy",
    "light",
    "heavy",
    "fast",
    "lite",
    "high",
    "very",
    "plus",
    "low",
    "old",
    "new",
    "off",
    "max",
    "no",
    // Abbreviations (lowercase here for matching, but title_case will uppercase them)
    "tv",
    "bw",
    "nvg",
    "rtx",
    "pvp",
    "vhs",
    "ibm",
    "dof",
    "fps",
    "hd",
    "hq",
    "lq",
    "rt",
    "gi",
    "ssr",
    "ssao",
    "taa",
    "sspt",
    "hdr",
];

/// Maximum length for an unknown segment to be treated as an abbreviation
/// (kept uppercase). Longer unknowns are title-cased as probable words.
/// Set to 3 because most real abbreviations are 2-3 chars (FX, GI, RT, HDR)
/// while 4+ char unknowns are more likely unrecognized words. True 4-char
/// abbreviations (SSPT, SSAO, etc.) are handled by the ABBREVIATIONS list.
const ABBREVIATION_LENGTH_THRESHOLD: usize = 3;

/// Compute a normalized display name for a shader profile.
///
/// Pipeline:
/// 1. Try `label` if non-empty
/// 2. Strip Minecraft formatting codes (§X sequences)
/// 3. Strip decorative Unicode (ornamental symbols, CJK brackets, emoji)
/// 4. Strip metadata suffixes like (Default), (Recommended), - Performance
/// 5. Strip surrounding punctuation/brackets
/// 6. Collapse whitespace
/// 7. If result is empty, fall back to humanized `name`
/// 8. If humanized name is also empty, fall back to raw `name`
/// 9. If raw name is known-bad, return "Default"
pub fn normalize_display_name(name: &str, label: Option<&str>) -> String {
    let label_text = label.filter(|l| !l.trim().is_empty());

    if let Some(label) = label_text {
        let cleaned = strip_formatting_codes(label);
        let cleaned = strip_decorative_unicode(&cleaned);
        let cleaned = strip_metadata_suffixes(&cleaned);
        let cleaned = strip_surrounding_punctuation(&cleaned);
        let cleaned = collapse_whitespace(&cleaned);

        if !cleaned.is_empty() {
            return cleaned;
        }
    }

    // Label was empty/missing/stripped-to-nothing — check for known-bad names
    if BOGUS_NAMES.contains(&name) {
        return "Default".to_string();
    }

    let humanized = humanize_name(name);
    if humanized.is_empty() {
        if name.is_empty() {
            return "Default".to_string();
        }
        return name.to_string();
    }
    humanized
}

/// Humanize a raw profile name by splitting on underscores and camelCase
/// boundaries, then title-casing each word. For all-caps segments without
/// separators, attempts dictionary-based word splitting. For mixed-case
/// segments with uppercase runs (e.g. "RTXHigh"), splits at the boundary
/// between the run and the lowercase word.
fn humanize_name(name: &str) -> String {
    // Split on underscores first
    let segments: Vec<&str> = name.split('_').filter(|s| !s.is_empty()).collect();

    let mut words = Vec::new();
    for segment in segments {
        if is_all_caps(segment) && segment.len() > 1 {
            // All-caps segment: try dictionary splitting (VERYHIGH → Very High)
            words.extend(dictionary_split(segment));
        } else if is_all_lower(segment) && segment.len() > 1 {
            // All-lowercase segment: try dictionary splitting
            // (extremeplus → Extreme Plus)
            words.extend(dictionary_split(segment));
        } else if has_upper_run_boundary(segment) {
            // Uppercase run followed by lowercase: "RTXHigh" → "RTX" + "High"
            words.extend(split_upper_run(segment));
        } else if has_camel_case_boundary(segment) {
            // Split on camelCase boundaries
            words.extend(split_camel_case(segment));
        } else {
            words.push(title_case(segment));
        }
    }

    words.join(" ")
}

/// Strip Minecraft formatting codes from a string.
/// Removes `§X` sequences where X is any single character (color, bold, italic, etc.)
/// The § character is U+00A7 (SECTION SIGN).
///
/// Also strips text that was marked as obfuscated (`§k`). In Minecraft, `§k` makes
/// text appear as random scrambling characters — the actual characters are placeholders
/// and meaningless. We skip them until the next formatting code or end of string.
fn strip_formatting_codes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut obfuscated = false;

    while let Some(ch) = chars.next() {
        if ch == '\u{00A7}' {
            if let Some(&code) = chars.peek() {
                chars.next(); // consume the code character
                if code == 'k' {
                    // Enter obfuscated mode — skip subsequent characters
                    obfuscated = true;
                } else {
                    // Any other formatting code exits obfuscated mode
                    obfuscated = false;
                }
            }
        } else if !obfuscated {
            result.push(ch);
        }
        // If obfuscated, skip the character (it's placeholder scramble text)
    }

    result
}

/// Strip decorative Unicode characters (ornamental symbols, CJK brackets, emoji).
/// Preserves ASCII, basic Latin, and common punctuation like +, -, /, &.
fn strip_decorative_unicode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for ch in input.chars() {
        if should_keep_char(ch) {
            result.push(ch);
        }
    }

    result
}

/// Determine whether a character should be kept during decorative stripping.
fn should_keep_char(ch: char) -> bool {
    match ch {
        // ASCII printable range (space through tilde) — keep all
        ' '..='~' => true,
        // Common accented Latin characters
        '\u{00C0}'..='\u{00FF}' => true,
        // Allow non-breaking space (normalize to regular space later)
        '\u{00A0}' => true,
        // Everything else (CJK brackets, ornamental symbols, emoji, etc.) — strip
        _ => false,
    }
}

/// Known metadata suffixes to strip from profile labels.
/// These describe the profile's purpose, not its name.
const METADATA_SUFFIXES_PARENS: &[&str] = &[
    "Default",
    "Recommended",
    "Good GPU",
    "Performance",
    "Balanced",
    "Best Quality",
    "Integrated GPU",
    "Standard",
    "DoF",
];

const METADATA_SUFFIXES_DASH: &[&str] =
    &["Recommended", "Performance", "Maximum Quality", "Balanced"];

/// Strip metadata suffixes like (Default), (Recommended), - Performance, etc.
/// Loops until no more suffixes are found (handles multiple stacked suffixes).
/// Matching is case-insensitive.
fn strip_metadata_suffixes(input: &str) -> String {
    let mut result = input.to_string();

    loop {
        let before = result.len();

        // Strip parenthesized metadata: " (Default)", " (Recommended)", etc.
        for suffix in METADATA_SUFFIXES_PARENS {
            // Try with leading space
            let pattern = format!(" ({suffix})");
            if ends_with_ignore_case(&result, &pattern) {
                result.truncate(result.len() - pattern.len());
                break;
            }
            // Try without leading space
            let pattern_no_space = format!("({suffix})");
            if ends_with_ignore_case(&result, &pattern_no_space) {
                result.truncate(result.len() - pattern_no_space.len());
                break;
            }
        }

        // Strip dash-separated metadata: " - Recommended", " - Performance", etc.
        for suffix in METADATA_SUFFIXES_DASH {
            let pattern = format!(" - {suffix}");
            if ends_with_ignore_case(&result, &pattern) {
                result.truncate(result.len() - pattern.len());
                break;
            }
        }

        // No more suffixes found — done
        if result.len() == before {
            break;
        }
    }

    result
}

/// Case-insensitive `ends_with` for ASCII strings.
fn ends_with_ignore_case(haystack: &str, needle: &str) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    let start = haystack.len() - needle.len();
    haystack[start..].eq_ignore_ascii_case(needle)
}

/// Collapse multiple whitespace characters into single spaces and trim.
fn collapse_whitespace(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_space = true; // Start true to trim leading whitespace

    for ch in input.chars() {
        if ch.is_whitespace() || ch == '\u{00A0}' {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }

    // Trim trailing space
    if result.ends_with(' ') {
        result.pop();
    }

    result
}

/// Characters that are decorative when they appear at the edges of a display name.
/// These get trimmed from both the start and end.
const DECORATIVE_EDGE_CHARS: &[char] = &[
    '-', '.', '~', '|', '*', ':', ';', '!', '?', '#', '=', '_', '^', '\\',
];

/// Bracket pairs that fully wrap a name are unwrapped: `[Default]` → `Default`.
const BRACKET_PAIRS: &[(char, char)] = &[('(', ')'), ('[', ']'), ('{', '}'), ('<', '>')];

/// Strip decorative punctuation and brackets from the start and end of a string.
///
/// 1. Trim whitespace
/// 2. Strip known decorative edge characters (dashes, dots, tildes, etc.)
///    and whitespace from both ends
/// 3. If the result is fully wrapped in matching brackets, unwrap one layer
fn strip_surrounding_punctuation(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let is_decorative_edge =
        |c: char| -> bool { c.is_whitespace() || DECORATIVE_EDGE_CHARS.contains(&c) };

    // Strip decorative chars from edges
    let stripped = trimmed
        .trim_start_matches(|c: char| is_decorative_edge(c))
        .trim_end_matches(|c: char| is_decorative_edge(c));

    if stripped.is_empty() {
        return String::new();
    }

    // Unwrap matching bracket pairs
    let chars: Vec<char> = stripped.chars().collect();
    if chars.len() >= 2 {
        let (first, last) = (chars[0], *chars.last().unwrap());
        for &(open, close) in BRACKET_PAIRS {
            if first == open && last == close {
                let inner: String = chars[1..chars.len() - 1].iter().collect();
                // Recursively strip in case of nested decoration: "-- [High] --"
                // first pass strips dashes → "[High]", second unwraps brackets
                return strip_surrounding_punctuation(&inner);
            }
        }
    }

    stripped.to_string()
}

/// Check if a string is entirely uppercase ASCII letters.
fn is_all_caps(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_uppercase())
}

/// Check if a string is entirely lowercase ASCII letters.
fn is_all_lower(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase())
}

/// Check if a string has camelCase boundaries (lowercase followed by uppercase).
fn has_camel_case_boundary(s: &str) -> bool {
    s.chars()
        .zip(s.chars().skip(1))
        .any(|(a, b)| a.is_lowercase() && b.is_uppercase())
}

/// Check if a string has an uppercase run followed by a lowercase letter.
/// e.g. "RTXHigh" has "RTX" (3 uppercase) then "H" starts "High" — the boundary
/// is between X and H where the run ends and a new word begins.
/// More precisely: uppercase, uppercase, lowercase — the split point is before
/// the second uppercase character.
fn has_upper_run_boundary(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len().saturating_sub(2) {
        if chars[i].is_uppercase() && chars[i + 1].is_uppercase() && chars[i + 2].is_lowercase() {
            return true;
        }
    }
    false
}

/// Split a string with uppercase runs into words.
/// "RTXHigh" → ["RTX", "High"]
/// "SSAOEnabled" → ["SSAO", "Enabled"]
/// "HDRQuality" → ["HDR", "Quality"]
/// Also handles subsequent camelCase after the run.
fn split_upper_run(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut words = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_uppercase() {
            // Count the uppercase run
            let start = i;
            while i < chars.len() && chars[i].is_uppercase() {
                i += 1;
            }
            // If followed by lowercase, the last uppercase char starts the next word
            if i < chars.len() && chars[i].is_lowercase() && i - start > 1 {
                let abbrev = &chars[start..i - 1];
                words.push(title_case(&abbrev.iter().collect::<String>()));

                // Now collect the rest of the lowercase word
                let word_start = i - 1;
                while i < chars.len() && !chars[i].is_uppercase() {
                    i += 1;
                }
                let word: String = chars[word_start..i].iter().collect();
                words.push(title_case(&word));
            } else {
                // Pure uppercase segment (no following lowercase)
                let segment: String = chars[start..i].iter().collect();
                words.push(title_case(&segment));
            }
        } else {
            // Lowercase segment — collect until next uppercase
            let start = i;
            while i < chars.len() && !chars[i].is_uppercase() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            words.push(title_case(&word));
        }
    }

    words
}

/// Split a string on camelCase boundaries.
/// "DeepFried" → ["Deep", "Fried"]
/// "RealismH" → ["Realism", "H"]
fn split_camel_case(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in s.chars() {
        if ch.is_uppercase() && !current.is_empty() {
            words.push(title_case(&current));
            current.clear();
        }
        current.push(ch);
    }

    if !current.is_empty() {
        words.push(title_case(&current));
    }

    words
}

/// Title-case a word: first letter uppercase, rest lowercase.
/// Exception: known abbreviations stay uppercase.
fn title_case(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    // Check if it's a known abbreviation that should stay uppercase
    if ABBREVIATIONS.iter().any(|a| a.eq_ignore_ascii_case(word)) {
        return word.to_ascii_uppercase();
    }

    let lower = word.to_ascii_lowercase();
    let mut chars = lower.chars();
    let first = chars.next().unwrap().to_ascii_uppercase();
    let mut result = String::with_capacity(word.len());
    result.push(first);
    result.extend(chars);
    result
}

/// Attempt dictionary-based word splitting for a string without separators.
/// Uses greedy left-to-right matching against the known word list.
/// Unmatched segments ≤ `ABBREVIATION_LENGTH_THRESHOLD` chars are kept uppercase
/// (likely abbreviations like FPS, GI). Longer unknowns are title-cased.
fn dictionary_split(input: &str) -> Vec<String> {
    let lower = input.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut pos = 0;
    let mut words = Vec::new();

    while pos < bytes.len() {
        let remaining = &lower[pos..];

        // Try to match the longest dictionary word at this position
        let mut best_match: Option<&str> = None;
        for &word in DICTIONARY {
            if remaining.starts_with(word) && best_match.is_none_or(|best| word.len() > best.len())
            {
                best_match = Some(word);
            }
        }

        if let Some(matched) = best_match {
            words.push(title_case(matched));
            pos += matched.len();
        } else {
            // No dictionary match — consume characters until we find a match or run out.
            let start = pos;
            pos += 1;
            while pos < bytes.len() {
                let remaining = &lower[pos..];
                if DICTIONARY.iter().any(|w| remaining.starts_with(w)) {
                    break;
                }
                pos += 1;
            }
            let unknown = &input[start..pos];
            if unknown.len() <= ABBREVIATION_LENGTH_THRESHOLD {
                // Short unknown segment — likely an abbreviation, keep uppercase
                words.push(unknown.to_ascii_uppercase());
            } else {
                words.push(title_case(unknown));
            }
        }
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_formatting_codes_basic_color() {
        assert_eq!(strip_formatting_codes("§aHigh"), "High");
    }

    #[test]
    fn test_strip_formatting_codes_multiple_codes() {
        assert_eq!(strip_formatting_codes("§e§lExtreme"), "Extreme");
    }

    #[test]
    fn test_strip_formatting_codes_with_reset() {
        assert_eq!(
            strip_formatting_codes("§eHIGH §r§7- Recommended"),
            "HIGH - Recommended"
        );
    }

    #[test]
    fn test_strip_formatting_codes_unicode_escape() {
        // \u00A7 is the unicode representation of §
        assert_eq!(strip_formatting_codes("\u{00A7}bHigh"), "High");
    }

    #[test]
    fn test_strip_formatting_codes_inline_color_changes() {
        // §bPeriwinkle §7(§5Ultra§7)
        assert_eq!(
            strip_formatting_codes("§bPeriwinkle §7(§5Ultra§7)"),
            "Periwinkle (Ultra)"
        );
    }

    #[test]
    fn test_strip_formatting_codes_no_codes() {
        assert_eq!(strip_formatting_codes("High"), "High");
    }

    #[test]
    fn test_strip_formatting_codes_empty() {
        assert_eq!(strip_formatting_codes(""), "");
    }

    #[test]
    fn test_strip_formatting_codes_obfuscated_text_stripped() {
        // §k produces obfuscated/scrambled text in Minecraft — content is stripped
        assert_eq!(strip_formatting_codes("§kAIWS"), "");
    }

    #[test]
    fn test_strip_formatting_codes_obfuscated_then_normal() {
        // §k starts obfuscated, §r resets to normal
        assert_eq!(strip_formatting_codes("§kXX§r Normal"), " Normal");
    }

    #[test]
    fn test_strip_formatting_codes_complex_label() {
        // §f۞ §6H§figh ۞
        assert_eq!(strip_formatting_codes("§f۞ §6H§figh ۞"), "۞ High ۞");
    }

    #[test]
    fn test_strip_formatting_codes_cjk_brackets_with_obfuscated() {
        // §f【§f§kO §4O§fld TV §f§kO §f】
        // §k enters obfuscated mode (O and space skipped), §4 exits it
        // §4O§fld = "O" in dark red + "ld" in white = "Old"
        assert_eq!(
            strip_formatting_codes("§f【§f§kO §4O§fld TV §f§kO §f】"),
            "【Old TV 】"
        );
    }

    #[test]
    fn test_strip_decorative_ornamental_symbols() {
        // Decorative stripping removes symbols but doesn't trim whitespace
        assert_eq!(strip_decorative_unicode("۞ High ۞"), " High ");
    }

    #[test]
    fn test_strip_decorative_cjk_brackets() {
        assert_eq!(strip_decorative_unicode("【Old TV 】"), "Old TV ");
    }

    #[test]
    fn test_strip_decorative_watch_emoji() {
        assert_eq!(strip_decorative_unicode("⌚ Screenshot"), " Screenshot");
    }

    #[test]
    fn test_strip_decorative_diamond_symbol() {
        // ✦ used in some labels — stripping leaves leading space
        assert_eq!(strip_decorative_unicode("✦ Quality"), " Quality");
    }

    #[test]
    fn test_strip_decorative_preserves_plus() {
        // + symbols are intentional tier markers
        assert_eq!(strip_decorative_unicode("Vanilla +++"), "Vanilla +++");
    }

    #[test]
    fn test_strip_decorative_preserves_ampersand() {
        assert_eq!(strip_decorative_unicode("Old TV B&W"), "Old TV B&W");
    }

    #[test]
    fn test_strip_decorative_no_decorations() {
        assert_eq!(strip_decorative_unicode("Medium High"), "Medium High");
    }

    #[test]
    fn test_strip_metadata_default_parens() {
        assert_eq!(strip_metadata_suffixes("High (Default)"), "High");
    }

    #[test]
    fn test_strip_metadata_recommended_parens() {
        assert_eq!(strip_metadata_suffixes("High (Recommended)"), "High");
    }

    #[test]
    fn test_strip_metadata_good_gpu_parens() {
        assert_eq!(strip_metadata_suffixes("High (Good GPU)"), "High");
    }

    #[test]
    fn test_strip_metadata_performance_parens() {
        assert_eq!(strip_metadata_suffixes("Low (Performance)"), "Low");
    }

    #[test]
    fn test_strip_metadata_balanced_parens() {
        assert_eq!(strip_metadata_suffixes("Medium (Balanced)"), "Medium");
    }

    #[test]
    fn test_strip_metadata_best_quality_parens() {
        assert_eq!(strip_metadata_suffixes("Ultra (Best Quality)"), "Ultra");
    }

    #[test]
    fn test_strip_metadata_integrated_gpu_parens() {
        assert_eq!(strip_metadata_suffixes("Low (Integrated GPU)"), "Low");
    }

    #[test]
    fn test_strip_metadata_standard_parens() {
        assert_eq!(strip_metadata_suffixes("Medium (Standard)"), "Medium");
    }

    #[test]
    fn test_strip_metadata_dof_parens() {
        assert_eq!(strip_metadata_suffixes("High (DoF)"), "High");
    }

    #[test]
    fn test_strip_metadata_dash_recommended() {
        assert_eq!(strip_metadata_suffixes("HIGH - Recommended"), "HIGH");
    }

    #[test]
    fn test_strip_metadata_dash_performance() {
        assert_eq!(strip_metadata_suffixes("LOW - Performance"), "LOW");
    }

    #[test]
    fn test_strip_metadata_dash_maximum_quality() {
        assert_eq!(strip_metadata_suffixes("ULTRA - Maximum Quality"), "ULTRA");
    }

    #[test]
    fn test_strip_metadata_dash_balanced() {
        assert_eq!(strip_metadata_suffixes("MEDIUM - Balanced"), "MEDIUM");
    }

    #[test]
    fn test_strip_metadata_no_suffix() {
        assert_eq!(strip_metadata_suffixes("High"), "High");
    }

    #[test]
    fn test_strip_metadata_preserves_non_metadata_parens() {
        // Periwinkle (Ultra) — "Ultra" is the tier name, not metadata
        assert_eq!(
            strip_metadata_suffixes("Periwinkle (Ultra)"),
            "Periwinkle (Ultra)"
        );
    }

    #[test]
    fn test_collapse_leading_space() {
        assert_eq!(collapse_whitespace(" High"), "High");
    }

    #[test]
    fn test_collapse_trailing_space() {
        assert_eq!(collapse_whitespace("High "), "High");
    }

    #[test]
    fn test_collapse_multiple_interior_spaces() {
        assert_eq!(collapse_whitespace("Medium  High"), "Medium High");
    }

    #[test]
    fn test_collapse_mixed() {
        assert_eq!(collapse_whitespace("  Very   High  "), "Very High");
    }

    #[test]
    fn test_humanize_screaming_snake() {
        assert_eq!(humanize_name("MEDIUM_HIGH"), "Medium High");
    }

    #[test]
    fn test_humanize_lower_snake() {
        assert_eq!(humanize_name("shadowless_low"), "Shadowless Low");
    }

    #[test]
    fn test_humanize_pascal_snake() {
        assert_eq!(
            humanize_name("Just_Colored_Lighting"),
            "Just Colored Lighting"
        );
    }

    #[test]
    fn test_humanize_very_high_underscore() {
        assert_eq!(humanize_name("VERY_HIGH"), "Very High");
    }

    #[test]
    fn test_humanize_very_low_underscore() {
        assert_eq!(humanize_name("VERY_LOW"), "Very Low");
    }

    #[test]
    fn test_humanize_single_word_upper() {
        assert_eq!(humanize_name("HIGH"), "High");
    }

    #[test]
    fn test_humanize_single_word_lower() {
        assert_eq!(humanize_name("high"), "High");
    }

    #[test]
    fn test_humanize_single_word_pascal() {
        assert_eq!(humanize_name("High"), "High");
    }

    #[test]
    fn test_humanize_camel_deep_fried() {
        assert_eq!(humanize_name("DeepFried"), "Deep Fried");
    }

    #[test]
    fn test_humanize_camel_realism_h() {
        assert_eq!(humanize_name("RealismH"), "Realism H");
    }

    #[test]
    fn test_humanize_camel_cel_m() {
        assert_eq!(humanize_name("CelM"), "Cel M");
    }

    #[test]
    fn test_humanize_camel_cel_s() {
        assert_eq!(humanize_name("CelS"), "Cel S");
    }

    #[test]
    fn test_humanize_dramatic_pbr() {
        assert_eq!(humanize_name("Dramatic_Pbr"), "Dramatic Pbr");
    }

    #[test]
    fn test_humanize_fancy_fast() {
        assert_eq!(humanize_name("Fancy_Fast"), "Fancy Fast");
    }

    #[test]
    fn test_humanize_potato_shadows() {
        assert_eq!(humanize_name("Potato_Shadows"), "Potato Shadows");
    }

    #[test]
    fn test_humanize_veryhigh_no_sep() {
        assert_eq!(humanize_name("VERYHIGH"), "Very High");
    }

    #[test]
    fn test_humanize_verylow_no_sep() {
        assert_eq!(humanize_name("VERYLOW"), "Very Low");
    }

    #[test]
    fn test_humanize_oldtv() {
        assert_eq!(humanize_name("OLDTV"), "Old TV");
    }

    #[test]
    fn test_humanize_oldtvbw() {
        assert_eq!(humanize_name("OLDTVBW"), "Old TV BW");
    }

    #[test]
    fn test_humanize_extremeplus() {
        assert_eq!(humanize_name("extremeplus"), "Extreme Plus");
    }

    #[test]
    fn test_humanize_rtx_stays_upper() {
        assert_eq!(humanize_name("RTX"), "RTX");
    }

    #[test]
    fn test_humanize_rtx_low() {
        assert_eq!(humanize_name("RTX_LOW"), "RTX Low");
    }

    #[test]
    fn test_humanize_nvg() {
        assert_eq!(humanize_name("NVG"), "NVG");
    }

    #[test]
    fn test_humanize_pvp() {
        assert_eq!(humanize_name("PVP"), "PVP");
    }

    #[test]
    fn test_humanize_ibm() {
        assert_eq!(humanize_name("IBM"), "IBM");
    }

    #[test]
    fn test_humanize_vhs() {
        assert_eq!(humanize_name("VHS"), "VHS");
    }

    #[test]
    fn test_humanize_dof() {
        // Intentionally not split — it's an abbreviation
        assert_eq!(humanize_name("DOF"), "DOF");
    }

    #[test]
    fn test_normalize_simple_label() {
        assert_eq!(normalize_display_name("HIGH", Some("§eHigh")), "High");
    }

    #[test]
    fn test_normalize_label_with_default_suffix() {
        assert_eq!(
            normalize_display_name("HIGH", Some("§bHigh (Default)")),
            "High"
        );
    }

    #[test]
    fn test_normalize_label_with_recommended_dash() {
        assert_eq!(
            normalize_display_name("HIGH", Some("§eHIGH §7- Recommended")),
            "HIGH"
        );
    }

    #[test]
    fn test_normalize_creative_label_preserved() {
        // Author chose "Caveman PC" for their potato profile
        assert_eq!(
            normalize_display_name("POTATO", Some("§aCaveman PC")),
            "Caveman PC"
        );
    }

    #[test]
    fn test_normalize_vanilla_plus_tiers() {
        assert_eq!(
            normalize_display_name("ONE", Some("§fVanilla §b+")),
            "Vanilla +"
        );
        assert_eq!(
            normalize_display_name("TWO", Some("§fVanilla §b++")),
            "Vanilla ++"
        );
        assert_eq!(
            normalize_display_name("THREE", Some("§fVanilla §b+++")),
            "Vanilla +++"
        );
    }

    #[test]
    fn test_normalize_streamer_tiers() {
        assert_eq!(
            normalize_display_name("CONTENT_ONE", Some("§fStreamer §b+")),
            "Streamer +"
        );
    }

    #[test]
    fn test_normalize_ornamental_label() {
        // §f۞ §6H§figh ۞
        assert_eq!(
            normalize_display_name("HIGH", Some("§f۞ §6H§figh ۞")),
            "High"
        );
    }

    #[test]
    fn test_normalize_cjk_bracket_label() {
        // §f【 §4O§fld TV §f】
        assert_eq!(
            normalize_display_name("OLDTV", Some("§f【 §4O§fld TV §f】")),
            "Old TV"
        );
    }

    #[test]
    fn test_normalize_screenshot_with_emoji() {
        // §c⌚§r §5§lScreenshot
        assert_eq!(
            normalize_display_name("SCREENSHOT", Some("§c⌚§r §5§lScreenshot")),
            "Screenshot"
        );
    }

    #[test]
    fn test_normalize_periwinkle_subpack() {
        assert_eq!(
            normalize_display_name("BHQ", Some("§bPeriwinkle §7(§5Ultra§7)")),
            "Periwinkle (Ultra)"
        );
    }

    #[test]
    fn test_normalize_rose_quartz_subpack() {
        assert_eq!(
            normalize_display_name("PHQ", Some("§dRose Quartz §7(§5Ultra§7)")),
            "Rose Quartz (Ultra)"
        );
    }

    #[test]
    fn test_normalize_lemon_subpack() {
        assert_eq!(
            normalize_display_name("YHQ", Some("§eLemon §7(§5Ultra§7)")),
            "Lemon (Ultra)"
        );
    }

    #[test]
    fn test_normalize_vibrantly_visual_strips_default() {
        assert_eq!(
            normalize_display_name("HIGH", Some("§eVibrantly Visual (Default)")),
            "Vibrantly Visual"
        );
    }

    #[test]
    fn test_normalize_thermal_imager() {
        assert_eq!(
            normalize_display_name("THERMAL_IMAGER", Some("§fThermal imager")),
            "Thermal imager"
        );
    }

    #[test]
    fn test_normalize_night_vision_goggles() {
        assert_eq!(
            normalize_display_name("BLUE_NVG", Some("§bBlue Night Vision Goggles")),
            "Blue Night Vision Goggles"
        );
    }

    #[test]
    fn test_normalize_optifine_compat() {
        assert_eq!(
            normalize_display_name("OPTIFINE_COMPAT", Some("Optifine Compatibility")),
            "Optifine Compatibility"
        );
    }

    #[test]
    fn test_normalize_vanilla_rtx() {
        assert_eq!(
            normalize_display_name("VANILLA", Some("§aVanilla / RTX-ish")),
            "Vanilla / RTX-ish"
        );
    }

    #[test]
    fn test_normalize_reimagined() {
        assert_eq!(
            normalize_display_name("REIMAGINED", Some("§5§lReima§d§lgined")),
            "Reimagined"
        );
    }

    #[test]
    fn test_normalize_no_label_simple() {
        assert_eq!(normalize_display_name("HIGH", None), "High");
    }

    #[test]
    fn test_normalize_no_label_underscore() {
        assert_eq!(normalize_display_name("MEDIUM_HIGH", None), "Medium High");
    }

    #[test]
    fn test_normalize_no_label_lower_snake() {
        assert_eq!(
            normalize_display_name("shadowless_low", None),
            "Shadowless Low"
        );
    }

    #[test]
    fn test_normalize_no_label_pascal() {
        assert_eq!(normalize_display_name("DeepFried", None), "Deep Fried");
    }

    #[test]
    fn test_normalize_no_label_camel() {
        assert_eq!(normalize_display_name("RealismH", None), "Realism H");
    }

    #[test]
    fn test_normalize_no_label_veryhigh() {
        assert_eq!(normalize_display_name("VERYHIGH", None), "Very High");
    }

    #[test]
    fn test_normalize_no_label_minecraft_nostalgic() {
        assert_eq!(
            normalize_display_name("Minecraft_Nostalgic", None),
            "Minecraft Nostalgic"
        );
    }

    #[test]
    fn test_normalize_no_label_no_effects() {
        assert_eq!(normalize_display_name("no_effects", None), "No Effects");
    }

    #[test]
    fn test_normalize_no_label_no_effects_shadows() {
        assert_eq!(
            normalize_display_name("no_effects_shadows", None),
            "No Effects Shadows"
        );
    }

    #[test]
    fn test_normalize_empty_label_falls_back() {
        assert_eq!(normalize_display_name("MEDIUM", Some("")), "Medium");
    }

    #[test]
    fn test_normalize_whitespace_only_label_falls_back() {
        assert_eq!(normalize_display_name("LOW", Some("  ")), "Low");
    }

    #[test]
    fn test_normalize_shader_version_label_placeholder() {
        // SHADER_VERSION_LABEL is a placeholder that was never resolved
        // With no label, it should still produce something reasonable
        // or be caught by a manual override
        let result = normalize_display_name("SHADER_VERSION_LABEL", None);
        // Should NOT produce "Shader Version Label" since that's meaningless
        // The override should map it to something else or empty → but we need
        // a fallback. Since it's truly bogus, the override should produce
        // a reasonable default like the humanized name minus the known-bad pattern.
        // For now: just verify it doesn't contain "SHADER_VERSION_LABEL"
        assert_ne!(result, "SHADER_VERSION_LABEL");
        assert_ne!(result, "Shader Version Label");
    }

    #[test]
    fn test_normalize_label_only_formatting_codes() {
        // A label that's entirely formatting codes with no text
        assert_eq!(normalize_display_name("DEFAULT", Some("§a§l")), "Default");
    }

    #[test]
    fn test_normalize_label_only_decorations() {
        // A label that's only decorative symbols after stripping codes
        assert_eq!(normalize_display_name("HIGH", Some("§f۞ §f۞")), "High");
    }

    #[test]
    fn test_normalize_leading_space_in_label() {
        // §a High — space after formatting code
        assert_eq!(normalize_display_name("HIGH", Some("§a High")), "High");
    }

    #[test]
    fn test_normalize_leading_space_potato() {
        assert_eq!(
            normalize_display_name("POTATO", Some("§4 Potato")),
            "Potato"
        );
    }

    #[test]
    fn test_normalize_borderland_obfuscated() {
        // §f۞ §f§kO §4B§forderland §4+§f §kO §f۞
        assert_eq!(
            normalize_display_name("BORDERLAND", Some("§f۞ §f§kO §4B§forderland §4+§f §kO §f۞")),
            "Borderland +"
        );
    }

    #[test]
    fn test_normalize_cinematic_on_ultra() {
        // Some authors label ULTRA as "Cinematic"
        assert_eq!(
            normalize_display_name("ULTRA", Some("§dCinematic")),
            "Cinematic"
        );
    }

    #[test]
    fn test_normalize_best_performance_label() {
        // VERYLOW labeled as "Best Performance"
        assert_eq!(
            normalize_display_name("VERYLOW", Some("§cBest Performance")),
            "Best Performance"
        );
    }

    #[test]
    fn test_normalize_dedicated_graphics() {
        assert_eq!(
            normalize_display_name("dgpu", Some("§bDedicated Graphics")),
            "Dedicated Graphics"
        );
    }

    #[test]
    fn test_normalize_integrated_graphics() {
        assert_eq!(
            normalize_display_name("igpu", Some("§aIntegrated Graphics")),
            "Integrated Graphics"
        );
    }

    #[test]
    fn test_normalize_max_allcaps_styled() {
        assert_eq!(normalize_display_name("max", Some("§5§l§oMAX")), "MAX");
    }

    #[test]
    fn test_normalize_no_effects_bold() {
        assert_eq!(
            normalize_display_name("no_effects", Some("§lNo effects")),
            "No effects"
        );
    }

    #[test]
    fn test_normalize_only_shadows() {
        assert_eq!(
            normalize_display_name("no_effects_shadows", Some("Only shadows")),
            "Only shadows"
        );
    }

    #[test]
    fn test_normalize_original_prefix() {
        // Some labels say "Original High", "Original Medium" etc.
        assert_eq!(
            normalize_display_name("High", Some("Original High")),
            "Original High"
        );
    }

    #[test]
    fn test_normalize_ultra_maximum_quality_formatted() {
        // §6§lULTRA §r§7- Maximum Quality
        assert_eq!(
            normalize_display_name("ULTRA", Some("§6§lULTRA §r§7- Maximum Quality")),
            "ULTRA"
        );
    }

    #[test]
    fn test_normalize_medium_balanced_formatted() {
        // §aMEDIUM §r§7- Balanced
        assert_eq!(
            normalize_display_name("MEDIUM", Some("§aMEDIUM §r§7- Balanced")),
            "MEDIUM"
        );
    }

    #[test]
    fn test_normalize_low_performance_formatted() {
        // §cLOW §r§7- Performance
        assert_eq!(
            normalize_display_name("LOW", Some("§cLOW §r§7- Performance")),
            "LOW"
        );
    }

    #[test]
    fn test_normalize_medium_default_parens() {
        assert_eq!(
            normalize_display_name("MEDIUM", Some("Medium (Default)")),
            "Medium"
        );
    }

    #[test]
    fn test_normalize_default_name_with_styled_label() {
        // DEFAULT with §a§lDEFAULT label
        assert_eq!(
            normalize_display_name("DEFAULT", Some("§a§lDEFAULT")),
            "DEFAULT"
        );
    }

    #[test]
    fn test_normalize_render_name() {
        assert_eq!(
            normalize_display_name("RENDER", Some("§a§lRENDER")),
            "RENDER"
        );
    }

    #[test]
    fn test_normalize_timelapse() {
        assert_eq!(
            normalize_display_name("TIMELAPSE", Some("§6Timelapse")),
            "Timelapse"
        );
    }

    #[test]
    fn test_normalize_grey_weather() {
        assert_eq!(
            normalize_display_name("GREY_W", Some("§7Grey weather")),
            "Grey weather"
        );
    }

    #[test]
    fn test_normalize_medium_low() {
        assert_eq!(
            normalize_display_name("MEDIUM_LOW", Some("§aMedium Low")),
            "Medium Low"
        );
    }

    #[test]
    fn test_normalize_minimalism_typo() {
        // MIMIMALISM is a typo, but label correctly says Minimalism
        assert_eq!(
            normalize_display_name("MIMIMALISM", Some("§bMinimalism")),
            "Minimalism"
        );
    }

    #[test]
    fn test_normalize_standart_typo() {
        // STANDART is a typo, but label correctly says Standard
        assert_eq!(
            normalize_display_name("STANDART", Some("§aStandard")),
            "Standard"
        );
    }

    #[test]
    fn test_humanize_medium_no_sep() {
        // Just "MEDIUM" — single dictionary word
        assert_eq!(humanize_name("MEDIUM"), "Medium");
    }

    #[test]
    fn test_humanize_ultra_no_sep() {
        assert_eq!(humanize_name("ULTRA"), "Ultra");
    }

    #[test]
    fn test_humanize_extreme_no_sep() {
        assert_eq!(humanize_name("EXTREME"), "Extreme");
    }

    #[test]
    fn test_humanize_potato_no_sep() {
        assert_eq!(humanize_name("POTATO"), "Potato");
    }

    #[test]
    fn test_humanize_toaster_lower() {
        assert_eq!(humanize_name("toaster"), "Toaster");
    }

    #[test]
    fn test_humanize_vanilla_lower() {
        assert_eq!(humanize_name("vanilla"), "Vanilla");
    }

    #[test]
    fn test_humanize_dgpu() {
        // "dgpu" — not in dictionary, keep as-is title-cased
        assert_eq!(humanize_name("dgpu"), "Dgpu");
    }

    #[test]
    fn test_humanize_igpu() {
        assert_eq!(humanize_name("igpu"), "Igpu");
    }

    #[test]
    fn test_normalize_bogus_name_with_valid_label_uses_label() {
        // Bliss sets name=SHADER_VERSION_LABEL but label="Bliss V2.1.2"
        assert_eq!(
            normalize_display_name("SHADER_VERSION_LABEL", Some("Bliss V2.1.2")),
            "Bliss V2.1.2"
        );
    }

    #[test]
    fn test_normalize_bogus_name_with_formatted_label_uses_label() {
        assert_eq!(
            normalize_display_name("SHADER_VERSION_LABEL", Some("§aDefault Profile")),
            "Default Profile"
        );
    }

    #[test]
    fn test_normalize_bogus_name_without_label_returns_default() {
        assert_eq!(
            normalize_display_name("SHADER_VERSION_LABEL", None),
            "Default"
        );
    }

    #[test]
    fn test_normalize_bogus_name_with_empty_label_returns_default() {
        assert_eq!(
            normalize_display_name("SHADER_VERSION_LABEL", Some("")),
            "Default"
        );
    }

    #[test]
    fn test_humanize_sspt_stays_uppercase() {
        // SSPT = Screen Space Path Tracing, a rendering abbreviation
        assert_eq!(humanize_name("SSPT"), "SSPT");
    }

    #[test]
    fn test_humanize_jpl_stays_uppercase() {
        assert_eq!(humanize_name("JPL"), "JPL");
    }

    #[test]
    fn test_humanize_highfps_splits_with_uppercase_abbrev() {
        // FPS is a common abbreviation; should stay uppercase
        assert_eq!(humanize_name("HIGHFPS"), "High FPS");
    }

    #[test]
    fn test_dictionary_split_unknown_short_segment_stays_uppercase() {
        // "NOFX" — "no" matches but "fx" is short unknown → uppercase
        let result = dictionary_split("NOFX");
        assert_eq!(result, vec!["No", "FX"]);
    }

    #[test]
    fn test_dictionary_split_unknown_long_segment_title_cased() {
        // Longer unknown segments get title-cased (likely a word, not abbreviation)
        let result = dictionary_split("COMPLEMENTARY");
        assert_eq!(result, vec!["Complementary"]);
    }

    #[test]
    fn test_humanize_fps_alone() {
        assert_eq!(humanize_name("FPS"), "FPS");
    }

    #[test]
    fn test_humanize_hd_alone() {
        assert_eq!(humanize_name("HD"), "HD");
    }

    #[test]
    fn test_humanize_hq_alone() {
        assert_eq!(humanize_name("HQ"), "HQ");
    }

    #[test]
    fn test_humanize_ssr_alone() {
        assert_eq!(humanize_name("SSR"), "SSR");
    }

    #[test]
    fn test_humanize_ssao_alone() {
        assert_eq!(humanize_name("SSAO"), "SSAO");
    }

    #[test]
    fn test_humanize_taa_alone() {
        assert_eq!(humanize_name("TAA"), "TAA");
    }

    #[test]
    fn test_humanize_gi_alone() {
        assert_eq!(humanize_name("GI"), "GI");
    }

    #[test]
    fn test_humanize_rt_alone() {
        assert_eq!(humanize_name("RT"), "RT");
    }

    #[test]
    fn test_humanize_lq_alone() {
        assert_eq!(humanize_name("LQ"), "LQ");
    }

    #[test]
    fn test_humanize_rtx_high_mixed() {
        // RTXHigh should split as "RTX" + "High", not "Rtxhigh"
        assert_eq!(humanize_name("RTXHigh"), "RTX High");
    }

    #[test]
    fn test_humanize_ssao_enabled_mixed() {
        assert_eq!(humanize_name("SSAOEnabled"), "SSAO Enabled");
    }

    #[test]
    fn test_humanize_hdr_quality_mixed() {
        assert_eq!(humanize_name("HDRQuality"), "HDR Quality");
    }

    #[test]
    fn test_strip_metadata_multiple_suffixes() {
        // Both dash and paren suffixes present
        assert_eq!(
            strip_metadata_suffixes("High (Default) - Recommended"),
            "High"
        );
    }

    #[test]
    fn test_strip_metadata_double_parens() {
        // Two paren suffixes — both get stripped by the loop
        assert_eq!(
            strip_metadata_suffixes("Ultra (Best Quality) (Default)"),
            "Ultra"
        );
    }

    #[test]
    fn test_strip_metadata_lowercase_default() {
        assert_eq!(strip_metadata_suffixes("High (default)"), "High");
    }

    #[test]
    fn test_strip_metadata_uppercase_recommended() {
        assert_eq!(strip_metadata_suffixes("High (RECOMMENDED)"), "High");
    }

    #[test]
    fn test_strip_metadata_dash_lowercase_performance() {
        assert_eq!(strip_metadata_suffixes("LOW - performance"), "LOW");
    }

    #[test]
    fn test_normalize_empty_name_no_label() {
        // Empty name should NOT produce empty string (sentinel collision)
        let result = normalize_display_name("", None);
        assert!(
            !result.is_empty(),
            "Empty name must not produce empty display_name (sentinel collision)"
        );
    }

    #[test]
    fn test_normalize_underscore_only_name() {
        let result = normalize_display_name("___", None);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_strip_formatting_codes_lone_section_sign_at_end() {
        // Malformed: trailing § with no following character — silently dropped
        assert_eq!(strip_formatting_codes("High§"), "High");
    }

    #[test]
    fn test_strip_formatting_codes_hex_color_1_16() {
        // §x§1§2§3§4§5§6 = 24-bit color in Minecraft 1.16+
        assert_eq!(strip_formatting_codes("§x§1§2§3§4§5§6Hello"), "Hello");
    }

    #[test]
    fn test_strip_formatting_codes_ampersand_not_stripped() {
        // & formatting codes are a server plugin convention, NOT shader convention
        // We deliberately don't strip them
        assert_eq!(strip_formatting_codes("&aHigh"), "&aHigh");
    }

    #[test]
    fn test_normalize_cjk_label_falls_back_to_name() {
        // Chinese labels get stripped by should_keep_char — falls back to humanize
        assert_eq!(normalize_display_name("QUALITY", Some("质量")), "Quality");
    }

    #[test]
    fn test_normalize_cyrillic_label_falls_back_to_name() {
        assert_eq!(normalize_display_name("LOW", Some("§eНизкие")), "Low");
    }

    #[test]
    fn test_strip_decorative_zero_width_space() {
        // Zero-width space (U+200B) between letters — should be stripped
        assert_eq!(
            strip_decorative_unicode("H\u{200B}i\u{200B}g\u{200B}h"),
            "High"
        );
    }

    #[test]
    fn test_strip_decorative_preserves_accented_latin() {
        // Accented characters in Latin-1 Supplement should be preserved
        assert_eq!(strip_decorative_unicode("Café Über"), "Café Über");
    }

    #[test]
    fn test_strip_decorative_extended_latin_stripped() {
        // Extended Latin characters (U+0100+) get stripped — design decision
        // Polish ą (U+0105) is outside our kept range
        assert_eq!(strip_decorative_unicode("Świat"), "wiat");
    }

    #[test]
    fn test_normalize_label_only_whitespace_after_stripping() {
        // Label is formatting codes + spaces → collapses to empty → fallback
        assert_eq!(normalize_display_name("TEST", Some("§a  §b  ")), "Test");
    }

    #[test]
    fn test_collapse_whitespace_nbsp() {
        // Non-breaking space should be treated as whitespace
        assert_eq!(collapse_whitespace("Very\u{00A0}High"), "Very High");
    }

    #[test]
    fn test_humanize_v2_high() {
        assert_eq!(humanize_name("V2_HIGH"), "V2 High");
    }

    #[test]
    fn test_humanize_1080p() {
        assert_eq!(humanize_name("1080p"), "1080p");
    }

    #[test]
    fn test_humanize_hyphenated() {
        assert_eq!(humanize_name("photo-realistic"), "Photo-realistic");
    }

    #[test]
    fn test_humanize_plus_plus_plus() {
        // Plus symbols as tier markers — keep as-is
        assert_eq!(humanize_name("+++"), "+++");
    }

    #[test]
    fn test_humanize_dotted_version() {
        assert_eq!(humanize_name("v1.0"), "V1.0");
    }

    #[test]
    fn test_normalize_output_never_contains_section_sign() {
        let cases = vec![
            ("HIGH", Some("§eHigh")),
            ("LOW", Some("§c§lLow §r§7(Performance)")),
            ("X", Some("§x§1§2§3§4§5§6Test")),
            ("K", Some("§kObfuscated§rVisible")),
        ];
        for (name, label) in cases {
            let result = normalize_display_name(name, label);
            assert!(
                !result.contains('\u{00A7}'),
                "Output for name={name:?} label={label:?} contains §: {result:?}"
            );
        }
    }

    #[test]
    fn test_normalize_output_never_has_leading_trailing_whitespace() {
        let cases = vec![
            ("HIGH", Some("§a High")),
            ("LOW", Some("§f۞ §6L§flow ۞")),
            ("X", Some("  §eTest  ")),
            ("Y", None),
        ];
        for (name, label) in cases {
            let result = normalize_display_name(name, label);
            assert_eq!(
                result.trim(),
                result,
                "Output for name={name:?} label={label:?} has leading/trailing whitespace: {result:?}"
            );
        }
    }

    #[test]
    fn test_normalize_output_never_has_consecutive_spaces() {
        let cases = vec![
            ("X", Some("§a  Two  Spaces")),
            ("MEDIUM_HIGH", None),
            ("VERYHIGH", None),
        ];
        for (name, label) in cases {
            let result = normalize_display_name(name, label);
            assert!(
                !result.contains("  "),
                "Output for name={name:?} label={label:?} has consecutive spaces: {result:?}"
            );
        }
    }

    #[test]
    fn test_strip_surrounding_square_brackets() {
        assert_eq!(strip_surrounding_punctuation("[Default]"), "Default");
    }

    #[test]
    fn test_strip_surrounding_curly_brackets() {
        assert_eq!(strip_surrounding_punctuation("{High}"), "High");
    }

    #[test]
    fn test_strip_surrounding_angle_brackets() {
        assert_eq!(strip_surrounding_punctuation("<Ultra>"), "Ultra");
    }

    #[test]
    fn test_strip_surrounding_parens() {
        assert_eq!(strip_surrounding_punctuation("(Low)"), "Low");
    }

    #[test]
    fn test_strip_surrounding_dashes() {
        assert_eq!(strip_surrounding_punctuation("- High -"), "High");
    }

    #[test]
    fn test_strip_surrounding_dots() {
        assert_eq!(strip_surrounding_punctuation("..Medium.."), "Medium");
    }

    #[test]
    fn test_strip_surrounding_mixed_punctuation() {
        assert_eq!(strip_surrounding_punctuation("-- [High] --"), "High");
    }

    #[test]
    fn test_strip_surrounding_preserves_plus() {
        // + is a meaningful tier marker, not decoration
        assert_eq!(strip_surrounding_punctuation("Vanilla +++"), "Vanilla +++");
    }

    #[test]
    fn test_strip_surrounding_preserves_interior_parens() {
        // Parens in the middle are content (subpack names), not wrapping
        assert_eq!(
            strip_surrounding_punctuation("Periwinkle (Ultra)"),
            "Periwinkle (Ultra)"
        );
    }

    #[test]
    fn test_strip_surrounding_preserves_interior_hyphens() {
        assert_eq!(strip_surrounding_punctuation("RTX-ish"), "RTX-ish");
    }

    #[test]
    fn test_strip_surrounding_tilde_decoration() {
        assert_eq!(strip_surrounding_punctuation("~ Fancy ~"), "Fancy");
    }

    #[test]
    fn test_strip_surrounding_pipe_decoration() {
        assert_eq!(strip_surrounding_punctuation("| High |"), "High");
    }

    #[test]
    fn test_strip_surrounding_star_decoration() {
        assert_eq!(strip_surrounding_punctuation("* Ultra *"), "Ultra");
    }

    #[test]
    fn test_strip_surrounding_only_punctuation_returns_empty() {
        assert_eq!(strip_surrounding_punctuation("---"), "");
    }

    #[test]
    fn test_strip_surrounding_no_punctuation() {
        assert_eq!(strip_surrounding_punctuation("High"), "High");
    }

    #[test]
    fn test_normalize_label_with_brackets_stripped() {
        // Full pipeline: label has brackets that should be stripped
        assert_eq!(
            normalize_display_name("DEFAULT", Some("[Default]")),
            "Default"
        );
    }

    #[test]
    fn test_normalize_label_with_dash_decoration() {
        assert_eq!(normalize_display_name("HIGH", Some("§e-- High --")), "High");
    }

    #[test]
    fn test_normalize_label_trailing_plus_preserved() {
        // "Vanilla +" should keep the +
        assert_eq!(
            normalize_display_name("ONE", Some("Vanilla +")),
            "Vanilla +"
        );
    }
}
