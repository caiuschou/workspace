use mcp_server::{McpServer, ServerError};
use mcp_core::{
    types::{
        BaseMetadata, Icons, Tool, CallToolResult, ContentBlock, TextContent,
    },
    protocol::RequestContext,
};
use crate::gitlab::GitLabClient;
use crate::config::Config;
use serde_json::json;

/// GitLab MCP server
pub struct GitLabMcpServer {
    _client: GitLabClient,
}

impl GitLabMcpServer {
    /// Create a new GitLab MCP server
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        config.validate()
            .map_err(|e| format!("Invalid config: {}", e))?;

        let _client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)?;

        Ok(Self { _client })
    }

    /// Register tools with the MCP server
    pub fn register_tools(server: &mut McpServer) -> Result<(), ServerError> {
        // === Configuration Tools ===

        // Register config_status tool
        let config_status_tool = Tool {
            base: BaseMetadata {
                name: "config_status".to_string(),
                title: Some("Get Configuration Status".to_string()),
            },
            icons: Icons::default(),
            description: Some("Check the current GitLab MCP server configuration status without exposing sensitive data".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            config_status_tool,
            |_arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let config = Config::from_env();
                    let config_file = Config::config_file();

                    let mut status = vec![];
                    status.push("## GitLab MCP Configuration Status\n".to_string());

                    // Config file location
                    match &config_file {
                        Ok(path) => {
                            status.push(format!("**Config File:** `{}`", path.display()));
                            if path.exists() {
                                status.push("**Status:** Config file exists".to_string());
                            } else {
                                status.push("**Status:** Config file not found (using environment variables or defaults)".to_string());
                            }
                        }
                        Err(e) => {
                            status.push(format!("**Config File Error:** {}", e));
                        }
                    }

                    status.push(String::new());

                    // GitLab URL
                    status.push(format!("**GitLab URL:** {}", config.gitlab_url));

                    // Token status (masked)
                    if config.gitlab_token.is_empty() {
                        status.push("**Token:** Not configured".to_string());
                    } else {
                        let preview = if config.gitlab_token.len() > 12 {
                            format!("{}***...***{}", &config.gitlab_token[..4], &config.gitlab_token[config.gitlab_token.len()-4..])
                        } else {
                            "***".to_string()
                        };
                        status.push(format!("**Token:** {} ({} chars)", preview, config.gitlab_token.len()));
                    }

                    // Log level
                    status.push(format!("**Log Level:** {}", config.log_level));

                    // Validation
                    match config.validate() {
                        Ok(_) => status.push("\n**Configuration:** Valid".to_string()),
                        Err(e) => status.push(format!("\n**Configuration Error:** {}", e)),
                    }

                    // Instructions
                    if config.gitlab_token.is_empty() {
                        status.push("\n### Setup Instructions".to_string());
                        status.push("To configure the GitLab MCP server:".to_string());
                        status.push("1. Create a Personal Access Token in GitLab".to_string());
                        status.push("2. Run: `gitlab-mcp config set-token <your-token>`".to_string());
                        status.push("   Or set environment variable: `export GITLAB_TOKEN=glpat-...`".to_string());
                    }

                    Ok(CallToolResult {
                        content: vec![ContentBlock::Text(TextContent::new(status.join("\n")))],
                        ..Default::default()
                    })
                })
            },
        )?;

        // Register get_config_info tool
        let get_config_info_tool = Tool {
            base: BaseMetadata {
                name: "get_config_info".to_string(),
                title: Some("Get Configuration Info".to_string()),
            },
            icons: Icons::default(),
            description: Some("Get information about available configuration options and how to set them".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            get_config_info_tool,
            |_arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let info = vec![
                        "## GitLab MCP Configuration\n".to_string(),
                        "### Configuration Methods\n".to_string(),
                        "**Via CLI:**".to_string(),
                        "```bash".to_string(),
                        "gitlab-mcp config set-url <url>     # Set GitLab instance URL".to_string(),
                        "gitlab-mcp config set-token <token> # Set Personal Access Token".to_string(),
                        "gitlab-mcp config show              # Show current config".to_string(),
                        "```".to_string(),
                        "".to_string(),
                        "**Via Environment Variables:**".to_string(),
                        "```bash".to_string(),
                        "export GITLAB_URL=\"https://gitlab.com\"".to_string(),
                        "export GITLAB_TOKEN=\"glpat-xxxxxxxxxxxxxx\"".to_string(),
                        "```".to_string(),
                        "".to_string(),
                        "**Via Config File:**".to_string(),
                        format!("Location: `{}`", Config::config_file().map(|p| p.display().to_string()).unwrap_or_else(|_| "Unknown".to_string())),
                        "".to_string(),
                        "```toml".to_string(),
                        "gitlab_url = \"https://gitlab.com\"".to_string(),
                        "gitlab_token = \"glpat-xxxxxxxxxxxxxx\"".to_string(),
                        "log_level = \"info\"".to_string(),
                        "```".to_string(),
                        "".to_string(),
                        "### Configuration Options\n".to_string(),
                        "- **gitlab_url**: GitLab instance URL (default: https://gitlab.com)".to_string(),
                        "- **gitlab_token**: Personal Access Token for authentication".to_string(),
                        "- **log_level**: Logging level (trace, debug, info, warn, error)".to_string(),
                        "".to_string(),
                        "### Priority Order".to_string(),
                        "1. Environment variables (highest priority)".to_string(),
                        "2. Config file".to_string(),
                        "3. Default values (lowest priority)".to_string(),
                    ];

                    Ok(CallToolResult {
                        content: vec![ContentBlock::Text(TextContent::new(info.join("\n")))],
                        ..Default::default()
                    })
                })
            },
        )?;

        // Register set_config tool
        let set_config_tool = Tool {
            base: BaseMetadata {
                name: "set_config".to_string(),
                title: Some("Set Configuration".to_string()),
            },
            icons: Icons::default(),
            description: Some("Set GitLab MCP configuration (saves to config file). Use this to configure your GitLab token and URL.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "gitlab_url": {
                        "type": "string",
                        "description": "GitLab instance URL (e.g., https://gitlab.com)"
                    },
                    "gitlab_token": {
                        "type": "string",
                        "description": "Personal Access Token (starts with glpat_)"
                    }
                }
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            set_config_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.and_then(|a| a.as_object().cloned()).unwrap_or_default();

                    let mut config = Config::from_env();

                    // Load existing config file if exists
                    if let Ok(path) = Config::config_file() {
                        if path.exists() {
                            if let Ok(file_config) = Config::from_file(path.clone()) {
                                config = file_config;
                            }
                        }
                    }

                    let mut updated = false;
                    let mut results = vec![];

                    // Update gitlab_url if provided
                    if let Some(url) = args.get("gitlab_url").and_then(|v| v.as_str()) {
                        if !url.is_empty() {
                            config.gitlab_url = url.to_string();
                            results.push(format!("✓ GitLab URL set to: {}", url));
                            updated = true;
                        }
                    }

                    // Update gitlab_token if provided
                    if let Some(token) = args.get("gitlab_token").and_then(|v| v.as_str()) {
                        if !token.is_empty() {
                            config.gitlab_token = token.to_string();
                            let preview = if token.len() > 12 {
                                format!("{}***...***{}", &token[..4], &token[token.len()-4..])
                            } else {
                                "***".to_string()
                            };
                            results.push(format!("✓ Token set to: {} ({} chars)", preview, token.len()));
                            updated = true;
                        }
                    }

                    if !updated {
                        results.push("No changes made. Please provide gitlab_url and/or gitlab_token.".to_string());
                    } else {
                        // Save to config file
                        match config.save() {
                            Ok(_) => {
                                results.push(format!("✓ Configuration saved to: {}", Config::config_file().map(|p| p.display().to_string()).unwrap_or_else(|_| "Unknown".to_string())));

                                // Validate
                                match config.validate() {
                                    Ok(_) => results.push("✓ Configuration is valid!".to_string()),
                                    Err(e) => results.push(format!("⚠ Warning: {}", e)),
                                }

                                results.push("\n**Note:** You may need to restart Claude Desktop for changes to take effect.".to_string());
                            }
                            Err(e) => {
                                results.push(format!("✗ Failed to save config: {}", e));
                            }
                        }
                    }

                    Ok(CallToolResult {
                        content: vec![ContentBlock::Text(TextContent::new(results.join("\n")))],
                        ..Default::default()
                    })
                })
            },
        )?;

        // === Project Tools ===

        // Register get_project tool
        let get_project_tool = Tool {
            base: BaseMetadata {
                name: "get_project".to_string(),
                title: Some("Get Project Details".to_string()),
            },
            icons: Icons::default(),
            description: Some("Get detailed information about a GitLab project".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    }
                },
                "required": ["project_id"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            get_project_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let path = format!("projects/{}", urlencoding::encode(project_id));

                    #[derive(serde::Deserialize, serde::Serialize)]
                    struct Project {
                        id: u64,
                        name: String,
                        path_with_namespace: String,
                        description: Option<String>,
                        default_branch: Option<String>,
                        web_url: String,
                        created_at: String,
                        last_activity_at: String,
                        visibility: String,
                        star_count: u64,
                        forks_count: u64,
                        #[serde(default)]
                        ssh_url_to_repo: Option<String>,
                        #[serde(default)]
                        http_url_to_repo: Option<String>,
                        #[serde(default)]
                        topics: Option<Vec<String>>,
                    }

                    match client.get::<Project>(&path).await {
                        Ok(project) => {
                            let json = serde_json::to_string_pretty(&project)
                                .unwrap_or_else(|_| {
                                    let fallback = serde_json::json!({
                                        "id": project.id,
                                        "name": project.name
                                    });
                                    fallback.to_string()
                                });
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(json))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error fetching project: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // Register list_projects tool
        let list_projects_tool = Tool {
            base: BaseMetadata {
                name: "list_projects".to_string(),
                title: Some("List Projects".to_string()),
            },
            icons: Icons::default(),
            description: Some("List projects accessible by the current user".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "search": {
                        "type": "string",
                        "description": "Search string to filter projects"
                    },
                    "per_page": {
                        "type": "integer",
                        "description": "Number of items per page (default: 20, max: 100)"
                    },
                    "page": {
                        "type": "integer",
                        "description": "Page number (default: 1)"
                    },
                    "owned": {
                        "type": "boolean",
                        "description": "Limit by projects owned by the current user"
                    },
                    "membership": {
                        "type": "boolean",
                        "description": "Limit by projects that the current user is a member of"
                    }
                }
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            list_projects_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.and_then(|a| a.as_object().cloned()).unwrap_or_default();

                    let search = args.get("search").and_then(|v| v.as_str());
                    let per_page = args.get("per_page").and_then(|v| v.as_u64()).unwrap_or(20);
                    let page = args.get("page").and_then(|v| v.as_u64()).unwrap_or(1);
                    let membership = args.get("membership").and_then(|v| v.as_bool()).unwrap_or(true);

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    tracing::info!("Listing projects: per_page={}, page={}, membership={}", per_page, page, membership);

                    // Build query parameters using get_with_query
                    let mut query = vec![
                        ("per_page".to_string(), per_page.to_string()),
                        ("page".to_string(), page.to_string()),
                        ("membership".to_string(), membership.to_string()),
                    ];
                    if let Some(s) = search {
                        query.push(("search".to_string(), s.to_string()));
                        query.push(("order_by".to_string(), "last_activity_at".to_string()));
                        query.push(("sort".to_string(), "desc".to_string()));
                    }

                    tracing::debug!("Query parameters: {:?}", query);

                    #[derive(serde::Deserialize, serde::Serialize)]
                    struct Project {
                        id: u64,
                        name: String,
                        path_with_namespace: String,
                        description: Option<String>,
                        web_url: String,
                        visibility: String,
                        #[serde(default)]
                        created_at: Option<String>,
                        #[serde(default)]
                        last_activity_at: Option<String>,
                        #[serde(default)]
                        default_branch: Option<String>,
                    }

                    match client.get_with_query::<Vec<Project>>("projects", &query).await {
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
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error listing projects: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // Register create_project tool
        let create_project_tool = Tool {
            base: BaseMetadata {
                name: "create_project".to_string(),
                title: Some("Create Project".to_string()),
            },
            icons: Icons::default(),
            description: Some("Create a new GitLab project".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Project name (required)"
                    },
                    "path": {
                        "type": "string",
                        "description": "Repository path (defaults to name slugified)"
                    },
                    "namespace_id": {
                        "type": "integer",
                        "description": "Namespace ID (omit to create in user's namespace)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Project description"
                    },
                    "visibility": {
                        "type": "string",
                        "description": "Visibility level",
                        "enum": ["private", "public", "internal"]
                    },
                    "initialize_with_readme": {
                        "type": "boolean",
                        "description": "Initialize with README.md"
                    },
                    "default_branch": {
                        "type": "string",
                        "description": "Default branch name (default: main)"
                    }
                },
                "required": ["name"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            create_project_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    tracing::info!("create_project tool called");

                    let args = arguments
                        .and_then(|v| v.as_object().cloned())
                        .ok_or_else(|| ServerError::Handler("Expected object arguments".to_string()))?;

                    let name = args
                        .get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("name is required".to_string()))?
                        .to_string();

                    tracing::info!("Creating project: {}", name);

                    let path = args.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let namespace_id = args.get("namespace_id").and_then(|v| v.as_u64());
                    let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let visibility = args.get("visibility").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let initialize_with_readme = args.get("initialize_with_readme").and_then(|v| v.as_bool());
                    let default_branch = args.get("default_branch").and_then(|v| v.as_str()).map(|s| s.to_string());

                    tracing::debug!("Project options - visibility: {:?}, namespace_id: {:?}", visibility, namespace_id);

                    // Validate visibility
                    if let Some(ref vis) = visibility {
                        if !matches!(vis.as_str(), "private" | "public" | "internal") {
                            tracing::error!("Invalid visibility value: {}", vis);
                            return Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(
                                    "Error: visibility must be one of: private, public, internal".to_string(),
                                ))],
                                is_error: Some(true),
                                ..Default::default()
                            });
                        }
                    }

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    #[derive(serde::Serialize)]
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

                    let request = CreateProjectRequest {
                        name: name.clone(),
                        path,
                        namespace_id,
                        description,
                        visibility,
                        initialize_with_readme,
                        default_branch,
                    };

                    #[derive(serde::Deserialize)]
                    struct GitLabProject {
                        id: u64,
                        name: String,
                        #[serde(rename = "path")]
                        _path: String,
                        path_with_namespace: String,
                        web_url: String,
                        description: Option<String>,
                        visibility: String,
                        created_at: String,
                        default_branch: Option<String>,
                    }

                    match client.post::<GitLabProject, _>("projects", &request).await {
                        Ok(project) => {
                            tracing::info!("Project created successfully: {} (ID: {})", project.name, project.id);

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

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            tracing::error!("Failed to create project '{}': {}", name, e);
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error: Failed to create project: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // === Issue Tools ===

        // Register list_issues tool
        let list_issues_tool = Tool {
            base: BaseMetadata {
                name: "list_issues".to_string(),
                title: Some("List Issues".to_string()),
            },
            icons: Icons::default(),
            description: Some("List issues for a project".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "state": {
                        "type": "string",
                        "description": "Issue state (opened, closed, all)",
                        "enum": ["opened", "closed", "all"]
                    },
                    "labels": {
                        "type": "string",
                        "description": "Comma-separated list of label names"
                    },
                    "per_page": {
                        "type": "integer",
                        "description": "Number per page (default: 20)"
                    },
                    "page": {
                        "type": "integer",
                        "description": "Page number"
                    }
                },
                "required": ["project_id"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            list_issues_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;

                    let state = args.and_then(|a| a.get("state")).and_then(|v| v.as_str()).unwrap_or("opened");
                    let labels = args.and_then(|a| a.get("labels")).and_then(|v| v.as_str());
                    let per_page = args.and_then(|a| a.get("per_page")).and_then(|v| v.as_u64()).unwrap_or(20);
                    let page = args.and_then(|a| a.get("page")).and_then(|v| v.as_u64()).unwrap_or(1);

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let mut path = format!("projects/{}/issues?per_page={}&page={}&state={}", encoded_project, per_page, page, state);
                    if let Some(l) = labels {
                        path.push_str(&format!("&labels={}", urlencoding::encode(l)));
                    }

                    #[derive(serde::Deserialize)]
                    struct Issue {
                        iid: u64,
                        title: String,
                        state: String,
                        web_url: String,
                        created_at: String,
                        #[serde(rename = "updated_at")]
                        _updated_at: String,
                        author: serde_json::Value,
                        assignees: Vec<serde_json::Value>,
                        labels: Vec<String>,
                    }

                    match client.get::<Vec<Issue>>(&path).await {
                        Ok(issues) => {
                            let mut output = vec![];
                            output.push(format!("## Issues ({} found)\n", issues.len()));

                            for i in issues {
                                let author = i.author.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                let assignee_names: Vec<&str> = i.assignees.iter()
                                    .filter_map(|a| a.get("name"))
                                    .filter_map(|n| n.as_str())
                                    .collect();
                                let labels_str = if i.labels.is_empty() {
                                    String::new()
                                } else {
                                    format!("Labels: {}", i.labels.join(", "))
                                };

                                output.push(format!("### !{} - {}", i.iid, i.title));
                                output.push(format!("**State:** {}", i.state));
                                output.push(format!("**Author:** {}", author));
                                if !assignee_names.is_empty() {
                                    output.push(format!("**Assignees:** {}", assignee_names.join(", ")));
                                }
                                if !labels_str.is_empty() {
                                    output.push(format!("**{}**", labels_str));
                                }
                                output.push(format!("**Created:** {}", i.created_at));
                                output.push(format!("**URL:** {}", i.web_url));
                                output.push(String::new());
                            }

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error listing issues: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // Register get_issue tool
        let get_issue_tool = Tool {
            base: BaseMetadata {
                name: "get_issue".to_string(),
                title: Some("Get Issue Details".to_string()),
            },
            icons: Icons::default(),
            description: Some("Get detailed information about a single issue".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "issue_iid": {
                        "type": "integer",
                        "description": "Issue IID (internal project ID)"
                    }
                },
                "required": ["project_id", "issue_iid"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            get_issue_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;
                    let issue_iid = args
                        .and_then(|a| a.get("issue_iid"))
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| ServerError::Handler("issue_iid is required".to_string()))?;

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let path = format!("projects/{}/issues/{}", encoded_project, issue_iid);

                    #[derive(serde::Deserialize)]
                    struct Issue {
                        iid: u64,
                        title: String,
                        description: Option<String>,
                        state: String,
                        web_url: String,
                        created_at: String,
                        #[serde(rename = "updated_at")]
                        _updated_at: String,
                        author: serde_json::Value,
                        assignees: Vec<serde_json::Value>,
                        labels: Vec<String>,
                        milestone: Option<serde_json::Value>,
                    }

                    match client.get::<Issue>(&path).await {
                        Ok(issue) => {
                            let mut output = vec![];
                            output.push(format!("# !{} - {}", issue.iid, issue.title));
                            output.push(format!("**State:** {}", issue.state));

                            let author = issue.author.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                            output.push(format!("**Author:** {}", author));

                            let assignee_names: Vec<&str> = issue.assignees.iter()
                                .filter_map(|a| a.get("name"))
                                .filter_map(|n| n.as_str())
                                .collect();
                            if !assignee_names.is_empty() {
                                output.push(format!("**Assignees:** {}", assignee_names.join(", ")));
                            }

                            if !issue.labels.is_empty() {
                                output.push(format!("**Labels:** {}", issue.labels.join(", ")));
                            }

                            if let Some(m) = &issue.milestone {
                                if let Some(title) = m.get("title").and_then(|v| v.as_str()) {
                                    output.push(format!("**Milestone:** {}", title));
                                }
                            }

                            output.push(format!("**Created:** {}", issue.created_at));
                            output.push(format!("**Updated:** {}", issue._updated_at));
                            output.push(format!("**URL:** {}", issue.web_url));

                            if let Some(desc) = &issue.description {
                                output.push(format!("\n## Description\n{}", desc));
                            }

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error getting issue: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // === Merge Request Tools ===

        // Register list_merge_requests tool
        let list_mrs_tool = Tool {
            base: BaseMetadata {
                name: "list_merge_requests".to_string(),
                title: Some("List Merge Requests".to_string()),
            },
            icons: Icons::default(),
            description: Some("List merge requests for a project".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "state": {
                        "type": "string",
                        "description": "MR state (opened, closed, merged, all)",
                        "enum": ["opened", "closed", "merged", "all"]
                    },
                    "per_page": {
                        "type": "integer",
                        "description": "Number per page (default: 20)"
                    },
                    "page": {
                        "type": "integer",
                        "description": "Page number"
                    }
                },
                "required": ["project_id"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            list_mrs_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;

                    let state = args.and_then(|a| a.get("state")).and_then(|v| v.as_str()).unwrap_or("opened");
                    let per_page = args.and_then(|a| a.get("per_page")).and_then(|v| v.as_u64()).unwrap_or(20);
                    let page = args.and_then(|a| a.get("page")).and_then(|v| v.as_u64()).unwrap_or(1);

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let path = format!("projects/{}/merge_requests?per_page={}&page={}&state={}", encoded_project, per_page, page, state);

                    #[derive(serde::Deserialize)]
                    struct MergeRequest {
                        iid: u64,
                        title: String,
                        state: String,
                        web_url: String,
                        created_at: String,
                        #[serde(rename = "updated_at")]
                        _updated_at: String,
                        author: serde_json::Value,
                        source_branch: String,
                        target_branch: String,
                        merge_status: Option<String>,
                    }

                    match client.get::<Vec<MergeRequest>>(&path).await {
                        Ok(mrs) => {
                            let mut output = vec![];
                            output.push(format!("## Merge Requests ({} found)\n", mrs.len()));

                            for mr in mrs {
                                let author = mr.author.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                let status = mr.merge_status.as_deref().unwrap_or("unknown");

                                output.push(format!("### !{} - {}", mr.iid, mr.title));
                                output.push(format!("**State:** {}", mr.state));
                                output.push(format!("**Author:** {}", author));
                                output.push(format!("**Branch:** {} → {}", mr.source_branch, mr.target_branch));
                                output.push(format!("**Merge Status:** {}", status));
                                output.push(format!("**Created:** {}", mr.created_at));
                                output.push(format!("**URL:** {}", mr.web_url));
                                output.push(String::new());
                            }

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error listing MRs: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // Register get_merge_request tool
        let get_mr_tool = Tool {
            base: BaseMetadata {
                name: "get_merge_request".to_string(),
                title: Some("Get Merge Request Details".to_string()),
            },
            icons: Icons::default(),
            description: Some("Get detailed information about a merge request".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "mr_iid": {
                        "type": "integer",
                        "description": "Merge Request IID"
                    }
                },
                "required": ["project_id", "mr_iid"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            get_mr_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;
                    let mr_iid = args
                        .and_then(|a| a.get("mr_iid"))
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| ServerError::Handler("mr_iid is required".to_string()))?;

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let path = format!("projects/{}/merge_requests/{}", encoded_project, mr_iid);

                    #[derive(serde::Deserialize)]
                    struct MergeRequest {
                        iid: u64,
                        title: String,
                        description: Option<String>,
                        state: String,
                        web_url: String,
                        created_at: String,
                        #[serde(rename = "updated_at")]
                        _updated_at: String,
                        author: serde_json::Value,
                        assignees: Vec<serde_json::Value>,
                        reviewers: Vec<serde_json::Value>,
                        source_branch: String,
                        target_branch: String,
                        merge_status: Option<String>,
                        has_conflicts: bool,
                        draft: bool,
                        work_in_progress: bool,
                    }

                    match client.get::<MergeRequest>(&path).await {
                        Ok(mr) => {
                            let mut output = vec![];
                            output.push(format!("# !{} - {}", mr.iid, mr.title));
                            output.push(format!("**State:** {}", mr.state));

                            let author = mr.author.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                            output.push(format!("**Author:** {}", author));

                            output.push(format!("**Branch:** {} → {}", mr.source_branch, mr.target_branch));

                            if let Some(status) = &mr.merge_status {
                                output.push(format!("**Merge Status:** {}", status));
                            }

                            if mr.has_conflicts {
                                output.push("**Has Conflicts:** Yes".to_string());
                            }

                            if mr.draft || mr.work_in_progress {
                                output.push("**Status:** Draft / WIP".to_string());
                            }

                            let assignee_names: Vec<&str> = mr.assignees.iter()
                                .filter_map(|a| a.get("name"))
                                .filter_map(|n| n.as_str())
                                .collect();
                            if !assignee_names.is_empty() {
                                output.push(format!("**Assignees:** {}", assignee_names.join(", ")));
                            }

                            let reviewer_names: Vec<&str> = mr.reviewers.iter()
                                .filter_map(|a| a.get("name"))
                                .filter_map(|n| n.as_str())
                                .collect();
                            if !reviewer_names.is_empty() {
                                output.push(format!("**Reviewers:** {}", reviewer_names.join(", ")));
                            }

                            output.push(format!("**Created:** {}", mr.created_at));
                            output.push(format!("**Updated:** {}", mr._updated_at));
                            output.push(format!("**URL:** {}", mr.web_url));

                            if let Some(desc) = &mr.description {
                                output.push(format!("\n## Description\n{}", desc));
                            }

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error getting MR: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // === Branch Tools ===

        // Register list_branches tool
        let list_branches_tool = Tool {
            base: BaseMetadata {
                name: "list_branches".to_string(),
                title: Some("List Branches".to_string()),
            },
            icons: Icons::default(),
            description: Some("List branches in a project".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "search": {
                        "type": "string",
                        "description": "Search string to filter branches"
                    }
                },
                "required": ["project_id"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            list_branches_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;

                    let search = args.and_then(|a| a.get("search")).and_then(|v| v.as_str());

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let mut path = format!("projects/{}/repository/branches", encoded_project);
                    if let Some(s) = search {
                        path.push_str(&format!("?search={}", urlencoding::encode(s)));
                    }

                    #[derive(serde::Deserialize)]
                    struct Branch {
                        name: String,
                        commit: serde_json::Value,
                        protected: bool,
                        default: bool,
                        web_url: String,
                    }

                    match client.get::<Vec<Branch>>(&path).await {
                        Ok(branches) => {
                            let mut output = vec![];
                            output.push(format!("## Branches ({} found)\n", branches.len()));

                            for b in branches {
                                let short_id = b.commit.get("short_id").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let title = b.commit.get("title").and_then(|v| v.as_str()).unwrap_or("");
                                let author = b.commit.get("author_name").and_then(|v| v.as_str()).unwrap_or("Unknown");

                                output.push(format!("### {} {}", b.name, if b.default { "(default)" } else { "" }));
                                output.push(format!("**Commit:** {} - {}", short_id, title));
                                output.push(format!("**Author:** {}", author));
                                output.push(format!("**Protected:** {}", if b.protected { "Yes" } else { "No" }));
                                output.push(format!("**URL:** {}", b.web_url));
                                output.push(String::new());
                            }

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error listing branches: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // === Commit Tools ===

        // Register list_commits tool
        let list_commits_tool = Tool {
            base: BaseMetadata {
                name: "list_commits".to_string(),
                title: Some("List Commits".to_string()),
            },
            icons: Icons::default(),
            description: Some("List commits in a project".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "ref_name": {
                        "type": "string",
                        "description": "The name of a branch or tag"
                    },
                    "per_page": {
                        "type": "integer",
                        "description": "Number per page (default: 20)"
                    }
                },
                "required": ["project_id"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            list_commits_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;

                    let ref_name = args.and_then(|a| a.get("ref_name")).and_then(|v| v.as_str());
                    let per_page = args.and_then(|a| a.get("per_page")).and_then(|v| v.as_u64()).unwrap_or(20);

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let mut path = format!("projects/{}/repository/commits?per_page={}", encoded_project, per_page);
                    if let Some(r) = ref_name {
                        path.push_str(&format!("&ref_name={}", urlencoding::encode(r)));
                    }

                    #[derive(serde::Deserialize)]
                    struct Commit {
                        #[serde(rename = "id")]
                        _id: String,
                        short_id: String,
                        title: String,
                        message: String,
                        author_name: String,
                        authored_date: String,
                        web_url: String,
                    }

                    match client.get::<Vec<Commit>>(&path).await {
                        Ok(commits) => {
                            let mut output = vec![];
                            output.push(format!("## Commits ({} found)\n", commits.len()));

                            for c in commits {
                                output.push(format!("### {} - {}", c.short_id, c.title));
                                output.push(format!("**Author:** {}", c.author_name));
                                output.push(format!("**Date:** {}", c.authored_date));
                                output.push(format!("**Message:** {}", c.message.lines().next().unwrap_or(&c.title)));
                                output.push(format!("**URL:** {}", c.web_url));
                                output.push(String::new());
                            }

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error listing commits: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // === Pipeline Tools ===

        // Register list_pipelines tool
        let list_pipelines_tool = Tool {
            base: BaseMetadata {
                name: "list_pipelines".to_string(),
                title: Some("List Pipelines".to_string()),
            },
            icons: Icons::default(),
            description: Some("List CI/CD pipelines for a project".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "status": {
                        "type": "string",
                        "description": "Status filter (pending, running, success, failed, canceled, skipped)"
                    },
                    "ref": {
                        "type": "string",
                        "description": "Ref name (branch or tag)"
                    },
                    "per_page": {
                        "type": "integer",
                        "description": "Number per page (default: 20)"
                    }
                },
                "required": ["project_id"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            list_pipelines_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;

                    let status = args.and_then(|a| a.get("status")).and_then(|v| v.as_str());
                    let ref_name = args.and_then(|a| a.get("ref")).and_then(|v| v.as_str());
                    let per_page = args.and_then(|a| a.get("per_page")).and_then(|v| v.as_u64()).unwrap_or(20);

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let mut path = format!("projects/{}/pipelines?per_page={}", encoded_project, per_page);
                    if let Some(s) = status {
                        path.push_str(&format!("&status={}", s));
                    }
                    if let Some(r) = ref_name {
                        path.push_str(&format!("&ref={}", urlencoding::encode(r)));
                    }

                    #[derive(serde::Deserialize)]
                    struct Pipeline {
                        id: u64,
                        iid: u64,
                        #[serde(rename = "project_id")]
                        _project_id: u64,
                        status: String,
                        ref_name: String,
                        sha: String,
                        created_at: String,
                        #[serde(rename = "updated_at")]
                        _updated_at: String,
                        web_url: String,
                        user: serde_json::Value,
                    }

                    match client.get::<Vec<Pipeline>>(&path).await {
                        Ok(pipelines) => {
                            let mut output = vec![];
                            output.push(format!("## Pipelines ({} found)\n", pipelines.len()));

                            for p in pipelines {
                                let user = p.user.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                                let short_sha = if p.sha.len() > 8 { &p.sha[..8] } else { &p.sha };

                                output.push(format!("### Pipeline #{} - {}", p.iid, p.status));
                                output.push(format!("**ID:** {}", p.id));
                                output.push(format!("**User:** {}", user));
                                output.push(format!("**Branch:** {}", p.ref_name));
                                output.push(format!("**SHA:** {}", short_sha));
                                output.push(format!("**Created:** {}", p.created_at));
                                output.push(format!("**URL:** {}", p.web_url));
                                output.push(String::new());
                            }

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error listing pipelines: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // === Repository/File Tools ===

        // Register list_files tool
        let list_files_tool = Tool {
            base: BaseMetadata {
                name: "list_files".to_string(),
                title: Some("List Repository Files".to_string()),
            },
            icons: Icons::default(),
            description: Some("List files in a project repository".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "path": {
                        "type": "string",
                        "description": "Path inside repository (default: root)"
                    },
                    "ref": {
                        "type": "string",
                        "description": "Branch, tag, or commit (default: default branch)"
                    }
                },
                "required": ["project_id"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            list_files_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;

                    let path = args.and_then(|a| a.get("path")).and_then(|v| v.as_str()).unwrap_or("");
                    let ref_name = args.and_then(|a| a.get("ref")).and_then(|v| v.as_str());

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let encoded_path = urlencoding::encode(path);
                    let mut url = format!("projects/{}/repository/tree/{}?path={}", encoded_project, encoded_path, encoded_path);
                    if let Some(r) = ref_name {
                        url.push_str(&format!("&ref={}", urlencoding::encode(r)));
                    }

                    #[derive(serde::Deserialize)]
                    struct FileInfo {
                        #[serde(rename = "id")]
                        _id: String,
                        name: String,
                        r#type: String,
                        path: String,
                        #[serde(rename = "mode")]
                        _mode: String,
                    }

                    match client.get::<Vec<FileInfo>>(&url).await {
                        Ok(files) => {
                            let mut output = vec![];
                            output.push(format!("## Files in `{}`\n", if path.is_empty() { "/" } else { path }));
                            output.push(format!("Found {} items\n", files.len()));

                            for f in &files {
                                let icon = if f.r#type == "tree" { "📁" } else { "📄" };
                                output.push(format!("{} **{}** `{}`", icon, f.name, f.path));
                            }

                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                                ..Default::default()
                            })
                        }
                        Err(e) => {
                            Ok(CallToolResult {
                                content: vec![ContentBlock::Text(TextContent::new(format!("Error listing files: {}", e)))],
                                is_error: Some(true),
                                ..Default::default()
                            })
                        }
                    }
                })
            },
        )?;

        // Register get_file tool
        let get_file_tool = Tool {
            base: BaseMetadata {
                name: "get_file".to_string(),
                title: Some("Get File Content".to_string()),
            },
            icons: Icons::default(),
            description: Some("Get the content of a file from the repository".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "string",
                        "description": "Project ID or URL-encoded path"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Full path to the file"
                    },
                    "ref": {
                        "type": "string",
                        "description": "Branch, tag, or commit (default: default branch)"
                    }
                },
                "required": ["project_id", "file_path"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        };

        server.register_tool(
            get_file_tool,
            |arguments: Option<serde_json::Value>, _context: RequestContext| {
                Box::pin(async move {
                    let args = arguments.as_ref().and_then(|a| a.as_object());
                    let project_id = args
                        .and_then(|a| a.get("project_id"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("project_id is required".to_string()))?;
                    let file_path = args
                        .and_then(|a| a.get("file_path"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ServerError::Handler("file_path is required".to_string()))?;

                    let ref_name = args.and_then(|a| a.get("ref")).and_then(|v| v.as_str());

                    let config = Config::from_env();
                    let client = GitLabClient::new(&config.gitlab_url, &config.gitlab_token)
                        .map_err(|e| ServerError::Handler(format!("Failed to create client: {}", e)))?;

                    let encoded_project = urlencoding::encode(project_id);
                    let encoded_path = urlencoding::encode(file_path);
                    let url = format!("projects/{}/repository/files/{}/raw?ref={}", encoded_project, encoded_path, ref_name.unwrap_or("HEAD"));

                    // Get raw file content
                    let response = client.get_bytes(&url).await
                        .map_err(|e| ServerError::Handler(format!("Failed to get file: {}", e)))?;

                    // Try to decode as UTF-8
                    let content = String::from_utf8_lossy(&response);

                    let mut output = vec![];
                    output.push(format!("## File: `{}`", file_path));
                    output.push(format!("**Ref:** {}", ref_name.unwrap_or("HEAD")));
                    output.push(format!("**Size:** {} bytes", response.len()));
                    output.push(String::new());
                    output.push("```".to_string());
                    output.push(content.into_owned());
                    output.push("```".to_string());

                    Ok(CallToolResult {
                        content: vec![ContentBlock::Text(TextContent::new(output.join("\n")))],
                        ..Default::default()
                    })
                })
            },
        )?;

        Ok(())
    }

    /// Run the server (stdio transport)
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // This will be implemented with the stdio loop
        Ok(())
    }
}
