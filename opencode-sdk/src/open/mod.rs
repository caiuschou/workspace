//! OpenCode::open - connect to or start the OpenCode server.
//!
//! Provides a one-shot entry point that either connects to an existing server
//! or spawns `opencode serve` and waits for it to be ready.
//! Supports project path and initial chat content.

use crate::error::Error;
use crate::server::{
    detect_command, install_opencode, kill_by_pid, spawn_server, DetectResult, SpawnOptions,
};
use reqwest::Client as ReqwestClient;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

mod chat;
mod health;
mod logging;
mod options;

pub use options::{OpenOptions, OpenResult, ServerHandle};

/// Poll interval when waiting for server to become ready.
const SERVER_POLL_INTERVAL_MS: u64 = 500;

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

        debug!(
            base_url = %base_url,
            "options: hostname={} port={}",
            options.hostname,
            options.port
        );
        if let Some(wd) = working_dir {
            debug!(working_dir = ?wd);
        }

        if !options.auto_start {
            info!("auto_start=false, connecting to existing server only");
            let (client, _, session, assistant_reply) =
                chat::connect_and_maybe_chat(&base_url, working_dir, &options).await?;
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
        if health::check_server_healthy(&base_url, &health_client).await {
            info!("server already running at {}", base_url);
            let (client, _, session, assistant_reply) =
                chat::connect_and_maybe_chat(&base_url, working_dir, &options).await?;
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

            if health::check_server_healthy(&base_url, &health_client).await {
                info!(elapsed_ms = elapsed.as_millis(), "server ready");
                let (client, _, session, assistant_reply) =
                    chat::connect_and_maybe_chat(&base_url, working_dir, &options).await?;
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
