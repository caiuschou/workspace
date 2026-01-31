//! BDD-style tests for detect_command.
//!
//! Tests absolute path logic (env-independent). which/where results depend on PATH.

use opencode_sdk::server::detect_command;

/// Given an absolute path that exists (e.g. /bin/true on Unix),
/// When detect_command is called with it,
/// Then available is true and path is Some.
#[test]
fn absolute_path_exists_returns_available() {
    #[cfg(unix)]
    let abs_path = "/bin/true"; // exists on most Unix
    #[cfg(windows)]
    let abs_path = "C:\\Windows\\System32\\cmd.exe"; // exists on Windows

    let result = detect_command(abs_path);
    assert!(result.available, "absolute path that exists should be available");
    assert_eq!(result.path.as_deref(), Some(abs_path));
}

/// Given an absolute path that does not exist,
/// When detect_command is called with it,
/// Then available is false and path is None.
#[test]
fn absolute_path_not_exists_returns_unavailable() {
    #[cfg(unix)]
    let abs_path = "/nonexistent/path/to/opencode";
    #[cfg(windows)]
    let abs_path = "C:\\Nonexistent\\Path\\opencode.exe";

    let result = detect_command(abs_path);
    assert!(!result.available);
    assert!(result.path.is_none());
}
