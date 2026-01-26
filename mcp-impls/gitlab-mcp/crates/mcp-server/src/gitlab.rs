use reqwest::{header, Client as HttpClient};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use crate::error::{GitLabError, Result};

/// GitLab API client
pub struct GitLabClient {
    http_client: HttpClient,
    base_url: Url,
    token: String,
}

impl GitLabClient {
    /// Create a new GitLab client
    pub fn new(base_url: impl AsRef<str>, token: impl AsRef<str>) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())
            .map_err(|e| GitLabError::invalid_parameter(format!("Invalid GitLab URL: {}", e)))?;

        let token = token.as_ref().to_string();

        if token.is_empty() {
            return Err(GitLabError::auth_error("GITLAB_TOKEN is required"));
        }

        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| GitLabError::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            base_url,
            token,
        })
    }

    /// Create client from environment variables
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("GITLAB_URL").unwrap_or_else(|_| "https://gitlab.com".to_string());
        let token = std::env::var("GITLAB_TOKEN")
            .map_err(|_| GitLabError::auth_error("GITLAB_TOKEN environment variable not set"))?;

        Self::new(&base_url, &token)
    }

    /// Get the base URL
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Make a GET request to the GitLab API
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = self.api_url(path);
        let response = self
            .http_client
            .get(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::USER_AGENT, "gitlab-mcp-server/0.1.0")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a GET request with query parameters to the GitLab API
    pub async fn get_with_query<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<T> {
        let mut url = self.api_url(path);
        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in query {
                query_pairs.append_pair(key, value);
            }
        }
        let response = self
            .http_client
            .get(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::USER_AGENT, "gitlab-mcp-server/0.1.0")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the GitLab API
    pub async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.api_url(path);
        let response = self
            .http_client
            .post(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::USER_AGENT, "gitlab-mcp-server/0.1.0")
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a PUT request to the GitLab API
    pub async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.api_url(path);
        let response = self
            .http_client
            .put(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::USER_AGENT, "gitlab-mcp-server/0.1.0")
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a DELETE request to the GitLab API
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = self.api_url(path);
        let response = self
            .http_client
            .delete(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::USER_AGENT, "gitlab-mcp-server/0.1.0")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a GET request and return raw bytes
    pub async fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let url = self.api_url(path);
        let response = self
            .http_client
            .get(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.token))
            .header(header::USER_AGENT, "gitlab-mcp-server/0.1.0")
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let bytes = response
                .bytes()
                .await
                .map_err(|e| GitLabError::ApiError(e))?;
            Ok(bytes.to_vec())
        } else if status.as_u16() == 401 {
            Err(GitLabError::auth_error("Invalid or expired token"))
        } else if status.as_u16() == 404 {
            Err(GitLabError::not_found("Resource not found"))
        } else {
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GitLabError::api_response(status.as_u16(), text))
        }
    }

    /// Handle API response
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| GitLabError::ApiError(e))
        } else if status.as_u16() == 401 {
            Err(GitLabError::auth_error("Invalid or expired token"))
        } else if status.as_u16() == 404 {
            Err(GitLabError::not_found("Resource not found"))
        } else if status.as_u16() == 429 {
            Err(GitLabError::RateLimitExceeded)
        } else {
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(GitLabError::api_response(status.as_u16(), text))
        }
    }

    /// Build full API URL
    fn api_url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("Invalid base URL")
            .push("api");
        url.path_segments_mut()
            .expect("Invalid base URL")
            .push("v4");

        // Append the path
        let path = path.trim_start_matches('/');
        for segment in path.split('/') {
            url.path_segments_mut()
                .expect("Invalid base URL")
                .push(segment);
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_api_url() {
        let client = GitLabClient::new("https://gitlab.com", "test_token").unwrap();
        let url = client.api_url("projects/123");
        assert_eq!(url.as_str(), "https://gitlab.com/api/v4/projects/123");
    }

    #[test]
    fn test_build_api_url_with_leading_slash() {
        let client = GitLabClient::new("https://gitlab.com", "test_token").unwrap();
        let url = client.api_url("/projects/123");
        assert_eq!(url.as_str(), "https://gitlab.com/api/v4/projects/123");
    }
}
