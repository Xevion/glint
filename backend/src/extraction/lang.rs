use std::collections::HashMap;

use super::ExtractionError;
use super::properties::parse_properties;

/// Parsed data from a shader pack's .lang file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LangData {
    /// profile.<NAME> -> display label
    pub profile_labels: HashMap<String, String>,
    /// option.<NAME> -> display label
    pub option_labels: HashMap<String, String>,
    /// option.<NAME>.comment -> tooltip text
    pub option_comments: HashMap<String, String>,
    /// screen.<NAME> -> display label
    pub screen_labels: HashMap<String, String>,
    /// value.<OPTION>.<VALUE> -> display label
    pub value_labels: HashMap<(String, String), String>,
}

/// Parse a .lang file into structured label data.
pub fn parse_lang(input: &str) -> Result<LangData, ExtractionError> {
    let pairs = parse_properties(input)?;
    let mut data = LangData::default();

    for (key, value) in pairs {
        if let Some(name) = key.strip_prefix("profile.") {
            data.profile_labels.insert(name.to_string(), value);
        } else if let Some(rest) = key.strip_prefix("option.") {
            // Check .comment suffix BEFORE plain option
            if let Some(name) = rest.strip_suffix(".comment") {
                data.option_comments.insert(name.to_string(), value);
            } else {
                data.option_labels.insert(rest.to_string(), value);
            }
        } else if let Some(name) = key.strip_prefix("screen.") {
            data.screen_labels.insert(name.to_string(), value);
        } else if let Some(rest) = key.strip_prefix("value.") {
            // value.<OPTION>.<VALUE> — split at first dot
            if let Some((option, val)) = rest.split_once('.') {
                data.value_labels
                    .insert((option.to_string(), val.to_string()), value);
            }
        }
        // Everything else: skip
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_labels() {
        let input = "profile.LOW=Low Quality\nprofile.HIGH=High Quality\n";
        let result = parse_lang(input).unwrap();
        assert_eq!(
            result.profile_labels.get("LOW"),
            Some(&"Low Quality".into())
        );
        assert_eq!(
            result.profile_labels.get("HIGH"),
            Some(&"High Quality".into())
        );
    }

    #[test]
    fn test_option_labels_and_comments() {
        let input = "\
            option.SHADOWS=Shadow Quality\n\
            option.SHADOWS.comment=Controls shadow rendering quality\n\
            option.DOF=Depth of Field\n";
        let result = parse_lang(input).unwrap();
        assert_eq!(
            result.option_labels.get("SHADOWS"),
            Some(&"Shadow Quality".into())
        );
        assert_eq!(
            result.option_comments.get("SHADOWS"),
            Some(&"Controls shadow rendering quality".into())
        );
        assert_eq!(
            result.option_labels.get("DOF"),
            Some(&"Depth of Field".into())
        );
    }

    #[test]
    fn test_screen_labels() {
        let input = "screen.POST=Post Processing\nscreen.LIGHTING=Lighting\n";
        let result = parse_lang(input).unwrap();
        assert_eq!(
            result.screen_labels.get("POST"),
            Some(&"Post Processing".into())
        );
    }

    #[test]
    fn test_value_labels() {
        let input =
            "value.SHADOW_QUALITY.0=Off\nvalue.SHADOW_QUALITY.1=Low\nvalue.SHADOW_QUALITY.2=High\n";
        let result = parse_lang(input).unwrap();
        assert_eq!(
            result
                .value_labels
                .get(&("SHADOW_QUALITY".into(), "0".into())),
            Some(&"Off".into())
        );
    }

    #[test]
    fn test_empty_input() {
        let result = parse_lang("").unwrap();
        assert_eq!(result, LangData::default());
    }

    #[test]
    fn test_unrelated_keys_ignored() {
        let input = "some.random.key=value\nof.no.interest=whatever\n";
        let result = parse_lang(input).unwrap();
        assert_eq!(result, LangData::default());
    }
}
