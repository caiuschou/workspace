//! Configuration for Zoekt MCP server.
//!
//! - `ZOEKT_BASE_URL`: base URL (default `http://127.0.0.1:6070`)
//! - `ZOEKT_USERNAME` / `ZOEKT_PASSWORD`: optional HTTP Basic auth; both must be set to enable.

const DEFAULT_ZOEKT_BASE_URL: &str = "http://127.0.0.1:6070";

/// Zoekt API base URL. Uses env `ZOEKT_BASE_URL` or default.
pub fn zoekt_base_url() -> String {
    std::env::var("ZOEKT_BASE_URL").unwrap_or_else(|_| DEFAULT_ZOEKT_BASE_URL.to_string())
}

/// HTTP Basic auth for Zoekt. Uses `ZOEKT_USERNAME` and `ZOEKT_PASSWORD`; returns `Some((user, pass))` only when both are set.
pub fn zoekt_basic_auth() -> Option<(String, String)> {
    let user = std::env::var("ZOEKT_USERNAME").ok()?;
    let pass = std::env::var("ZOEKT_PASSWORD").ok()?;
    Some((user, pass))
}
