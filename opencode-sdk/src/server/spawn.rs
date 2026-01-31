//! Server process spawning.
//!
//! Starts the OpenCode server as a detached process via `opencode serve`.

use crate::error::Error;
use tracing::info;
use std::process::{Child, Command, Stdio};

/// Options for starting the server.
#[derive(Debug, Clone)]
pub struct SpawnOptions<'a> {
    /// Command to execute (e.g. "opencode" or "/usr/local/bin/opencode").
    pub command: &'a str,
    /// Server port.
    pub port: u16,
    /// Server hostname.
    pub hostname: &'a str,
    /// Extra arguments (e.g. ["--cors", "http://localhost:3000"]).
    pub extra_args: &'a [String],
    /// Working directory for the process.
    pub working_directory: Option<&'a std::path::Path>,
}

/// Spawns the OpenCode server process.
///
/// Returns the child process handle. The process is spawned with null stdio
/// and will continue running independently after this function returns.
///
/// # Errors
///
/// Returns `Error::SpawnFailed` if the process could not be started.
pub fn spawn_server(opts: SpawnOptions<'_>) -> Result<Child, Error> {
    info!(command = %opts.command, port = opts.port, hostname = %opts.hostname, "spawning opencode serve");
    let mut args = vec![
        "serve".to_string(),
        "--port".to_string(),
        opts.port.to_string(),
        "--hostname".to_string(),
        opts.hostname.to_string(),
    ];
    args.extend(opts.extra_args.iter().cloned());

    let mut cmd = Command::new(opts.command);
    cmd.args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(cwd) = opts.working_directory {
        cmd.current_dir(cwd);
    }

    cmd.spawn().map_err(Error::SpawnFailed)
}
