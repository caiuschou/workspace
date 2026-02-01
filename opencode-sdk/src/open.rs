//! OpenCode::open - connect to or start the OpenCode server.
//!
//! Provides a one-shot entry point that either connects to an existing server
//! or spawns `opencode serve` and waits for it to be ready.
//! Supports project path and initial chat content.

use crate::client::Client;
use tracing::{debug, info};
use crate::error::Error;
use crate::server::{
    detect_command, install_opencode, kill_by_pid, spawn_server, DetectResult, SpawnOptions,
};
use crate::event;
use crate::session::{CreateSessionRequest, MessageListItem, Part, SendMessageRequest, Session};
use reqwest::Client as ReqwestClient;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::{sleep, timeout};

/// Poll interval when waiting for server to become ready.
const SERVER_POLL_INTERVAL_MS: u64 = 500;

/// Options for `OpenCode::open`.
///
/// Use the builder pattern for customization:
/// ```ignore
/// OpenCode::open(
///     OpenOptions::default()
///         .project_path("/path/to/project")
///         .chat_content("分析这段代码")
/// )
/// ```
#[derive(Debug, Clone)]
pub struct OpenOptions {
    /// Server hostname (default: "127.0.0.1").
    pub hostname: String,

    /// Server port (default: 4096).
    pub port: u16,

    /// Whether to auto-start the server if not running (default: true).
    pub auto_start: bool,

    /// Command to run for server (default: "opencode").
    pub command: String,

    /// Extra arguments for `opencode serve`.
    pub server_args: Vec<String>,

    /// Timeout for server health check when probing (ms).
    pub health_check_timeout_ms: u64,

    /// Max time to wait for server to become ready after spawn (ms).
    pub startup_timeout_ms: u64,

    /// Project directory. Used as cwd when spawning serve, and as `directory`
    /// query param for session API calls.
    pub project_path: Option<PathBuf>,

    /// Initial chat message. When set, a session is created and this message
    /// is sent after opening. The created session is returned.
    pub chat_content: Option<String>,

    /// Working directory for the server process (overridden by project_path if set).
    pub working_directory: Option<PathBuf>,

    /// When true, if opencode is not found, attempt to install it via npm/brew/curl.
    pub auto_install: bool,

    /// Max time to wait for AI response after sending chat_content (ms). Default 5 min.
    pub wait_for_response_ms: u64,

    /// When true, stream assistant output to logs in real-time via event subscription.
    pub stream_output: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            hostname: "127.0.0.1".to_string(),
            port: 4096,
            auto_start: true,
            command: "opencode".to_string(),
            server_args: Vec::new(),
            health_check_timeout_ms: 3000,
            startup_timeout_ms: 30_000,
            project_path: None,
            chat_content: None,
            working_directory: None,
            auto_install: true,
            wait_for_response_ms: 300_000, // 5 minutes
            stream_output: false,
        }
    }
}

impl OpenOptions {
    /// Sets the hostname.
    pub fn hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = hostname.into();
        self
    }

    /// Sets the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets whether to auto-start the server.
    pub fn auto_start(mut self, auto_start: bool) -> Self {
        self.auto_start = auto_start;
        self
    }

    /// Sets the command to run (e.g. "opencode" or full path).
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = command.into();
        self
    }

    /// Sets extra server arguments.
    pub fn server_args(mut self, args: Vec<String>) -> Self {
        self.server_args = args;
        self
    }

    /// Sets the health check timeout in milliseconds.
    pub fn health_check_timeout_ms(mut self, ms: u64) -> Self {
        self.health_check_timeout_ms = ms;
        self
    }

    /// Sets the startup timeout in milliseconds.
    pub fn startup_timeout_ms(mut self, ms: u64) -> Self {
        self.startup_timeout_ms = ms;
        self
    }

    /// Sets the project directory. Used as cwd when spawning serve and for session API.
    pub fn project_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_path = Some(path.into());
        self
    }

    /// Sets the initial chat message. When provided, a session is created and
    /// this message is sent after opening.
    pub fn chat_content(mut self, content: impl Into<String>) -> Self {
        self.chat_content = Some(content.into());
        self
    }

    /// Sets the working directory for the server process.
    pub fn working_directory(mut self, dir: Option<PathBuf>) -> Self {
        self.working_directory = dir;
        self
    }

    /// Sets whether to auto-install opencode when not found.
    pub fn auto_install(mut self, enable: bool) -> Self {
        self.auto_install = enable;
        self
    }

    /// Sets max time to wait for AI response after sending chat (ms).
    pub fn wait_for_response_ms(mut self, ms: u64) -> Self {
        self.wait_for_response_ms = ms;
        self
    }

    /// Sets whether to stream assistant output in real-time via event subscription.
    pub fn stream_output(mut self, enable: bool) -> Self {
        self.stream_output = enable;
        self
    }
}

/// Handle to a server process started by this SDK.
///
/// Use [`ServerHandle::shutdown`] to gracefully terminate the server.
#[derive(Debug)]
pub struct ServerHandle {
    pid: u32,
}

impl ServerHandle {
    /// Creates a new handle for the given process ID.
    pub fn new(pid: u32) -> Self {
        Self { pid }
    }

    /// Returns the process ID.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Gracefully shuts down the server (SIGTERM on Unix, taskkill on Windows).
    ///
    /// This is best-effort; errors (e.g. process already exited) are ignored.
    pub fn shutdown(&self) {
        kill_by_pid(self.pid);
    }
}

/// Result of `OpenCode::open`.
#[derive(Debug)]
pub struct OpenResult {
    /// Connected client.
    pub client: Client,
    /// Server handle if we started the server.
    pub server: Option<ServerHandle>,
    /// Session if `chat_content` was provided.
    pub session: Option<Session>,
    /// Last assistant message (agent reply) when chat_content was provided and we waited.
    /// Contains text parts and tool call parts (tool_name, tool_input, tool_output).
    pub assistant_reply: Option<crate::session::MessageListItem>,
}

/// OpenCode SDK namespace.
///
/// Use [`OpenCode::open`] to connect to or start the OpenCode server.
#[derive(Debug)]
pub struct OpenCode;

impl OpenCode {
    /// Opens OpenCode: connects to existing server or starts one.
    ///
    /// If the server is already running at the given address, returns a client
    /// connected to it. Otherwise spawns `opencode serve` in the project directory
    /// (if set) and waits for it to be ready.
    ///
    /// When `project_path` is set, it is used as the working directory for the
    /// serve process and for session API calls. When `chat_content` is set, a
    /// session is created and the message is sent; the session is returned.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opencode_sdk::{OpenCode, OpenOptions};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), opencode_sdk::Error> {
    ///     let result = OpenCode::open(
    ///         OpenOptions::default()
    ///             .project_path("/path/to/project")
    ///             .chat_content("分析项目结构")
    ///     ).await?;
    ///     println!("version: {:?}", result.client.health().await?.version);
    ///     if let Some(s) = result.server {
    ///         s.shutdown();
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn open(options: OpenOptions) -> Result<OpenResult, Error> {
        info!("OpenCode::open started");
        let base_url = format!("http://{}:{}", options.hostname, options.port);
        let working_dir = options
            .project_path
            .as_ref()
            .or(options.working_directory.as_ref())
            .map(|p| p.as_path());

        debug!(base_url = %base_url, "options: hostname={} port={}", options.hostname, options.port);
        if let Some(wd) = working_dir {
            debug!(working_dir = ?wd);
        }

        if !options.auto_start {
            info!("auto_start=false, connecting to existing server only");
            let client = Client::new(&base_url);
            let (session, assistant_reply) =
                maybe_send_chat(
            &client,
            working_dir,
            &options.chat_content,
            options.wait_for_response_ms,
            options.stream_output,
        )
        .await?;
            return Ok(OpenResult {
                client,
                server: None,
                session,
                assistant_reply,
            });
        }

        // 1. Detect command, optionally install if not found
        info!("step 1: detect opencode command");
        let mut detect = detect_command(&options.command);
        debug!(available = detect.available, path = ?detect.path);

        if !detect.available && options.auto_install && options.command == "opencode" {
            info!("opencode not found, attempting auto-install");
            if let Ok(installed) = install_opencode() {
                info!(path = %installed, "opencode installed successfully");
                detect = DetectResult {
                    available: true,
                    path: Some(installed),
                };
            }
        }

        if !detect.available {
            info!("opencode command not found, aborting");
            return Err(Error::CommandNotFound(options.command));
        }

        let command_path = detect
            .path
            .as_deref()
            .unwrap_or(&options.command);
        info!(command = %command_path, "opencode command ready");

        // 2. Health check - is server already running?
        info!("step 2: health check");
        let health_client = ReqwestClient::builder()
            .timeout(Duration::from_millis(options.health_check_timeout_ms))
            .build()
            .unwrap_or_else(|_| ReqwestClient::new());
        if check_server_healthy(&base_url, &health_client).await {
            info!("server already running at {}", base_url);
            let client = Client::new(&base_url);
            let (session, assistant_reply) =
                maybe_send_chat(
            &client,
            working_dir,
            &options.chat_content,
            options.wait_for_response_ms,
            options.stream_output,
        )
        .await?;
            return Ok(OpenResult {
                client,
                server: None,
                session,
                assistant_reply,
            });
        }

        // 3. Spawn server in project directory
        info!("step 3: spawn opencode serve");
        let child = spawn_server(SpawnOptions {
            command: command_path,
            port: options.port,
            hostname: &options.hostname,
            extra_args: &options.server_args,
            working_directory: working_dir,
        })?;

        let pid = child.id();
        info!(pid = pid, "server process spawned");
        drop(child);

        // 4. Poll until ready
        info!("step 4: poll until server ready");
        let poll_interval = Duration::from_millis(SERVER_POLL_INTERVAL_MS);
        let deadline = Duration::from_millis(options.startup_timeout_ms);
        let mut elapsed = Duration::ZERO;

        while elapsed < deadline {
            sleep(poll_interval).await;
            elapsed += poll_interval;

            if check_server_healthy(&base_url, &health_client).await {
                info!(elapsed_ms = elapsed.as_millis(), "server ready");
                let client = Client::new(&base_url);
                let (session, assistant_reply) =
                    maybe_send_chat(
            &client,
            working_dir,
            &options.chat_content,
            options.wait_for_response_ms,
            options.stream_output,
        )
        .await?;
                info!("OpenCode::open completed successfully");
                return Ok(OpenResult {
                    client,
                    server: Some(ServerHandle::new(pid)),
                    session,
                    assistant_reply,
                });
            }
        }

        // Timeout - try to kill the process we started
        info!("startup timeout, killing server process");
        kill_by_pid(pid);
        Err(Error::StartupTimeout {
            url: base_url,
            timeout_ms: options.startup_timeout_ms,
        })
    }
}

/// Logs assistant reply parts (text, tool calls, reasoning, etc.) at info level.
fn log_assistant_reply(reply: &MessageListItem) {
    info!(parts_count = reply.parts.len(), "assistant reply received");
    let text = reply.text_content();
    if !text.is_empty() {
        const PREVIEW_LEN: usize = 2000;
        let preview: String = text.chars().take(PREVIEW_LEN).collect();
        let truncated = text.len() > PREVIEW_LEN;
        info!("assistant reply text (len={}, truncated={}):\n---\n{}\n---", text.len(), truncated, preview);
    } else {
        info!("assistant reply has no text content");
    }
    for (i, part) in reply.parts.iter().enumerate() {
        match part.part_type.as_str() {
            "tool" | "tool_call" => {
                info!(part_index = i, tool_name = ?part.tool_name, finished = ?part.finished, "assistant part tool call");
            }
            "tool_result" => {
                info!(part_index = i, tool_name = ?part.tool_name, tool_call_id = ?part.tool_call_id, is_error = ?part.is_error, "assistant part tool result");
            }
            "reasoning" => {
                if let Some(r) = part.reasoning.as_ref().filter(|s| !s.is_empty()) {
                    const LEN: usize = 500;
                    let preview: String = r.chars().take(LEN).collect();
                    let suffix = if r.len() > LEN { "\n..." } else { "" };
                    info!("assistant part[{}] reasoning:\n---\n{}{}\n---", i, preview, suffix);
                } else {
                    info!(part_index = i, "assistant part reasoning");
                }
            }
            "image" | "image_url" => {
                info!(part_index = i, image_url = ?part.image_url, "assistant part image");
            }
            "binary" => {
                info!(part_index = i, "assistant part binary");
            }
            "finish" => {
                info!(part_index = i, finish_reason = ?part.finish_reason, "assistant part finish");
            }
            _ => {
                if let Some(t) = part.text.as_ref().filter(|s| !s.is_empty()) {
                    const LEN: usize = 500;
                    let preview: String = t.chars().take(LEN).collect();
                    let suffix = if t.len() > LEN { "\n..." } else { "" };
                    info!("assistant part[{}] {}:\n---\n{}{}\n---", i, part.part_type, preview, suffix);
                } else {
                    info!(part_index = i, part_type = %part.part_type, "assistant part (no text)");
                }
            }
        }
    }
}

/// Fetches the last assistant message with content in one request (no polling).
fn fetch_last_assistant_message(
    messages: &[MessageListItem],
) -> Option<MessageListItem> {
    let last_assistant = messages
        .iter()
        .rev()
        .find(|m| m.info.role.eq_ignore_ascii_case("assistant"))?;
    let has_content = !last_assistant.text_content().is_empty() || !last_assistant.parts.is_empty();
    if has_content {
        Some(last_assistant.clone())
    } else {
        None
    }
}

/// Sends message with streaming: subscribes to SSE, sends, waits for completion via stream (no polling), then fetches reply once.
async fn run_with_stream(
    client: &Client,
    directory: Option<&Path>,
    session_id: &str,
    content: &str,
    wait_for_response_ms: u64,
) -> Result<Option<MessageListItem>, Error> {
    let (tx, rx) = oneshot::channel::<()>();
    let client_clone = client.clone();
    let dir = directory.map(|p| p.to_path_buf());
    let session_id_clone = session_id.to_string();

    let event_handle = tokio::spawn(async move {
        let dir_ref = dir.as_deref();
        let _ = event::subscribe_and_stream_until_done(
            &client_clone,
            dir_ref,
            &session_id_clone,
            |text| {
                info!("assistant stream: {}", text);
                print!("{}", text);
                let _ = std::io::Write::flush(&mut std::io::stdout());
            },
        )
        .await;
        let _ = tx.send(());
    });

    client
        .session_send_message_async(
            session_id,
            directory,
            SendMessageRequest {
                parts: vec![Part::text(content)],
            },
        )
        .await?;
    info!("message sent (streaming)");

    let timeout_duration = Duration::from_millis(wait_for_response_ms);
    match timeout(timeout_duration, rx).await {
        Ok(Ok(())) | Ok(Err(_)) => {
            event_handle.abort();
            let messages = client.session_list_messages(session_id, directory).await?;
            info!(count = messages.len(), "received message list (after SSE completion)");
            Ok(fetch_last_assistant_message(&messages))
        }
        Err(_) => {
            event_handle.abort();
            info!(timeout_ms = wait_for_response_ms, "wait for SSE completion timeout");
            Err(Error::WaitResponseTimeout {
                timeout_ms: wait_for_response_ms,
            })
        }
    }
}

/// Sends message without streaming; optionally waits for reply via SSE completion (no polling).
async fn run_without_stream(
    client: &Client,
    directory: Option<&Path>,
    session_id: &str,
    content: &str,
    wait_for_response_ms: u64,
) -> Result<Option<MessageListItem>, Error> {
    client
        .session_send_message_async(
            session_id,
            directory,
            SendMessageRequest {
                parts: vec![Part::text(content)],
            },
        )
        .await?;
    info!("message sent");

    if wait_for_response_ms > 0 {
        info!(timeout_ms = wait_for_response_ms, "step: wait for assistant response (SSE completion)");
        let (tx, rx) = oneshot::channel::<()>();
        let client_clone = client.clone();
        let dir = directory.map(|p| p.to_path_buf());
        let session_id_clone = session_id.to_string();

        let event_handle = tokio::spawn(async move {
            let dir_ref = dir.as_deref();
            let _ = event::subscribe_and_stream_until_done(
                &client_clone,
                dir_ref,
                &session_id_clone,
                |_| {},
            )
            .await;
            let _ = tx.send(());
        });

        let timeout_duration = Duration::from_millis(wait_for_response_ms);
        match timeout(timeout_duration, rx).await {
            Ok(Ok(())) | Ok(Err(_)) => {
                event_handle.abort();
                let messages = client.session_list_messages(session_id, directory).await?;
                info!(count = messages.len(), "received message list (after SSE completion)");
                Ok(fetch_last_assistant_message(&messages))
            }
            Err(_) => {
                event_handle.abort();
                info!(timeout_ms = wait_for_response_ms, "wait for SSE completion timeout");
                Err(Error::WaitResponseTimeout {
                    timeout_ms: wait_for_response_ms,
                })
            }
        }
    } else {
        Ok(None)
    }
}

/// Creates session and sends message when chat_content is provided.
/// Waits for AI response when wait_for_response_ms > 0.
/// When stream_output is true, subscribes to events and logs text deltas in real-time.
async fn maybe_send_chat(
    client: &Client,
    directory: Option<&Path>,
    chat_content: &Option<String>,
    wait_for_response_ms: u64,
    stream_output: bool,
) -> Result<(Option<Session>, Option<MessageListItem>), Error> {
    let content = match chat_content {
        Some(s) if !s.is_empty() => s,
        _ => {
            debug!("no chat_content, skipping session creation");
            return Ok((None, None));
        }
    };

    info!("step: create session");
    let session = client
        .session_create(directory, CreateSessionRequest { title: None })
        .await?;
    info!(session_id = %session.id, "session created");

    info!("step: send message (prompt_async)");
    let session_id = session.id.clone();

    let assistant_reply = if stream_output && wait_for_response_ms > 0 {
        run_with_stream(client, directory, &session_id, content, wait_for_response_ms).await?
    } else {
        run_without_stream(client, directory, &session_id, content, wait_for_response_ms).await?
    };

    if let Some(ref reply) = assistant_reply {
        log_assistant_reply(reply);
    }
    Ok((Some(session), assistant_reply))
}

/// Checks if the OpenCode server is responding at the given base URL.
async fn check_server_healthy(base_url: &str, client: &ReqwestClient) -> bool {
    debug!(%base_url, "health check");
    let url = format!("{}/global/health", base_url);

    let ok = match client.get(&url).send().await {
        Ok(res) => res.status().is_success(),
        Err(e) => {
            debug!(error = %e, "health check failed");
            false
        }
    };
    if ok {
        debug!("health check ok");
    }
    ok
}
