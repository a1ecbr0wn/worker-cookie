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

`worker-cookie` is a library — you don't deploy it directly. Instead, use
[worker-cookie-template](https://github.com/a1ecbr0wn/worker-cookie-template) to
create your own **private** repository containing just your configuration. The
template wires up the entry point and CI for you; all the banner logic comes from
the `worker-cookie` crate.

### Prerequisites

- A [Cloudflare account](https://dash.cloudflare.com/sign-up)
- A Cloudflare API token with Workers edit permissions

### 1. Create your repository from the template

Click **Use this template** on the
[worker-cookie-template GitHub page](https://github.com/a1ecbr0wn/worker-cookie-template)
and choose **Create a new repository**. Make the repository **private** — your
configuration will live here.

Your new repository contains:

| File                        | Purpose                                  |
| --------------------------- | ---------------------------------------- |
| `src/lib.rs`                | Entry point — calls into `worker-cookie` |
| `Cargo.toml`                | Declares the `worker-cookie` dependency  |
| `wrangler.jsonc`            | Worker name and build config             |
| `config/cookie-banner.toml` | Your banner configuration                |
| `.github/workflows/`        | CI to build and upload on every push     |

### 2. Customise your worker name

Open `wrangler.jsonc` and change the `name` field to something unique within
your Cloudflare account:

```jsonc
{
  "name": "my-site-cookie-worker",
  ...
}
```

### 3. Write your configuration

Edit `config/cookie-banner.toml` to match your site's language, message, and scripts.
See the [Configuration](#configuration) section below, or the
[full configuration reference](https://cookies.a1ecbr0wn.com/configuration).

At minimum you need a `[banner.<locale>]` section, a `[buttons.<locale>]` section,
and a `[scripts]` section. Commit the file — CI reads it and passes it to the worker
at deploy time. No secrets required for the configuration.

### 4. Add your Cloudflare API token as a GitHub secret

In your repository, go to **Settings → Secrets and variables → Actions** and add:

| Secret name            | Value                                                          |
| ---------------------- | -------------------------------------------------------------- |
| `CLOUDFLARE_API_TOKEN` | A Cloudflare API token with _Workers Scripts: Edit_ permission |

This is the only secret needed.

### 5. Push and deploy

Commit your changes and push. The CI workflow will:

1. Run `cargo fmt` and `cargo clippy` checks
2. Build the worker to WebAssembly
3. Upload a new draft version to Cloudflare

The upload creates a **draft** — it does not go live automatically. To promote it, run:

```sh
wrangler versions deploy --name=<your-worker-name> --yes
```

Or deploy from the Cloudflare dashboard under **Workers → your worker → Deployments**.

### 6. Route traffic through your worker

Your site's DNS must be managed by Cloudflare and the record must be proxied
(orange cloud). Then in the Cloudflare dashboard:

1. Select your account and site zone.
2. Go to **Workers Routes** in the left sidebar.
3. Click **Add route** and set the pattern to match your site (e.g. `example.com/*`).
4. Set the **Worker** to the name from your `wrangler.jsonc`.

Every HTML request matching the route will have the cookie banner injected automatically.

## Configuration

Configuration is a TOML string in `config/cookie-banner.toml`, passed to the worker
via the `WORKER_CONFIG` environment variable by CI. All locale-keyed sections can be
repeated for as many locales as needed.

### Unqualified defaults

`[banner]`, `[buttons]`, and `[privacy_policy]` can each be written without a locale
suffix. These unqualified sections act as a catch-all for visitors whose locale does
not match any specific entry. This lets you configure a single language without locale
keys at all:

```toml
[banner]
theme = "minimal"
style = "bottom"
overlay_opacity = 0
message = "We use cookies to improve your experience."

[buttons]
accept_label = "Accept"
decline_label = "Decline"
```

You can also mix unqualified defaults with locale-specific sections. The locale-specific
entry takes priority when it matches; the unqualified section is the final fallback.

### Locale resolution

The worker reads the visitor's `Accept-Language` header and resolves it using a
three-tier fallback:

1. Exact match — `fr_FR` matches `[banner.fr_FR]`
2. Language prefix — `en_US` matches `en_GB` (first key starting with `en`)
3. Unqualified default — falls back to `[banner]` if no locale key matches; if there
   is no unqualified default either, the page is passed through unmodified

### `[banner.<locale>]`

| Field             | Type            | Description                                                                                    |
| ----------------- | --------------- | ---------------------------------------------------------------------------------------------- |
| `theme`           | string          | Visual theme. See [Themes](https://cookies.a1ecbr0wn.com/themes).                             |
| `style`           | string          | Banner position. See [Themes](https://cookies.a1ecbr0wn.com/themes).                          |
| `overlay_opacity` | integer (0–100) | Background overlay opacity in percent. Use `0` for position styles that don't show an overlay. |
| `message`         | string          | The consent message shown to the visitor.                                                      |

### `[buttons.<locale>]`

| Field           | Type   | Description                   |
| --------------- | ------ | ----------------------------- |
| `accept_label`  | string | Label for the accept button.  |
| `decline_label` | string | Label for the decline button. |

### `[privacy_policy.<locale>]` _(optional)_

When present, a link to your privacy policy is shown inside the banner.

| Field       | Type   | Description                      |
| ----------- | ------ | -------------------------------- |
| `url`       | string | URL of your privacy policy page. |
| `link_text` | string | Visible link text.               |

### `[scripts]`

| Field       | Type  | Description                                           |
| ----------- | ----- | ----------------------------------------------------- |
| `essential` | array | Scripts always loaded, regardless of consent.         |
| `tracking`  | array | Scripts loaded only when the visitor accepts cookies. |

### `[settings]` _(optional)_

Global settings that apply across all locales. Controls the position and appearance
of the settings button (the cookie icon that lets visitors reopen the banner).

| Field    | Type    | Default            | Description                                                                                                                                     |
| -------- | ------- | ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `bottom` | integer | CSS default (16px) | Pixels from the bottom of the viewport for the settings button.                                                                                 |
| `right`  | integer | CSS default (16px) | Pixels from the right of the viewport for the settings button.                                                                                  |
| `color`  | string  | `#c8973f`          | Hex colour for the cookie icon fill (`#rgb`, `#rrggbb`, `#rgba`, `#rrggbbaa`), or `none`/`transparent`. Other formats fall back to the default. |

### Full example

```toml
[banner]
theme = "hacker"
style = "box-bottom-right"
overlay_opacity = 0
message = "We use cookies to improve your experience. Essential cookies are always enabled."

[buttons]
accept_label = "Accept All"
decline_label = "Decline Non-Essential"

[privacy_policy]
url = "https://example.com/privacy"
link_text = "Read our privacy policy"

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

[settings]
bottom = 24
right = 32
color = "#c8973f"
```

## Contributing

### Prerequisites

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

### Local development

```sh
wrangler dev --var "WORKER_CONFIG:$(cat config/cookie-banner.toml)"
```

### Running tests

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
