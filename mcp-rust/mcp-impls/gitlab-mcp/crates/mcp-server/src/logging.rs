//! Logging initialization for gitlab-mcp server.
//!
//! Provides file logging to ~/.mcp/logs/gitlab-mcp.log.

use std::io;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer, EnvFilter};

/// Log directory path: ~/.mcp/logs/
fn log_directory() -> PathBuf {
    let mut path = dirs::home_dir().expect("Unable to determine home directory");
    path.push(".mcp");
    path.push("logs");
    path
}

/// Initialize tracing with file and stderr logging.
///
/// Returns a `WorkerGuard` that must be kept alive for the duration of the program
/// to ensure logs are flushed properly.
///
/// # Log Configuration
/// - **File**: `~/.mcp/logs/gitlab-mcp.log`
///   - Level: DEBUG and above
///   - Includes timestamps and full context
/// - **Stderr**: Error level only
///   - Critical errors that need immediate attention
///
/// # Example
/// ```no_run
/// use gitlab_mcp_server::logging;
///
/// fn main() -> anyhow::Result<()> {
///     let _guard = logging::init_logging();
///     // Your application code here
///     Ok(())
/// }
/// ```
pub fn init_logging() -> anyhow::Result<WorkerGuard> {
    let log_dir = log_directory();

    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;

    // Set up file appender
    // File will be named: gitlab-mcp.log
    let log_file = log_dir.join("gitlab-mcp.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file);

    // Build environment filter
    // Try RUST_LOG first, fall back to sensible defaults
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("error,mcp_core=off,gitlab_mcp_server=debug"));

    // File layer: DEBUG and above with full context
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_file)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false);

    // Stderr layer: ERROR only, no ANSI colors
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(io::stderr)
        .with_target(false)
        .with_ansi(false);

    // Combine layers
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            file_layer
                .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
        )
        .with(
            stderr_layer
                .with_filter(tracing_subscriber::filter::LevelFilter::ERROR),
        )
        .init();

    Ok(guard)
}
