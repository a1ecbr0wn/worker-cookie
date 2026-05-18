---
layout: docs
title: "Configuration | worker-cookie"
nav_order: 3
permalink: /configuration
---

## Configuration

The worker reads its configuration from the `WORKER_CONFIG` environment variable, which must contain valid TOML. Configuration is keyed by locale (e.g. `en_GB`, `fr_FR`, `de_DE`), so you can serve different messages to visitors in different regions.

### Locale resolution

The worker reads the visitor's `Accept-Language` request header and resolves it to a configuration locale using a three-tier fallback:

1. Exact match — `fr_FR` matches `[banner.fr_FR]`
2. Language prefix — `en_US` matches `en_GB` (first key starting with `en`)
3. Default — falls back to `en_GB` if no match is found

### `[banner.<locale>]`

| Field | Type | Description |
|---|---|---|
| `theme` | string | Visual theme. See [Themes](../themes). |
| `style` | string | Banner position. See [Themes](../themes). |
| `overlay_opacity` | integer (0–100) | Background overlay opacity in percent. Use `0` for position styles that don't show an overlay. |
| `message` | string | The consent message shown to the visitor. |

### `[buttons.<locale>]`

| Field | Type | Description |
|---|---|---|
| `accept_label` | string | Label for the accept button. |
| `decline_label` | string | Label for the decline button. |

### `[privacy_policy.<locale>]` _(optional)_

When present, a link to your privacy policy is shown inside the banner.

| Field | Type | Description |
|---|---|---|
| `url` | string | URL of your privacy policy page. |
| `link_text` | string | Visible link text. |

### `[scripts]`

Lists the scripts to load based on consent. Each entry has a `name` (for your reference only) and a `src` (the script URL).

| Field | Type | Description |
|---|---|---|
| `essential` | array | Scripts always loaded, regardless of consent. |
| `tracking` | array | Scripts loaded only when the visitor accepts cookies. |

### Full example

```toml
[banner.en_GB]
theme = "hacker"
style = "box-bottom-right"
overlay_opacity = 0
message = "We use cookies to improve your experience. Essential cookies are always enabled."

[buttons.en_GB]
accept_label = "Accept All"
decline_label = "Decline Non-Essential"

[privacy_policy.en_GB]
url = "https://example.com/privacy"
link_text = "Read our privacy policy"

[banner.fr_FR]
theme = "hacker"
style = "box-bottom-right"
overlay_opacity = 0
message = "Nous utilisons des cookies pour améliorer votre expérience."

[buttons.fr_FR]
accept_label = "Tout accepter"
decline_label = "Refuser les non essentiels"

[privacy_policy.fr_FR]
url = "https://example.com/privacy"
link_text = "Lire notre politique de confidentialité"

[scripts]
essential = [
  { name = "site-core", src = "/js/core.js" }
]
tracking = [
  { name = "google-analytics", src = "https://www.googletagmanager.com/gtag/js" }
]
```
