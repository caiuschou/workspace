//! OpenOptions, OpenResult, and ServerHandle for OpenCode::open.

use crate::client::Client;
use crate::server::kill_by_pid;
use crate::session::{MessageListItem, Session};
use std::path::PathBuf;

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
    pub assistant_reply: Option<MessageListItem>,
}
