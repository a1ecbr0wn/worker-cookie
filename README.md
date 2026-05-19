# cookie-worker

A Cloudflare Worker written in Rust that injects a cookie consent banner into HTML
responses. It intercepts page requests, detects the user's locale, and injects a
themed banner with accept/decline controls before serving the page. Consent is
persisted in a `userConsent` browser cookie; subsequent requests skip the banner
and load scripts immediately.

## Features

- **7 themes**: Hacker, Minimal, Dark Elegant, Sky, Sage, Fire, Earth
- **5 positioning styles**: centre, bottom, top, box-bottom-right, box-bottom-left
- **Multi-locale**: configure banner text and buttons per locale (e.g. `en_GB`,
  `fr_FR`, `de_DE`)
- **Script gating**: essential scripts always load; tracking scripts only load
  with consent
- **MutationObserver**: catches dynamically-injected tracking scripts and removes
  them if consent was declined
- **Consent revocation**: cookie icon in the corner lets users reopen the banner
  at any time
- **Server-side bypass**: when `userConsent` cookie is already set, the banner
  is pre-hidden and scripts are loaded inline — no client-side cookie read needed

## Setup

`worker-cookie` is a library — you don’t deploy it directly. Instead, use
[worker-cookie-template](https://github.com/a1ecbr0wn/worker-cookie-template) to
create your own private repository containing just your configuration. The
template wires up the entry point and CI for you; all the banner logic comes from
the worker-cookie crate.

See the [setup documentation](https://cookies.a1ecbr0wn.com/setup) for more infomation.

## Prerequisites for building this repository

- [Rust](https://rustup.rs) with the `wasm32-unknown-unknown` target:

```sh
  rustup target add wasm32-unknown-unknown
```

- [worker-build](https://github.com/cloudflare/workers-rs):

```sh
  cargo install worker-build
```

- [Wrangler v4+](https://developers.cloudflare.com/workers/wrangler/):

```sh
  npm install -D wrangler@latest
```

## Local development

1. Copy `config/cookie-banner.toml` content into `.dev.vars`:

   ```sh
   # .dev.vars
   WORKER_CONFIG="""

   [banner]
   theme = "hacker"
   style = "box-bottom-right"
   overlay_opacity = 50
   message = "We use cookies to improve your experience."

   [buttons]
   accept_label = "Accept All"
   decline_label = "Decline Non-Essential"

   [privacy_policy]
   url = "https://example.com/privacy"
   link_text = "Read our privacy policy"

   [banner.fr_FR]
   theme = "fire"
   style = "box-bottom-left"
   overlay_opacity = 0
   message = "Nous utilisons des cookies pour améliorer votre expérience. Les cookies essentiels sont toujours activés."

   [buttons.fr_FR]
   accept_label = "Tout accepter"
   decline_label = "Refuser les non-essentiels"

   [privacy_policy.fr_FR]
   url = "https://example.com/privacy"
   link_text = "Lire notre politique de confidentialité"

   [scripts]
   essential = [{ name = "site-core", src = "/js/core.js" }]
   tracking  = [
     { name = "google-analytics", src = "https://www.googletagmanager.com/gtag/js" },
   ]
   """
   ```

2. Start the dev server:

   ```sh
   wrangler dev
   ```

   The worker listens on `http://localhost:8787` and proxies requests to the upstream
   URL derived from the incoming request.

## Configuration reference

Configuration is a TOML string passed via the `WORKER_CONFIG` environment variable.
All sections that accept locale codes (e.g. `en_GB`, `en_US`, `fr_FR`) can be repeated
for as many locales as needed, default is no locale.

### `[banner.<locale>]`

| Key               | Type   | Description                                                                     |
| ----------------- | ------ | ------------------------------------------------------------------------------- |
| `theme`           | string | Theme name: `hacker`, `minimal`, `dark-elegant`, `sky`, `sage`, `fire`, `earth` |
| `style`           | string | Position: `centre`, `bottom`, `top`, `box-bottom-right`, `box-bottom-left`      |
| `overlay_opacity` | u8     | Background overlay opacity 0–100 (ignored for box styles)                       |
| `message`         | string | Banner body text                                                                |

### `[buttons.<locale>]`

| Key             | Type   | Description         |
| --------------- | ------ | ------------------- |
| `accept_label`  | string | Accept button text  |
| `decline_label` | string | Decline button text |

### `[privacy_policy.<locale>]` _(optional)_

| Key         | Type   | Description        |
| ----------- | ------ | ------------------ |
| `url`       | string | Privacy policy URL |
| `link_text` | string | Link display text  |

### `[scripts]`

| Key         | Type              | Description                         |
| ----------- | ----------------- | ----------------------------------- |
| `essential` | `[{ name, src }]` | Always loaded regardless of consent |
| `tracking`  | `[{ name, src }]` | Only loaded when user accepts       |

### `[settings]` _(optional)_

| Key      | Type    | Default            | Description                                                                                                                                     |
| -------- | ------- | ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `bottom` | integer | CSS default (16px) | Pixels from the bottom of the viewport for the settings button.                                                                                 |
| `right`  | integer | CSS default (16px) | Pixels from the right of the viewport for the settings button.                                                                                  |
| `color`  | string  | `#c8973f`          | Hex colour for the cookie icon fill (`#rgb`, `#rrggbb`, `#rgba`, `#rrggbbaa`), or `none`/`transparent`. Other formats fall back to the default. |

## Deployment

1. Set `WORKER_CONFIG` as a Wrangler secret (paste the TOML content when prompted):

   ```sh
   wrangler secret put WORKER_CONFIG
   ```

2. Build and deploy:

   ```sh
   worker-build --release
   wrangler deploy
   ```

3. To roll back to the previous version:

   ```sh
   wrangler rollback
   ```

## Running tests

Tests run natively (no Wasm runtime needed) since all business logic is pure Rust:

```sh
cargo test
```

Test coverage includes: config parsing, cookie parsing, locale detection, banner
injection, script filtering, visibility state, locale fallback, and edge cases
(missing `</head>`, missing `</body>`, malformed cookies).

## Project structure

```text
src/
  lib.rs        — fetch event handler
  config.rs     — TOML config deserialization
  consent.rs    — locale detection, cookie parsing
  banner.rs     — HTML/JS banner renderer
  injector.rs   — HTML rewriter (CSS + banner injection)
  utils.rs      — Cloudflare Workers runtime utilities
assets/
  themes.css    — all 7 themes and 5 position styles
config/
  cookie-banner.toml   — configuration template
specs/
  requirements.md          — feature requirements
  cookie-banner-themes.html — visual theme mockups
```
