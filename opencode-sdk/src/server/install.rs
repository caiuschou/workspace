//! Auto-install OpenCode when not found.
//!
//! Tries in order: npm, brew, curl install script.

use super::runner::{CommandRunner, DefaultCommandRunner};
use crate::error::Error;
use crate::server::detect_command_with;
use tracing::{debug, info};

/// Attempts to install OpenCode using available package managers (uses default `CommandRunner`).
///
/// Tries: npm install -g opencode, then brew, then curl script.
/// Returns the path to opencode if installation succeeded.
pub fn install_opencode() -> Result<String, Error> {
    install_opencode_with_runner(&DefaultCommandRunner)
}

/// Attempts to install OpenCode using the given `CommandRunner`.
///
/// Use this to inject a mock runner in tests.
pub fn install_opencode_with_runner(runner: &dyn CommandRunner) -> Result<String, Error> {
    if let Some(path) = try_npm_install(runner) {
        return Ok(path);
    }
    if let Some(path) = try_brew_install(runner) {
        return Ok(path);
    }
    if let Some(path) = try_curl_install(runner) {
        return Ok(path);
    }
    Err(Error::InstallFailed(
        "no install method available (tried npm, brew, curl). \
         Install manually: npm install -g opencode-ai, or see https://opencode.ai/install"
            .to_string(),
    ))
}

fn try_npm_install(runner: &dyn CommandRunner) -> Option<String> {
    info!("try install: npm");
    let check = runner.run("npm", &["--version"]);
    if check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        info!("npm available, running npm install -g opencode-ai");
        let out = runner.run("npm", &["install", "-g", "opencode-ai"]);
        if out.as_ref().map(|o| o.status.success()).unwrap_or(false) {
            let detect = detect_command_with("opencode", runner);
            if detect.available {
                info!("npm install succeeded");
                return detect.path;
            }
        }
    }
    debug!("npm install failed or npm not available");
    None
}

fn try_brew_install(runner: &dyn CommandRunner) -> Option<String> {
    info!("try install: brew");
    let check = runner.run("brew", &["--version"]);
    if check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        info!("brew available, running brew install");
        let out = runner.run("brew", &["install", "opencode-ai/tap/opencode"]);
        if out.as_ref().map(|o| o.status.success()).unwrap_or(false) {
            let detect = detect_command_with("opencode", runner);
            if detect.available {
                info!("brew install succeeded");
                return detect.path;
            }
        }
    }
    debug!("brew install failed or brew not available");
    None
}

fn try_curl_install(runner: &dyn CommandRunner) -> Option<String> {
    info!("try install: curl script");
    let check = runner.run("curl", &["--version"]);
    if !check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        return None;
    }
    let curl_out = runner.run("curl", &["-fsSL", "https://opencode.ai/install"]);
    let curl_out = curl_out.ok()?;
    if !curl_out.status.success() {
        return None;
    }
    let script = String::from_utf8_lossy(&curl_out.stdout);
    let bash_out = runner.run("bash", &["-c", &*script]);
    if bash_out.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        let detect = detect_command_with("opencode", runner);
        if detect.available {
            info!("curl install succeeded");
            return detect.path;
        }
    }
    debug!("curl install failed");
    None
}
