//! Server process management.
//!
//! Detects the opencode command, spawns the server, provides install, and shutdown.

mod detect;
mod install;
mod shutdown;
mod spawn;

pub use detect::{detect_command, DetectResult};
pub use install::install_opencode;
pub use shutdown::kill_by_pid;
pub use spawn::{spawn_server, SpawnOptions};
