use std::collections::HashMap;

use super::ExtractionError;

/// A single option setting within a profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileOption {
    /// Enable a toggle (bare `KEY` in profile line)
    Enable(String),
    /// Disable a toggle (`!KEY` in profile line)
    Disable(String),
    /// Set a value (`KEY=VALUE` in profile line)
    Set(String, String),
}

/// A parsed profile from shaders.properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedProfile {
    pub name: String,
    pub inherits_from: Option<String>,
    pub options: Vec<ProfileOption>,
    pub sort_order: i32,
}

/// A screen definition from shaders.properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenDefinition {
    pub name: String,
    pub options: Vec<ScreenEntry>,
    pub columns: Option<i32>,
}

/// An entry in a settings screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenEntry {
    /// A regular option toggle/slider
    Option(String),
    /// A sub-screen link: `[ScreenName]`
    SubScreen(String),
    /// An empty slot: `<empty>`
    Empty,
}

/// Pipeline feature flags and booleans extracted from shaders.properties.
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineFeatures {
    pub booleans: HashMap<String, bool>,
    pub cloud_setting: Option<String>,
}

/// Full extraction result from shaders.properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderPropertiesData {
    pub profiles: Vec<ParsedProfile>,
    pub screens: Vec<ScreenDefinition>,
    pub main_screen: Option<ScreenDefinition>,
    pub iris_features_required: Vec<String>,
    pub iris_features_optional: Vec<String>,
    pub pipeline: PipelineFeatures,
    pub slider_options: Vec<String>,
}

/// Known pipeline boolean keys (excluding `clouds` which is handled specially).
const PIPELINE_BOOL_KEYS: &[&str] = &[
    "separateAo",
    "sun",
    "moon",
    "oldLighting",
    "shadowTerrain",
    "shadowEntities",
    "shadowPlayer",
    "shadowBlockEntities",
    "underwaterOverlay",
    "vignette",
    "stars",
    "sky",
    "weather",
    "rainDepth",
    "beaconBeamDepth",
    "separateEntityDraws",
    "frustumCulling",
    "occlusionCulling",
];

/// Interpret raw properties key-value pairs into structured shader properties.
pub fn interpret_shader_properties(
    pairs: &[(String, String)],
) -> Result<ShaderPropertiesData, ExtractionError> {
    let mut profiles = Vec::new();
    let mut screens: Vec<ScreenDefinition> = Vec::new();
    let mut main_screen: Option<ScreenDefinition> = None;
    let mut iris_features_required = Vec::new();
    let mut iris_features_optional = Vec::new();
    let mut pipeline = PipelineFeatures {
        booleans: HashMap::new(),
        cloud_setting: None,
    };
    let mut slider_options = Vec::new();

    // Track profile count for sort_order
    let mut profile_index: i32 = 0;

    for (key, value) in pairs {
        if let Some(name) = key.strip_prefix("profile.") {
            let profile = parse_profile(name, value, profile_index);
            profiles.push(profile);
            profile_index += 1;
        } else if key == "screen" {
            // Main screen definition (no sub-name)
            let entries = parse_screen_entries(value);
            match main_screen.as_mut() {
                Some(ms) => ms.options = entries,
                None => {
                    main_screen = Some(ScreenDefinition {
                        name: String::new(),
                        options: entries,
                        columns: None,
                    });
                }
            }
        } else if key == "screen.columns" {
            // Main screen column count
            let cols = value.trim().parse::<i32>().ok();
            match main_screen.as_mut() {
                Some(ms) => ms.columns = cols,
                None => {
                    main_screen = Some(ScreenDefinition {
                        name: String::new(),
                        options: Vec::new(),
                        columns: cols,
                    });
                }
            }
        } else if let Some(rest) = key.strip_prefix("screen.") {
            // Named sub-screen or sub-screen columns
            if let Some(screen_name) = rest.strip_suffix(".columns") {
                // screen.<NAME>.columns
                let cols = value.trim().parse::<i32>().ok();
                if let Some(screen) = screens.iter_mut().find(|s| s.name == screen_name) {
                    screen.columns = cols;
                } else {
                    screens.push(ScreenDefinition {
                        name: screen_name.to_string(),
                        options: Vec::new(),
                        columns: cols,
                    });
                }
            } else {
                // screen.<NAME> — sub-screen entries
                let entries = parse_screen_entries(value);
                if let Some(screen) = screens.iter_mut().find(|s| s.name == rest) {
                    screen.options = entries;
                } else {
                    screens.push(ScreenDefinition {
                        name: rest.to_string(),
                        options: entries,
                        columns: None,
                    });
                }
            }
        } else if key == "iris.features.required" {
            iris_features_required = split_whitespace_tokens(value);
        } else if key == "iris.features.optional" {
            iris_features_optional = split_whitespace_tokens(value);
        } else if key == "sliders" {
            slider_options = split_whitespace_tokens(value);
        } else if key == "clouds" {
            pipeline.cloud_setting = Some(value.clone());
        } else if PIPELINE_BOOL_KEYS.contains(&key.as_str()) {
            pipeline
                .booleans
                .insert(key.clone(), value.trim() == "true");
        }
    }

    Ok(ShaderPropertiesData {
        profiles,
        screens,
        main_screen,
        iris_features_required,
        iris_features_optional,
        pipeline,
        slider_options,
    })
}

/// Parse a single profile definition from its value string.
fn parse_profile(name: &str, value: &str, sort_order: i32) -> ParsedProfile {
    let tokens: Vec<&str> = value.split_whitespace().collect();
    let mut inherits_from = None;
    let mut options = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        if i == 0 && token.starts_with("profile.") {
            // Inheritance: first token is `profile.<OTHER>`
            if let Some(parent) = token.strip_prefix("profile.") {
                inherits_from = Some(parent.to_string());
            }
        } else if let Some(rest) = token.strip_prefix('!') {
            options.push(ProfileOption::Disable(rest.to_string()));
        } else if let Some((k, v)) = token.split_once('=') {
            options.push(ProfileOption::Set(k.to_string(), v.to_string()));
        } else {
            options.push(ProfileOption::Enable(token.to_string()));
        }
    }

    ParsedProfile {
        name: name.to_string(),
        inherits_from,
        options,
        sort_order,
    }
}

/// Parse screen entry tokens from a value string.
fn parse_screen_entries(value: &str) -> Vec<ScreenEntry> {
    let mut entries = Vec::new();
    let mut chars = value.chars().peekable();

    // Skip leading whitespace between tokens
    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }

        match chars.peek() {
            None => break,
            Some('[') => {
                // Sub-screen reference: [Name]
                chars.next(); // consume '['
                let name: String = chars.by_ref().take_while(|&c| c != ']').collect();
                if !name.is_empty() {
                    entries.push(ScreenEntry::SubScreen(name));
                }
            }
            Some('<') => {
                // Could be <empty>
                let tag: String = chars.by_ref().take_while(|&c| !c.is_whitespace()).collect();
                if tag == "<empty>" {
                    entries.push(ScreenEntry::Empty);
                }
            }
            Some(_) => {
                // Regular option token
                let token: String = chars.by_ref().take_while(|c| !c.is_whitespace()).collect();
                if !token.is_empty() {
                    entries.push(ScreenEntry::Option(token));
                }
            }
        }
    }

    entries
}

/// Split a value string by whitespace into non-empty tokens.
fn split_whitespace_tokens(value: &str) -> Vec<String> {
    value.split_whitespace().map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Task 4 tests

    #[test]
    fn test_simple_profile() {
        let pairs = vec![("profile.LOW".into(), "SHADOWS AO_QUALITY=1 !DOF".into())];
        let result = interpret_shader_properties(&pairs).unwrap();
        assert_eq!(result.profiles.len(), 1);

        let p = &result.profiles[0];
        assert_eq!(p.name, "LOW");
        assert_eq!(p.inherits_from, None);
        assert_eq!(p.sort_order, 0);
        assert_eq!(
            p.options,
            vec![
                ProfileOption::Enable("SHADOWS".into()),
                ProfileOption::Set("AO_QUALITY".into(), "1".into()),
                ProfileOption::Disable("DOF".into()),
            ]
        );
    }

    #[test]
    fn test_profile_inheritance() {
        let pairs = vec![
            ("profile.LOW".into(), "SHADOWS".into()),
            ("profile.MEDIUM".into(), "profile.LOW BLOOM".into()),
        ];
        let result = interpret_shader_properties(&pairs).unwrap();
        assert_eq!(result.profiles.len(), 2);

        let med = &result.profiles[1];
        assert_eq!(med.name, "MEDIUM");
        assert_eq!(med.inherits_from, Some("LOW".into()));
        assert_eq!(med.sort_order, 1);
        assert_eq!(med.options, vec![ProfileOption::Enable("BLOOM".into()),]);
    }

    #[test]
    fn test_multiple_profiles_preserve_order() {
        let pairs = vec![
            ("profile.LOW".into(), "A".into()),
            ("profile.MEDIUM".into(), "B".into()),
            ("profile.HIGH".into(), "C".into()),
            ("profile.ULTRA".into(), "D".into()),
        ];
        let result = interpret_shader_properties(&pairs).unwrap();
        let names: Vec<&str> = result.profiles.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["LOW", "MEDIUM", "HIGH", "ULTRA"]);
    }

    #[test]
    fn test_no_profiles() {
        let pairs = vec![("clouds".into(), "fancy".into())];
        let result = interpret_shader_properties(&pairs).unwrap();
        assert!(result.profiles.is_empty());
    }

    #[test]
    fn test_iris_features() {
        let pairs = vec![
            ("iris.features.required".into(), "SSBO CUSTOM_IMAGES".into()),
            ("iris.features.optional".into(), "COMPUTE_SHADERS".into()),
        ];
        let result = interpret_shader_properties(&pairs).unwrap();
        assert_eq!(result.iris_features_required, vec!["SSBO", "CUSTOM_IMAGES"]);
        assert_eq!(result.iris_features_optional, vec!["COMPUTE_SHADERS"]);
    }

    #[test]
    fn test_pipeline_booleans() {
        let pairs = vec![
            ("separateAo".into(), "true".into()),
            ("clouds".into(), "fancy".into()),
            ("sun".into(), "false".into()),
            ("oldLighting".into(), "true".into()),
        ];
        let result = interpret_shader_properties(&pairs).unwrap();
        assert_eq!(result.pipeline.booleans.get("separateAo"), Some(&true));
        assert_eq!(result.pipeline.booleans.get("sun"), Some(&false));
        assert_eq!(result.pipeline.booleans.get("oldLighting"), Some(&true));
        assert_eq!(result.pipeline.cloud_setting, Some("fancy".into()));
    }

    #[test]
    fn test_slider_options() {
        let pairs = vec![(
            "sliders".into(),
            "shadowMapResolution shadowDistance sunPathRotation".into(),
        )];
        let result = interpret_shader_properties(&pairs).unwrap();
        assert_eq!(
            result.slider_options,
            vec!["shadowMapResolution", "shadowDistance", "sunPathRotation"]
        );
    }

    // Task 5 tests

    #[test]
    fn test_main_screen() {
        let pairs = vec![(
            "screen".into(),
            "[Lighting] [Effects] [Post] <empty> shadowMapResolution".into(),
        )];
        let result = interpret_shader_properties(&pairs).unwrap();
        let main = result.main_screen.as_ref().unwrap();
        assert_eq!(
            main.options,
            vec![
                ScreenEntry::SubScreen("Lighting".into()),
                ScreenEntry::SubScreen("Effects".into()),
                ScreenEntry::SubScreen("Post".into()),
                ScreenEntry::Empty,
                ScreenEntry::Option("shadowMapResolution".into()),
            ]
        );
    }

    #[test]
    fn test_sub_screens() {
        let pairs = vec![
            (
                "screen.POST".into(),
                "TAA FXAA [BLOOM_CONFIG] MOTION_BLUR".into(),
            ),
            (
                "screen.BLOOM_CONFIG".into(),
                "BLOOM_STRENGTH BLOOM_CONTRAST".into(),
            ),
        ];
        let result = interpret_shader_properties(&pairs).unwrap();
        assert_eq!(result.screens.len(), 2);

        let post = result.screens.iter().find(|s| s.name == "POST").unwrap();
        assert_eq!(
            post.options,
            vec![
                ScreenEntry::Option("TAA".into()),
                ScreenEntry::Option("FXAA".into()),
                ScreenEntry::SubScreen("BLOOM_CONFIG".into()),
                ScreenEntry::Option("MOTION_BLUR".into()),
            ]
        );
    }

    #[test]
    fn test_screen_columns() {
        let pairs = vec![
            ("screen.columns".into(), "2".into()),
            ("screen.POST.columns".into(), "3".into()),
            ("screen.POST".into(), "TAA FXAA".into()),
        ];
        let result = interpret_shader_properties(&pairs).unwrap();
        let main = result.main_screen.as_ref().unwrap();
        assert_eq!(main.columns, Some(2));

        let post = result.screens.iter().find(|s| s.name == "POST").unwrap();
        assert_eq!(post.columns, Some(3));
    }
}
