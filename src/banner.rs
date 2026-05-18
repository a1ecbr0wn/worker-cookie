use crate::config::{
    BannerConfig, ButtonsConfig, PrivacyPolicyConfig, ScriptsConfig, SettingsConfig,
};

/// Renders the cookie consent banner HTML and embedded JavaScript.
///
/// When `preset_choice` is `Some`, the banner starts hidden and the settings button starts
/// visible (consent already known server-side). When `None`, the banner starts visible
/// and the script reads the cookie to determine initial state. Either way the banner div
/// is always present so the settings flow can show it again.
///
/// The `settings` parameter provides global positioning for the settings button; its `bottom`
/// and `right` fields are applied as inline CSS overrides, allowing fine-tuned viewport positioning.
pub fn render_banner_html(
    banner: &BannerConfig,
    buttons: &ButtonsConfig,
    privacy: Option<&PrivacyPolicyConfig>,
    scripts: &ScriptsConfig,
    settings: &SettingsConfig,
    preset_choice: Option<&str>,
) -> String {
    let privacy_link = privacy
        .map(|p| {
            format!(
                r#"<a href="{}" class="cookie-privacy-link">{}</a>"#,
                p.url, p.link_text
            )
        })
        .unwrap_or_default();

    let essential_srcs = script_src_list(
        &scripts
            .essential
            .iter()
            .map(|s| s.src.as_str())
            .collect::<Vec<_>>(),
    );
    let tracking_srcs = script_src_list(
        &scripts
            .tracking
            .iter()
            .map(|s| s.src.as_str())
            .collect::<Vec<_>>(),
    );

    let raw_color = settings.color.as_deref().unwrap_or("#d2ebff");
    let icon = cookie_svg(sanitize_svg_color(raw_color));

    let mut pos = String::new();
    if let Some(b) = settings.bottom {
        pos.push_str(&format!("bottom:{b}px;"));
    }
    if let Some(r) = settings.right {
        pos.push_str(&format!("right:{r}px;"));
    }
    let btn_display = match preset_choice {
        Some(_) => "display:block",
        None => "display:none",
    };
    let banner_style = match preset_choice {
        Some(_) => r#"style="display:none""#.to_string(),
        None => String::new(),
    };
    let btn_style = if pos.is_empty() {
        format!(r#"style="{btn_display}""#)
    } else {
        format!(r#"style="{btn_display};{pos}""#)
    };

    format!(
        r#"
<div id="cookie-banner" class="cookie-banner cookie-theme-{theme} cookie-style-{style}" role="dialog" aria-modal="true" aria-label="Cookie consent" {banner_style}>
  <div class="cookie-overlay" style="opacity:{opacity}%"></div>
  <div class="cookie-box">
    <div class="cookie-title">&gt; NOTICE</div>
    <div class="cookie-message">{message}</div>
    {privacy_link}
    <div class="cookie-actions">
      <button class="cookie-btn cookie-btn-decline" onclick="cookieConsent('declined')">{decline}</button>
      <button class="cookie-btn cookie-btn-accept primary" onclick="cookieConsent('accepted')">{accept}</button>
    </div>
  </div>
</div>
<button id="cookie-settings-btn" class="cookie-settings-btn" onclick="cookieSettings()" aria-label="Review cookie settings" title="Review cookie settings" {btn_style}>{icon}</button>
{script}"#,
        theme = banner.theme,
        style = banner.style,
        opacity = banner.overlay_opacity,
        message = banner.message,
        banner_style = banner_style,
        btn_style = btn_style,
        privacy_link = privacy_link,
        decline = buttons.decline_label,
        accept = buttons.accept_label,
        icon = icon,
        script = consent_script(&essential_srcs, &tracking_srcs, preset_choice),
    )
}

/// Generates the inline SVG cookie icon used in the settings button.
///
/// Renders a 24×24 cookie as a closed outline path: the cookie body (circle r=10, centre 12,12)
/// with a bite taken from the upper-right (bite circle r=6, centre 20,4). The two circles
/// intersect at approximately (21.7, 9.7) and (14.3, 2.3); the path traces the short concave bite arc
/// between those points, then the long cookie arc back around. No `fill-rule` tricks or
/// `<clipPath>` IDs are needed, so there are no fill artifacts and no ID-collision risk.
/// Chip dots are `<circle>` elements overlaid with translucent black. The SVG is aria-hidden;
/// the button carries its own accessible label.
fn cookie_svg(color: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" aria-hidden="true"><path fill="{color}" d="M21.7 9.7A6 6 0 0 1 14.3 2.3A10 10 0 1 0 21.7 9.7Z"/><circle cx="9" cy="11" r="1.5" fill="rgba(0,0,0,0.22)"/><circle cx="14" cy="16" r="1.5" fill="rgba(0,0,0,0.22)"/><circle cx="8" cy="17" r="1.2" fill="rgba(0,0,0,0.22)"/><circle cx="15" cy="10" r="1.2" fill="rgba(0,0,0,0.22)"/></svg>"#,
        color = color
    )
}

/// Validates that `color` is safe to interpolate into an SVG `fill` attribute.
///
/// Uses a strict allowlist: accepts only CSS hex colours (`#rgb`, `#rrggbb`, `#rgba`,
/// `#rrggbbaa`) and the keywords `none` and `transparent`. Any other value — including
/// named colours, `rgb(...)` functions, and `url(...)` references — falls back to
/// `"#d2ebff"`. Allowlist is intentionally narrow to eliminate the entire class of
/// CSS-injection and resource-fetch bypasses rather than enumerating forbidden characters.
fn sanitize_svg_color(color: &str) -> &str {
    let s = color.trim();
    if is_safe_svg_color(s) { s } else { "#d2ebff" }
}

/// Returns `true` when `s` is a safe CSS color for SVG `fill`: hex colours or `none`/`transparent`.
///
/// `s` must already be trimmed of whitespace; `sanitize_svg_color` handles trimming before calling this.
fn is_safe_svg_color(s: &str) -> bool {
    if let Some(hex) = s.strip_prefix('#') {
        return matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit());
    }
    matches!(s, "none" | "transparent")
}

/// Formats a slice of URL strings as a JavaScript array literal body (comma-separated quoted strings).
fn script_src_list(srcs: &[&str]) -> String {
    srcs.iter()
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generates the inline `<script>` block for consent management.
///
/// When `preset_choice` is `Some`, scripts are loaded immediately using that choice.
/// When `None`, the script reads the `userConsent` cookie. If the cookie contains
/// 'accepted' or 'declined' (via strict equality), the banner is hidden, the settings
/// button is shown, and the consent choice is applied. Otherwise, the banner remains
/// visible and no scripts are loaded until the user makes a choice.
/// The `window.cookieConsent` function validates its input, accepting only 'accepted'
/// or 'declined'; any other value is silently rejected.
fn consent_script(
    essential_srcs: &str,
    tracking_srcs: &str,
    preset_choice: Option<&str>,
) -> String {
    let init = match preset_choice {
        Some(choice) => format!(r#"  applyConsent("{choice}");"#),
        None => r#"  var existing = getCookie('userConsent');
  if (existing === 'accepted' || existing === 'declined') {
    document.getElementById('cookie-banner').style.display = 'none';
    document.getElementById('cookie-settings-btn').style.display = 'block';
    applyConsent(existing);
  }"#
        .to_string(),
    };

    format!(
        r#"<script>
(function() {{
  var ESSENTIAL = [{essential}];
  var TRACKING = [{tracking}];

  function getCookie(name) {{
    var m = document.cookie.match('(?:^|;)\\s*' + name + '=([^;]*)');
    return m ? decodeURIComponent(m[1]) : null;
  }}

  function setCookie(name, value) {{
    document.cookie = name + '=' + value + '; path=/; max-age=31536000; SameSite=Lax';
  }}

  function loadScripts(srcs) {{
    srcs.forEach(function(src) {{
      var s = document.createElement('script');
      s.src = src;
      s.async = true;
      document.head.appendChild(s);
    }});
  }}

  function applyConsent(choice) {{
    loadScripts(ESSENTIAL);
    if (choice === 'accepted') loadScripts(TRACKING);
    var obs = new MutationObserver(function(mutations) {{
      mutations.forEach(function(m) {{
        m.addedNodes.forEach(function(node) {{
          if (node.tagName === 'SCRIPT' && node.src) {{
            var isTracking = TRACKING.some(function(src) {{ return node.src.indexOf(src) !== -1; }});
            if (isTracking && choice !== 'accepted') {{
              node.parentNode && node.parentNode.removeChild(node);
            }}
          }}
        }});
      }});
    }});
    obs.observe(document.documentElement, {{ childList: true, subtree: true }});
  }}

  window.cookieConsent = function(choice) {{
    if (choice !== 'accepted' && choice !== 'declined') return;
    setCookie('userConsent', choice);
    document.getElementById('cookie-banner').style.display = 'none';
    document.getElementById('cookie-settings-btn').style.display = 'block';
    applyConsent(choice);
  }};

  window.cookieSettings = function() {{
    document.getElementById('cookie-banner').style.display = '';
    document.getElementById('cookie-settings-btn').style.display = 'none';
  }};

{init}
}})();
</script>"#,
        essential = essential_srcs,
        tracking = tracking_srcs,
        init = init,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BannerConfig, ButtonsConfig, PrivacyPolicyConfig, ScriptEntry, ScriptsConfig,
        SettingsConfig,
    };

    fn test_scripts() -> ScriptsConfig {
        ScriptsConfig {
            essential: vec![ScriptEntry {
                name: "core".to_string(),
                src: "/js/core.js".to_string(),
            }],
            tracking: vec![ScriptEntry {
                name: "ga".to_string(),
                src: "https://www.googletagmanager.com/gtag/js".to_string(),
            }],
        }
    }

    fn test_banner() -> BannerConfig {
        BannerConfig {
            theme: "hacker".to_string(),
            style: "box-bottom-right".to_string(),
            overlay_opacity: 50,
            message: "We use cookies.".to_string(),
        }
    }

    fn test_buttons() -> ButtonsConfig {
        ButtonsConfig {
            accept_label: "Accept All".to_string(),
            decline_label: "Decline".to_string(),
        }
    }

    #[test]
    fn banner_html_contains_message() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(html.contains("We use cookies."));
    }

    #[test]
    fn banner_html_contains_button_labels() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(html.contains("Accept All"));
        assert!(html.contains("Decline"));
    }

    #[test]
    fn banner_html_contains_essential_scripts() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(html.contains("/js/core.js"));
    }

    #[test]
    fn banner_html_contains_tracking_scripts() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(html.contains("googletagmanager.com"));
    }

    #[test]
    fn banner_html_applies_theme_and_style_classes() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(html.contains("cookie-theme-hacker"));
        assert!(html.contains("cookie-style-box-bottom-right"));
    }

    #[test]
    fn banner_html_omits_privacy_link_when_none() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(!html.contains("cookie-privacy-link"));
    }

    #[test]
    fn banner_html_includes_privacy_link_when_provided() {
        let privacy = PrivacyPolicyConfig {
            url: "https://example.com/privacy".to_string(),
            link_text: "Privacy Policy".to_string(),
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            Some(&privacy),
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(html.contains("https://example.com/privacy"));
        assert!(html.contains("Privacy Policy"));
    }

    #[test]
    fn banner_visible_when_no_preset_consent() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        let banner_tag_start = html
            .find(r#"id="cookie-banner""#)
            .expect("banner div missing");
        let tag_end = html[banner_tag_start..]
            .find('>')
            .expect("banner tag not closed");
        let banner_tag = &html[banner_tag_start..banner_tag_start + tag_end];
        assert!(
            !banner_tag.contains("display:none"),
            "banner should be visible: {}",
            banner_tag
        );
        assert!(html.contains(r#"id="cookie-settings-btn""#));
        assert!(html.contains(r#"style="display:none""#));
    }

    #[test]
    fn banner_hidden_when_preset_consent() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            Some("accepted"),
        );
        assert!(html.contains(r#"style="display:none""#));
        assert!(html.contains(r#"style="display:block""#));
    }

    #[test]
    fn preset_accepted_embeds_choice_in_script() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            Some("accepted"),
        );
        assert!(html.contains(r#"applyConsent("accepted")"#));
    }

    #[test]
    fn preset_declined_embeds_choice_in_script() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            Some("declined"),
        );
        assert!(html.contains(r#"applyConsent("declined")"#));
    }

    #[test]
    fn no_preset_reads_cookie_in_script() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(html.contains("getCookie('userConsent')"));
    }

    #[test]
    fn cookie_consent_function_has_allowlist_guard() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(
            html.contains("if (choice !== 'accepted' && choice !== 'declined') return;"),
            "cookieConsent allowlist guard missing from generated script"
        );
    }

    /// Extracts the opening tag of the settings button from the rendered HTML.
    ///
    /// Locates the settings button element by its `id` attribute and returns the substring
    /// from the element ID through the closing `>`, representing the complete opening tag
    /// with all its attributes.
    fn settings_btn_tag(html: &str) -> String {
        let pos = html
            .find(r#"id="cookie-settings-btn""#)
            .expect("settings btn missing");
        let end = html[pos..].find('>').expect("settings btn tag not closed");
        html[pos..pos + end].to_string()
    }

    #[test]
    fn settings_button_no_position_override_when_settings_default() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        let tag = settings_btn_tag(&html);
        assert!(!tag.contains("bottom:"), "unexpected bottom in: {}", tag);
        assert!(!tag.contains("right:"), "unexpected right in: {}", tag);
    }

    #[test]
    fn settings_button_applies_bottom_override() {
        let settings = SettingsConfig {
            bottom: Some(48),
            right: None,
            color: None,
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            None,
        );
        let tag = settings_btn_tag(&html);
        assert!(
            tag.contains("bottom:48px;"),
            "expected bottom override in: {}",
            tag
        );
        assert!(!tag.contains("right:"), "unexpected right in: {}", tag);
    }

    #[test]
    fn settings_button_applies_both_position_overrides() {
        let settings = SettingsConfig {
            bottom: Some(24),
            right: Some(32),
            color: None,
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            None,
        );
        let tag = settings_btn_tag(&html);
        assert!(tag.contains("bottom:24px;"), "expected bottom in: {}", tag);
        assert!(tag.contains("right:32px;"), "expected right in: {}", tag);
    }

    #[test]
    fn settings_button_applies_position_overrides_when_preset_consent() {
        let settings = SettingsConfig {
            bottom: Some(24),
            right: Some(32),
            color: None,
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            Some("accepted"),
        );
        let tag = settings_btn_tag(&html);
        assert!(
            tag.contains("display:block"),
            "expected block display in: {}",
            tag
        );
        assert!(tag.contains("bottom:24px;"), "expected bottom in: {}", tag);
        assert!(tag.contains("right:32px;"), "expected right in: {}", tag);
    }

    #[test]
    fn settings_icon_uses_default_color() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &SettingsConfig::default(),
            None,
        );
        assert!(
            html.contains(r##"fill="#d2ebff""##),
            "expected default icon color in fill attribute"
        );
    }

    #[test]
    fn settings_icon_uses_custom_color() {
        let settings = SettingsConfig {
            bottom: None,
            right: None,
            color: Some("#ff6600".to_string()),
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            None,
        );
        assert!(
            html.contains(r##"fill="#ff6600""##),
            "expected custom icon color in fill attribute"
        );
        assert!(
            !html.contains(r##"fill="#d2ebff""##),
            "default color should not appear in fill when overridden"
        );
    }

    #[test]
    fn settings_icon_sanitizes_css_injection_bypass() {
        // url(...) contains no denylist chars but must be rejected by the allowlist
        let settings = SettingsConfig {
            bottom: None,
            right: None,
            color: Some("url(javascript:alert(1))".to_string()),
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            None,
        );
        assert!(
            html.contains(r##"fill="#d2ebff""##),
            "url() color must fall back to default"
        );
        assert!(
            !html.contains("javascript"),
            "javascript URI must not appear in output"
        );
    }

    #[test]
    fn settings_icon_sanitizes_named_color() {
        // Named colors like "red" are not in the allowlist
        let settings = SettingsConfig {
            bottom: None,
            right: None,
            color: Some("red".to_string()),
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            None,
        );
        assert!(
            html.contains(r##"fill="#d2ebff""##),
            "named color must fall back to default"
        );
        assert!(
            !html.contains(r##"fill="red""##),
            "named color must not appear as fill value"
        );
    }

    #[test]
    fn settings_icon_accepts_short_hex_color() {
        let settings = SettingsConfig {
            bottom: None,
            right: None,
            color: Some("#fff".to_string()),
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            None,
        );
        assert!(
            html.contains(r##"fill="#fff""##),
            "3-digit hex color must pass through"
        );
    }

    #[test]
    fn settings_icon_accepts_eight_digit_hex_color() {
        let settings = SettingsConfig {
            bottom: None,
            right: None,
            color: Some("#ff660080".to_string()),
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            None,
        );
        assert!(
            html.contains(r##"fill="#ff660080""##),
            "8-digit hex color must pass through"
        );
    }

    #[test]
    fn settings_icon_accepts_transparent_keyword() {
        let settings = SettingsConfig {
            bottom: None,
            right: None,
            color: Some("transparent".to_string()),
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            &settings,
            None,
        );
        assert!(
            html.contains(r##"fill="transparent""##),
            "transparent keyword must pass through"
        );
    }
}
