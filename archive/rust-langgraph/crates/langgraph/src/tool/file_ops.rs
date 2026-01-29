//! 文件操作工具：在指定根目录下安全地 read/write/list/exists。
//!
//! - `FileOpsTool`: 实现 `Tool`，参数 operation、path，write 需 content
//! - 路径限制在 `base_dir` 内，禁止 `..` 逃逸

use crate::error::ToolError;
use crate::tool::Tool;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// 文件操作工具：在 `base_dir` 下执行 read/write/list/exists，路径做安全检查。
///
/// Paths are resolved as `base_dir.join(path)` and canonicalized; the result must
/// lie under `base_dir` to prevent path traversal. Interacts with `ToolRegistry::execute`.
pub struct FileOpsTool {
    /// 根目录；所有 path 必须在此目录下。
    base_dir: PathBuf,
}

impl FileOpsTool {
    /// 新建工具，限制在 `base_dir` 下操作。
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// 解析并校验路径：必须落在 base_dir 内，禁止 `..`。
    fn resolve(&self, path: &str) -> Result<PathBuf, ToolError> {
        if path.contains("..") {
            return Err(ToolError::ValidationFailed(
                "path must not contain '..'".into(),
            ));
        }
        let base = self
            .base_dir
            .canonicalize()
            .map_err(|e| ToolError::ExecutionFailed(format!("base_dir canonicalize: {}", e)))?;
        let full = base.join(path.trim_start_matches('/'));
        let full = if full.exists() {
            full.canonicalize()
                .map_err(|e| ToolError::ExecutionFailed(format!("path resolve: {}", e)))?
        } else {
            full
        };
        if !full.starts_with(&base) {
            return Err(ToolError::ValidationFailed(
                "path escapes base directory".into(),
            ));
        }
        Ok(full)
    }
}

impl Tool for FileOpsTool {
    fn name(&self) -> &str {
        "file_ops"
    }

    fn description(&self) -> &str {
        "Read, write, list, or check existence of files under a base directory. Operations: read, write, list, exists."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": { "type": "string", "description": "One of: read, write, list, exists" },
                "path": { "type": "string", "description": "Path relative to base directory" },
                "content": { "type": "string", "description": "Content for write operation" }
            },
            "required": ["operation", "path"]
        })
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let op = args
            .get("operation")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::ValidationFailed("missing or non-string 'operation'".into()))?
            .to_lowercase();
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::ValidationFailed("missing or non-string 'path'".into()))?;

        let resolved = self.resolve(path)?;

        match op.as_str() {
            "read" => {
                let s = std::fs::read_to_string(&resolved)
                    .map_err(|e| ToolError::ExecutionFailed(format!("read: {}", e)))?;
                Ok(serde_json::json!({ "content": s }))
            }
            "write" => {
                let content = args
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if let Some(p) = resolved.parent() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| ToolError::ExecutionFailed(format!("create_dir_all: {}", e)))?;
                }
                std::fs::write(&resolved, content)
                    .map_err(|e| ToolError::ExecutionFailed(format!("write: {}", e)))?;
                Ok(serde_json::json!({ "written": true, "path": resolved.to_string_lossy() }))
            }
            "list" => {
                let entries = std::fs::read_dir(&resolved)
                    .map_err(|e| ToolError::ExecutionFailed(format!("list: {}", e)))?;
                let names: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect();
                Ok(serde_json::json!({ "entries": names }))
            }
            "exists" => {
                let ok = resolved.exists();
                Ok(serde_json::json!({ "exists": ok }))
            }
            _ => Err(ToolError::ValidationFailed(format!(
                "unknown operation: {}",
                op
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn file_ops_exists_and_read() {
        let dir = std::env::temp_dir().join("langgraph_file_ops_test");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("hello.txt");
        std::fs::File::create(&f)
            .unwrap()
            .write_all(b"hello")
            .unwrap();

        let tool = FileOpsTool::new(&dir);
        let args = serde_json::json!({"operation": "exists", "path": "hello.txt"});
        let out = tool.execute(args).unwrap();
        assert_eq!(out.get("exists"), Some(&serde_json::json!(true)));

        let args = serde_json::json!({"operation": "read", "path": "hello.txt"});
        let out = tool.execute(args).unwrap();
        assert_eq!(out.get("content"), Some(&serde_json::json!("hello")));

        let _ = std::fs::remove_file(&f);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn file_ops_reject_dotdot() {
        let dir = std::env::temp_dir().join("langgraph_file_ops_test2");
        let _ = std::fs::create_dir_all(&dir);
        let tool = FileOpsTool::new(&dir);
        let args = serde_json::json!({"operation": "read", "path": "../etc/passwd"});
        let err = tool.execute(args).unwrap_err();
        assert!(matches!(err, ToolError::ValidationFailed(_)));
        let _ = std::fs::remove_dir(&dir);
    }
}
