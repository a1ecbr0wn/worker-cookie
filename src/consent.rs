use worker::Request;

/// Extracts and normalises a locale from an `Accept-Language` header value.
///
/// Takes the highest-priority tag, strips quality weighting, and converts hyphens to
/// underscores (e.g. `en-GB;q=0.9` → `en_GB`). Returns `"en_GB"` when the header is
/// absent or yields an empty string.
pub fn detect_locale(accept_language: Option<&str>) -> String {
    let parsed = accept_language.and_then(|h| {
        h.split(',').next().map(|s| {
            s.split(';')
                .next()
                .map(|part| part.trim().replace('-', "_"))
                .unwrap_or_default()
        })
    });
    match parsed {
        Some(s) if !s.is_empty() => s,
        _ => "en_GB".to_string(),
    }
}

/// Extracts the value of the `userConsent` cookie from a raw `Cookie` header value.
///
/// Returns `None` when the header is absent, the cookie is not present, or the value is not
/// one of the recognised consent states (`"accepted"` or `"declined"`). Unrecognised values
/// are silently discarded to prevent cookie-derived content from reaching HTML/JS output.
pub fn get_consent_cookie(cookie_header: Option<&str>) -> Option<String> {
    let header = cookie_header?;
    header.split(';').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let name = parts.next()?.trim();
        let value = parts.next()?.trim();
        if name == "userConsent" && (value == "accepted" || value == "declined") {
            Some(value.to_string())
        } else {
            None
        }
    })
}

/// Extracts locale and consent state from a live `worker::Request`.
pub fn extract_from_request(req: &Request) -> (String, Option<String>) {
    let accept_lang = req.headers().get("accept-language").ok().flatten();
    let cookie = req.headers().get("cookie").ok().flatten();
    let locale = detect_locale(accept_lang.as_deref());
    let consent = get_consent_cookie(cookie.as_deref());
    (locale, consent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_locale_exact_match() {
        assert_eq!(detect_locale(Some("fr_FR")), "fr_FR");
    }

    #[test]
    fn detect_locale_normalises_hyphen() {
        assert_eq!(detect_locale(Some("en-GB")), "en_GB");
    }

    #[test]
    fn detect_locale_strips_quality() {
        assert_eq!(detect_locale(Some("de-DE;q=0.9")), "de_DE");
    }

    #[test]
    fn detect_locale_picks_first_tag() {
        assert_eq!(detect_locale(Some("fr-FR,en-GB;q=0.8")), "fr_FR");
    }

    #[test]
    fn detect_locale_defaults_when_absent() {
        assert_eq!(detect_locale(None), "en_GB");
    }

    #[test]
    fn detect_locale_defaults_on_empty() {
        assert_eq!(detect_locale(Some("")), "en_GB");
    }

    #[test]
    fn get_consent_cookie_present() {
        assert_eq!(
            get_consent_cookie(Some("session=abc; userConsent=accepted; foo=bar")),
            Some("accepted".to_string())
        );
    }

    #[test]
    fn get_consent_cookie_declined() {
        assert_eq!(
            get_consent_cookie(Some("userConsent=declined")),
            Some("declined".to_string())
        );
    }

    #[test]
    fn get_consent_cookie_absent() {
        assert_eq!(get_consent_cookie(Some("session=abc; foo=bar")), None);
    }

    #[test]
    fn get_consent_cookie_no_header() {
        assert_eq!(get_consent_cookie(None), None);
    }

    #[test]
    fn get_consent_cookie_handles_whitespace() {
        assert_eq!(
            get_consent_cookie(Some("  userConsent = accepted  ")),
            Some("accepted".to_string())
        );
    }
}
