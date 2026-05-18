---
layout: docs
title: "Setup | worker-cookie"
nav_order: 2
permalink: /setup
---

## Setup

`worker-cookie` is a library — you don't deploy it directly. Instead, use
[worker-cookie-template](https://github.com/a1ecbr0wn/worker-cookie-template)
to create your own **private** repository containing just your configuration.
The template wires up the entry point and CI for you; all the banner logic
comes from the `worker-cookie` crate.

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
See the [Configuration](../configuration) page for a full reference.

At minimum you need a `[banner.<locale>]` section, a `[buttons.<locale>]`
section, and a `[scripts]` section. Commit the file — CI reads it and passes
it to the worker at deploy time. No secrets required for the configuration.

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
3. Read `config/worker.toml` and upload a new draft version to Cloudflare

The upload creates a **draft** — it does not go live automatically. To promote
it, run:

```sh
wrangler versions deploy --name=<your-worker-name> --yes
```

Or deploy from the Cloudflare dashboard under **Workers → your worker → Deployments**.

### 6. Route traffic through your worker

**Prerequisite**: your site's DNS must be managed by Cloudflare and the record must
be proxied (orange cloud icon in the Cloudflare DNS dashboard). If your site is not
behind Cloudflare's proxy, workers cannot intercept its traffic.

The worker acts as a transparent proxy: it receives the request, fetches the page
from your origin server, injects the cookie banner into HTML responses, and returns
the result to the visitor. Non-HTML responses (images, scripts, JSON) are passed
through unmodified.

To wire this up:

1. In the [Cloudflare dashboard](https://dash.cloudflare.com), select your account
   and then your site's zone.
2. Go to **Workers Routes** (under the **Workers** section in the left sidebar).
3. Click **Add route**.
4. Set the **Route** pattern to match all pages on your site, e.g.:
   ```
   example.com/*
   ```
5. Set the **Worker** to the name you gave your worker in `wrangler.jsonc`.
6. Save.

From this point on, every request matching the route pattern runs through your
worker. All HTML pages will have the cookie banner injected automatically.

### Releases

To cut a versioned release, trigger the **Tag a release** workflow manually
from the **Actions** tab. It will bump the version in `Cargo.toml`, commit,
tag, and kick off the release workflow which builds, uploads, and creates a
GitHub Release.

### Local development

```sh
wrangler dev --var "WORKER_CONFIG:$(cat config/cookie-banner.toml)"
```
