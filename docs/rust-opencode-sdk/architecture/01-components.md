# Core Components

## SDK 与 Serve API 模块对应

SDK 能力层与 [OpenCode Serve API](../../opencode-serve-api.md) 按模块对应；完整接口与状态见 [opencode-serve-api/README.md](../../opencode-serve-api/README.md)。

| 组件 | Serve API 模块 | 已实现接口 | 状态 |
|------|----------------|------------|------|
| client/ | Global (01) | `GET /global/health`、`POST /global/dispose` | 已实现 |
| instance.rs | Instance (02) | POST /instance/dispose | 已实现 |
| project.rs | Project (03) | GET /project, GET /project/current, PATCH /project/{id} | 已实现 |
| path_vcs.rs | Path & VCS (04) | GET /path, GET /vcs | 已实现 |
| config.rs | Config (05) | GET/PATCH /config, GET /config/providers | 已实现 |
| provider.rs, auth.rs | Provider (06), Auth (07) | GET /provider, OAuth, PUT /auth/{id} | 已实现 |
| session/ | Session / Message (08) | 创建/列表/消息/发送/diff 等 | 已实现 |
| permission.rs, question.rs, command.rs | Permission, Question, Command (09–11) | GET/POST 对应接口 | 已实现 |
| file/ | File (12) | GET /file, GET /file/content, GET /file/status | 已实现 |
| find.rs, lsp_mcp.rs, agent_skill.rs, api_log.rs | Find, LSP/MCP, Agent, Logging (13–16) | 对应 GET/POST | 已实现 |
| event/ | Event (17) | GET /event、GET /global/event (SSE) | 已实现 |
| pty.rs, tui.rs, experimental.rs | PTY, TUI, Experimental (18–20) | 对应 CRUD 与实验接口 | 已实现 |

---

## 1. Client

The HTTP client wraps `reqwest::Client` and provides type-safe access to all OpenCode Server endpoints.

```rust
pub struct Client {
    base_url: String,
    http: ReqwestClient,
}
```

**Design Decisions:**

- **Clone-friendly**: `Client` is `Clone` (via `Arc` in reqwest), enabling safe sharing across async tasks.
- **Builder pattern**: `ClientBuilder` allows timeout configuration while maintaining ergonomic defaults.
- **Internal visibility**: `http()` is `pub(crate)` to allow extension modules to reuse the underlying client.

**API Surface:**

| Method | Endpoint | Description |
|--------|----------|-------------|
| `new(base_url)` | - | Create client with defaults (30s timeout) |
| `builder(base_url)` | - | Create configurable client |
| `health()` | `GET /global/health` | Check server availability（对应 [01-global](../../opencode-serve-api/01-global.md)） |

## 2. OpenCode::open — One-Shot Entry Point

The `open.rs` module provides a unified entry point that handles the entire lifecycle:

```
OpenCode::open(OpenOptions) → OpenResult
```

**Lifecycle Steps:**

```
1. detect_command("opencode")
   └─► Check if opencode binary exists (which/where)

2. install_opencode() [if auto_install && not found]
   └─► Try: npm → brew → curl script

3. check_server_healthy(base_url)
   └─► GET /global/health

4. spawn_server() [if auto_start && not healthy]
   └─► Start `opencode serve --port N --hostname H`
   └─► Poll until healthy or timeout

5. maybe_send_chat() [if chat_content provided]
   └─► session_create()
   └─► session_send_message_async()
   └─► subscribe_and_stream_until_done() [wait for completion]
   └─► session_list_messages() [fetch final reply]
```

**OpenOptions Configuration:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `hostname` | `String` | `"127.0.0.1"` | Server bind address |
| `port` | `u16` | `4096` | Server port |
| `auto_start` | `bool` | `true` | Start server if not running |
| `auto_install` | `bool` | `true` | Install opencode if not found |
| `command` | `String` | `"opencode"` | Binary name or path |
| `project_path` | `Option<PathBuf>` | `None` | Project directory |
| `chat_content` | `Option<String>` | `None` | Initial message to send |
| `stream_output` | `bool` | `false` | Print assistant output in real-time |
| `wait_for_response_ms` | `u64` | `300_000` | Max wait for AI response (5 min) |
| `startup_timeout_ms` | `u64` | `30_000` | Max wait for server startup |
| `health_check_timeout_ms` | `u64` | `3_000` | Health probe timeout |

**OpenResult:**

| Field | Type | Description |
|-------|------|-------------|
| `client` | `Client` | Connected HTTP client |
| `server` | `Option<ServerHandle>` | Handle if SDK started the server |
| `session` | `Option<Session>` | Session if `chat_content` was provided |
| `assistant_reply` | `Option<MessageListItem>` | AI response if waited |

## 3. Server Module

Process lifecycle management for the OpenCode server.

| Submodule | Responsibility |
|-----------|----------------|
| `detect` | Find `opencode` binary via `which`/`where` or absolute path |
| `install` | Auto-install: npm → brew → curl (cascading fallback) |
| `spawn` | Start `opencode serve` with null stdio, return `Child` |
| `shutdown` | Kill process by PID (`SIGTERM` on Unix, `taskkill` on Windows) |

**Platform Abstraction:**

```rust
// detect.rs - Platform-specific command lookup
#[cfg(unix)]
let output = Command::new("which").arg(command).output();

#[cfg(windows)]
let output = Command::new("where").arg(command).output();
```

## 4. Session API

Session management implemented as `impl Client` extension methods；对应 [08-session](../../opencode-serve-api/08-session.md)。

| Method | Endpoint | Description |
|--------|----------|-------------|
| `session_create` | `POST /session` | Create new session |
| `session_send_message_async` | `POST /session/{id}/prompt_async` | Send message (non-blocking) |
| `session_list_messages` | `GET /session/{id}/message[s]` | Fetch message history |
| `session_diff` | `GET /session/{id}/diff` | Get file changes in session |

**Type Definitions:**

```rust
/// Session created by the server.
pub struct Session {
    pub id: String,           // e.g., "ses_abc123"
    pub title: Option<String>,
}

/// Message content part (text, tool call, image, etc.)
pub struct Part {
    pub part_type: String,    // "text", "tool_call", "finish", etc.
    pub text: Option<String>,
    pub reasoning: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<Value>,
    pub tool_output: Option<Value>,
    pub finish_reason: Option<String>,
    // ... additional fields
}

/// Complete message with metadata and parts.
pub struct MessageListItem {
    pub info: MessageInfo,    // id, role
    pub parts: Vec<Part>,
}
```

**Version Compatibility:**

The SDK handles different OpenCode server versions by:
- Trying both `/message` and `/messages` endpoints
- Parsing both `{messages: [...]}` and top-level array formats
- Graceful fallback with `unwrap_or_default()`

## 5. Event Module (SSE)

Real-time event streaming via Server-Sent Events；对应 [17-event](../../opencode-serve-api/17-event.md)。

**Core Functions:**

```rust
/// Stream events and invoke callback for each text delta.
pub async fn subscribe_and_stream<F>(
    client: &Client,
    directory: Option<&Path>,
    session_id: &str,
    on_text: F,
) -> Result<(), Error>
where F: FnMut(&str) + Send;

/// Stream until completion event (session.idle, finish, etc.)
pub async fn subscribe_and_stream_until_done<F>(
    client: &Client,
    directory: Option<&Path>,
    session_id: &str,
    on_text: F,
) -> Result<(), Error>
where F: FnMut(&str) + Send;
```

**Completion Detection:**

The SDK recognizes multiple completion signals per `17-event-format.md`:

| Event Type | Condition | Priority |
|------------|-----------|----------|
| `session.idle` | Always | Recommended |
| `session.status` | `status.type === "idle"` | High |
| `message.part.updated` | `part.type` is `"finish"` or `"step-finish"` | High |
| `message.updated` | `info.finish` present | Medium |
| Legacy | Type contains "finish"/"complete" | Low |

**Text Delta Extraction:**

Priority order for extracting streaming text:
1. `properties.delta` — Incremental chunk (preferred)
2. `properties.part.text` — Full part text
3. `properties.text` or `properties.content` — Fallback

## 6. File API

File system queries implemented as `impl Client` extension；对应 [12-file](../../opencode-serve-api/12-file.md)。

| Method | Endpoint | Returns |
|--------|----------|---------|
| `file_list(path, directory)` | `GET /file?path=...` | `serde_json::Value` |
| `file_status(directory)` | `GET /file/status` | `serde_json::Value` |

> **Note:** These methods return `serde_json::Value` for flexibility. Consider defining concrete types for production use.

## 7. Error Handling

Comprehensive error types using `thiserror`:

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("opencode command not found: {0}")]
    CommandNotFound(String),

    #[error("failed to install opencode: {0}")]
    InstallFailed(String),

    #[error("timeout waiting for AI response after {timeout_ms}ms")]
    WaitResponseTimeout { timeout_ms: u64 },

    #[error("failed to spawn opencode serve: {0}")]
    SpawnFailed(#[source] std::io::Error),

    #[error("server did not become ready within {timeout_ms}ms")]
    StartupTimeout { url: String, timeout_ms: u64 },
}
```

**Error Handling Philosophy:**

- **Typed Errors**: Each error variant captures relevant context for debugging.
- **Source Chaining**: `#[source]` and `#[from]` enable error chain traversal.
- **User-Friendly Messages**: Error messages include actionable hints (e.g., install URL).
- **No Panics**: All fallible operations return `Result<T, Error>`.

## 8. Logging

Structured logging via `tracing` ecosystem.

```rust
pub fn init_logger(log_dir: Option<PathBuf>) 
    -> Option<tracing_appender::non_blocking::WorkerGuard>;
```

**Features:**
- Dual output: stdout + rolling file
- Default log directory: `~/.local/share/opencode-sdk/`
- Default level: `opencode_sdk=debug` (override via `RUST_LOG`)
- Non-blocking file writes via `tracing_appender`

**Log Levels:**
- `INFO`: Major lifecycle events (server start, session create, message send)
- `DEBUG`: API responses, event payloads, internal state changes
- `ERROR`: Failures that need attention
