use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// GitLab MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// GitLab instance URL
    pub gitlab_url: String,
    /// GitLab personal access token
    pub gitlab_token: String,
    /// Log level
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gitlab_url: "https://gitlab.com".to_string(),
            gitlab_token: String::new(),
            log_level: "info".to_string(),
        }
    }
}

impl Config {
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

    /// Load configuration from file first, then override with environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Load from file if exists
        if let Ok(path) = Self::config_file() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(file_config) = toml::from_str::<Config>(&content) {
                        config = file_config;
                    }
                }
            }
        }

        // Environment variables override config file
        if let Ok(url) = std::env::var("GITLAB_URL") {
            config.gitlab_url = url;
        }

        if let Ok(token) = std::env::var("GITLAB_TOKEN") {
            config.gitlab_token = token;
        }

        if let Ok(level) = std::env::var("LOG_LEVEL") {
            config.log_level = level;
        }

        config
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.gitlab_token.is_empty() {
            return Err("GITLAB_TOKEN is required. Set it via environment variable or config file.".to_string());
        }

        // Validate URL format
        if let Err(e) = url::Url::parse(&self.gitlab_url) {
            return Err(format!("Invalid GITLAB_URL: {}", e));
        }

        Ok(())
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

    /// Load from config file only (no environment override)
    pub fn from_file(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save to specific file path
    pub fn save_to_file(&self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Print config location for user info
    pub fn print_config_info() {
        match Self::config_file() {
            Ok(path) => {
                println!("Config file location: {}", path.display());
                if path.exists() {
                    println!("Config file exists.");
                } else {
                    println!("Config file does not exist yet. It will be created when you save config.");
                }
            }
            Err(e) => {
                eprintln!("Could not determine config directory: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.gitlab_url, "https://gitlab.com");
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_config_validate_success() {
        let mut config = Config::default();
        config.gitlab_token = "test_token".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_no_token() {
        let config = Config::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_invalid_url() {
        let mut config = Config::default();
        config.gitlab_token = "test_token".to_string();
        config.gitlab_url = "not-a-url".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialize() {
        let config = Config {
            gitlab_url: "https://gitlab.example.com".to_string(),
            gitlab_token: "glpat_123456".to_string(),
            log_level: "debug".to_string(),
        };

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("gitlab_url"));
        assert!(toml_str.contains("gitlab_token"));
        assert!(toml_str.contains("log_level"));
    }
}
