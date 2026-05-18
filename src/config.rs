use serde::Deserialize;
use std::collections::HashMap;
use worker::{Env, Result};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct BannerConfig {
    pub theme: String,
    pub style: String,
    pub overlay_opacity: u8,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ButtonsConfig {
    pub accept_label: String,
    pub decline_label: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PrivacyPolicyConfig {
    pub url: String,
    pub link_text: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ScriptEntry {
    /// Human-readable label used in config for identification; not used at runtime.
    #[allow(dead_code)]
    pub name: String,
    pub src: String,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct ScriptsConfig {
    #[serde(default)]
    pub essential: Vec<ScriptEntry>,
    #[serde(default)]
    pub tracking: Vec<ScriptEntry>,
}

/// Locale-keyed configuration map where the empty string `""` represents the
/// unqualified default section (e.g. `[banner]` with no locale suffix).
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub banner: HashMap<String, BannerConfig>,
    pub buttons: HashMap<String, ButtonsConfig>,
    pub privacy_policy: HashMap<String, PrivacyPolicyConfig>,
    pub scripts: ScriptsConfig,
}

/// Splits a TOML table into a locale-keyed map, with an optional unqualified
/// default inserted under the `""` key.
///
/// Subtable entries (e.g. `[section.en_GB]`) become locale keys. Scalar fields
/// at the top level (e.g. `[banner]` with direct fields) form the default entry
/// stored under `""`. Returns an error if any entry fails to deserialise as `T`.
fn split_section<T>(table: &toml::Table) -> std::result::Result<HashMap<String, T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut default_fields = toml::Table::new();
    let mut result = HashMap::new();

    for (key, value) in table {
        if let toml::Value::Table(sub) = value {
            if key.is_empty() {
                return Err("locale key may not be an empty string".to_string());
            }
            let cfg: T = toml::Value::Table(sub.clone())
                .try_into()
                .map_err(|e| format!("[*.{key}]: {e}"))?;
            result.insert(key.clone(), cfg);
        } else {
            default_fields.insert(key.clone(), value.clone());
        }
    }

    if !default_fields.is_empty() {
        let cfg: T = toml::Value::Table(default_fields)
            .try_into()
            .map_err(|e| format!("default section: {e}"))?;
        result.insert(String::new(), cfg);
    }

    Ok(result)
}

/// Parses a TOML string into a `WorkerConfig`.
///
/// Each of `[banner]`, `[buttons]`, and `[privacy_policy]` may carry either
/// locale subtables (e.g. `[banner.en_GB]`), direct fields as an unqualified
/// default (e.g. `[banner]` with `theme = "..."`), or both. The unqualified
/// default is stored under the `""` key and acts as the final fallback during
/// locale resolution.
pub fn parse(raw: &str) -> std::result::Result<WorkerConfig, String> {
    let mut table: toml::Table = raw.parse().map_err(|e| format!("TOML parse error: {e}"))?;

    let banner_table = table
        .remove("banner")
        .and_then(|v| {
            if let toml::Value::Table(t) = v {
                Some(t)
            } else {
                None
            }
        })
        .ok_or("missing [banner] section")?;

    let buttons_table = table
        .remove("buttons")
        .and_then(|v| {
            if let toml::Value::Table(t) = v {
                Some(t)
            } else {
                None
            }
        })
        .ok_or("missing [buttons] section")?;

    let privacy_table = table
        .remove("privacy_policy")
        .and_then(|v| {
            if let toml::Value::Table(t) = v {
                Some(t)
            } else {
                None
            }
        })
        .unwrap_or_default();

    let scripts: ScriptsConfig = match table.remove("scripts") {
        Some(v) => v.try_into().map_err(|e| format!("[scripts]: {e}"))?,
        None => ScriptsConfig::default(),
    };

    Ok(WorkerConfig {
        banner: split_section(&banner_table)?,
        buttons: split_section(&buttons_table)?,
        privacy_policy: split_section(&privacy_table)?,
        scripts,
    })
}

/// Loads and parses the `WORKER_CONFIG` environment variable from TOML format.
///
/// Retrieves the environment variable and passes it to `parse()`. Returns a worker error
/// if the variable is missing or the TOML is invalid.
pub fn load(env: &Env) -> Result<WorkerConfig> {
    let raw = env.var("WORKER_CONFIG")?.to_string();
    parse(&raw).map_err(worker::Error::RustError)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_TOML: &str = r#"
[banner.en_GB]
theme = "hacker"
style = "box-bottom-right"
overlay_opacity = 50
message = "We use cookies."

[buttons.en_GB]
accept_label = "Accept All"
decline_label = "Decline"

[privacy_policy.en_GB]
url = "https://example.com/privacy"
link_text = "Privacy Policy"

[scripts]
essential = [{ name = "core", src = "/js/core.js" }]
tracking  = [{ name = "ga",   src = "https://ga.example.com/ga.js" }]
"#;

    const DEFAULT_TOML: &str = r#"
[banner]
theme = "minimal"
style = "bottom"
overlay_opacity = 0
message = "Default message."

[buttons]
accept_label = "OK"
decline_label = "No"

[scripts]
"#;

    const MIXED_TOML: &str = r#"
[banner]
theme = "minimal"
style = "bottom"
overlay_opacity = 0
message = "Default message."

[banner.en_GB]
theme = "hacker"
style = "box-bottom-right"
overlay_opacity = 50
message = "English message."

[buttons]
accept_label = "OK"
decline_label = "No"

[buttons.en_GB]
accept_label = "Accept All"
decline_label = "Decline"

[scripts]
"#;

    #[test]
    fn parses_valid_config() {
        let cfg = parse(VALID_TOML).expect("valid TOML should parse");
        assert_eq!(
            cfg.banner.get("en_GB"),
            Some(&BannerConfig {
                theme: "hacker".to_string(),
                style: "box-bottom-right".to_string(),
                overlay_opacity: 50,
                message: "We use cookies.".to_string(),
            })
        );
    }

    #[test]
    fn parses_button_labels() {
        let cfg = parse(VALID_TOML).expect("valid TOML should parse");
        assert_eq!(
            cfg.buttons.get("en_GB"),
            Some(&ButtonsConfig {
                accept_label: "Accept All".to_string(),
                decline_label: "Decline".to_string(),
            })
        );
    }

    #[test]
    fn parses_privacy_policy() {
        let cfg = parse(VALID_TOML).expect("valid TOML should parse");
        assert_eq!(
            cfg.privacy_policy.get("en_GB"),
            Some(&PrivacyPolicyConfig {
                url: "https://example.com/privacy".to_string(),
                link_text: "Privacy Policy".to_string(),
            })
        );
    }

    #[test]
    fn parses_scripts() {
        let cfg = parse(VALID_TOML).expect("valid TOML should parse");
        assert_eq!(
            cfg.scripts,
            ScriptsConfig {
                essential: vec![ScriptEntry {
                    name: "core".to_string(),
                    src: "/js/core.js".to_string(),
                }],
                tracking: vec![ScriptEntry {
                    name: "ga".to_string(),
                    src: "https://ga.example.com/ga.js".to_string(),
                }],
            }
        );
    }

    /// Verifies that absent or empty `[scripts]` section defaults to empty vectors.
    #[test]
    fn empty_scripts_default_to_empty_vecs() {
        let cfg = parse(DEFAULT_TOML).expect("should parse with no script entries");
        assert_eq!(cfg.scripts, ScriptsConfig::default());
    }

    #[test]
    fn rejects_invalid_toml() {
        assert!(parse("not valid toml :::").is_err());
    }

    #[test]
    fn rejects_missing_required_fields() {
        let toml = r#"
[banner.en_GB]
theme = "hacker"
"#;
        assert!(parse(toml).is_err());
    }

    #[test]
    fn privacy_policy_is_optional() {
        let cfg = parse(DEFAULT_TOML).expect("privacy_policy should be optional");
        assert!(cfg.privacy_policy.is_empty());
    }

    #[test]
    fn parses_unqualified_default_section() {
        let cfg = parse(DEFAULT_TOML).expect("default section should parse");
        assert_eq!(
            cfg.banner.get(""),
            Some(&BannerConfig {
                theme: "minimal".to_string(),
                style: "bottom".to_string(),
                overlay_opacity: 0,
                message: "Default message.".to_string(),
            })
        );
        assert_eq!(
            cfg.buttons.get(""),
            Some(&ButtonsConfig {
                accept_label: "OK".to_string(),
                decline_label: "No".to_string(),
            })
        );
    }

    #[test]
    fn parses_mixed_default_and_locale_sections() {
        let cfg = parse(MIXED_TOML).expect("mixed config should parse");
        assert_eq!(
            cfg.banner.get(""),
            Some(&BannerConfig {
                theme: "minimal".to_string(),
                style: "bottom".to_string(),
                overlay_opacity: 0,
                message: "Default message.".to_string(),
            })
        );
        assert_eq!(
            cfg.banner.get("en_GB"),
            Some(&BannerConfig {
                theme: "hacker".to_string(),
                style: "box-bottom-right".to_string(),
                overlay_opacity: 50,
                message: "English message.".to_string(),
            })
        );
    }

    /// Ensures parsing fails when an empty string is explicitly used as a locale key.
    ///
    /// The empty string `""` is reserved for the unqualified default section;
    /// it must not appear as an explicit subtable key.
    #[test]
    fn rejects_empty_string_locale_key() {
        let toml = r#"
[banner.""]
theme = "hacker"
style = "bottom"
overlay_opacity = 0
message = "x"

[buttons]
accept_label = "OK"
decline_label = "No"

[scripts]
"#;
        assert!(parse(toml).is_err());
    }
}
