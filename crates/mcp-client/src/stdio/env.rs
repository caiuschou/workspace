use std::collections::HashMap;

/// Environment variables considered safe to inherit on Unix.
#[cfg(not(target_os = "windows"))]
pub const DEFAULT_INHERITED_ENV_VARS: &[&str] =
    &["HOME", "LOGNAME", "PATH", "SHELL", "TERM", "USER"];

/// Environment variables considered safe to inherit on Windows.
#[cfg(target_os = "windows")]
pub const DEFAULT_INHERITED_ENV_VARS: &[&str] = &[
    "APPDATA",
    "HOMEDRIVE",
    "HOMEPATH",
    "LOCALAPPDATA",
    "PATH",
    "PROCESSOR_ARCHITECTURE",
    "SYSTEMDRIVE",
    "SYSTEMROOT",
    "TEMP",
    "USERNAME",
    "USERPROFILE",
    "PROGRAMFILES",
];

/// A sanitized subset of environment variables to share with spawned servers.
pub fn get_default_environment() -> HashMap<String, String> {
    let mut env = HashMap::new();
    for key in DEFAULT_INHERITED_ENV_VARS {
        if let Ok(value) = std::env::var(key) {
            if value.starts_with("()") {
                continue;
            }
            env.insert(key.to_string(), value);
        }
    }
    env
}
