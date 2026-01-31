//! Auto-install OpenCode when not found.
//!
//! Tries in order: npm, brew, curl install script.

use crate::error::Error;
use tracing::{debug, info};
use crate::server::detect_command;
use std::process::Command;

/// Attempts to install OpenCode using available package managers.
///
/// Tries: npm install -g opencode, then brew, then curl script.
/// Returns the path to opencode if installation succeeded.
pub fn install_opencode() -> Result<String, Error> {
    // 1. Try npm (most common in dev environments)
    if let Some(path) = try_npm_install() {
        return Ok(path);
    }

    // 2. Try Homebrew (macOS/Linux)
    if let Some(path) = try_brew_install() {
        return Ok(path);
    }

    // 3. Try curl install script
    if let Some(path) = try_curl_install() {
        return Ok(path);
    }

    Err(Error::InstallFailed(
        "no install method available (tried npm, brew, curl). \
         Install manually: npm install -g opencode-ai, or see https://opencode.ai/install"
            .to_string(),
    ))
}

fn try_npm_install() -> Option<String> {
    info!("try install: npm");
    let check = Command::new("npm").arg("--version").output();
    if check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        info!("npm available, running npm install -g opencode-ai");
        let status = Command::new("npm")
            .args(["install", "-g", "opencode-ai"])
            .status();
        if status.as_ref().map(|s| s.success()).unwrap_or(false) {
            let detect = detect_command("opencode");
            if detect.available {
                info!("npm install succeeded");
                return detect.path;
            }
        }
    }
    debug!("npm install failed or npm not available");
    None
}

fn try_brew_install() -> Option<String> {
    info!("try install: brew");
    let check = Command::new("brew").arg("--version").output();
    if check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        info!("brew available, running brew install");
        let status = Command::new("brew")
            .args(["install", "opencode-ai/tap/opencode"])
            .status();
        if status.as_ref().map(|s| s.success()).unwrap_or(false) {
            let detect = detect_command("opencode");
            if detect.available {
                info!("brew install succeeded");
                return detect.path;
            }
        }
    }
    debug!("brew install failed or brew not available");
    None
}

fn try_curl_install() -> Option<String> {
    info!("try install: curl script");
    let check = Command::new("curl").arg("--version").output();
    if !check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        return None;
    }

    // curl -fsSL https://opencode.ai/install | bash
    let curl = Command::new("curl")
        .args(["-fsSL", "https://opencode.ai/install"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();

    let curl_out = curl.ok()?;
    if !curl_out.status.success() {
        return None;
    }

    let script = String::from_utf8_lossy(&curl_out.stdout);
    let status = Command::new("bash")
        .arg("-c")
        .arg(&*script)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if status.as_ref().map(|s| s.success()).unwrap_or(false) {
        let detect = detect_command("opencode");
        if detect.available {
            info!("curl install succeeded");
            return detect.path;
        }
    }
    debug!("curl install failed");
    None
}
