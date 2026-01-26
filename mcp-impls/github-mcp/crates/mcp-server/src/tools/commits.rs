use mcp_core::protocol::RequestContext;
use mcp_core::types::*;
use mcp_server::{McpServer, ServerError};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::client::GithubClient;
use crate::tools::issues::{get_arg, to_result};

/// 注册提交相关工具
pub fn register_tools(
    server: &mut McpServer,
    state: Arc<super::GithubState>,
) -> Result<(), ServerError> {
    let client = GithubClient::new(state);

    // get_commit - 获取单个提交
    let get_commit = Tool {
        base: BaseMetadata {
            name: "get_commit".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Get a single commit from a repository".to_string()),
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
                "ref": {
                    "type": "string",
                    "description": "The commit SHA or branch/tag name"
                }
            },
            "required": ["owner", "repo", "ref"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(get_commit, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let reference = get_arg(args, "ref")?;

            let req = client.get(&format!("/repos/{}/{}/commits/{}", owner, repo, reference));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // compare_commits - 比较两个提交
    let compare = Tool {
        base: BaseMetadata {
            name: "compare_commits".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Compare two commits (diff)".to_string()),
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
                "base": {
                    "type": "string",
                    "description": "The base branch or SHA for comparison"
                },
                "head": {
                    "type": "string",
                    "description": "The head branch or SHA for comparison"
                }
            },
            "required": ["owner", "repo", "base", "head"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(compare, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let base = get_arg(args, "base")?;
            let head = get_arg(args, "head")?;

            let req = client.get(&format!(
                "/repos/{}/{}/compare/{}...{}",
                owner, repo, base, head
            ));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    Ok(())
}
