use mcp_core::protocol::RequestContext;
use mcp_core::types::*;
use mcp_server::{McpServer, ServerError};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::client::GithubClient;

/// 注册 Issue 相关工具
pub fn register_tools(
    server: &mut McpServer,
    state: Arc<super::GithubState>,
) -> Result<(), ServerError> {
    let client = GithubClient::new(state);

    // list_issues - 列出 Issues
    let list_issues = Tool {
        base: BaseMetadata {
            name: "list_issues".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List issues in a GitHub repository".to_string()),
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
                "state": {
                    "type": "string",
                    "description": "Issue state (open, closed, all)",
                    "enum": ["open", "closed", "all"]
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of issues (default: 30)"
                },
                "labels": {
                    "type": "string",
                    "description": "Comma separated label names"
                },
                "since": {
                    "type": "string",
                    "description": "Only show issues updated at or after this time (ISO 8601)"
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
    server.register_tool(list_issues, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let limit = get_arg_opt(args, "limit").unwrap_or(30);

            let mut path = format!("/repos/{}/{}/issues?per_page={}", owner, repo, limit);

            if let Some(state) = get_arg_opt::<String>(args, "state") {
                path.push_str(&format!("&state={}", state));
            }
            if let Some(labels) = get_arg_opt::<String>(args, "labels") {
                path.push_str(&format!("&labels={}", labels));
            }
            if let Some(since) = get_arg_opt::<String>(args, "since") {
                path.push_str(&format!("&since={}", since));
            }

            let req = client.get(&path);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // get_issue - 获取单个 Issue
    let get_issue = Tool {
        base: BaseMetadata {
            name: "get_issue".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Get a single issue in a repository".to_string()),
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
                "issue_number": {
                    "type": "number",
                    "description": "Issue number"
                }
            },
            "required": ["owner", "repo", "issue_number"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(get_issue, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "issue_number")?;

            let req = client.get(&format!("/repos/{}/{}/issues/{}", owner, repo, number));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // create_issue - 创建 Issue
    let create_issue = Tool {
        base: BaseMetadata {
            name: "create_issue".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Create a new issue in a repository".to_string()),
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
                "title": {
                    "type": "string",
                    "description": "Issue title"
                },
                "body": {
                    "type": "string",
                    "description": "Issue body/description"
                },
                "labels": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Issue labels"
                },
                "assignees": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Users to assign"
                }
            },
            "required": ["owner", "repo", "title"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(create_issue, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let title = get_arg(args, "title")?;

            let mut payload = json!({ "title": title });

            if let Some(body) = get_arg_opt::<String>(args, "body") {
                payload["body"] = json!(body);
            }
            if let Some(labels) = args.and_then(|a| a.get("labels").and_then(|v| v.as_array())) {
                payload["labels"] = json!(labels);
            }
            if let Some(assignees) = args.and_then(|a| a.get("assignees").and_then(|v| v.as_array())) {
                payload["assignees"] = json!(assignees);
            }

            let req = client.post(&format!("/repos/{}/{}/issues", owner, repo))
                .json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // update_issue - 更新 Issue
    let update_issue = Tool {
        base: BaseMetadata {
            name: "update_issue".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Update an issue in a repository".to_string()),
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
                "issue_number": {
                    "type": "number",
                    "description": "Issue number"
                },
                "title": {
                    "type": "string",
                    "description": "New issue title"
                },
                "body": {
                    "type": "string",
                    "description": "Issue body/description"
                },
                "state": {
                    "type": "string",
                    "description": "Issue state: open or closed",
                    "enum": ["open", "closed"]
                },
                "labels": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Issue labels (replaces all labels)"
                },
                "assignees": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Users to assign (replaces all assignees)"
                }
            },
            "required": ["owner", "repo", "issue_number"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(update_issue, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "issue_number")?;

            let mut payload = json!({});

            if let Some(title) = get_arg_opt::<String>(args, "title") {
                payload["title"] = json!(title);
            }
            if let Some(body) = get_arg_opt::<String>(args, "body") {
                payload["body"] = json!(body);
            }
            if let Some(state) = get_arg_opt::<String>(args, "state") {
                payload["state"] = json!(state);
            }
            if let Some(labels) = args.and_then(|a| a.get("labels").and_then(|v| v.as_array())) {
                payload["labels"] = json!(labels);
            }
            if let Some(assignees) = args.and_then(|a| a.get("assignees").and_then(|v| v.as_array())) {
                payload["assignees"] = json!(assignees);
            }

            let req = client.patch(&format!("/repos/{}/{}/issues/{}", owner, repo, number))
                .json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // list_issue_comments - 列出 Issue 评论
    let list_comments = Tool {
        base: BaseMetadata {
            name: "list_issue_comments".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List comments on an issue".to_string()),
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
                "issue_number": {
                    "type": "number",
                    "description": "Issue number"
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of comments (default: 30)"
                }
            },
            "required": ["owner", "repo", "issue_number"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(list_comments, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "issue_number")?;
            let limit = get_arg_opt(args, "limit").unwrap_or(30);

            let req = client.get(&format!(
                "/repos/{}/{}/issues/{}/comments?per_page={}",
                owner, repo, number, limit
            ));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // create_issue_comment - 创建 Issue 评论
    let create_comment = Tool {
        base: BaseMetadata {
            name: "create_issue_comment".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Create a comment on an issue".to_string()),
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
                "issue_number": {
                    "type": "number",
                    "description": "Issue number"
                },
                "body": {
                    "type": "string",
                    "description": "Comment body"
                }
            },
            "required": ["owner", "repo", "issue_number", "body"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(create_comment, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "issue_number")?;
            let body = get_arg(args, "body")?;

            let payload = json!({ "body": body });

            let req = client.post(&format!(
                "/repos/{}/{}/issues/{}/comments",
                owner, repo, number
            )).json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    Ok(())
}

/// 从参数中获取必需的字符串参数
pub fn get_arg(args: Option<&serde_json::Map<String, Value>>, key: &str) -> Result<String, ServerError> {
    args.and_then(|a| a.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| ServerError::Handler(format!("missing {}", key)))
}

/// 从参数中获取可选参数
pub fn get_arg_opt<T: FromJson>(args: Option<&serde_json::Map<String, Value>>, key: &str) -> Option<T> {
    args.and_then(|a| a.get(key))
        .and_then(|v| T::from_json(v))
}

/// 从 JSON 值转换的 trait
trait FromJson: Sized {
    fn from_json(value: &Value) -> Option<Self>;
}

impl FromJson for String {
    fn from_json(value: &Value) -> Option<Self> {
        value.as_str().map(|s| s.to_string())
    }
}

impl FromJson for u64 {
    fn from_json(value: &Value) -> Option<Self> {
        value.as_u64()
    }
}

impl FromJson for i64 {
    fn from_json(value: &Value) -> Option<Self> {
        value.as_i64()
    }
}

impl FromJson for i32 {
    fn from_json(value: &Value) -> Option<Self> {
        value.as_i64().and_then(|v| i32::try_from(v).ok())
    }
}

impl FromJson for bool {
    fn from_json(value: &Value) -> Option<Self> {
        value.as_bool()
    }
}

/// 将 GitHub 响应转换为 MCP 工具结果
pub fn to_result(resp: &crate::client::GithubResponse) -> CallToolResult {
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
