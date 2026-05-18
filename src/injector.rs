use crate::banner::render_banner_html;
use crate::config::WorkerConfig;

/// Injects the cookie consent banner and theme CSS into an HTML response.
///
/// When `existing_consent` is `Some`, the banner is rendered pre-hidden and the settings
/// button is shown; the inline script applies consent immediately without reading the cookie.
/// When `None`, the banner starts visible and the script reads the cookie on load.
/// CSS is always injected into `<head>`; the banner is inserted before `</body>`.
/// Global settings button positioning from `cfg.settings` is applied to all rendered banners.
/// Returns the original HTML unmodified if the resolved locale has no configuration.
pub fn inject(
    html: &str,
    cfg: &WorkerConfig,
    locale: &str,
    existing_consent: Option<&str>,
) -> String {
    let locale_key = resolve_locale(cfg, locale);

    let banner_cfg = cfg.banner.get(&locale_key);
    let buttons_cfg = cfg.buttons.get(&locale_key);

    let (Some(banner), Some(buttons)) = (banner_cfg, buttons_cfg) else {
        return html.to_string();
    };

    let privacy = cfg.privacy_policy.get(&locale_key);
    let css = include_str!("../assets/themes.css");

    let html = insert_before(html, "</head>", &format!("<style>{}</style>", css));
    let banner_html = render_banner_html(
        banner,
        buttons,
        privacy,
        &cfg.scripts,
        &cfg.settings,
        existing_consent,
    );
    insert_before_last(&html, "</body>", &banner_html)
}

/// Inserts `snippet` immediately before the first occurrence of `tag` in `html`.
///
/// Returns the original string if `tag` is not found.
fn insert_before(html: &str, tag: &str, snippet: &str) -> String {
    match html.find(tag) {
        Some(pos) => {
            let mut s = html.to_string();
            s.insert_str(pos, snippet);
            s
        }
        None => html.to_string(),
    }
}

/// Inserts `snippet` immediately before the last occurrence of `tag` in `html`.
///
/// Appends to the end of the string if `tag` is not found.
fn insert_before_last(html: &str, tag: &str, snippet: &str) -> String {
    match html.rfind(tag) {
        Some(pos) => {
            let mut s = html.to_string();
            s.insert_str(pos, snippet);
            s
        }
        None => format!("{}{}", html, snippet),
    }
}

/// Maps a detected locale to an available configuration locale, with three-tier fallback.
///
/// Attempts exact match first (e.g. `en_GB` → `en_GB`), then language-prefix match
/// (e.g. `en_US` → `en_GB`), then falls back to the unqualified default section stored
/// under `""`. Returns `""` when no locale matches and no default is configured, causing
/// `inject` to pass the page through unmodified. The language prefix is extracted by
/// splitting on the first underscore; if no underscore is present, the entire locale
/// string is used for matching.
fn resolve_locale(cfg: &WorkerConfig, locale: &str) -> String {
    if cfg.banner.contains_key(locale) {
        return locale.to_string();
    }
    let lang = locale.split_once('_').map_or(locale, |(prefix, _)| prefix);
    if let Some(k) = cfg
        .banner
        .keys()
        .find(|k| !k.is_empty() && k.starts_with(lang))
    {
        return k.clone();
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BannerConfig, ButtonsConfig, ScriptEntry, ScriptsConfig, SettingsConfig, WorkerConfig,
    };
    use std::collections::HashMap;

    /// Constructs a test WorkerConfig with both an unqualified default section (under `""`)
    /// and a locale-specific section for `en_GB`, used to verify fallback behavior.
    fn make_config_with_default() -> WorkerConfig {
        let mut banner = HashMap::new();
        banner.insert(
            String::new(),
            BannerConfig {
                theme: "minimal".to_string(),
                style: "bottom".to_string(),
                overlay_opacity: 0,
                message: "Default message.".to_string(),
            },
        );
        banner.insert(
            "en_GB".to_string(),
            BannerConfig {
                theme: "hacker".to_string(),
                style: "box-bottom-right".to_string(),
                overlay_opacity: 50,
                message: "English message.".to_string(),
            },
        );
        let mut buttons = HashMap::new();
        buttons.insert(
            String::new(),
            ButtonsConfig {
                accept_label: "OK".to_string(),
                decline_label: "No".to_string(),
            },
        );
        buttons.insert(
            "en_GB".to_string(),
            ButtonsConfig {
                accept_label: "Accept".to_string(),
                decline_label: "Decline".to_string(),
            },
        );
        WorkerConfig {
            banner,
            buttons,
            privacy_policy: HashMap::new(),
            scripts: ScriptsConfig::default(),
            settings: SettingsConfig::default(),
        }
    }

    fn make_config() -> WorkerConfig {
        let mut banner = HashMap::new();
        banner.insert(
            "en_GB".to_string(),
            BannerConfig {
                theme: "hacker".to_string(),
                style: "box-bottom-right".to_string(),
                overlay_opacity: 50,
                message: "We use cookies.".to_string(),
            },
        );
        let mut buttons = HashMap::new();
        buttons.insert(
            "en_GB".to_string(),
            ButtonsConfig {
                accept_label: "Accept".to_string(),
                decline_label: "Decline".to_string(),
            },
        );
        WorkerConfig {
            banner,
            buttons,
            privacy_policy: HashMap::new(),
            scripts: ScriptsConfig {
                essential: vec![ScriptEntry {
                    name: "core".to_string(),
                    src: "/js/core.js".to_string(),
                }],
                tracking: vec![ScriptEntry {
                    name: "ga".to_string(),
                    src: "https://ga.example.com/ga.js".to_string(),
                }],
            },
            settings: SettingsConfig::default(),
        }
    }

    const FULL_PAGE: &str = "<html><head></head><body><p>Hello</p></body></html>";
    const NO_HEAD: &str = "<html><body><p>Hello</p></body></html>";
    const NO_BODY: &str = "<html><head></head><p>Hello</p></html>";

    #[test]
    fn injects_css_into_head() {
        let result = inject(FULL_PAGE, &make_config(), "en_GB", None);
        assert!(result.contains("<style>"));
        assert!(result.find("<style>").unwrap() < result.find("</head>").unwrap());
    }

    #[test]
    fn injects_banner_before_body_close() {
        let result = inject(FULL_PAGE, &make_config(), "en_GB", None);
        assert!(result.contains("cookie-banner"));
        assert!(result.rfind("cookie-banner").unwrap() < result.rfind("</body>").unwrap());
    }

    #[test]
    fn banner_visible_when_no_consent() {
        let result = inject(FULL_PAGE, &make_config(), "en_GB", None);
        // Settings button should be hidden; banner should not have display:none
        assert!(result.contains(r#"id="cookie-settings-btn""#));
        // The banner div itself should not carry display:none
        let banner_pos = result.find(r#"id="cookie-banner""#).unwrap();
        let settings_pos = result.find(r#"id="cookie-settings-btn""#).unwrap();
        // display:none on settings btn, not on banner
        let between = &result[banner_pos..settings_pos];
        assert!(!between.contains("display:none"));
    }

    #[test]
    fn banner_hidden_when_consent_exists() {
        let result = inject(FULL_PAGE, &make_config(), "en_GB", Some("accepted"));
        // Banner div should carry display:none
        let banner_pos = result.find(r#"id="cookie-banner""#).unwrap();
        let close = result[banner_pos..].find('>').unwrap();
        let tag = &result[banner_pos..banner_pos + close];
        assert!(tag.contains("display:none"));
    }

    #[test]
    fn settings_button_visible_when_consent_exists() {
        let result = inject(FULL_PAGE, &make_config(), "en_GB", Some("accepted"));
        assert!(result.contains(r#"style="display:block""#));
    }

    #[test]
    fn preset_choice_embedded_in_script() {
        let result = inject(FULL_PAGE, &make_config(), "en_GB", Some("accepted"));
        assert!(result.contains(r#"applyConsent("accepted")"#));
    }

    #[test]
    fn no_consent_reads_cookie_in_script() {
        let result = inject(FULL_PAGE, &make_config(), "en_GB", None);
        assert!(result.contains("getCookie('userConsent')"));
    }

    #[test]
    fn handles_missing_head_tag() {
        let result = inject(NO_HEAD, &make_config(), "en_GB", None);
        assert!(result.contains("cookie-banner"));
    }

    #[test]
    fn appends_when_no_body_tag() {
        let result = inject(NO_BODY, &make_config(), "en_GB", None);
        assert!(result.contains("cookie-banner"));
    }

    #[test]
    fn locale_prefix_fallback() {
        let result = inject(FULL_PAGE, &make_config(), "en_US", None);
        assert!(result.contains("We use cookies."));
    }

    #[test]
    fn locale_no_match_without_default_passes_through() {
        let result = inject(FULL_PAGE, &make_config(), "zh_CN", None);
        assert_eq!(result, FULL_PAGE);
    }

    #[test]
    fn resolve_locale_exact_match() {
        let cfg = make_config();
        assert_eq!(resolve_locale(&cfg, "en_GB"), "en_GB");
    }

    #[test]
    fn resolve_locale_prefix_fallback() {
        let cfg = make_config();
        assert_eq!(resolve_locale(&cfg, "en_US"), "en_GB");
    }

    #[test]
    fn resolve_locale_no_match_returns_empty_string() {
        let cfg = make_config();
        assert_eq!(resolve_locale(&cfg, "zh_CN"), "");
    }

    #[test]
    fn resolve_locale_bare_lang_code_matches_by_prefix() {
        let cfg = make_config();
        // "en" has no underscore; the full string is used as the prefix,
        // which matches "en_GB" via starts_with.
        assert_eq!(resolve_locale(&cfg, "en"), "en_GB");
    }

    #[test]
    fn resolve_locale_falls_back_to_default_when_no_locale_matches() {
        let cfg = make_config_with_default();
        assert_eq!(resolve_locale(&cfg, "zh_CN"), "");
    }

    #[test]
    fn locale_no_match_uses_default_section() {
        let result = inject(FULL_PAGE, &make_config_with_default(), "zh_CN", None);
        assert!(result.contains("Default message."));
    }

    #[test]
    fn locale_exact_match_takes_priority_over_default() {
        let result = inject(FULL_PAGE, &make_config_with_default(), "en_GB", None);
        assert!(result.contains("English message."));
        assert!(!result.contains("Default message."));
    }
}
