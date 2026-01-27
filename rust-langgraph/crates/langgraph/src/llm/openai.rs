//! OpenAI 兼容的 LLM 客户端（Chat Completions API）。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::client::LlmClient;
use super::error::LlmError;
use super::types::{ChatMessage, ChatRequest, ChatResponse, MessageRole, Usage};

/// OpenAI 兼容配置。
#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    /// API Key，通常来自 `OPENAI_API_KEY` 环境变量。
    pub api_key: String,
    /// Base URL，默认 `https://api.openai.com/v1`，兼容兼容端（如 Azure、其他代理）。
    pub base_url: String,
    /// 模型 ID，如 `gpt-4o-mini`、`gpt-4o`。
    pub model: String,
    /// 默认 temperature，未在请求中指定时使用。
    pub default_temperature: Option<f32>,
}

impl OpenAiConfig {
    /// 从环境变量构造：`OPENAI_API_KEY` 必填，`OPENAI_BASE_URL`、`OPENAI_MODEL` 可选。
    pub fn from_env() -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| LlmError::Auth("OPENAI_API_KEY not set".to_string()))?;
        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        Ok(Self {
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            default_temperature: Some(0.7),
        })
    }
}

/// 请求体中 messages 项（OpenAI 格式）。
#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

fn role_to_str(r: &MessageRole) -> &'static str {
    match r {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
    }
}

#[derive(Debug, Serialize)]
struct OpenAiRequestBody {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageOut,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessageOut {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug)]
pub struct OpenAiClient {
    config: OpenAiConfig,
    client: reqwest::Client,
}

impl OpenAiClient {
    /// 使用给定配置构造客户端。
    pub fn new(config: OpenAiConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    /// 从环境变量构造（需 `OPENAI_API_KEY`）。
    pub fn from_env() -> Result<Self, LlmError> {
        OpenAiConfig::from_env().map(Self::new)
    }

    fn build_body(&self, req: &ChatRequest) -> OpenAiRequestBody {
        let messages: Vec<OpenAiMessage> = req
            .messages
            .iter()
            .map(|m| OpenAiMessage {
                role: role_to_str(&m.role).to_string(),
                content: m.content.clone(),
            })
            .collect();
        let temperature = req
            .temperature
            .or(self.config.default_temperature);
        OpenAiRequestBody {
            model: self.config.model.clone(),
            messages,
            temperature,
            max_tokens: req.max_tokens,
        }
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = self.build_body(&req);
        let res = self
            .client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;
        let status = res.status();
        let text = res.text().await.map_err(|e| LlmError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(match status.as_u16() {
                401 => LlmError::Auth(text),
                403 => LlmError::Auth(text),
                429 => LlmError::RateLimit(text),
                400..=499 => LlmError::InvalidRequest(text),
                _ => LlmError::ApiError(text),
            });
        }
        let parsed: OpenAiResponse = serde_json::from_str(&text)
            .map_err(|e| LlmError::Parsing(format!("{e}: {text}")))?;
        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        let usage = parsed
            .usage
            .map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
            })
            .unwrap_or_default();
        Ok(ChatResponse { content, usage })
    }
}
