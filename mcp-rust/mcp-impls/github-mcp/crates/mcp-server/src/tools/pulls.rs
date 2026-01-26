use mcp_core::protocol::RequestContext;
use mcp_core::types::*;
use mcp_server::{McpServer, ServerError};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::client::GithubClient;
use crate::tools::issues::{get_arg, get_arg_opt, to_result};

/// 注册 Pull Request 相关工具
pub fn register_tools(
    server: &mut McpServer,
    state: Arc<super::GithubState>,
) -> Result<(), ServerError> {
    let client = GithubClient::new(state);

    // list_pulls - 列出 Pull Requests
    let list_pulls = Tool {
        base: BaseMetadata {
            name: "list_pulls".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List pull requests in a repository".to_string()),
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
                    "description": "PR state (open, closed, all)",
                    "enum": ["open", "closed", "all"]
                },
                "head": {
                    "type": "string",
                    "description": "Filter by head user or branch (e.g., 'user:branch')"
                },
                "base": {
                    "type": "string",
                    "description": "Filter by base branch"
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of PRs (default: 30)"
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
    server.register_tool(list_pulls, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let limit = get_arg_opt(args, "limit").unwrap_or(30);

            let mut path = format!("/repos/{}/{}/pulls?per_page={}", owner, repo, limit);

            if let Some(state) = get_arg_opt::<String>(args, "state") {
                path.push_str(&format!("&state={}", state));
            }
            if let Some(head) = get_arg_opt::<String>(args, "head") {
                path.push_str(&format!("&head={}", head));
            }
            if let Some(base) = get_arg_opt::<String>(args, "base") {
                path.push_str(&format!("&base={}", base));
            }

            let req = client.get(&path);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // get_pull - 获取单个 Pull Request
    let get_pull = Tool {
        base: BaseMetadata {
            name: "get_pull".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Get a single pull request".to_string()),
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
                "pull_number": {
                    "type": "number",
                    "description": "Pull request number"
                }
            },
            "required": ["owner", "repo", "pull_number"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(get_pull, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "pull_number")?;

            let req = client.get(&format!("/repos/{}/{}/pulls/{}", owner, repo, number));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // create_pull - 创建 Pull Request
    let create_pull = Tool {
        base: BaseMetadata {
            name: "create_pull".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Create a new pull request".to_string()),
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
                    "description": "Pull request title"
                },
                "head": {
                    "type": "string",
                    "description": "The name of the branch where your changes are (e.g., 'user:branch')"
                },
                "base": {
                    "type": "string",
                    "description": "The name of the branch you want to merge into"
                },
                "body": {
                    "type": "string",
                    "description": "Pull request description"
                },
                "draft": {
                    "type": "boolean",
                    "description": "Create as draft PR"
                }
            },
            "required": ["owner", "repo", "title", "head", "base"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(create_pull, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let title = get_arg(args, "title")?;
            let head = get_arg(args, "head")?;
            let base = get_arg(args, "base")?;

            let mut payload = json!({
                "title": title,
                "head": head,
                "base": base
            });

            if let Some(body) = get_arg_opt::<String>(args, "body") {
                payload["body"] = json!(body);
            }
            if let Some(draft) = get_arg_opt::<bool>(args, "draft") {
                payload["draft"] = json!(draft);
            }

            let req = client.post(&format!("/repos/{}/{}/pulls", owner, repo))
                .json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // merge_pull - 合并 Pull Request
    let merge_pull = Tool {
        base: BaseMetadata {
            name: "merge_pull".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Merge a pull request".to_string()),
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
                "pull_number": {
                    "type": "number",
                    "description": "Pull request number"
                },
                "commit_title": {
                    "type": "string",
                    "description": "Title for the merge commit message"
                },
                "commit_message": {
                    "type": "string",
                    "description": "Extra detail for the commit message"
                },
                "merge_method": {
                    "type": "string",
                    "description": "Merge method: merge, squash, or rebase",
                    "enum": ["merge", "squash", "rebase"]
                }
            },
            "required": ["owner", "repo", "pull_number"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(merge_pull, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "pull_number")?;

            let mut payload = json!({});

            if let Some(title) = get_arg_opt::<String>(args, "commit_title") {
                payload["commit_title"] = json!(title);
            }
            if let Some(msg) = get_arg_opt::<String>(args, "commit_message") {
                payload["commit_message"] = json!(msg);
            }
            if let Some(method) = get_arg_opt::<String>(args, "merge_method") {
                payload["merge_method"] = json!(method);
            }

            let req = client.put(&format!(
                "/repos/{}/{}/pulls/{}/merge",
                owner, repo, number
            )).json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // list_pull_files - 列出 PR 修改的文件
    let list_files = Tool {
        base: BaseMetadata {
            name: "list_pull_files".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List files changed in a pull request".to_string()),
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
                "pull_number": {
                    "type": "number",
                    "description": "Pull request number"
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of files (default: 30)"
                }
            },
            "required": ["owner", "repo", "pull_number"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(list_files, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "pull_number")?;
            let limit = get_arg_opt(args, "limit").unwrap_or(30);

            let req = client.get(&format!(
                "/repos/{}/{}/pulls/{}/files?per_page={}",
                owner, repo, number, limit
            ));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // list_pull_comments - 列出 PR 评论
    let list_pr_comments = Tool {
        base: BaseMetadata {
            name: "list_pull_comments".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List review comments on a pull request".to_string()),
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
                "pull_number": {
                    "type": "number",
                    "description": "Pull request number"
                },
                "limit": {
                    "type": "number",
                    "description": "Maximum number of comments (default: 30)"
                }
            },
            "required": ["owner", "repo", "pull_number"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(list_pr_comments, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "pull_number")?;
            let limit = get_arg_opt(args, "limit").unwrap_or(30);

            let req = client.get(&format!(
                "/repos/{}/{}/pulls/{}/comments?per_page={}",
                owner, repo, number, limit
            ));
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    // create_pull_comment - 创建 PR 评论
    let create_pr_comment = Tool {
        base: BaseMetadata {
            name: "create_pull_comment".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Create a review comment on a pull request".to_string()),
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
                "pull_number": {
                    "type": "number",
                    "description": "Pull request number"
                },
                "body": {
                    "type": "string",
                    "description": "Comment body"
                },
                "commit_id": {
                    "type": "string",
                    "description": "The SHA of the commit needing a comment"
                },
                "path": {
                    "type": "string",
                    "description": "The relative path to the file being commented on"
                },
                "position": {
                    "type": "number",
                    "description": "Line index in the diff to comment on"
                }
            },
            "required": ["owner", "repo", "pull_number", "body"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    let client_clone = client.clone();
    server.register_tool(create_pr_comment, move |args: Option<Value>, _ctx: RequestContext| {
        let client = client_clone.clone();
        Box::pin(async move {
            let args = args.as_ref().and_then(|a| a.as_object());
            let owner = get_arg(args, "owner")?;
            let repo = get_arg(args, "repo")?;
            let number = get_arg(args, "pull_number")?;
            let body = get_arg(args, "body")?;

            let mut payload = json!({ "body": body });

            if let Some(commit_id) = get_arg_opt::<String>(args, "commit_id") {
                payload["commit_id"] = json!(commit_id);
            }
            if let Some(path) = get_arg_opt::<String>(args, "path") {
                payload["path"] = json!(path);
            }
            if let Some(position) = get_arg_opt::<u64>(args, "position") {
                payload["position"] = json!(position);
            }

            let req = client.post(&format!(
                "/repos/{}/{}/pulls/{}/comments",
                owner, repo, number
            )).json(&payload);
            let resp = client.send(req).await?;

            Ok(to_result(&resp))
        })
    })?;

    Ok(())
}
