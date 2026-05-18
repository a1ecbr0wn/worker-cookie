use worker::*;

mod banner;
mod config;
mod consent;
mod injector;
mod utils;

/// Handles incoming fetch requests, proxying HTML responses through the cookie consent banner injector.
///
/// For non-HTML content-types, returns the upstream response unmodified. When the request
/// already carries a `userConsent` cookie, injects only the script loader; otherwise injects
/// the full banner UI.
///
/// Call this from your own `#[event(fetch)]` handler — see `worker-cookie-template` for a
/// minimal entry point.
pub async fn run(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    utils::set_panic_hook();
    let cfg = config::load(&env)?;
    let url = req.url()?;
    let (locale, existing_consent) = consent::extract_from_request(&req);

    let upstream = Fetch::Request(Request::new(url.as_str(), Method::Get)?);
    let mut upstream_resp = upstream.send().await?;

    let content_type = upstream_resp
        .headers()
        .get("content-type")?
        .map_or_else(String::new, std::convert::identity);

    if !content_type.contains("text/html") {
        return Ok(upstream_resp);
    }

    let html = upstream_resp.text().await?;
    let injected = injector::inject(&html, &cfg, &locale, existing_consent.as_deref());

    let mut resp = Response::ok(injected)?;
    resp.headers_mut()
        .set("content-type", "text/html; charset=utf-8")?;
    Ok(resp)
}
