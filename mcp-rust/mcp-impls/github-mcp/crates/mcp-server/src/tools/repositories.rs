use mcp_core::protocol::RequestContext;
use mcp_core::types::*;
use mcp_server::{McpServer, ServerError};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::client::GithubClient;

/// 注册仓库相关工具
pub fn register_tools(
    server: &mut McpServer,
    state: Arc<super::GithubState>,
) -> Result<(), ServerError> {
    let client = GithubClient::new(state);

    // get_repository - 获取仓库信息
    let get_repo = Tool {
        base: BaseMetadata {
            name: "get_repository".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Get information about a GitHub repository".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Repository owner (user or organization)"
                },
                "repo": {
                    "type": "string",
                    "description": "Repository name"
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
    server.register_tool(get_repo, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = args
                .and_then(|a| a.get("owner"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing owner".to_string()))?;
            let repo = args
                .and_then(|a| a.get("repo"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing repo".to_string()))?;

            let req = client.get(&format!("/repos/{}/{}", owner, repo));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // list_branches - 列出分支
    let list_branches = Tool {
        base: BaseMetadata {
            name: "list_branches".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List branches in a GitHub repository".to_string()),
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
                "protected": {
                    "type": "boolean",
                    "description": "Filter for protected branches only"
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of branches (default: 30)"
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
    server.register_tool(list_branches, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = args
                .and_then(|a| a.get("owner"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing owner".to_string()))?;
            let repo = args
                .and_then(|a| a.get("repo"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing repo".to_string()))?;
            let protected = args.and_then(|a| a.get("protected").and_then(|v| v.as_bool()));
            let limit = args
                .and_then(|a| a.get("limit").and_then(|v| v.as_u64()))
                .unwrap_or(30);

            let mut path = format!("/repos/{}/{}/branches?per_page={}", owner, repo, limit);
            if let Some(true) = protected {
                path.push_str("&protected=true");
            }

            let req = client.get(&path);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // get_branch - 获取分支信息
    let get_branch = Tool {
        base: BaseMetadata {
            name: "get_branch".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Get information about a branch".to_string()),
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
                    "description": "Branch name"
                }
            },
            "required": ["owner", "repo", "branch"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(get_branch, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = args
                .and_then(|a| a.get("owner"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing owner".to_string()))?;
            let repo = args
                .and_then(|a| a.get("repo"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing repo".to_string()))?;
            let branch = args
                .and_then(|a| a.get("branch"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing branch".to_string()))?;

            let req = client.get(&format!("/repos/{}/{}/branches/{}", owner, repo, branch));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // list_commits - 列出提交
    let list_commits = Tool {
        base: BaseMetadata {
            name: "list_commits".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List commits in a repository".to_string()),
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
                "sha": {
                    "type": "string",
                    "description": "SHA or branch to start listing commits from"
                },
                "path": {
                    "type": "string",
                    "description": "Only commits containing this file path"
                },
                "author": {
                    "type": "string",
                    "description": "GitHub login or email to filter by"
                },
                "since": {
                    "type": "string",
                    "description": "Only commits after this ISO 8601 timestamp"
                },
                "until": {
                    "type": "string",
                    "description": "Only commits before this ISO 8601 timestamp"
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of commits (default: 30)"
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
    server.register_tool(list_commits, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = args
                .and_then(|a| a.get("owner"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing owner".to_string()))?;
            let repo = args
                .and_then(|a| a.get("repo"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::Handler("missing repo".to_string()))?;

            let mut path = format!("/repos/{}/{}/commits?per_page={}", owner, repo,
                args.and_then(|a| a.get("limit").and_then(|v| v.as_u64())).unwrap_or(30));

            if let Some(sha) = args.and_then(|a| a.get("sha").and_then(|v| v.as_str())) {
                path.push_str(&format!("&sha={}", sha));
            }
            if let Some(p) = args.and_then(|a| a.get("path").and_then(|v| v.as_str())) {
                path.push_str(&format!("&path={}", p));
            }
            if let Some(a) = args.and_then(|a| a.get("author").and_then(|v| v.as_str())) {
                path.push_str(&format!("&author={}", a));
            }
            if let Some(s) = args.and_then(|a| a.get("since").and_then(|v| v.as_str())) {
                path.push_str(&format!("&since={}", s));
            }
            if let Some(u) = args.and_then(|a| a.get("until").and_then(|v| v.as_str())) {
                path.push_str(&format!("&until={}", u));
            }

            let req = client.get(&path);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    Ok(())
}

/// 将 GitHub 响应转换为 MCP 工具结果
fn to_result(resp: &crate::client::GithubResponse) -> CallToolResult {
    let text = if let Some(json) = &resp.json {
        serde_json::to_string_pretty(json).unwrap_or_else(|_| resp.body.clone())
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
