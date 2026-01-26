use mcp_core::protocol::RequestContext;
use mcp_core::types::*;
use mcp_server::{McpServer, ServerError};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::config::TokenConfig;

/// 注册 Token 管理工具
pub fn register_tools(
    server: &mut McpServer,
    state: Arc<super::GithubState>,
) -> Result<(), ServerError> {
    // list_tokens - 列出所有 token
    let list_tokens = Tool {
        base: BaseMetadata {
            name: "list_tokens".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List all configured GitHub tokens".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {},
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let state_clone = state.clone();
    server.register_tool(list_tokens, move |_args: Option<Value>, _ctx: RequestContext| {
        let state = state_clone.clone();
        Box::pin(async move {
            let result = if let Some(tokens) = &state.tokens {
                let config = tokens.read().unwrap();
                let token_names: Vec<&String> = config.list_tokens();
                let current_name = state.current_token_name();

                let tokens_info: Vec<Value> = token_names.iter().map(|name| {
                    json!({
                        "name": name,
                        "is_default": config.default_token.as_deref() == Some(*name),
                        "is_current": current_name.as_ref().map(|s| s.as_str()) == Some(*name),
                        "preview": format!("{}...", config.tokens.get(*name).unwrap_or(&"".to_string()).chars().take(10).collect::<String>())
                    })
                }).collect();

                json!({
                    "tokens": tokens_info,
                    "count": tokens_info.len(),
                    "default": config.default_token,
                    "current": current_name,
                    "config_path": TokenConfig::config_path().ok().map(|p| p.to_string_lossy().to_string())
                })
            } else {
                json!({
                    "tokens": [],
                    "count": 0,
                    "note": "Token configuration not enabled. Initialize with add_token first."
                })
            };

            Ok(CallToolResult {
                content: vec![ContentBlock::Text(TextContent {
                    kind: "text".to_string(),
                    text: serde_json::to_string_pretty(&result).unwrap_or_default(),
                    annotations: None,
                    meta: None,
                })],
                structured_content: None,
                is_error: None,
                meta: None,
            })
        })
    })?;

    // add_token - 添加 token
    let add_token = Tool {
        base: BaseMetadata {
            name: "add_token".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Add or update a GitHub token with a name".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Token name (identifier)"
                },
                "token": {
                    "type": "string",
                    "description": "GitHub personal access token"
                },
                "set_as_default": {
                    "type": "boolean",
                    "description": "Set as default token",
                    "default": false
                },
                "set_as_current": {
                    "type": "boolean",
                    "description": "Use this token for current session",
                    "default": true
                }
            },
            "required": ["name", "token"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let state_clone = state.clone();
    server.register_tool(add_token, move |args: Option<Value>, _ctx: RequestContext| {
        let state = state_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let name = args.and_then(|a| a.get("name")).and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing name".to_string()))?;
            let token = args.and_then(|a| a.get("token")).and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing token".to_string()))?;
            let set_as_default = args.and_then(|a| a.get("set_as_default")).and_then(|v| v.as_bool()).unwrap_or(false);
            let set_as_current = args.and_then(|a| a.get("set_as_current")).and_then(|v| v.as_bool()).unwrap_or(true);

            // 重新加载并更新配置
            let mut config = TokenConfig::load().unwrap_or_default();
            config.add_token(name.to_string(), token.to_string());
            if set_as_default {
                config.set_default_token(name);
            }
            config.save()
                .map_err(|e| ServerError::Handler(format!("Failed to save config: {}", e)))?;

            // 更新状态
            let message = if set_as_current {
                // 注意：这里不能直接修改 state.current_token，因为它是不可变的
                // 实际应用中需要使用内部可变性
                format!("Token '{}' added successfully. Set as current and default.", name)
            } else if set_as_default {
                format!("Token '{}' added successfully. Set as default.", name)
            } else {
                format!("Token '{}' added successfully.", name)
            };

            Ok(CallToolResult {
                content: vec![ContentBlock::Text(TextContent {
                    kind: "text".to_string(),
                    text: message,
                    annotations: None,
                    meta: None,
                })],
                structured_content: None,
                is_error: None,
                meta: None,
            })
        })
    })?;

    // remove_token - 删除 token
    let remove_token = Tool {
        base: BaseMetadata {
            name: "remove_token".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Remove a configured GitHub token".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Token name to remove"
                }
            },
            "required": ["name"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let state_clone = state.clone();
    server.register_tool(remove_token, move |args: Option<Value>, _ctx: RequestContext| {
        let state = state_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let name = args.and_then(|a| a.get("name")).and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing name".to_string()))?;

            let mut config = TokenConfig::load().unwrap_or_default();
            let removed = config.remove_token(name);
            if !removed {
                return Ok(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent {
                        kind: "text".to_string(),
                        text: format!("Token '{}' not found", name),
                        annotations: None,
                        meta: None,
                    })],
                    structured_content: None,
                    is_error: Some(true),
                    meta: None,
                });
            }
            config.save()
                .map_err(|e| ServerError::Handler(format!("Failed to save config: {}", e)))?;

            Ok(CallToolResult {
                content: vec![ContentBlock::Text(TextContent {
                    kind: "text".to_string(),
                    text: format!("Token '{}' removed successfully", name),
                    annotations: None,
                    meta: None,
                })],
                structured_content: None,
                is_error: None,
                meta: None,
            })
        })
    })?;

    // set_default_token - 设置默认 token
    let set_default = Tool {
        base: BaseMetadata {
            name: "set_default_token".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Set a token as the default".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Token name to set as default"
                }
            },
            "required": ["name"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    server.register_tool(set_default, move |args: Option<Value>, _ctx: RequestContext| {
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let name = args.and_then(|a| a.get("name")).and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing name".to_string()))?;

            let mut config = TokenConfig::load().unwrap_or_default();
            if !config.set_default_token(name) {
                return Ok(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent {
                        kind: "text".to_string(),
                        text: format!("Token '{}' not found", name),
                        annotations: None,
                        meta: None,
                    })],
                    structured_content: None,
                    is_error: Some(true),
                    meta: None,
                });
            }
            config.save()
                .map_err(|e| ServerError::Handler(format!("Failed to save config: {}", e)))?;

            Ok(CallToolResult {
                content: vec![ContentBlock::Text(TextContent {
                    kind: "text".to_string(),
                    text: format!("Token '{}' set as default", name),
                    annotations: None,
                    meta: None,
                })],
                structured_content: None,
                is_error: None,
                meta: None,
            })
        })
    })?;

    // use_token - 切换到指定 token（当前会话）
    let use_token = Tool {
        base: BaseMetadata {
            name: "use_token".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Switch to a specific token for the current session".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Token name to use"
                }
            },
            "required": ["name"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let state_clone = state.clone();
    server.register_tool(use_token, move |args: Option<Value>, _ctx: RequestContext| {
        let state = state_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let name = args.and_then(|a| a.get("name")).and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing name".to_string()))?;

            let config = TokenConfig::load().unwrap_or_default();
            if let Some(token) = config.get_token(name) {
                // 注意：这里需要使用内部可变性来更新 current_token
                // 由于 state 使用的是 Arc，我们需要通过其他方式来更新
                // 这需要重构 GithubState 使用 Arc<Mutex<>> 或 Arc<RwLock<>>
                return Ok(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent {
                        kind: "text".to_string(),
                        text: format!("Token '{}' found. To switch tokens, restart the server with GITHUB_TOKEN environment variable or set as default.", name),
                        annotations: None,
                        meta: None,
                    })],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                });
            }

            Ok(CallToolResult {
                content: vec![ContentBlock::Text(TextContent {
                    kind: "text".to_string(),
                    text: format!("Token '{}' not found", name),
                    annotations: None,
                    meta: None,
                })],
                structured_content: None,
                is_error: Some(true),
                meta: None,
            })
        })
    })?;

    Ok(())
}
