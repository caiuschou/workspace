use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// GitLab MCP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// GitLab instance URL
    pub gitlab_url: String,
    /// GitLab personal access token
    pub gitlab_token: String,
    /// Output format: table, json, plain
    pub output_format: String,
    /// Enable colors
    pub color: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            gitlab_url: "https://gitlab.com".to_string(),
            gitlab_token: String::new(),
            output_format: "table".to_string(),
            color: true,
        }
    }
}

impl ClientConfig {
    /// Get config directory path
    pub fn config_dir() -> Result<PathBuf, anyhow::Error> {
        let dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("gitlab-mcp");
        Ok(dir)
    }

    /// Get config file path
    pub fn config_file() -> Result<PathBuf, anyhow::Error> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load configuration from file and environment
    pub fn load() -> Result<Self, anyhow::Error> {
        let mut config = Self::default();

        // Load from file if exists
        if let Ok(path) = Self::config_file() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(file_config) = toml::from_str::<ClientConfig>(&content) {
                        config = file_config;
                    }
                }
            }
        }

        // Override with environment variables
        if let Ok(url) = std::env::var("GITLAB_URL") {
            config.gitlab_url = url;
        }

        if let Ok(token) = std::env::var("GITLAB_TOKEN") {
            config.gitlab_token = token;
        }

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir)?;

        let config_file = Self::config_file()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_file, content)?;
        Ok(())
    }
}
