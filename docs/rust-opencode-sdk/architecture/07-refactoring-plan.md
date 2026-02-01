# Refactoring Plan

This section outlines prioritized code refactoring tasks to improve maintainability, reduce duplication, and strengthen the API without breaking changes. Tasks are ordered by impact and effort.

## Phase 1: High-Impact, Low-Risk

### 1.1 Extract `directory` Query Parameter Helper

**Problem:** The pattern of adding `directory` to HTTP requests is repeated 10+ times across `event.rs`, `file.rs`, and `session/mod.rs`:

```rust
// Repeated pattern
let mut req = self.http().get(&url);
if let Some(dir) = directory {
    if let Some(s) = dir.to_str() {
        req = req.query(&[("directory", s)]);
    }
}
```

**Solution:** Add a `RequestBuilder` extension trait in a new module `src/request.rs`:

```rust
// src/request.rs
use reqwest::RequestBuilder;
use std::path::Path;

pub(crate) trait RequestBuilderExt {
    fn with_directory(self, directory: Option<&Path>) -> Self;
}

impl RequestBuilderExt for RequestBuilder {
    fn with_directory(mut self, directory: Option<&Path>) -> Self {
        if let Some(dir) = directory.and_then(|p| p.to_str()) {
            self = self.query(&[("directory", dir)]);
        }
        self
    }
}
```

**Files to modify:** `event.rs`, `file.rs`, `session/mod.rs`

---

### 1.2 Unify `run_with_stream` and `run_without_stream` in open.rs

**Problem:** `run_with_stream` and `run_without_stream` share ~60 lines of duplicated logic (SSE subscription, timeout handling, `fetch_last_assistant_message`). Only the `on_text` callback differs.

**Solution:** Extract common flow into a single `run_send_and_wait` with an enum for callback type:

```rust
enum StreamMode {
    /// Stream text to stdout in real-time
    StreamToStdout,
    /// Silently wait (no output)
    Silent,
    /// Custom callback
    Custom(Box<dyn FnMut(&str) + Send>),
}

async fn run_send_and_wait(
    client: &Client,
    directory: Option<&Path>,
    session_id: &str,
    content: &str,
    wait_for_response_ms: u64,
    mode: StreamMode,
) -> Result<Option<MessageListItem>, Error>
```

**Files to modify:** `open.rs`

---

### 1.3 Extract `finish_open` Helper in open.rs

**Problem:** `OpenResult { client, server, session, assistant_reply }` is constructed in 4 places with identical structure. The `maybe_send_chat` call and result assembly are repeated.

**Solution:** Extract a helper that takes the post-chat state:

```rust
async fn connect_and_maybe_chat(
    base_url: &str,
    working_dir: Option<&Path>,
    options: &OpenOptions,
) -> Result<(Client, Option<ServerHandle>, Option<Session>, Option<MessageListItem>), Error> {
    let client = Client::new(base_url);
    let (session, assistant_reply) = maybe_send_chat(
        &client,
        working_dir,
        &options.chat_content,
        options.wait_for_response_ms,
        options.stream_output,
    ).await?;
    Ok((client, None, session, assistant_reply))
}
```

Then each branch (auto_start=false, already running, after spawn) calls this and adds `server: Some/None` as needed.

**Files to modify:** `open.rs`

---

## Phase 2: Type Safety and API Consistency

### 2.1 ClientBuilder::build() Return Result

**Problem:** `ClientBuilder::build()` uses `.expect("reqwest client build")`, which can panic if reqwest fails (e.g., TLS init issues on some platforms).

**Solution:** Return `Result<Client, Error>` and add `Error::ClientBuildFailed(String)`:

```rust
pub fn build(self) -> Result<Client, Error> {
    let http = ReqwestClient::builder()
        .timeout(self.timeout.unwrap_or(Duration::from_secs(30)))
        .build()
        .map_err(|e| Error::ClientBuildFailed(e.to_string()))?;
    Ok(Client { base_url: self.base_url, http })
}
```

**Note:** This is a breaking change. Consider a `try_build()` method that returns `Result` while keeping `build()` for backward compatibility (with a `#[deprecated]` or doc note about panic risk).

**Files to modify:** `client.rs`, `error.rs`, callers of `Client::new` (which uses `builder().build()`)

---

### 2.2 Introduce `ProjectDirectory` Newtype

**Problem:** `Option<&Path>` for `directory` is passed through many layers. It's easy to forget or pass the wrong value. No type-level distinction between "server cwd" and "explicit project path."

**Solution:** Introduce a newtype (or enum) for project context:

```rust
/// Project directory for API calls. Use `None` for server's cwd.
#[derive(Debug, Clone)]
pub struct ProjectDirectory(Option<PathBuf>);

impl ProjectDirectory {
    pub fn none() -> Self { Self(None) }
    pub fn from_path(path: impl Into<PathBuf>) -> Self { Self(Some(path.into())) }
    pub fn as_path(&self) -> Option<&Path> { self.0.as_deref() }
}
```

Use `ProjectDirectory` in `OpenOptions`, `OpenResult`, and API methods. Reduces `Option<&Path>` parameter count and improves discoverability.

**Files to modify:** New `src/project.rs`, `open.rs`, `session/mod.rs`, `file.rs`, `event.rs`

---

### 2.3 Concrete Types for file_list, file_status, session_diff

**Problem:** These methods return `serde_json::Value`, forcing callers to manually parse and risking runtime errors.

**Solution:** Define response types per API spec (see [08-roadmap](08-roadmap.md)). Start with `#[serde(untagged)]` or `#[serde(deny_unknown_fields)]` where appropriate to catch schema drift.

**Files to modify:** `file.rs`, `session/mod.rs`, new types in `src/file.rs` and `src/session/diff.rs`

---

## Phase 3: Observability and Configuration

### 3.1 Configurable Event Logging

**Problem:** `event.rs` logs full JSON payload at `INFO` level for every SSE event. In high-throughput scenarios this is noisy and can impact performance.

**Solution:** Add an `EventLogLevel` (or use `RUST_LOG` more granularly):

```rust
// Option A: Use tracing levels, document RUST_LOG=opencode_sdk::event=trace for full payload
// Option B: Add OpenOptions.event_debug_logging: bool (default false)
```

Move full-payload logging to `trace!` or `debug!`. Keep completion and error events at `info!`.

**Files to modify:** `event.rs`

---

### 3.2 Extract SSE Connection Setup in event.rs

**Problem:** `subscribe_and_stream` and `subscribe_and_stream_until_done` duplicate URL construction, header setup, and stream creation (~15 lines each).

**Solution:** Extract `connect_sse(client, directory) -> impl Stream<Item = Result<Event, Error>>`:

```rust
async fn connect_sse(
    client: &Client,
    directory: Option<&Path>,
) -> Result<impl Stream<Item = Result<Event, Error>>, Error> {
    let url = format!("{}/event", client.base_url());
    let mut req = client.http().get(&url).with_directory(directory);
    // ... build request and return stream
}
```

Then both public functions consume the stream with different logic.

**Files to modify:** `event.rs`

---

### 3.3 Logging Strategy for log_assistant_reply

**Problem:** `log_assistant_reply` is a 50-line function with repetitive `match` arms and magic constants (PREVIEW_LEN, LEN). Hard to maintain.

**Solution:** Extract `log_part(part: &Part, index: usize)` and use a small table or macro for part-type → log format. Consider moving to `session/mod.rs` or a `session/logging.rs` submodule if it grows.

**Files to modify:** `open.rs`

---

## Phase 4: Structural Improvements

### 4.1 Split open.rs by Responsibility

**Problem:** `open.rs` is ~640 lines. It mixes: OpenOptions/OpenResult types, OpenCode::open orchestration, chat flow, health check, and reply logging.

**Solution:** Split into:

```
src/open/
├── mod.rs         # OpenCode, OpenOptions, OpenResult, open()
├── options.rs     # OpenOptions + builder (or keep in mod)
├── chat.rs        # maybe_send_chat, run_with_stream, run_without_stream
├── health.rs      # check_server_healthy
└── logging.rs     # log_assistant_reply
```

Or keep single file but add `// --- Section ---` headers and consider extracting `chat` and `health` as modules if the file exceeds 800 lines.

---

### 4.2 session_list_messages Fallback Logic

**Problem:** `session_list_messages` tries `/message`, then `/messages`, then retries without directory. The logic is nested and hard to follow.

**Solution:** Use a small loop or iterator over `(path, directory)` combinations:

```rust
let attempts = [
    (format!("{}/session/{}/message", ...), directory),
    (format!("{}/session/{}/messages", ...), directory),
    (format!("{}/session/{}/message", ...), None), // fallback no dir
];
for (path, dir) in attempts {
    if let Ok(items) = self.session_list_messages_at(&path, dir).await {
        if !items.is_empty() {
            return Ok(items);
        }
    }
}
Ok(vec![])
```

---

### 4.3 Server Module Testability

**Problem:** `install.rs` and `spawn.rs` use `std::process::Command` directly, making unit tests require real npm/brew/curl. No dependency injection.

**Solution:** Introduce a `CommandRunner` trait (or similar) for process execution. Default impl uses `Command`; test impl can inject mock behavior. Lower priority unless adding integration tests for install/spawn.

---

## Refactoring Checklist

| Task | Phase | Effort | Breaking |
|------|-------|--------|----------|
| 1.1 directory query helper | 1 | S | No |
| 1.2 unify run_with_* | 1 | M | No |
| 1.3 finish_open helper | 1 | S | No |
| 2.1 ClientBuilder Result | 2 | S | Yes* |
| 2.2 ProjectDirectory newtype | 2 | M | Yes |
| 2.3 concrete response types | 2 | L | Yes |
| 3.1 event log level | 3 | S | No |
| 3.2 SSE connection extraction | 3 | M | No |
| 3.3 log_assistant_reply | 3 | S | No |
| 4.1 split open.rs | 4 | M | No |
| 4.2 session_list_messages | 4 | S | No |
| 4.3 CommandRunner trait | 4 | L | No |

*2.1 can be done backward-compatibly with `try_build()`.

---

## Suggested Implementation Order

1. **1.1** — Quick win, reduces boilerplate immediately.
2. **1.2, 1.3** — Simplify open.rs control flow.
3. **3.1** — Reduce log noise before other work.
4. **3.2** — Clean up event.rs.
5. **4.2** — Simplify session fallback logic.
6. **2.1** — Add `try_build()`, document `build()` panic.
7. **2.2, 2.3** — When ready for a minor version bump with API changes.
