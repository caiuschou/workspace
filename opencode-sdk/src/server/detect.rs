//! Command detection utilities.
//!
//! Uses `which` on Unix/macOS and `where` on Windows to find the opencode binary in PATH.
//! Accepts absolute paths directly.

use std::path::Path;
use tracing::debug;
use std::process::Command;

/// Result of command availability check.
#[derive(Debug, Clone)]
pub struct DetectResult {
    /// Whether the command is available.
    pub available: bool,
    /// Full path to the command (if found).
    pub path: Option<String>,
}

/// Checks if a command is available.
///
/// If `command` is an absolute path, checks that the file exists and is executable.
/// Otherwise uses `which` (Unix) or `where` (Windows) to find it in PATH.
///
/// # Arguments
///
/// * `command` - Command name (e.g. "opencode") or absolute path (e.g. "/usr/local/bin/opencode")
pub fn detect_command(command: &str) -> DetectResult {
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
    let output = Command::new("which").arg(command).output();

    #[cfg(windows)]
    let output = Command::new("where").arg(command).output();

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
