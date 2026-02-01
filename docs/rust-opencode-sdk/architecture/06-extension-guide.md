# Extension Guide

## Adding a New API Endpoint

1. **Create or extend a module** (e.g., `src/project.rs`):

```rust
use crate::{Client, Error};

impl Client {
    /// Gets project configuration.
    ///
    /// `GET /project/config`
    pub async fn project_config(
        &self,
        directory: Option<&std::path::Path>,
    ) -> Result<ProjectConfig, Error> {
        let url = format!("{}/project/config", self.base_url());
        let mut req = self.http().get(&url);
        
        if let Some(dir) = directory {
            if let Some(s) = dir.to_str() {
                req = req.query(&[("directory", s)]);
            }
        }
        
        let response = req.send().await?;
        let config: ProjectConfig = response.json().await?;
        Ok(config)
    }
}
```

2. **Define response types**:

```rust
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    pub name: String,
    pub root_path: String,
    // ...
}
```

3. **Export in `lib.rs`**:

```rust
pub mod project;
pub use project::ProjectConfig;
```

## Adding a New Error Variant

```rust
// error.rs
#[derive(Error, Debug)]
pub enum Error {
    // ... existing variants ...
    
    #[error("project not found: {path}")]
    ProjectNotFound { path: String },
}
```
