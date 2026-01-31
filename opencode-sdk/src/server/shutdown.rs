//! Server process shutdown utilities.
//!
//! Terminates the OpenCode server process by PID.
//! Uses `kill -TERM` on Unix and `taskkill /PID` on Windows.

use std::process::Command;

/// Terminates a process by PID.
///
/// On Unix: sends SIGTERM via `kill -TERM <pid>`.
/// On Windows: uses `taskkill /PID <pid> /T` to kill the process tree.
///
/// Ignores errors (e.g. process already exited).
pub fn kill_by_pid(pid: u32) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output();
    }

    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output();
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
    }
}
