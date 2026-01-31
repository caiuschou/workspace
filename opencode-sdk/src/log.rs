//! Logging configuration for OpenCode SDK.
//!
//! Logs to both stdout and a file. Call [init_logger] before using the SDK.

use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static mut LOG_FILE_GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;

/// Initializes logging to stdout and file.
///
/// Log file: `{dir}/opencode-sdk.log` (default: current directory or `~/.local/share/opencode-sdk`)
///
/// Default level is `opencode_sdk=debug` when `RUST_LOG` is not set; set `RUST_LOG` to override
/// (e.g. `RUST_LOG=opencode_sdk=info` to reduce noise).
pub fn init_logger(log_dir: Option<PathBuf>) {
    let dir = log_dir.unwrap_or_else(|| {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share"))
            })
            .map(|p| p.join("opencode-sdk"))
            .unwrap_or_else(|_| PathBuf::from("."))
    });

    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("opencode-sdk: failed to create log dir {:?}: {}", dir, e);
        return;
    }
    let log_file = dir.join("opencode-sdk.log");
    eprintln!("opencode-sdk: log file: {}", log_file.display());

    let file_appender = tracing_appender::rolling::never(&dir, "opencode-sdk.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("opencode_sdk=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true),
        )
        .init();

    unsafe {
        LOG_FILE_GUARD = Some(guard);
    }
}
