//! HTTP 请求工具：按 url、method、headers、body 发请求。
//!
//! - `HttpFetcher`: 注入的同步 HTTP 后端（mock 或真实）
//! - `HttpRequestTool`: 实现 `Tool`，参数 url、method、headers、body
//! - `MockHttpFetcher`: 单元测试用，返回可配置的 body
//! - `ReqwestHttpFetcher`: 真实 HTTP（需 feature `http`）

use crate::error::ToolError;
use crate::tool::Tool;
use serde_json::Value;
use std::collections::HashMap;

/// HTTP 后端 trait：同步发起请求并返回响应体字符串。
///
/// Used by `HttpRequestTool::execute`. Tests use `MockHttpFetcher`;
/// production uses `ReqwestHttpFetcher` when feature `http` is enabled.
pub trait HttpFetcher: Send + Sync {
    /// 发起请求，返回响应体；错误时返回 `ToolError`。
    fn fetch(
        &self,
        url: &str,
        method: &str,
        headers: &HashMap<String, String>,
        body: Option<&str>,
    ) -> Result<String, ToolError>;
}

/// Mock HTTP 后端：返回预设的 body，供单元测试使用。
///
/// Used by `HttpRequestTool` in tests; inject via `HttpRequestTool::new(Box::new(MockHttpFetcher::new(...)))`.
#[derive(Debug, Clone)]
pub struct MockHttpFetcher {
    /// 每次 `fetch` 返回的字符串。
    pub response_body: String,
}

impl MockHttpFetcher {
    /// 新建 mock，所有请求返回 `response_body`。
    pub fn new(response_body: impl Into<String>) -> Self {
        Self {
            response_body: response_body.into(),
        }
    }
}

impl HttpFetcher for MockHttpFetcher {
    fn fetch(
        &self,
        _url: &str,
        _method: &str,
        _headers: &HashMap<String, String>,
        _body: Option<&str>,
    ) -> Result<String, ToolError> {
        Ok(self.response_body.clone())
    }
}

/// HTTP 请求工具：对给定 url 发 method 请求，支持 headers 和 body。
///
/// Uses an injected `HttpFetcher` so tests can use `MockHttpFetcher` and production
/// can use `ReqwestHttpFetcher` (when feature `http` is enabled). Interacts with
/// `ToolRegistry::execute` via `Tool::execute`.
pub struct HttpRequestTool {
    fetcher: Box<dyn HttpFetcher>,
}

impl HttpRequestTool {
    /// 使用给定的 fetcher 构造。测试传 `MockHttpFetcher`，生产可传 `ReqwestHttpFetcher`。
    pub fn new(fetcher: Box<dyn HttpFetcher>) -> Self {
        Self { fetcher }
    }
}

impl Tool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "Makes an HTTP request to the given URL. Supports method, headers, and optional body."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "Request URL" },
                "method": { "type": "string", "description": "HTTP method, default GET", "default": "GET" },
                "headers": { "type": "object", "description": "Optional headers as key-value object" },
                "body": { "type": "string", "description": "Optional request body" }
            },
            "required": ["url"]
        })
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let url = args
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::ValidationFailed("missing or non-string 'url'".into()))?
            .to_string();
        let method = args
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("GET")
            .to_uppercase();
        let headers: HashMap<String, String> = args
            .get("headers")
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        let body = args.get("body").and_then(Value::as_str).map(String::from);

        let resp = self.fetcher.fetch(
            &url,
            &method,
            &headers,
            body.as_deref(),
        )?;
        Ok(serde_json::json!({ "status": "ok", "body": resp }))
    }
}

#[cfg(feature = "http")]
/// 使用 reqwest blocking 的真实 HTTP 后端。需启用 feature `http`。
///
/// Used by `HttpRequestTool::new(Box::new(ReqwestHttpFetcher))` when making real HTTP calls.
pub struct ReqwestHttpFetcher;

#[cfg(feature = "http")]
impl ReqwestHttpFetcher {
    /// 新建真实 HTTP fetcher。
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl Default for ReqwestHttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "http")]
impl HttpFetcher for ReqwestHttpFetcher {
    fn fetch(
        &self,
        url: &str,
        method: &str,
        headers: &HashMap<String, String>,
        body: Option<&str>,
    ) -> Result<String, ToolError> {
        let client = reqwest::blocking::Client::new();
        let mut req = match method {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            "HEAD" => client.head(url),
            _ => return Err(ToolError::ExecutionFailed(format!("unsupported method: {}", method))),
        };
        for (k, v) in headers {
            req = req.header(k.as_str(), v.as_str());
        }
        if let Some(b) = body {
            req = req.body(b.to_string());
        }
        let resp = req.send().map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
        resp.text().map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_request_tool_mock() {
        let fetcher = MockHttpFetcher::new(r#"{"ok":true}"#);
        let tool = HttpRequestTool::new(Box::new(fetcher));
        let args = serde_json::json!({"url": "https://example.com"});
        let out = tool.execute(args).unwrap();
        assert!(out.get("body").is_some());
        assert_eq!(out.get("body").and_then(Value::as_str), Some(r#"{"ok":true}"#));
    }

    #[test]
    fn http_request_tool_missing_url() {
        let fetcher = MockHttpFetcher::new("");
        let tool = HttpRequestTool::new(Box::new(fetcher));
        let args = serde_json::json!({});
        let err = tool.execute(args).unwrap_err();
        assert!(matches!(err, ToolError::ValidationFailed(_)));
    }
}
