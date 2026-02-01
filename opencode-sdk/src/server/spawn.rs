//! Server process spawning.
//!
//! Starts the OpenCode server as a detached process via `opencode serve`.

use super::runner::{CommandRunner, DefaultCommandRunner};
use crate::error::Error;
use std::process::Child;
use tracing::info;

/// Options for starting the server via [`spawn_server`].
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

/// Spawns the OpenCode server process (uses default `CommandRunner`).
///
/// Returns the child process handle. The process is spawned with null stdio
/// and will continue running independently after this function returns.
///
/// # Errors
///
/// Returns `Error::SpawnFailed` if the process could not be started.
pub fn spawn_server(opts: SpawnOptions<'_>) -> Result<Child, Error> {
    spawn_server_with_runner(opts, &DefaultCommandRunner)
}

/// Spawns the OpenCode server process using the given `CommandRunner`.
///
/// Use this to inject a mock runner in tests.
pub fn spawn_server_with_runner(
    opts: SpawnOptions<'_>,
    runner: &dyn CommandRunner,
) -> Result<Child, Error> {
    info!(command = %opts.command, port = opts.port, hostname = %opts.hostname, "spawning opencode serve");
    let mut args: Vec<String> = vec![
        "serve".to_string(),
        "--port".to_string(),
        opts.port.to_string(),
        "--hostname".to_string(),
        opts.hostname.to_string(),
    ];
    args.extend(opts.extra_args.iter().cloned());
    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    runner
        .spawn(opts.command, &args_refs, opts.working_directory)
        .map_err(Error::SpawnFailed)
}
