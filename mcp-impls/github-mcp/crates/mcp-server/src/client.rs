use reqwest::Client;
use serde_json::Value;
use std::sync::{Arc, RwLock};
use mcp_server::ServerError;

/// GitHub API 客户端状态
#[derive(Clone)]
pub struct GithubState {
    /// 当前使用的 token
    pub current_token: Option<String>,
    /// 所有可用的 token (从配置加载)
    pub tokens: Option<Arc<RwLock<crate::config::TokenConfig>>>,
    pub client: Client,
}

impl GithubState {
    pub fn new() -> Self {
        Self {
            current_token: std::env::var("GITHUB_TOKEN").ok(),
            tokens: None,
            client: Client::new(),
        }
    }

    pub fn with_config(mut self) -> Self {
        if let Ok(config) = crate::config::TokenConfig::load() {
            if self.current_token.is_none() {
                self.current_token = config.get_default_token().map(|s| s.to_string());
            }
            self.tokens = Some(Arc::new(RwLock::new(config)));
        }
        self
    }

    /// 切换到指定名称的 token
    pub fn switch_token(&self, name: &str) -> Result<(), String> {
        if let Some(tokens) = &self.tokens {
            let config = tokens.read().unwrap();
            if let Some(_token) = config.get_token(name) {
                // 注意：这里需要更新 current_token，但由于不可变引用，
                // 实际使用时需要通过其他方式处理
                return Err("需要重新创建客户端以切换 token".to_string());
            }
            return Err(format!("Token '{}' 不存在", name));
        }
        Err("未启用 token 配置".to_string())
    }

    pub fn auth_header(&self) -> Option<String> {
        self.current_token.as_ref().map(|t| format!("Bearer {}", t))
    }

    /// 获取当前 token 名称
    pub fn current_token_name(&self) -> Option<String> {
        if let Some(tokens) = &self.tokens {
            let config = tokens.read().unwrap();
            let current = self.current_token.as_ref()?;
            for (name, token) in config.tokens.iter() {
                if token == current {
                    return Some(name.clone());
                }
            }
        }
        None
    }
}

impl Default for GithubState {
    fn default() -> Self {
        Self::new()
    }
}

/// GitHub API 响应
#[derive(Debug)]
pub struct GithubResponse {
    pub status: reqwest::StatusCode,
    pub body: String,
    pub json: Option<Value>,
}

impl GithubResponse {
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

/// GitHub API 客户端
#[derive(Clone)]
pub struct GithubClient {
    state: Arc<GithubState>,
    pub base_url: String,
}

impl GithubClient {
    pub fn new(state: Arc<GithubState>) -> Self {
        Self {
            state,
            base_url: "https://api.github.com".to_string(),
        }
    }

    pub fn with_base_url(state: Arc<GithubState>, base_url: String) -> Self {
        Self { state, base_url }
    }

    /// 构建 GET 请求
    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.state.client.get(&url);
        if let Some(auth) = self.state.auth_header() {
            req = req.header("Authorization", auth);
        }
        req = req.header("User-Agent", "github-mcp")
            .header("Accept", "application/vnd.github.v3+json");
        req
    }

    /// 构建 POST 请求
    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.state.client.post(&url);
        if let Some(auth) = self.state.auth_header() {
            req = req.header("Authorization", auth);
        }
        req = req.header("User-Agent", "github-mcp")
            .header("Accept", "application/vnd.github.v3+json");
        req
    }

    /// 构建 PUT 请求
    pub fn put(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.state.client.put(&url);
        if let Some(auth) = self.state.auth_header() {
            req = req.header("Authorization", auth);
        }
        req = req.header("User-Agent", "github-mcp")
            .header("Accept", "application/vnd.github.v3+json");
        req
    }

    /// 构建 PATCH 请求
    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.state.client.patch(&url);
        if let Some(auth) = self.state.auth_header() {
            req = req.header("Authorization", auth);
        }
        req = req.header("User-Agent", "github-mcp")
            .header("Accept", "application/vnd.github.v3+json");
        req
    }

    /// 构建 DELETE 请求
    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.state.client.delete(&url);
        if let Some(auth) = self.state.auth_header() {
            req = req.header("Authorization", auth);
        }
        req = req.header("User-Agent", "github-mcp")
            .header("Accept", "application/vnd.github.v3+json");
        req
    }

    /// 发送请求并获取响应
    pub async fn send(&self, req: reqwest::RequestBuilder) -> Result<GithubResponse, ServerError> {
        let resp = req
            .send()
            .await
            .map_err(|e| ServerError::Handler(format!("Request failed: {}", e)))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| ServerError::Handler(format!("Response read failed: {}", e)))?;

        let json = serde_json::from_str::<Value>(&body).ok();

        Ok(GithubResponse { status, body, json })
    }

    /// 发送请求并期望成功的响应
    pub async fn send_ok(&self, req: reqwest::RequestBuilder) -> Result<GithubResponse, ServerError> {
        let resp = self.send(req).await?;
        if !resp.is_success() {
            return Err(ServerError::Handler(format!("GitHub API error ({}): {}", resp.status, resp.body)));
        }
        Ok(resp)
    }
}
