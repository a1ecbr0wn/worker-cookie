use crate::config::{BannerConfig, ButtonsConfig, PrivacyPolicyConfig, ScriptsConfig};

/// Renders the cookie consent banner HTML and embedded JavaScript.
///
/// When `preset_choice` is `Some`, the banner starts hidden and the revoke button starts
/// visible (consent already known server-side). When `None`, the banner starts visible
/// and the script reads the cookie to determine initial state. Either way the banner div
/// is always present so the revoke flow can show it again.
pub fn render_banner_html(
    banner: &BannerConfig,
    buttons: &ButtonsConfig,
    privacy: Option<&PrivacyPolicyConfig>,
    scripts: &ScriptsConfig,
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

    let (banner_style, revoke_style) = match preset_choice {
        Some(_) => (r#"style="display:none""#, r#"style="display:block""#),
        None => ("", r#"style="display:none""#),
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
<button id="cookie-revoke-btn" class="cookie-revoke-btn" onclick="cookieRevoke()" aria-label="Review cookie settings" title="Review cookie settings" {revoke_style}>&#x1F36A;</button>
{script}"#,
        theme = banner.theme,
        style = banner.style,
        opacity = banner.overlay_opacity,
        message = banner.message,
        banner_style = banner_style,
        revoke_style = revoke_style,
        privacy_link = privacy_link,
        decline = buttons.decline_label,
        accept = buttons.accept_label,
        script = consent_script(&essential_srcs, &tracking_srcs, preset_choice),
    )
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
/// 'accepted' or 'declined' (via strict equality), the banner is hidden, the revoke
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
        Some(choice) => format!(r#"  applyConsent("{choice}");"#, choice = choice),
        None => r#"  var existing = getCookie('userConsent');
  if (existing === 'accepted' || existing === 'declined') {
    document.getElementById('cookie-banner').style.display = 'none';
    document.getElementById('cookie-revoke-btn').style.display = 'block';
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
    document.getElementById('cookie-revoke-btn').style.display = 'block';
    applyConsent(choice);
  }};

  window.cookieRevoke = function() {{
    document.getElementById('cookie-banner').style.display = '';
    document.getElementById('cookie-revoke-btn').style.display = 'none';
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
    use crate::config::{BannerConfig, ButtonsConfig, ScriptEntry, ScriptsConfig};

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
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        assert!(html.contains("We use cookies."));
    }

    #[test]
    fn banner_html_contains_button_labels() {
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        assert!(html.contains("Accept All"));
        assert!(html.contains("Decline"));
    }

    #[test]
    fn banner_html_contains_essential_scripts() {
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        assert!(html.contains("/js/core.js"));
    }

    #[test]
    fn banner_html_contains_tracking_scripts() {
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        assert!(html.contains("googletagmanager.com"));
    }

    #[test]
    fn banner_html_applies_theme_and_style_classes() {
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        assert!(html.contains("cookie-theme-hacker"));
        assert!(html.contains("cookie-style-box-bottom-right"));
    }

    #[test]
    fn banner_html_omits_privacy_link_when_none() {
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        assert!(!html.contains("cookie-privacy-link"));
    }

    #[test]
    fn banner_html_includes_privacy_link_when_provided() {
        let privacy = crate::config::PrivacyPolicyConfig {
            url: "https://example.com/privacy".to_string(),
            link_text: "Privacy Policy".to_string(),
        };
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            Some(&privacy),
            &test_scripts(),
            None,
        );
        assert!(html.contains("https://example.com/privacy"));
        assert!(html.contains("Privacy Policy"));
    }

    #[test]
    fn banner_visible_when_no_preset_consent() {
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        // The banner div opening tag must not carry display:none
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
        // Revoke button should start hidden
        assert!(html.contains(r#"id="cookie-revoke-btn""#));
        assert!(html.contains(r#"style="display:none""#));
    }

    #[test]
    fn banner_hidden_when_preset_consent() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
            Some("accepted"),
        );
        // Banner div should be hidden
        assert!(html.contains(r#"style="display:none""#));
        // Revoke button should be visible
        assert!(html.contains(r#"style="display:block""#));
    }

    #[test]
    fn preset_accepted_embeds_choice_in_script() {
        let html = render_banner_html(
            &test_banner(),
            &test_buttons(),
            None,
            &test_scripts(),
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
            Some("declined"),
        );
        assert!(html.contains(r#"applyConsent("declined")"#));
    }

    #[test]
    fn no_preset_reads_cookie_in_script() {
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        assert!(html.contains("getCookie('userConsent')"));
    }

    #[test]
    fn cookie_consent_function_has_allowlist_guard() {
        let html = render_banner_html(&test_banner(), &test_buttons(), None, &test_scripts(), None);
        assert!(
            html.contains("if (choice !== 'accepted' && choice !== 'declined') return;"),
            "cookieConsent allowlist guard missing from generated script"
        );
    }
}
