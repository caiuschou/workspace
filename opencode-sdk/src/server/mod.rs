//! Server process management.
//!
//! Detects the opencode command, spawns the server, provides install, and shutdown.

mod detect;
mod install;
mod runner;
mod shutdown;
mod spawn;

pub use detect::{detect_command, detect_command_with, DetectResult};
pub use install::{install_opencode, install_opencode_with_runner};
pub use runner::{CommandRunner, DefaultCommandRunner};
pub use shutdown::kill_by_pid;
pub use spawn::{spawn_server, spawn_server_with_runner, SpawnOptions};
