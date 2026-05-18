# Cookie Consent Banner - Requirements Document

## Project Overview

A Rust-based Cloudflare Worker that injects a cookie consent banner onto web pages to manage user consent for non-essential cookies and third-party scripts.

## Technology Stack

- **Language**: Rust
- **Platform**: Cloudflare Workers (WASM via `worker-build`)
- **Deployment**: Cloudflare Workers runtime
- **Distribution**: Published to crates.io as `worker-cookie`; users deploy via `worker-cookie-template`

## Core Functionality

### 1. Cookie Banner Display

- Banner appears as an overlay on initial page load
- Banner persists until user makes a choice via button click
- No close button (X); only consent/decline buttons available
- Multiple positioning styles available (configurable):
  - **Centre**: Centered overlay, blocks page interaction
  - **Bottom**: Fixed to bottom of viewport, spans full width
  - **Top**: Fixed to top of viewport, spans full width
  - **Box bottom-right**: Fixed box in bottom-right corner
  - **Box bottom-left**: Fixed box in bottom-left corner
- Background overlay opacity configurable (0-100%), applies to all styles
- Page scrollable/clickable behind banner (except centre which blocks interaction)

### 2. User Consent Options

- **Essential Cookies Button**: User accepts essential cookies only
- **Decline Button**: User rejects non-essential cookies and tracking
- Button text customizable via configuration

### 3. Consent Storage

- User's consent choice stored in a browser cookie (`userConsent`)
- Cookie marked as essential (allowed without prior consent)
- Consent preference persists across page visits
- Values: `accepted` or `declined`

### 4. Conditional Script Loading

- On page load, worker checks for existing consent cookie
- If consent exists, respect the stored preference
- If no consent cookie, display banner and wait for user choice
- Only load third-party scripts if user accepted
- Essential scripts and cookies always load regardless of choice
- Mutation observer catches dynamically-injected scripts and respects consent state

### 5. Consent Revocation

- Small cookie icon/button fixed to the viewport corner allows user to reopen banner
- User can change consent choice at any time
- Updated choice overwrites previous cookie value
- Icon is an inline SVG (24×24) — no external request, no emoji dependency
- Icon colour configurable via `[settings]` (see Configuration below); defaults to `#d2ebff`
- Icon position configurable via `[settings]` `bottom` and `right` pixel offsets

### 6. Multi-language Support

- Configuration file supports multiple language sections with locale codes (e.g., `[banner.en_GB]`, `[banner.fr_FR]`, `[banner.de_DE]`)
- Worker detects user language/locale and applies appropriate section
- Banner text, button labels, and messages customizable per locale

### 7. Privacy Policy Link Integration

- Optional in configuration
- If supplied, displays link in banner with configurable text
- If not supplied, link hidden
- Configurable URL and link text per locale

## Styling & Themes

### Available Themes

#### Hacker
- **Background**: Dark navy (`#0a0e27`) with green scanline grid overlay
- **Text**: Neon green (`#00ff00`), monospace (`Courier New`)
- **Border**: 2px solid neon green with green glow box-shadow
- **Buttons**: Outlined neon green; primary button filled green with dark text
- **Title prefix**: `> NOTICE`

#### Minimal
- **Background**: White
- **Text**: Dark (`#1a1a1a`) title in Georgia serif; body in system sans-serif (`#666`)
- **Border**: 3px solid dark top accent, no radius
- **Buttons**: Light grey background; primary button black with white text

#### Dark Elegant
- **Background**: Dark navy gradient (`#1a1a2e` → `#16213e`) with backdrop blur
- **Text**: White title, grey body (`#b0b0b0`); left accent bar in red (`#e94560`)
- **Border**: 4px left border accent, 4px radius
- **Buttons**: Outlined white/grey; primary button red (`#e94560`)

#### Sky
- **Background**: Light blue (`#e3f2fd`) with blue border
- **Text**: Deep blue (`#0d47a1`) title, medium blue body (`#1565c0`)
- **Border**: 2px solid `#1976d2`, 8px radius, soft blue shadow
- **Buttons**: White with blue border; primary button solid blue

#### Sage
- **Background**: Light green (`#c8e6c9`) with green border
- **Text**: Dark green (`#1b5e20`) title, medium green body (`#2e7d32`)
- **Border**: 2px solid `#388e3c`, 8px radius, soft green shadow
- **Buttons**: White with green border; primary button solid green

#### Fire
- **Background**: Red (`#d32f2f`) with dark red border
- **Text**: White title and body
- **Border**: 2px solid `#b71c1c`, 8px radius, soft red shadow
- **Buttons**: Transparent with white border; primary button white with red text

#### Earth
- **Background**: Brown (`#6d4c41`) with darker brown border
- **Text**: Gold (`#ffd700`) title and body
- **Border**: 2px solid `#5d4037`, 8px radius, soft brown shadow
- **Buttons**: Transparent with gold border; primary button solid gold with dark brown text

### Configuration for Themes

- Support multiple CSS themes via configuration
- Allow easy switching between themes without code changes
- Future themes expandable

## Configuration

### Config File Location and Delivery

- Config lives in `config/cookie-banner.toml` in the user's private template repo
- Committed directly — no secrets required (nothing in the config is sensitive)
- Injected at deploy time by CI via `wrangler versions upload --var "WORKER_CONFIG:$(cat config/cookie-banner.toml)"`
- Local development: `wrangler dev --var "WORKER_CONFIG:$(cat config/cookie-banner.toml)"`
- No `.dev.vars` file; no Cloudflare secrets needed for configuration

### Config File Format

Configuration via TOML file with sections for theme, button labels, banner text, and script definitions.

Example structure:
```toml
[banner.en_GB]
theme = "hacker"
style = "box-bottom-right"  # Options: centre, bottom, top, box-bottom-right, box-bottom-left
overlay_opacity = 50        # 0-100, percentage of background overlay opacity
message = "We use cookies to improve your experience. Essential cookies are always enabled."

[buttons.en_GB]
accept_label = "Accept All"
decline_label = "Decline Non-Essential"

[privacy_policy.en_GB]
url = "https://example.com/privacy"
link_text = "Read our privacy policy"

[scripts]
essential = [
  { name = "site-core", src = "/js/core.js" }
]
tracking = [
  { name = "google-analytics", src = "https://www.googletagmanager.com/gtag/js" },
  { name = "facebook-pixel", src = "https://connect.facebook.net/en_US/fbevents.js" }
]
```

### Configurable Elements

- **Theme Selection**: Choose active theme (e.g., "hacker", "light", etc.)
- **Button Labels**: Customize text for both consent buttons per locale
- **Banner Text**: Customizable message displayed on banner per locale
- **Banner Style**: Choose positioning style and overlay opacity
- **Privacy Policy**: Optional URL and link text per locale
- **Script Lists**: Define which scripts are essential vs. non-essential with URLs and metadata
- **Settings button position**: Optional `bottom` and `right` pixel offsets via `[settings]`
- **Settings button icon colour**: Optional CSS colour string via `[settings].color` (default `#d2ebff`)

## Implementation Details

### Essential vs Non-Essential

- **Essential Scripts**: Site functionality, core features (always load)
- **Non-Essential Scripts**: Analytics, advertising, tracking (conditional on consent)

### Consent Cookie Structure

- Cookie name: `userConsent`
- Values: `accepted` or `declined`
- Marked as essential
- Secure and SameSite attributes set appropriately for Cloudflare Workers context
- Persistent across sessions

### Script Injection Strategy

- Worker rewrites HTML response, injects banner HTML + loader script before `</body>` tag
- Loader script includes mutation observer to catch dynamically-injected scripts
- Third-party scripts filtered based on `userConsent` cookie value
- Non-essential scripts blocked until consent obtained

### Worker Logic Flow

1. Worker intercepts page request
2. Check for existing `userConsent` cookie
3. If cookie exists, inject conditional script loader only
4. If no cookie exists, inject banner + script loader
5. Banner displays with accept/decline buttons
6. User clicks button, choice captured and stored in cookie
7. Script loader executes, allowing essential scripts and (if accepted) non-essential scripts

### Library vs Entry Point Split

- `worker-cookie` is a pure library crate (`crate-type = ["cdylib", "rlib"]`) exposing `pub async fn run(req, env, ctx)`
- The `#[event(fetch)]` macro entry point lives in `worker-cookie-template/src/lib.rs`
- This separation allows the library to be published to crates.io and used as a dependency

### Security Requirements

- **XSS via cookie**: `userConsent` cookie value validated server-side to only `"accepted"` or `"declined"` before any HTML/JS interpolation
- **Client-side guard**: JS cookie read uses strict equality (`=== 'accepted' || === 'declined'`), not a truthy check
- **`window.cookieConsent` allowlist**: The public JS function rejects any choice value not in `{'accepted', 'declined'}` before writing the cookie
- **CI injection safety**: GitHub Actions workflow inputs passed via `env:` block, never interpolated directly into `run:` shell steps
- **No hardcoded secrets**: Discord webhook and similar credentials must come from repository secrets, not be committed

## Testing

### Unit Test Coverage

- **Consent cookie creation**: Verify correct name, value, and expiry
- **Cookie parsing**: Verify correct interpretation on subsequent requests
- **Script filtering**: Verify essential scripts always load, non-essential conditional
- **Banner injection**: Verify HTML structure and styling applied correctly
- **Configuration parsing**: Verify valid and invalid configs handled appropriately
- **Edge cases**: Malformed cookies, missing config, concurrent requests
- **Mutation observer**: Verify dynamically-injected scripts are caught and filtered
- **Locale detection**: Verify correct locale section applied
- **Privacy policy link**: Verify link shown/hidden based on configuration

## Deliverables

- [x] Rust Worker source code (`worker-cookie` library crate)
- [x] `worker-cookie-template` GitHub template repository (thin wrapper + CI)
- [x] Configuration file (`config/cookie-banner.toml` in template repo)
- [x] CSS theme files (7 themes: hacker, minimal, dark-elegant, sky, sage, fire, earth)
- [x] Banner HTML/JS injection logic
- [x] Script conditional loading logic with mutation observer
- [x] Consent revocation UI (reopen button/icon)
- [x] Unit tests (47 tests covering consent, injection, config, banner, locale)
- [x] Jekyll documentation site (`docs/`) with setup, configuration, themes showcase
- [ ] Publish `worker-cookie` to crates.io (prerequisite for template to work)

## Future Enhancements

- Additional theme options
- Analytics integration for tracking consent choices
- Cookie category granularity (Analytics, Marketing, Functional, etc.)
