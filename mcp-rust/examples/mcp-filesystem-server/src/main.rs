use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use mcp_core::stdio::{JsonRpcMessage, serialize_message};
use mcp_core::types::{
    BaseMetadata, CallToolResult, ContentBlock, Icons, Implementation, ReadResourceResult,
    RequestMessage, Resource, ServerCapabilities, TextContent, Tool,
};
use mcp_server::{McpServer, ServerError, ServerOptions};
use serde_json::{Value, json};

struct FilesystemState {
    roots: Vec<Value>,
    initialized: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Filesystem server error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let server_info = Implementation {
        base: BaseMetadata {
            name: "mcp-filesystem-server".to_string(),
            title: None,
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: None,
    };

    let mut server_options = ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        resources: Some(mcp_core::types::ResourceCapabilities {
            subscribe: None,
            list_changed: Some(true),
        }),
        ..Default::default()
    });
    server_options.instructions =
        Some("Filesystem MCP server that provides file operations.".to_string());

    let mut server = McpServer::new(server_info.clone(), server_options);

    register_filesystem_tools(&mut server)?;
    register_filesystem_resources(&mut server)?;

    let state = Arc::new(Mutex::new(FilesystemState {
        roots: Vec::new(),
        initialized: false,
    }));

    let state_for_init = state.clone();
    server
        .server_mut()
        .set_on_initialized(Some(Arc::new(move || {
            let mut state = state_for_init.lock().unwrap();
            state.initialized = true;
        })));

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();
    let mut read_buffer = mcp_core::stdio::ReadBuffer::default();

    loop {
        buffer.clear();
        let bytes_read = reader.read_line(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        read_buffer.append(buffer.as_bytes());
        while let Ok(Some(message)) = read_buffer.read_message() {
            match message {
                JsonRpcMessage::Request(request) => {
                    let response = block_on(server.server().handle_request(request, None))?;
                    let response_msg = JsonRpcMessage::Result(response);
                    let serialized = serialize_message(&response_msg)?;
                    stdout.write_all(serialized.as_bytes())?;
                    stdout.flush()?;
                }
                JsonRpcMessage::Notification(notification) => {
                    let method = notification.method.clone();
                    block_on(server.server().handle_notification(notification, None))?;

                    let state = state.lock().unwrap();
                    if state.initialized && method == "notifications/initialized" {
                        drop(state);
                        send_roots_list_request(&mut stdout)?;
                    }
                }
                JsonRpcMessage::Result(result) => {
                    let state = state.lock().unwrap();
                    if let Some(result_value) = &result.result {
                        if let Some(roots) = result_value.get("roots") {
                            if let Some(roots_array) = roots.as_array() {
                                let mut state = state;
                                state.roots = roots_array.clone();
                                // Register resources directly without modifying capabilities
                                register_resources_from_roots_after_init(&server, &state.roots);
                                // Send resource list changed notification
                                let notification = server.resource_list_changed_notification();
                                let notification_msg = JsonRpcMessage::Notification(notification);
                                let serialized = serialize_message(&notification_msg)?;
                                stdout.write_all(serialized.as_bytes())?;
                                stdout.flush()?;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn send_roots_list_request(stdout: &mut io::Stdout) -> Result<(), Box<dyn std::error::Error>> {
    let request = RequestMessage::new("server-roots-list", "roots/list", json!({}));
    let request_msg = JsonRpcMessage::Request(request);
    let serialized = serialize_message(&request_msg)?;
    stdout.write_all(serialized.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

fn register_filesystem_tools(server: &mut McpServer) -> Result<(), ServerError> {
    let read_file_tool = Tool {
        base: BaseMetadata {
            name: "read_file".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Read the contents of a file".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read"
                }
            },
            "required": ["path"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    server.register_tool(
        read_file_tool,
        |arguments: Option<Value>, _context: mcp_core::protocol::RequestContext| {
            Box::pin(async move {
                let args = arguments.as_ref().and_then(|a| a.as_object());
                let path = args
                    .and_then(|a| a.get("path"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| ServerError::Handler("missing path argument".to_string()))?;

                let path = uri_to_path(&path)?;
                let contents = std::fs::read_to_string(&path)
                    .map_err(|e| ServerError::Handler(format!("failed to read file: {e}")))?;

                Ok(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent {
                        kind: "text".to_string(),
                        text: contents,
                        annotations: None,
                        meta: None,
                    })],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    let write_file_tool = Tool {
        base: BaseMetadata {
            name: "write_file".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("Write contents to a file".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "contents": {
                    "type": "string",
                    "description": "The contents to write to the file"
                }
            },
            "required": ["path", "contents"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    server.register_tool(
        write_file_tool,
        |arguments: Option<Value>, _context: mcp_core::protocol::RequestContext| {
            Box::pin(async move {
                let args = arguments.as_ref().and_then(|a| a.as_object());
                let path = args
                    .and_then(|a| a.get("path"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| ServerError::Handler("missing path argument".to_string()))?;

                let contents = args
                    .and_then(|a| a.get("contents"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| ServerError::Handler("missing contents argument".to_string()))?;

                let path = uri_to_path(&path)?;
                std::fs::write(&path, contents)
                    .map_err(|e| ServerError::Handler(format!("failed to write file: {e}")))?;

                Ok(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent {
                        kind: "text".to_string(),
                        text: "File written successfully".to_string(),
                        annotations: None,
                        meta: None,
                    })],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    let list_directory_tool = Tool {
        base: BaseMetadata {
            name: "list_directory".to_string(),
            title: None,
        },
        icons: Icons::default(),
        description: Some("List the contents of a directory".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the directory to list"
                }
            },
            "required": ["path"]
        }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    server.register_tool(
        list_directory_tool,
        |arguments: Option<Value>, _context: mcp_core::protocol::RequestContext| {
            Box::pin(async move {
                let args = arguments.as_ref().and_then(|a| a.as_object());
                let path = args
                    .and_then(|a| a.get("path"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| ServerError::Handler("missing path argument".to_string()))?;

                let path = uri_to_path(&path)?;
                let entries = std::fs::read_dir(&path)
                    .map_err(|e| ServerError::Handler(format!("failed to read directory: {e}")))?;

                let mut files = Vec::new();
                for entry in entries {
                    let entry = entry.map_err(|e| {
                        ServerError::Handler(format!("failed to read directory entry: {e}"))
                    })?;
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let is_dir = path.is_dir();
                    files.push(json!({
                        "name": name,
                        "path": path_to_file_uri(&path),
                        "type": if is_dir { "directory" } else { "file" }
                    }));
                }

                Ok(CallToolResult {
                    content: vec![],
                    structured_content: Some(json!({ "files": files })),
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    Ok(())
}

fn register_filesystem_resources(_server: &mut McpServer) -> Result<(), ServerError> {
    Ok(())
}

fn register_resources_from_roots_after_init(server: &McpServer, roots: &[Value]) {
    for root in roots {
        if let Some(uri) = root.get("uri").and_then(|u| u.as_str()) {
            if let Ok(path) = uri_to_path(uri) {
                if path.is_dir() {
                    let resource = Resource {
                        base: BaseMetadata {
                            name: root
                                .get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| {
                                    path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("root")
                                        .to_string()
                                }),
                            title: None,
                        },
                        icons: Icons::default(),
                        uri: uri.to_string(),
                        description: Some(format!("Directory: {}", path.display())),
                        mime_type: Some("inode/directory".to_string()),
                        annotations: None,
                        meta: None,
                    };

                    let uri = uri.to_string();
                    server.add_resource_after_init(resource, move |_uri, _context| {
                        let uri = uri.clone();
                        Box::pin(async move {
                            let path = uri_to_path(&uri)?;
                            let entries = std::fs::read_dir(&path).map_err(|e| {
                                ServerError::Handler(format!("failed to read directory: {e}"))
                            })?;

                            let mut files = Vec::new();
                            for entry in entries {
                                let entry = entry.map_err(|e| {
                                    ServerError::Handler(format!(
                                        "failed to read directory entry: {e}"
                                    ))
                                })?;
                                let path = entry.path();
                                let name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                files.push(json!({
                                    "name": name,
                                    "path": path_to_file_uri(&path),
                                    "type": if path.is_dir() { "directory" } else { "file" }
                                }));
                            }

                            Ok(ReadResourceResult {
                                contents: vec![mcp_core::types::ResourceContents::Text(
                                    mcp_core::types::TextResourceContents {
                                        base: mcp_core::types::ResourceContentsBase {
                                            uri: uri.clone(),
                                            mime_type: None,
                                            meta: None,
                                        },
                                        text: serde_json::to_string(&files).unwrap(),
                                    },
                                )],
                                meta: None,
                            })
                        })
                    });
                } else if path.is_file() {
                    let resource = Resource {
                        base: BaseMetadata {
                            name: root
                                .get("name")
                                .and_then(|n| n.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| {
                                    path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("file")
                                        .to_string()
                                }),
                            title: None,
                        },
                        icons: Icons::default(),
                        uri: uri.to_string(),
                        description: Some(format!("File: {}", path.display())),
                        mime_type: None,
                        annotations: None,
                        meta: None,
                    };

                    let uri = uri.to_string();
                    server.add_resource_after_init(resource, move |_uri, _context| {
                        let uri = uri.clone();
                        Box::pin(async move {
                            let path = uri_to_path(&uri)?;
                            let contents = std::fs::read_to_string(&path).map_err(|e| {
                                ServerError::Handler(format!("failed to read file: {e}"))
                            })?;

                            Ok(ReadResourceResult {
                                contents: vec![mcp_core::types::ResourceContents::Text(
                                    mcp_core::types::TextResourceContents {
                                        base: mcp_core::types::ResourceContentsBase {
                                            uri: uri.clone(),
                                            mime_type: None,
                                            meta: None,
                                        },
                                        text: contents,
                                    },
                                )],
                                meta: None,
                            })
                        })
                    });
                }
            }
        }
    }
}

fn uri_to_path(uri: &str) -> Result<PathBuf, ServerError> {
    if uri.starts_with("file://") {
        let path_str = uri.strip_prefix("file://").unwrap();
        let path_str = if path_str.starts_with("//") {
            &path_str[1..]
        } else {
            path_str
        };
        Ok(PathBuf::from(path_str))
    } else {
        Ok(PathBuf::from(uri))
    }
}

fn path_to_file_uri(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if path_str.starts_with('/') {
        format!("file://{path_str}")
    } else {
        format!("file:///{path_str}")
    }
}
