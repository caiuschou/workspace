//! Command execution abstraction for server module.
//!
//! Enables dependency injection for detect/install/spawn so tests can inject a mock.

use std::io;
use std::path::Path;
use std::process::{Child, Output};

/// Runs a command and returns its output (for detection and install steps).
pub trait CommandRunner: Send + Sync {
    /// Runs `cmd` with `args` and returns the output.
    fn run(&self, cmd: &str, args: &[&str]) -> io::Result<Output>;

    /// Spawns `cmd` with `args` and optional working directory; returns the child process.
    fn spawn(&self, cmd: &str, args: &[&str], cwd: Option<&Path>) -> io::Result<Child>;
}

/// Default implementation using `std::process::Command`.
#[derive(Debug, Clone, Default)]
pub struct DefaultCommandRunner;

impl CommandRunner for DefaultCommandRunner {
    fn run(&self, cmd: &str, args: &[&str]) -> io::Result<Output> {
        std::process::Command::new(cmd).args(args).output()
    }

    fn spawn(&self, cmd: &str, args: &[&str], cwd: Option<&Path>) -> io::Result<Child> {
        let mut command = std::process::Command::new(cmd);
        command.args(args);
        if let Some(cwd) = cwd {
            command.current_dir(cwd);
        }
        command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
    }
}
