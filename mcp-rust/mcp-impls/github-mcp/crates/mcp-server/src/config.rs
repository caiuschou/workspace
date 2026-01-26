use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Token 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// 所有命名的 token
    #[serde(default)]
    pub tokens: HashMap<String, String>,
    /// 默认 token 名称
    pub default_token: Option<String>,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            tokens: HashMap::new(),
            default_token: None,
        }
    }
}

impl TokenConfig {
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf, io::Error> {
        let mut path = config_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "无法找到配置目录")
        })?;
        path.push("github-mcp");
        fs::create_dir_all(&path)?;
        path.push("config.toml");
        Ok(path)
    }

    /// 加载配置
    pub fn load() -> Result<Self, io::Error> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: TokenConfig = toml::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// 保存配置
    pub fn save(&self) -> Result<(), io::Error> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// 添加或更新 token
    pub fn add_token(&mut self, name: String, token: String) {
        self.tokens.insert(name, token);
    }

    /// 删除 token
    pub fn remove_token(&mut self, name: &str) -> bool {
        let removed = self.tokens.remove(name).is_some();
        if self.default_token.as_deref() == Some(name) {
            self.default_token = None;
        }
        removed
    }

    /// 获取 token
    pub fn get_token(&self, name: &str) -> Option<&str> {
        self.tokens.get(name).map(|s| s.as_str())
    }

    /// 获取默认 token
    pub fn get_default_token(&self) -> Option<&str> {
        if let Some(name) = &self.default_token {
            self.tokens.get(name).map(|s| s.as_str())
        } else {
            // 如果没有设置默认，返回第一个 token
            self.tokens.values().next().map(|s| s.as_str())
        }
    }

    /// 设置默认 token
    pub fn set_default_token(&mut self, name: &str) -> bool {
        if self.tokens.contains_key(name) {
            self.default_token = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// 列出所有 token 名称
    pub fn list_tokens(&self) -> Vec<&String> {
        self.tokens.keys().collect()
    }
}
