use mcp_core::protocol::RequestContext;
use mcp_core::types::*;
use mcp_server::{McpServer, ServerError};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::client::GithubClient;
use crate::tools::issues::{get_arg, get_arg_opt, to_result};

/// 注册文件操作相关工具
pub fn register_tools(
    server: &mut McpServer,
    state: Arc<super::GithubState>,
) -> Result<(), ServerError> {
    let client = GithubClient::new(state);

    // get_file - 获取文件内容
    let get_file = Tool {
        base: BaseMetadata {
            name: "get_file".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Get file content from a repository".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "path": {
                    "type": "string",
                    "description": "File path in the repository"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (default: main)"
                }
            },
            "required": ["owner", "repo", "path"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(get_file, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let path = get_arg(args, "path")?;
            let branch = get_arg_opt(args, "branch").unwrap_or_else(|| "main".to_string());

            let req = client.get(&format!(
                "/repos/{}/{}/contents/{}?ref={}",
                owner, repo, path, branch
            ));
            let resp = client.send(req).await?;

            Ok(to_file_result(&resp))
        })
    })?;

    // list_directory - 列出目录内容
    let list_directory = Tool {
        base: BaseMetadata {
            name: "list_directory".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List contents of a directory in a repository".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "path": {
                    "type": "string",
                    "description": "Directory path (default: root)"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (default: main)"
                }
            },
            "required": ["owner", "repo"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(list_directory, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let path = get_arg_opt(args, "path").unwrap_or_else(|| ".".to_string());
            let branch = get_arg_opt(args, "branch").unwrap_or_else(|| "main".to_string());

            let req = client.get(&format!(
                "/repos/{}/{}/contents/{}?ref={}",
                owner, repo, path, branch
            ));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // create_file - 创建文件
    let create_file = Tool {
        base: BaseMetadata {
            name: "create_file".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Create a new file in a repository".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "path": {
                    "type": "string",
                    "description": "File path"
                },
                "content": {
                    "type": "string",
                    "description": "File content"
                },
                "message": {
                    "type": "string",
                    "description": "Commit message"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (default: main)"
                }
            },
            "required": ["owner", "repo", "path", "content", "message"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(create_file, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let path = get_arg(args, "path")?;
            let content = get_arg(args, "content")?;
            let message = get_arg(args, "message")?;
            let branch = get_arg_opt(args, "branch").unwrap_or_else(|| "main".to_string());

            use base64::{Engine as _, engine::general_purpose};
            let encoded_content = general_purpose::STANDARD.encode(content);

            let payload = json!({
                "message": message,
                "content": encoded_content,
                "branch": branch
            });

            let req = client.put(&format!(
                "/repos/{}/{}/contents/{}",
                owner, repo, path
            )).json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // update_file - 更新文件
    let update_file = Tool {
        base: BaseMetadata {
            name: "update_file".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Update an existing file in a repository".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "path": {
                    "type": "string",
                    "description": "File path"
                },
                "content": {
                    "type": "string",
                    "description": "New file content"
                },
                "message": {
                    "type": "string",
                    "description": "Commit message"
                },
                "sha": {
                    "type": "string",
                    "description": "The blob SHA of the file being replaced"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (default: main)"
                }
            },
            "required": ["owner", "repo", "path", "content", "message", "sha"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(update_file, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let path = get_arg(args, "path")?;
            let content = get_arg(args, "content")?;
            let message = get_arg(args, "message")?;
            let sha = get_arg(args, "sha")?;
            let branch = get_arg_opt(args, "branch").unwrap_or_else(|| "main".to_string());

            use base64::{Engine as _, engine::general_purpose};
            let encoded_content = general_purpose::STANDARD.encode(content);

            let payload = json!({
                "message": message,
                "content": encoded_content,
                "sha": sha,
                "branch": branch
            });

            let req = client.put(&format!(
                "/repos/{}/{}/contents/{}",
                owner, repo, path
            )).json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // delete_file - 删除文件
    let delete_file = Tool {
        base: BaseMetadata {
            name: "delete_file".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Delete a file from a repository".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "path": {
                    "type": "string",
                    "description": "File path"
                },
                "message": {
                    "type": "string",
                    "description": "Commit message"
                },
                "sha": {
                    "type": "string",
                    "description": "The blob SHA of the file being deleted"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (default: main)"
                }
            },
            "required": ["owner", "repo", "path", "message", "sha"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(delete_file, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let path = get_arg(args, "path")?;
            let message = get_arg(args, "message")?;
            let sha = get_arg(args, "sha")?;
            let branch = get_arg_opt(args, "branch").unwrap_or_else(|| "main".to_string());

            let payload = json!({
                "message": message,
                "sha": sha,
                "branch": branch
            });

            let req = client.delete(&format!(
                "/repos/{}/{}/contents/{}",
                owner, repo, path
            )).json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // get_readme - 获取 README
    let get_readme = Tool {
        base: BaseMetadata {
            name: "get_readme".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Get the README from a repository".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Repository owner"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (default: default branch)"
                }
            },
            "required": ["owner", "repo"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(get_readme, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;

            let mut path = format!("/repos/{}/{}/readme", owner, repo);
            if let Some(branch) = get_arg_opt::<String>(args, "branch") {
                path.push_str(&format!("?ref={}", branch));
            }

            let req = client.get(&path);
            let resp = client.send(req).await?;

            Ok(to_file_result(&resp))
        })
    })?;

    Ok(())
}

/// 将文件响应转换为 MCP 工具结果（解码 base64）
fn to_file_result(resp: &crate::client::GithubResponse) -> CallToolResult {
    let text = if let Some(json) = &resp.json {
        // 尝试解码文件内容
        if let Some(encoded) = json.get("content").and_then(|v| v.as_str()) {
            use base64::{Engine as _, engine::general_purpose};
            if let Ok(decoded) = general_purpose::STANDARD.decode(encoded) {
                if let Ok(content) = String::from_utf8(decoded) {
                    content
                } else {
                    serde_json::to_string_pretty(json).unwrap_or_else(|_| resp.body.clone())
                }
            } else {
                serde_json::to_string_pretty(json).unwrap_or_else(|_| resp.body.clone())
            }
        } else {
            serde_json::to_string_pretty(json).unwrap_or_else(|_| resp.body.clone())
        }
    } else {
        resp.body.clone()
    };

    CallToolResult {
        content: vec![ContentBlock::Text(TextContent {
            kind: "text".to_string(),
            text,
            annotations: None,
            meta: None,
        })],
        structured_content: None,
        is_error: if resp.is_success() { None } else { Some(true) },
        meta: None,
    }
}
