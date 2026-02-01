//! Health check for OpenCode server.

use reqwest::Client as ReqwestClient;
use tracing::debug;

/// Checks if the OpenCode server is responding at the given base URL.
pub(crate) async fn check_server_healthy(base_url: &str, client: &ReqwestClient) -> bool {
    debug!(%base_url, "health check");
    let url = format!("{}/global/health", base_url);

    let ok = match client.get(&url).send().await {
        Ok(res) => res.status().is_success(),
        Err(e) => {
            debug!(error = %e, "health check failed");
            false
        }
    };
    if ok {
        debug!("health check ok");
    }
    ok
}
