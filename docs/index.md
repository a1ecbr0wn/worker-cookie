---
layout: docs
title: "cookie-worker | Cookie Consent Banner for Cloudflare Workers"
nav_order: 1
permalink: /
---

<!-- markdownlint-configure-file {
  "MD033": false,
  "MD041": false
} -->

## Cookie-worker

The `cookie-worker` is a [Rust library](https://crates.io/crates/cookie-worker)
that adds a cookie consent banner to any Cloudflare Worker site — without touching
your site's code. Use
[this template](https://github.com/a1ecbr0wn/worker-cookie-template)
to create your own repository containing just your configuration, and let GitHub
Actions handle the deploy.

It acts as a transparent reverse proxy in front of your existing Cloudflare Worker
or Pages site. HTML responses pass through the worker, which injects the banner
HTML, CSS, and JavaScript before returning them to the browser. Non-HTML responses
(images, scripts, JSON, etc.) are passed through unmodified.

## How it works

1. A visitor requests a page from your site.
2. The `cookie-worker` worker intercepts the request and fetches it from your
   upstream site.
3. If the response is HTML, the banner is injected before `</body>` and the theme
   CSS is injected into `<head>`.
4. If the visitor already has a `userConsent` cookie, the banner is rendered hidden
   and consent is applied immediately — no flash of the banner on repeat visits.
5. The visitor's choice (accepted / declined) is stored in a `userConsent` cookie
   valid for one year.
6. Essential scripts are always loaded. Tracking scripts are only loaded when the
   visitor accepts.
