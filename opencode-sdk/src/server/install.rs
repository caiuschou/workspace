//! Auto-install OpenCode when not found.
//!
//! Tries in order: npm, platform-specific methods (brew/curl on Unix; choco/scoop/powershell on Windows).

use super::runner::{CommandRunner, DefaultCommandRunner};
use crate::error::Error;
use crate::server::detect_command_with;
use tracing::{debug, info};

/// Attempts to install OpenCode using available package managers (uses default `CommandRunner`).
///
/// Tries npm first (cross-platform), then platform-specific: brew/curl on Unix;
/// choco, scoop, powershell download on Windows.
/// Returns the path to opencode if installation succeeded.
///
/// # Errors
///
/// Returns [`Error::InstallFailed`] when no install method succeeds.
pub fn install_opencode() -> Result<String, Error> {
    install_opencode_with_runner(&DefaultCommandRunner)
}

/// Attempts to install OpenCode using the given `CommandRunner`.
///
/// Use this to inject a mock runner in tests.
pub fn install_opencode_with_runner(runner: &dyn CommandRunner) -> Result<String, Error> {
    #[cfg(windows)]
    if let Some(path) = check_windows_default_path() {
        return Ok(path);
    }

    if let Some(path) = try_npm_install(runner) {
        return Ok(path);
    }
    #[cfg(windows)]
    {
        if let Some(path) = try_choco_install(runner) {
            return Ok(path);
        }
        if let Some(path) = try_scoop_install(runner) {
            return Ok(path);
        }
        if let Some(path) = try_powershell_install(runner) {
            return Ok(path);
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(path) = try_brew_install(runner) {
            return Ok(path);
        }
        if let Some(path) = try_curl_install(runner) {
            return Ok(path);
        }
    }
    #[cfg(windows)]
    let msg = "no install method available (tried npm, choco, scoop, powershell). \
         Install manually: choco install opencode, scoop install opencode, \
         or npm install -g opencode-ai, or see https://opencode.ai/install";
    #[cfg(not(windows))]
    let msg = "no install method available (tried npm, brew, curl). \
         Install manually: npm install -g opencode-ai, or see https://opencode.ai/install";
    Err(Error::InstallFailed(msg.to_string()))
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

#[cfg(not(windows))]
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

#[cfg(not(windows))]
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

#[cfg(windows)]
fn check_windows_default_path() -> Option<String> {
    let home = std::env::var("USERPROFILE").ok()?;
    let path = std::path::Path::new(&home).join(".opencode").join("bin").join("opencode.exe");
    if path.exists() {
        info!(
            path = %path.display(),
            "opencode found at Windows default path, skip install"
        );
        return Some(path.to_string_lossy().into_owned());
    }
    None
}

#[cfg(windows)]
fn try_choco_install(runner: &dyn CommandRunner) -> Option<String> {
    info!("try install: choco");
    let check = runner.run("choco", &["--version"]);
    if !check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        debug!("choco not available");
        return None;
    }
    info!("choco available, running choco install opencode");
    let out = runner.run("choco", &["install", "opencode", "-y"]);
    if out.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        let detect = detect_command_with("opencode", runner);
        if detect.available {
            info!("choco install succeeded");
            return detect.path;
        }
    }
    debug!("choco install failed");
    None
}

#[cfg(windows)]
fn try_scoop_install(runner: &dyn CommandRunner) -> Option<String> {
    info!("try install: scoop");
    let check = runner.run("scoop", &["--version"]);
    if !check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        debug!("scoop not available");
        return None;
    }
    info!("scoop available, running scoop install opencode");
    let out = runner.run("scoop", &["install", "opencode"]);
    if out.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        let detect = detect_command_with("opencode", runner);
        if detect.available {
            info!("scoop install succeeded");
            return detect.path;
        }
    }
    debug!("scoop install failed");
    None
}

#[cfg(windows)]
fn try_powershell_install(runner: &dyn CommandRunner) -> Option<String> {
    info!("try install: powershell download");
    let url = "https://github.com/anomalyco/opencode/releases/latest/download/opencode-windows-x64.zip";
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$installDir = Join-Path $env:USERPROFILE '.opencode\bin'
$tempDir = Join-Path $env:TEMP ('opencode-install-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null
try {{
  Invoke-WebRequest -Uri '{}' -OutFile (Join-Path $tempDir 'opencode.zip') -UseBasicParsing
  Expand-Archive -Path (Join-Path $tempDir 'opencode.zip') -DestinationPath $tempDir -Force
  $exe = Get-ChildItem -Path $tempDir -Recurse -Filter 'opencode.exe' -ErrorAction SilentlyContinue | Select-Object -First 1
  if ($exe) {{
    Copy-Item $exe.FullName -Destination (Join-Path $installDir 'opencode.exe') -Force
    Write-Output (Join-Path $installDir 'opencode.exe')
  }} else {{
    throw 'opencode.exe not found in archive'
  }}
}} finally {{
  Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}}
"#,
        url
    );
    let out = runner.run("powershell", &["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script]);
    if let Ok(o) = out {
        if o.status.success() {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let path = stdout.lines().next().map(str::trim).filter(|s| s.ends_with("opencode.exe"));
            if let Some(p) = path {
                if std::path::Path::new(p).exists() {
                    info!("powershell download install succeeded");
                    return Some(p.to_string());
                }
            }
        }
    }
    debug!("powershell install failed");
    None
}
