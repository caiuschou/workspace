# Design Patterns

## 1. Builder Pattern

Fluent configuration with sensible defaults:

```rust
let result = OpenCode::open(
    OpenOptions::default()
        .project_path("/path/to/project")
        .chat_content("Analyze this code")
        .stream_output(true)
        .wait_for_response_ms(60_000)
).await?;
```

**Implementation Pattern:**

```rust
impl OpenOptions {
    pub fn project_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_path = Some(path.into());
        self
    }
    // ... other setters
}
```

## 2. Extension Methods (impl Client)

API modules extend `Client` without modifying the core:

```rust
// session/mod.rs
impl Client {
    pub async fn session_create(&self, ...) -> Result<Session, Error>;
    pub async fn session_send_message_async(&self, ...) -> Result<(), Error>;
}

// file.rs
impl Client {
    pub async fn file_list(&self, ...) -> Result<Value, Error>;
    pub async fn file_status(&self, ...) -> Result<Value, Error>;
}
```

**Benefits:**
- **Separation of Concerns**: Each file owns its API surface.
- **Discoverability**: IDE autocomplete shows all available methods.
- **Extensibility**: Add new modules without touching existing code.

## 3. Graceful Degradation

The SDK handles various server versions and edge cases:

```rust
// Try multiple endpoint variants
let items = self.session_list_messages_at(&path, directory).await;
if items.as_ref().map(|v| v.is_empty()).unwrap_or(true) {
    let path_plural = format!("{}/session/{}/messages", ...);
    if let Ok(plural_items) = self.session_list_messages_at(&path_plural, ...).await {
        // Use alternative endpoint
    }
}
```

## 4. Async Callback Pattern

Streaming with user-provided callbacks:

```rust
subscribe_and_stream_until_done(
    &client,
    directory,
    session_id,
    |text| {
        print!("{}", text);
        std::io::Write::flush(&mut std::io::stdout()).ok();
    },
).await?;
```

---

# Concurrency Model

## Async Runtime

The SDK is built on `tokio` with full async/await support:

- All I/O operations are non-blocking
- `Client` is `Clone + Send + Sync`, safe for concurrent use
- SSE streaming runs in spawned tasks with cancellation support

## SSE + Message Flow

When waiting for AI responses:

```rust
// Spawn SSE listener task
let event_handle = tokio::spawn(async move {
    subscribe_and_stream_until_done(&client, dir, session_id, on_text).await
});

// Send message (non-blocking)
client.session_send_message_async(session_id, ...).await?;

// Wait for completion with timeout
match timeout(Duration::from_millis(wait_ms), rx).await {
    Ok(_) => { /* SSE completed */ }
    Err(_) => { event_handle.abort(); /* Timeout */ }
}
```

## Cancellation

- SSE tasks can be aborted via `JoinHandle::abort()`
- `oneshot::channel` used for completion signaling
- Timeout wraps the completion receiver, not the entire operation
