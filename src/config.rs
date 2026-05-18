use serde::Deserialize;
use std::collections::HashMap;
use worker::{Env, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct BannerConfig {
    pub theme: String,
    pub style: String,
    pub overlay_opacity: u8,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ButtonsConfig {
    pub accept_label: String,
    pub decline_label: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrivacyPolicyConfig {
    pub url: String,
    pub link_text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptEntry {
    /// Human-readable label used in config for identification; not used at runtime.
    #[allow(dead_code)]
    pub name: String,
    pub src: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptsConfig {
    #[serde(default)]
    pub essential: Vec<ScriptEntry>,
    #[serde(default)]
    pub tracking: Vec<ScriptEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub banner: HashMap<String, BannerConfig>,
    pub buttons: HashMap<String, ButtonsConfig>,
    #[serde(default)]
    pub privacy_policy: HashMap<String, PrivacyPolicyConfig>,
    pub scripts: ScriptsConfig,
}

/// Deserializes the `WORKER_CONFIG` environment variable from TOML format.
pub fn load(env: &Env) -> Result<WorkerConfig> {
    let raw = env.var("WORKER_CONFIG")?.to_string();
    toml::from_str(&raw).map_err(|e| worker::Error::RustError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deserializes TOML directly into a `WorkerConfig` for tests.
    fn parse(toml: &str) -> std::result::Result<WorkerConfig, toml::de::Error> {
        toml::from_str(toml)
    }

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

    #[test]
    fn parses_valid_config() {
        let cfg = parse(VALID_TOML).expect("valid TOML should parse");
        let banner = cfg.banner.get("en_GB").expect("en_GB banner missing");
        assert_eq!(banner.theme, "hacker");
        assert_eq!(banner.style, "box-bottom-right");
        assert_eq!(banner.overlay_opacity, 50);
        assert_eq!(banner.message, "We use cookies.");
    }

    #[test]
    fn parses_button_labels() {
        let cfg = parse(VALID_TOML).expect("valid TOML should parse");
        let buttons = cfg.buttons.get("en_GB").expect("en_GB buttons missing");
        assert_eq!(buttons.accept_label, "Accept All");
        assert_eq!(buttons.decline_label, "Decline");
    }

    #[test]
    fn parses_privacy_policy() {
        let cfg = parse(VALID_TOML).expect("valid TOML should parse");
        let pp = cfg
            .privacy_policy
            .get("en_GB")
            .expect("en_GB privacy_policy missing");
        assert_eq!(pp.url, "https://example.com/privacy");
        assert_eq!(pp.link_text, "Privacy Policy");
    }

    #[test]
    fn parses_scripts() {
        let cfg = parse(VALID_TOML).expect("valid TOML should parse");
        assert_eq!(cfg.scripts.essential.len(), 1);
        assert_eq!(cfg.scripts.essential[0].src, "/js/core.js");
        assert_eq!(cfg.scripts.tracking.len(), 1);
        assert_eq!(cfg.scripts.tracking[0].src, "https://ga.example.com/ga.js");
    }

    #[test]
    fn empty_scripts_default_to_empty_vecs() {
        let toml = r#"
[banner.en_GB]
theme = "minimal"
style = "bottom"
overlay_opacity = 0
message = "Cookies."

[buttons.en_GB]
accept_label = "OK"
decline_label = "No"

[scripts]
"#;
        let cfg = parse(toml).expect("should parse with no script entries");
        assert!(cfg.scripts.essential.is_empty());
        assert!(cfg.scripts.tracking.is_empty());
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
        let toml = r#"
[banner.en_GB]
theme = "hacker"
style = "bottom"
overlay_opacity = 0
message = "Cookies."

[buttons.en_GB]
accept_label = "OK"
decline_label = "No"

[scripts]
"#;
        let cfg = parse(toml).expect("privacy_policy should be optional");
        assert!(cfg.privacy_policy.is_empty());
    }
}
