use governor::{clock::DefaultClock, state::NotKeyed, state::InMemoryState, Quota, RateLimiter};
use reqwest::Client as HttpClient;
use std::num::NonZeroU32;
use std::time::Duration;
use tracing::{debug, warn};

use crate::error::{Result, ScraperError};

/// HTTP client for scraping GitLab documentation
pub struct DocScraperClient {
    http_client: HttpClient,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    base_url: String,
    max_retries: u32,
}

impl DocScraperClient {
    /// Create a new documentation scraper client
    pub fn new() -> Result<Self> {
        // Rate limit: 1 request per second, burst of 5
        let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
        let rate_limiter = RateLimiter::direct(quota);

        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ScraperError::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            rate_limiter,
            base_url: "https://docs.gitlab.com".to_string(),
            max_retries: 3,
        })
    }

    /// Set the base URL (for testing)
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Fetch a page with retry logic
    pub async fn fetch_page(&self, path: &str) -> Result<String> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        self.fetch_with_retry(&url, self.max_retries).await
    }

    /// Fetch with exponential backoff retry
    async fn fetch_with_retry(&self, url: &str, max_retries: u32) -> Result<String> {
        let mut delay = Duration::from_millis(500);
        let mut last_error = None;

        for attempt in 0..=max_retries {
            // Respect rate limit
            self.rate_limiter.until_ready().await;

            debug!("Fetching {} (attempt {}/{})", url, attempt + 1, max_retries + 1);

            match self.fetch_once(url).await {
                Ok(content) => {
                    if attempt > 0 {
                        debug!("Success on retry attempt {}", attempt + 1);
                    }
                    return Ok(content);
                }
                Err(e) => {
                    // Don't retry on 404
                    let is_404 = matches!(e, ScraperError::HttpError(ref req_err) if req_err.status().map_or(false, |s| s.as_u16() == 404));
                    if is_404 {
                        return Err(e);
                    }

                    last_error = Some(e);

                    // Don't retry on last attempt
                    if attempt >= max_retries {
                        break;
                    }

                    let error_msg = last_error.as_ref().map(|e| e.to_string()).unwrap_or_default();
                    warn!("Request failed: {}, retrying in {:?}...", error_msg, delay);
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, Duration::from_secs(10));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ScraperError::max_retries_exceeded(url)))
    }

    /// Single fetch attempt
    async fn fetch_once(&self, url: &str) -> Result<String> {
        let response = self
            .http_client
            .get(url)
            .header("User-Agent", "gitlab-api-doc-scraper/0.1.0")
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let text = response.text().await?;
            Ok(text)
        } else if status.as_u16() == 404 {
            Err(ScraperError::network(format!("Not found: {}", url)))
        } else if status.as_u16() == 429 {
            Err(ScraperError::RateLimitExceeded)
        } else {
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(ScraperError::network(format!("HTTP {}: {}", status.as_u16(), text)))
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Default for DocScraperClient {
    fn default() -> Self {
        Self::new().expect("Failed to create DocScraperClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = DocScraperClient::new();
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url(), "https://docs.gitlab.com");
    }
}
