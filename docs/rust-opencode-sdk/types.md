# 类型定义

## 核心

### Session

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}
```

### Message

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub role: String,  // "user" | "assistant"
    #[serde(rename = "createdAt")]
    pub created_at: String,
}
```

### MessageResponse

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MessageResponse {
    pub info: Message,
    pub parts: Vec<Part>,
}
```

### Part

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Part {
    #[serde(rename = "type")]
    pub part_type: String,  // "text" | "tool" | "image" | etc.
    pub text: Option<String>,
    #[serde(rename = "toolName")]
    pub tool_name: Option<String>,
    #[serde(rename = "toolInput")]
    pub tool_input: Option<serde_json::Value>,
    #[serde(rename = "toolOutput")]
    pub tool_output: Option<serde_json::Value>,
}
```

---

## 搜索

### TextMatch

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TextMatch {
    pub path: String,
    pub lines: String,
    #[serde(rename = "lineNumber")]
    pub line_number: usize,
    #[serde(rename = "absoluteOffset")]
    pub absolute_offset: usize,
    pub submatches: Vec<Submatch>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Submatch {
    pub text: String,
    pub start: usize,
    pub end: usize,
}
```

### Symbol

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Symbol {
    pub name: String,
    pub path: Option<String>,
    pub kind: Option<String>,  // "function" | "class" | "variable" | etc.
}
```

---

## 文件

### FileContent

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FileContent {
    #[serde(rename = "type")]
    pub content_type: String,  // "raw" | "patch"
    pub content: String,
}
```

### File

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct File {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,  // "file" | "directory"
}
```

### FileNode

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FileNode {
    pub path: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub children: Option<Vec<FileNode>>,
}
```

---

## 项目 (Project)

### Project

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Project {
    pub path: String,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}
```

### Path

当前工作目录路径信息：

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Path {
    pub path: String,
    #[serde(rename = "absolute")]
    pub absolute_path: String,
    #[serde(rename = "project")]
    pub is_in_project: bool,
}
```

### VcsInfo

版本控制系统信息：

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VcsInfo {
    #[serde(rename = "type")]
    pub vcs_type: Option<String>,  // "git" | "hg" | etc.
    pub branch: Option<String>,
    pub commit: Option<String>,
    #[serde(rename = "remoteUrl")]
    pub remote_url: Option<String>,
    pub status: Option<VcsStatus>,
    #[serde(rename = "isDirty")]
    pub is_dirty: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct VcsStatus {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub untracked: Vec<String>,
}
```

### SessionStatus

会话状态（用于 `/session/status` 端点）：

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionStatus {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub state: String,  // "idle" | "busy" | "error"
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
```

---

## 配置

### Config

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub model: Option<String>,
    pub provider: Option<serde_json::Value>,
    pub agent: Option<serde_json::Value>,
    pub permission: Option<serde_json::Value>,
}
```

### Provider

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub models: Vec<String>,
}
```

### ProvidersInfo

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ProvidersInfo {
    pub providers: Vec<Provider>,
    #[serde(rename = "default")]
    pub default_models: std::collections::HashMap<String, String>,
}
```

---

## 代理

### Agent

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}
```

---

## 命令

### Command

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub arguments: Vec<CommandArgument>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CommandArgument {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub arg_type: String,
    pub required: bool,
}
```

---

## 事件

### Event

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub properties: serde_json::Value,
}
```

### 事件类型

| 事件类型 | 说明 |
|----------|------|
| `server.connected` | 服务器已连接 |
| `session.created` | 会话已创建 |
| `session.deleted` | 会话已删除 |
| `message.created` | 消息已创建 |
| `message.updated` | 消息已更新 |

---

## 错误

### Error

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("JSON deserialize error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## 选项类型

### CreateSessionOptions

```rust
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct CreateSessionOptions {
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
```

### SendMessageOptions

```rust
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct SendMessageOptions {
    #[serde(rename = "messageId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<Model>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(rename = "noReply", skip_serializing_if = "Option::is_none")]
    pub no_reply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub parts: Vec<Part>,
}
```

### Model

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct Model {
    #[serde(rename = "providerID")]
    pub provider_id: String,
    #[serde(rename = "modelID")]
    pub model_id: String,
}
```

### FindFilesOptions

```rust
#[derive(Debug, Clone, Default)]
pub struct FindFilesOptions {
    pub query: String,
    pub r#type: Option<String>,     // "file" | "directory"
    pub directory: Option<String>,
    pub limit: Option<u32>,
}
```
