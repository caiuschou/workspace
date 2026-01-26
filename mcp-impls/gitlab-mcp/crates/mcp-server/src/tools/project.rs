use mcp_core::types::{CallToolResult, ContentBlock, TextContent};
use crate::gitlab::GitLabClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use mcp_server::ServerError;

/// Project creation request
#[derive(Serialize)]
struct CreateProjectRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    initialize_with_readme: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_branch: Option<String>,
}

/// Project response from GitLab API
#[derive(Deserialize, Serialize)]
struct GitLabProject {
    id: u64,
    #[serde(default)]
    iid: Option<u64>,
    name: String,
    #[serde(rename = "path")]
    _path: String,
    path_with_namespace: String,
    web_url: String,
    description: Option<String>,
    visibility: String,
    created_at: String,
    #[serde(default)]
    last_activity_at: Option<String>,
    default_branch: Option<String>,
    #[serde(default)]
    ssh_url_to_repo: Option<String>,
    #[serde(default)]
    http_url_to_repo: Option<String>,
    #[serde(default)]
    star_count: Option<u64>,
    #[serde(default)]
    forks_count: Option<u64>,
    #[serde(default)]
    topics: Option<Vec<String>>,
}

/// Create a new GitLab project
pub async fn create_project(
    client: Arc<GitLabClient>,
    arguments: Option<Value>,
) -> Result<CallToolResult, ServerError> {
    let args = arguments
        .and_then(|v| v.as_object().cloned())
        .ok_or_else(|| ServerError::Handler("Expected object arguments".to_string()))?;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::Handler("name is required".to_string()))?
        .to_string();

    let path = args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
    let namespace_id = args.get("namespace_id").and_then(|v| v.as_u64());
    let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let visibility = args.get("visibility").and_then(|v| v.as_str()).map(|s| s.to_string());
    let initialize_with_readme = args.get("initialize_with_readme").and_then(|v| v.as_bool());
    let default_branch = args.get("default_branch").and_then(|v| v.as_str()).map(|s| s.to_string());

    // Validate visibility
    if let Some(ref vis) = visibility {
        if !matches!(vis.as_str(), "private" | "public" | "internal") {
            return Ok(to_tool_error(
                "visibility must be one of: private, public, internal",
            ));
        }
    }

    let request = CreateProjectRequest {
        name,
        path,
        namespace_id,
        description,
        visibility,
        initialize_with_readme,
        default_branch,
    };

    match client.post::<GitLabProject, _>("projects", &request).await {
        Ok(project) => {
            let mut output = vec![
                format!("## Project Created Successfully\n"),
                format!("**Name:** {}", project.name),
                format!("**Path:** {}", project.path_with_namespace),
                format!("**ID:** {}", project.id),
                format!("**URL:** {}", project.web_url),
                format!("**Visibility:** {}", project.visibility),
            ];

            if let Some(desc) = &project.description {
                output.push(format!("**Description:** {}", desc));
            }
            if let Some(branch) = &project.default_branch {
                output.push(format!("**Default Branch:** {}", branch));
            }

            output.push(format!("**Created at:** {}", project.created_at));

            Ok(to_tool_result(output.join("\n")))
        }
        Err(e) => Ok(to_tool_error(format!("Failed to create project: {}", e))),
    }
}

/// Get project details
pub async fn get_project(
    client: Arc<GitLabClient>,
    arguments: Option<Value>,
) -> Result<CallToolResult, ServerError> {
    let args = arguments
        .and_then(|v| v.as_object().cloned())
        .ok_or_else(|| ServerError::Handler("Expected object arguments".to_string()))?;

    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;

    // URL encode the project_id
    let encoded_id = urlencoding::encode(project_id);
    let path = format!("projects/{}", encoded_id);

    match client.get::<GitLabProject>(&path).await {
        Ok(project) => {
            let json = serde_json::to_string_pretty(&project)
                .unwrap_or_else(|_| {
                    serde_json::json!({"id": project.id, "name": project.name}).to_string()
                });
            Ok(CallToolResult {
                content: vec![ContentBlock::Text(TextContent::new(json))],
                ..Default::default()
            })
        }
        Err(e) => Ok(to_tool_error(format!("Failed to get project: {}", e))),
    }
}

/// List projects
pub async fn list_projects(
    client: Arc<GitLabClient>,
    arguments: Option<Value>,
) -> Result<CallToolResult, ServerError> {
    let args = arguments
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    // Extract query parameters
    let search = args.get("search").and_then(|v| v.as_str()).map(|s| s.to_string());
    let per_page = args.get("per_page").and_then(|v| as_u64(v)).unwrap_or(20);
    let page = args.get("page").and_then(|v| as_u64(v));

    tracing::info!("Listing projects: per_page={}, page={:?}", per_page, page);

    // Build query parameters
    let mut query = vec![
        ("membership".to_string(), "true".to_string()),
        ("per_page".to_string(), per_page.to_string()),
    ];
    if let Some(s) = search {
        query.push(("search".to_string(), s));
        query.push(("order_by".to_string(), "last_activity_at".to_string()));
        query.push(("sort".to_string(), "desc".to_string()));
    }
    if let Some(p) = page {
        query.push(("page".to_string(), p.to_string()));
    }

    tracing::debug!("Query parameters: {:?}", query);

    match client.get_with_query::<Vec<GitLabProject>>("projects", &query).await {
        Ok(projects) => {
            tracing::info!("Successfully retrieved {} projects", projects.len());
            let json = serde_json::to_string_pretty(&projects)
                .unwrap_or_else(|_| "[]".to_string());
            Ok(CallToolResult {
                content: vec![ContentBlock::Text(TextContent::new(json))],
                ..Default::default()
            })
        }
        Err(e) => {
            tracing::error!("Failed to list projects: {}", e);
            Ok(to_tool_error(format!("Failed to list projects: {}", e)))
        }
    }
}

/// Helper to convert Value to u64
fn as_u64(v: &serde_json::Value) -> Option<u64> {
    v.as_u64().or_else(|| v.as_i64().and_then(|i| u64::try_from(i).ok()))
}

/// Convert a result to MCP tool result
fn to_tool_result(content: String) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::Text(TextContent::new(content))],
        ..Default::default()
    }
}

/// Convert an error to MCP tool error result
fn to_tool_error(error: impl std::fmt::Display) -> CallToolResult {
    CallToolResult {
        content: vec![ContentBlock::Text(TextContent::new(format!("Error: {}", error)))],
        is_error: Some(true),
        ..Default::default()
    }
}
