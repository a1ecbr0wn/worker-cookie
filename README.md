# cookie-worker

A Cloudflare Worker written in Rust that injects a cookie consent banner into HTML responses. It intercepts page requests, detects the user's locale, and injects a themed banner with accept/decline controls before serving the page. Consent is persisted in a `userConsent` browser cookie; subsequent requests skip the banner and load scripts immediately.

## Features

- **7 themes**: Hacker, Minimal, Dark Elegant, Sky, Sage, Fire, Earth
- **5 positioning styles**: centre, bottom, top, box-bottom-right, box-bottom-left
- **Multi-locale**: configure banner text and buttons per locale (e.g. `en_GB`, `fr_FR`, `de_DE`)
- **Script gating**: essential scripts always load; tracking scripts only load with consent
- **MutationObserver**: catches dynamically-injected tracking scripts and removes them if consent was declined
- **Consent revocation**: cookie icon in the corner lets users reopen the banner at any time
- **Server-side bypass**: when `userConsent` cookie is already set, the banner is pre-hidden and scripts are loaded inline — no client-side cookie read needed

## Prerequisites

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

1. Copy `config/worker.toml` content into `.dev.vars`:

   ```sh
   # .dev.vars
   WORKER_CONFIG="""
   [banner.en_GB]
   theme = "hacker"
   style = "box-bottom-right"
   overlay_opacity = 50
   message = "We use cookies to improve your experience."

   [buttons.en_GB]
   accept_label = "Accept All"
   decline_label = "Decline Non-Essential"

   [privacy_policy.en_GB]
   url = "https://example.com/privacy"
   link_text = "Read our privacy policy"

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

   The worker listens on `http://localhost:8787` and proxies requests to the upstream URL derived from the incoming request.

## Configuration reference

Configuration is a TOML string passed via the `WORKER_CONFIG` environment variable. All sections that accept locale codes (e.g. `en_GB`, `fr_FR`) can be repeated for as many locales as needed.

### `[banner.<locale>]`

| Key               | Type   | Description                                               |
|-------------------|--------|-----------------------------------------------------------|
| `theme`           | string | Theme name: `hacker`, `minimal`, `dark-elegant`, `sky`, `sage`, `fire`, `earth` |
| `style`           | string | Position: `centre`, `bottom`, `top`, `box-bottom-right`, `box-bottom-left` |
| `overlay_opacity` | u8     | Background overlay opacity 0–100 (ignored for box styles) |
| `message`         | string | Banner body text                                          |

### `[buttons.<locale>]`

| Key             | Type   | Description                  |
|-----------------|--------|------------------------------|
| `accept_label`  | string | Accept button text           |
| `decline_label` | string | Decline button text          |

### `[privacy_policy.<locale>]` _(optional)_

| Key         | Type   | Description           |
|-------------|--------|-----------------------|
| `url`       | string | Privacy policy URL    |
| `link_text` | string | Link display text     |

### `[scripts]`

| Key         | Type                              | Description                                      |
|-------------|-----------------------------------|--------------------------------------------------|
| `essential` | `[{ name, src }]`                 | Always loaded regardless of consent              |
| `tracking`  | `[{ name, src }]`                 | Only loaded when user accepts                    |

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

Test coverage includes: config parsing, cookie parsing, locale detection, banner injection, script filtering, visibility state, locale fallback, and edge cases (missing `</head>`, missing `</body>`, malformed cookies).

## Project structure

```
src/
  lib.rs        — fetch event handler
  config.rs     — TOML config deserialization
  consent.rs    — locale detection, cookie parsing
  banner.rs     — HTML/JS banner renderer
  injector.rs   — HTML rewriter (CSS + banner injection)
assets/
  themes.css    — all 7 themes and 5 position styles
config/
  worker.toml   — configuration template
specs/
  cookie-requirements.md   — feature requirements
  cookie-banner-themes.html — visual theme mockups
```
