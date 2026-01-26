//! MCP transport layer for communicating with gitlab-mcp-server

use std::collections::HashMap;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use mcp_client::stdio::{JsonRpcMessage, StdioClientTransport, StdioServerParameters, StdioStream};
use mcp_core::{NotificationMessage, RequestMessage, ResultMessage};
use serde_json::{json, Value};

use crate::Result;

/// MCP client that communicates with gitlab-mcp-server via stdio
pub struct McpServerClient {
    transport: StdioClientTransport,
    receiver: mpsc::Receiver<JsonRpcMessage>,
}

impl McpServerClient {
    /// Start the MCP server and create a new client connection
    pub fn start(server_command: &str, server_args: &[String]) -> Result<Self> {
        let (message_tx, message_rx) = mpsc::channel();

        // Collect environment variables to pass to the server
        let mut server_env = HashMap::new();
        if let Ok(token) = std::env::var("GITLAB_TOKEN") {
            server_env.insert("GITLAB_TOKEN".to_string(), token);
        }
        if let Ok(url) = std::env::var("GITLAB_URL") {
            server_env.insert("GITLAB_URL".to_string(), url);
        }
        // Also pass through HOME for config file location
        if let Ok(home) = std::env::var("HOME") {
            server_env.insert("HOME".to_string(), home);
        }

        let mut params = StdioServerParameters::new(server_command)
            .args(server_args)
            .stderr(StdioStream::Inherit);
        if !server_env.is_empty() {
            params = params.env(server_env);
        }

        let mut transport = StdioClientTransport::new(params);

        transport.on_message(move |message| {
            let _ = message_tx.send(message);
        });

        transport.on_error(|error| eprintln!("MCP transport error: {error}"));

        // Start the server process
        transport.start()?;

        let mut client = Self {
            transport,
            receiver: message_rx,
        };

        // Initialize the MCP session
        client.initialize()?;

        Ok(client)
    }

    /// Initialize the MCP session
    fn initialize(&mut self) -> Result<()> {
        let params = json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "gitlab-mcp-client",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        self.send_request("client-initialize", "initialize", params)?;

        // Wait for initialize response
        match self.wait_for_result("client-initialize", Duration::from_secs(10))? {
            Some(result) => {
                if let Some(error) = result.error {
                    return Err(anyhow::anyhow!("Initialize failed: {}", error.message));
                }
                // Send initialized notification
                let notification = JsonRpcMessage::Notification(NotificationMessage::new(
                    "notifications/initialized",
                    Some(json!({})),
                ));
                self.transport.send(&notification)?;
            }
            None => return Err(anyhow::anyhow!("Timeout waiting for initialize response")),
        }

        Ok(())
    }

    /// List available tools from the server
    pub fn list_tools(&mut self) -> Result<Vec<Tool>> {
        self.send_request("client-tools-list", "tools/list", json!({}))?;

        match self.wait_for_result("client-tools-list", Duration::from_secs(5))? {
            Some(result) => {
                if let Some(error) = result.error {
                    return Err(anyhow::anyhow!("List tools failed: {}", error.message));
                }
                if let Some(result_value) = result.result {
                    if let Some(tools_value) = result_value.get("tools") {
                        let tools: Vec<Tool> = serde_json::from_value(tools_value.clone())
                            .map_err(|e| anyhow::anyhow!("Failed to parse tools: {}", e))?;
                        return Ok(tools);
                    }
                }
            }
            None => return Err(anyhow::anyhow!("Timeout waiting for tools list")),
        }
        Ok(Vec::new())
    }

    /// Call a tool on the server
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<ToolResponse> {
        let request_id = format!("call-tool-{}", name);

        self.send_request(
            &request_id,
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )?;

        match self.wait_for_result(&request_id, Duration::from_secs(30))? {
            Some(result) => {
                if let Some(error) = result.error {
                    return Err(anyhow::anyhow!("Tool call failed: {}", error.message));
                }
                Ok(ToolResponse {
                    result: result.result.unwrap_or(Value::Null),
                })
            }
            None => Err(anyhow::anyhow!(
                "Timeout waiting for tool response: {}",
                name
            )),
        }
    }

    /// List projects
    pub fn list_projects(
        &mut self,
        search: Option<String>,
        per_page: u64,
        page: Option<u64>,
    ) -> Result<Vec<Project>> {
        let mut args = json!({
            "per_page": per_page,
        });
        if let Some(search) = search {
            args["search"] = json!(search);
        }
        if let Some(page) = page {
            args["page"] = json!(page);
        }

        let response = self.call_tool("list_projects", args)?;

        // Check if the response is an error
        if let Some(is_error) = response.result.get("isError").and_then(|v| v.as_bool()) {
            if is_error {
                if let Some(content) = response.result.get("content").and_then(|c| c.as_array()) {
                    if let Some(error_msg) = content.first().and_then(|c| c.get("text").and_then(|t| t.as_str())) {
                        if error_msg.starts_with("Error") {
                            return Err(anyhow::anyhow!("{}", error_msg));
                        }
                    }
                }
            }
        }

        // Parse the response content
        if let Some(content) = response.result.get("content").and_then(|c| c.as_array()) {
            if let Some(text) = content.first().and_then(|c| c.get("text").and_then(|t| t.as_str())) {
                // Parse the JSON from the text content
                let projects: Vec<Project> = serde_json::from_str(text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse projects: {}", e))?;
                return Ok(projects);
            }
        }

        Ok(Vec::new())
    }

    /// Get a project by ID
    pub fn get_project(&mut self, project_id: &str) -> Result<Project> {
        let args = json!({ "project_id": project_id });
        let response = self.call_tool("get_project", args)?;

        if let Some(content) = response.result.get("content").and_then(|c| c.as_array()) {
            if let Some(text) = content.first().and_then(|c| c.get("text").and_then(|t| t.as_str())) {
                let project: Project = serde_json::from_str(text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse project: {}", e))?;
                return Ok(project);
            }
        }

        Err(anyhow::anyhow!("No project data in response"))
    }

    /// List issues for a project
    pub fn list_issues(&mut self, project_id: &str) -> Result<Vec<Issue>> {
        let args = json!({ "project_id": project_id });
        let response = self.call_tool("list_issues", args)?;

        if let Some(content) = response.result.get("content").and_then(|c| c.as_array()) {
            if let Some(text) = content.first().and_then(|c| c.get("text").and_then(|t| t.as_str())) {
                let issues: Vec<Issue> = serde_json::from_str(text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse issues: {}", e))?;
                return Ok(issues);
            }
        }

        Ok(Vec::new())
    }

    /// Get an issue by IID
    pub fn get_issue(&mut self, project_id: &str, issue_iid: u64) -> Result<Issue> {
        let args = json!({ "project_id": project_id, "issue_iid": issue_iid });
        let response = self.call_tool("get_issue", args)?;

        if let Some(content) = response.result.get("content").and_then(|c| c.as_array()) {
            if let Some(text) = content.first().and_then(|c| c.get("text").and_then(|t| t.as_str())) {
                let issue: Issue = serde_json::from_str(text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse issue: {}", e))?;
                return Ok(issue);
            }
        }

        Err(anyhow::anyhow!("No issue data in response"))
    }

    /// List merge requests for a project
    pub fn list_merge_requests(&mut self, project_id: &str) -> Result<Vec<MergeRequest>> {
        let args = json!({ "project_id": project_id });
        let response = self.call_tool("list_merge_requests", args)?;

        if let Some(content) = response.result.get("content").and_then(|c| c.as_array()) {
            if let Some(text) = content.first().and_then(|c| c.get("text").and_then(|t| t.as_str())) {
                let mrs: Vec<MergeRequest> = serde_json::from_str(text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse merge requests: {}", e))?;
                return Ok(mrs);
            }
        }

        Ok(Vec::new())
    }

    /// Get a merge request by IID
    pub fn get_merge_request(&mut self, project_id: &str, mr_iid: u64) -> Result<MergeRequest> {
        let args = json!({ "project_id": project_id, "mr_iid": mr_iid });
        let response = self.call_tool("get_merge_request", args)?;

        if let Some(content) = response.result.get("content").and_then(|c| c.as_array()) {
            if let Some(text) = content.first().and_then(|c| c.get("text").and_then(|t| t.as_str())) {
                let mr: MergeRequest = serde_json::from_str(text)
                    .map_err(|e| anyhow::anyhow!("Failed to parse merge request: {}", e))?;
                return Ok(mr);
            }
        }

        Err(anyhow::anyhow!("No merge request data in response"))
    }

    /// Close the connection
    pub fn close(mut self) -> Result<()> {
        self.transport.close()?;
        Ok(())
    }

    // Helper methods

    fn send_request(
        &mut self,
        request_id: &str,
        method: &str,
        params: Value,
    ) -> Result<()> {
        let request = RequestMessage::new(request_id, method, params);
        self.transport
            .send(&JsonRpcMessage::Request(request))
            .map_err(|e| anyhow::anyhow!("Failed to send request: {}", e))
    }

    fn wait_for_result(
        &mut self,
        request_id: &str,
        timeout: Duration,
    ) -> Result<Option<ResultMessage>> {
        let deadline = std::time::Instant::now() + timeout;

        while std::time::Instant::now() < deadline {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .unwrap_or_else(|| Duration::from_secs(0));

            match self.receiver.recv_timeout(remaining.min(Duration::from_secs(1))) {
                Ok(JsonRpcMessage::Result(message)) if message_id_matches(&message.id, request_id) => {
                    return Ok(Some(message));
                }
                Ok(JsonRpcMessage::Request(request)) if request.method == "roots/list" => {
                    // Respond with empty roots for now
                    let result = ResultMessage::success(request.id.clone(), json!({ "roots": [] }));
                    let _ = self.transport.send(&JsonRpcMessage::Result(result));
                }
                Ok(JsonRpcMessage::Notification(_)) => {}
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        Ok(None)
    }
}

fn message_id_matches(message_id: &mcp_core::MessageId, expected: &str) -> bool {
    message_id.as_str() == Some(expected)
}

/// Response from a tool call
#[derive(Debug)]
pub struct ToolResponse {
    pub result: Value,
}

/// Tool metadata
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
}

/// GitLab Project
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Project {
    pub id: u64,
    pub iid: Option<u64>,
    pub name: String,
    pub path: Option<String>,
    pub path_with_namespace: String,
    pub description: Option<String>,
    pub default_branch: Option<String>,
    pub web_url: String,
    pub ssh_url_to_repo: Option<String>,
    pub http_url_to_repo: Option<String>,
    pub created_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub visibility: Option<String>,
    pub star_count: Option<u64>,
    pub forks_count: Option<u64>,
    pub topics: Option<Vec<String>>,
}

/// GitLab Issue
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Issue {
    pub id: u64,
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub web_url: String,
    pub author: Option<User>,
    pub assignees: Option<Vec<User>>,
    pub created_at: String,
    pub updated_at: String,
    pub labels: Option<Vec<String>>,
}

/// GitLab Merge Request
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MergeRequest {
    pub id: u64,
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub web_url: String,
    pub author: Option<User>,
    pub assignees: Option<Vec<User>>,
    pub source_branch: String,
    pub target_branch: String,
    pub created_at: String,
    pub updated_at: String,
    pub labels: Option<Vec<String>>,
}

/// GitLab User
#[derive(Debug, Clone, serde::Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub name: String,
    pub web_url: Option<String>,
}
