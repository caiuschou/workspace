use mcp_core::protocol::RequestContext;
use mcp_core::types::*;
use mcp_server::{McpServer, ServerError};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::client::GithubClient;
use crate::tools::issues::{get_arg, get_arg_opt, to_result};

/// 注册分支相关工具
pub fn register_tools(
    server: &mut McpServer,
    state: Arc<super::GithubState>,
) -> Result<(), ServerError> {
    let client = GithubClient::new(state);

    // merge_branch - 合并分支
    let merge_branch = Tool {
        base: BaseMetadata {
            name: "merge_branch".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Merge a branch into another branch".to_string()),
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
                    "description": "The name of the base branch to merge into"
                },
                "head": {
                    "type": "string",
                    "description": "The head branch to merge"
                },
                "commit_message": {
                    "type": "string",
                    "description": "The commit message for the merge"
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
    server.register_tool(merge_branch, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let base = get_arg(args, "base")?;
            let head = get_arg(args, "head")?;

            let mut payload = json!({
                "base": base,
                "head": head
            });

            if let Some(msg) = get_arg_opt::<String>(args, "commit_message") {
                payload["commit_message"] = json!(msg);
            }

            let req = client.post(&format!("/repos/{}/{}/merges", owner, repo))
                .json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    Ok(())
}
