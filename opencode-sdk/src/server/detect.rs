//! Command detection utilities.
//!
//! Uses `which` on Unix/macOS and `where` on Windows to find the opencode binary in PATH.
//! Accepts absolute paths directly.

use super::runner::{CommandRunner, DefaultCommandRunner};
use std::path::Path;
use tracing::debug;

/// Result of command availability check.
///
/// Returned by [`detect_command`] and [`detect_command_with`].
#[derive(Debug, Clone)]
pub struct DetectResult {
    /// Whether the command is available.
    pub available: bool,
    /// Full path to the command (if found).
    pub path: Option<String>,
}

/// Checks if a command is available (uses default `CommandRunner`).
///
/// If `command` is an absolute path, checks that the file exists and is executable.
/// Otherwise uses `which` (Unix) or `where` (Windows) to find it in PATH.
///
/// # Arguments
///
/// * `command` - Command name (e.g. "opencode") or absolute path (e.g. "/usr/local/bin/opencode")
pub fn detect_command(command: &str) -> DetectResult {
    detect_command_with(command, &DefaultCommandRunner)
}

/// Checks if a command is available using the given `CommandRunner`.
///
/// Use this to inject a mock runner in tests.
pub fn detect_command_with(command: &str, runner: &dyn CommandRunner) -> DetectResult {
    let path = Path::new(command);

    if path.is_absolute() {
        debug!(command = %command, "checking absolute path");
        if path.exists() {
            debug!(path = %command, "absolute path exists");
            return DetectResult {
                available: true,
                path: Some(command.to_string()),
            };
        }
        debug!(path = %command, "absolute path not found");
        return DetectResult {
            available: false,
            path: None,
        };
    }

    #[cfg(unix)]
    let output = runner.run("which", &[command]);

    #[cfg(windows)]
    let output = runner.run("where", &[command]);

    #[cfg(not(any(unix, windows)))]
    let output: Result<std::process::Output, _> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "command detection not implemented for this platform",
    ));

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let found = stdout.lines().next().map(|s| s.trim().to_string());
            debug!(command = %command, found = ?found, "which/where succeeded");
            DetectResult {
                available: found.is_some(),
                path: found,
            }
        }
        _ => DetectResult {
            available: false,
            path: None,
        },
    }
}
