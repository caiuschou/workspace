//! OpenAI 兼容的 LLM 客户端（Chat Completions API）。
//!
//! 支持非流式 `chat` 与流式 `chat_stream`（SSE 解析，遇 `data: [DONE]` 结束）。

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::client::LlmClient;
use super::error::LlmError;
use super::stream::{ChatStreamEvent, LlmStreamClient};
use super::types::{ChatRequest, ChatResponse, MessageRole, Usage};

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

/// 流式请求体：在非流式基础上增加 `stream: true`，服务端以 SSE 按 chunk 返回。
#[derive(Debug, Serialize)]
struct OpenAiStreamRequestBody {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    /// 固定为 `true`，启用 Server-Sent Events 流式响应。
    stream: bool,
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

/// 流式 chunk 中的 delta 字段，对应 API 的 `choices[].delta`。
/// 仅解析 `content`，用于 incremental 文本；首条可能含 `role`，此处忽略。
#[derive(Debug, Default, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

/// 流式 chunk 中的单条 choice，对应 API 的 `choices[]`。
#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
}

/// 流式 SSE 单条事件的 JSON 结构，即每行 `data: {...}` 的反序列化类型。
/// 与非流式不同，流式下每块为 `chat.completion.chunk`，用 `delta` 递推内容。
#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Option<Vec<OpenAiStreamChoice>>,
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

    /// 构建流式请求体：与 `build_body` 相同结构，但固定 `stream: true`，供 `chat_stream` 使用。
    fn build_stream_body(&self, req: &ChatRequest) -> OpenAiStreamRequestBody {
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
        OpenAiStreamRequestBody {
            model: self.config.model.clone(),
            messages,
            temperature,
            max_tokens: req.max_tokens,
            stream: true,
        }
    }
}

/// 在后台任务中执行流式请求：解析 SSE，遇 `data: [DONE]` 结束，通过 `tx` 发送事件。
///
/// 流程：POST 带 `stream: true` → 按字节流读取 → 按行拆分，处理 `data: ...` →
/// - `data: [DONE]`：发 `Done(full)` 并退出；
/// - `data: {"choices":[...]}`：取 `choices[0].delta.content`，发 `Token(content)` 并累积到 `full`。
async fn run_sse_stream(
    config: OpenAiConfig,
    client: reqwest::Client,
    body: OpenAiStreamRequestBody,
    tx: mpsc::Sender<ChatStreamEvent>,
) {
    let url = format!("{}/chat/completions", config.base_url);

    // --- 发起请求与 HTTP 错误处理 ---
    let res = match client
        .post(&url)
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let _ = tx
                .send(ChatStreamEvent::Error(LlmError::Network(e.to_string())))
                .await;
            return;
        }
    };
    let status = res.status();
    if !status.is_success() {
        let text = res.text().await.unwrap_or_default();
        let err = match status.as_u16() {
            401 | 403 => LlmError::Auth(text),
            429 => LlmError::RateLimit(text),
            400..=499 => LlmError::InvalidRequest(text),
            _ => LlmError::ApiError(text),
        };
        let _ = tx.send(ChatStreamEvent::Error(err)).await;
        return;
    }

    // --- SSE 字节流 → 行缓冲 → 解析 data 行 ---
    let mut stream = res.bytes_stream();
    let mut buf = Vec::<u8>::new(); // 未读完的行可能跨 chunk，保留在 buf
    let mut full = String::new();   // 已解析的完整回复，用于最后的 Done(full)
    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let bytes = match chunk {
            Ok(b) => b,
            Err(e) => {
                let _ = tx
                    .send(ChatStreamEvent::Error(LlmError::Network(e.to_string())))
                    .await;
                break;
            }
        };
        buf.extend_from_slice(&bytes);
        let s = match std::str::from_utf8(&buf) {
            Ok(x) => x,
            Err(_) => continue, // 可能截断在多字节 UTF-8 中间，等后续 chunk
        };
        for line in s.split('\n') {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if !line.starts_with("data: ") {
                continue;
            }
            let payload = line["data: ".len()..].trim();
            // OpenAI 流结束标记：收到后发 Done 并退出
            if payload == "[DONE]" {
                let _ = tx.send(ChatStreamEvent::Done(full.clone())).await;
                return;
            }
            let chunk: OpenAiStreamChunk = match serde_json::from_str(payload) {
                Ok(c) => c,
                Err(_) => continue,
            };
            // 取 choices[0].delta.content，非空则发 Token 并累积
            if let Some(choices) = chunk.choices.and_then(|c| c.into_iter().next())
                && let Some(ref content) = choices.delta.content
                && !content.is_empty()
            {
                full.push_str(content);
                if tx.send(ChatStreamEvent::Token(content.clone())).await.is_err() {
                    return;
                }
            }
        }
        // 只保留最后一个 \n 之后的未完成行，供下一轮与后续 chunk 拼接
        let start = s.rfind('\n').map(|i| i + 1).unwrap_or(0);
        buf = s.as_bytes()[start..].to_vec();
    }
    // 流提前结束（如断开）仍发一次 Done，带上已累积内容
    let _ = tx.send(ChatStreamEvent::Done(full)).await;
}

impl LlmStreamClient for OpenAiClient {
    /// 发起流式对话：在后台 spawn 任务做 HTTP+SSE 解析，本方法立即返回一个事件流。
    ///
    /// 调用方须处于 tokio runtime 内（如 `#[tokio::main]`），否则 `tokio::spawn` 会 panic。
    /// 事件顺序通常为若干 `Token(...)`，最后一条 `Done(full)`；出错时会有 `Error(...)`。
    fn chat_stream(
        &self,
        req: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = ChatStreamEvent> + Send + '_>> {
        let (tx, rx) = mpsc::channel(32);
        let config = self.config.clone();
        let client = self.client.clone();
        let body = self.build_stream_body(&req);
        tokio::spawn(async move { run_sse_stream(config, client, body, tx).await });
        Box::pin(ReceiverStream::new(rx))
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
