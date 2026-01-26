use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use mcp_client::stdio::{
    JsonRpcMessage, StdioClientTransport, StdioClientTransportError, StdioServerParameters,
    StdioStream,
};
use mcp_core::{CoreConfig, MessageId, NotificationMessage, RequestMessage, ResultMessage, Role};
use serde_json::{Value, json};

const FILESYSTEM_DEFAULT_COMMAND: &str = "cargo";
const FILESYSTEM_DEFAULT_ARGS: &[&str] = &["run", "-p", "mcp-filesystem-server", "--quiet"];
const INITIALIZE_REQUEST_ID: &str = "client-initialize";
const TOOL_LIST_REQUEST_ID: &str = "client-tools-list";
const CALL_TOOL_REQUEST_ID: &str = "client-call-tool";
const LATEST_PROTOCOL_VERSION: &str = "2025-11-25";
const LIST_DIRECTORY_TOOL: &str = "list_directory";

fn main() {
    if let Err(error) = run() {
        eprintln!("Filesystem example failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = CoreConfig::dev("filesystem-example");
    announce_role(Role::Client, &config);

    let (command, args) = resolve_filesystem_server_command();
    println!(
        "Running filesystem service via `{}` with args {args:?}",
        command
    );

    let (message_tx, message_rx) = mpsc::channel::<JsonRpcMessage>();

    let mut transport = StdioClientTransport::new(
        StdioServerParameters::new(command.clone())
            .args(args.clone())
            .stderr(StdioStream::Inherit),
    );

    transport.on_message(move |message| {
        let _ = message_tx.send(message);
    });

    transport.on_error(|error| eprintln!("Filesystem transport error: {error}"));

    let roots = build_root_payloads()?;

    transport.start()?;
    println!("Transport ready, initializing server...");
    send_initialize(&mut transport)?;

    if let Some(result) = drive_initialization_and_roots(
        &mut transport,
        &message_rx,
        &roots,
        Duration::from_secs(20),
    )? {
        describe_roots(&result);
    } else {
        eprintln!("Timeout waiting for initialization/roots exchange");
    }

    send_request(
        &mut transport,
        TOOL_LIST_REQUEST_ID,
        "tools/list",
        json!({}),
    )?;

    if let Some(tools_result) = wait_for_result(
        &mut transport,
        &message_rx,
        &roots,
        TOOL_LIST_REQUEST_ID,
        Duration::from_secs(10),
    )? {
        println!(
            "Tools list response: {}",
            tools_result.result.as_ref().unwrap_or(&json!({}))
        );
        if tool_is_available(&tools_result, LIST_DIRECTORY_TOOL) {
            // Use the first root directory for testing
            let test_path = roots
                .first()
                .and_then(|r| r.get("uri"))
                .and_then(|u| u.as_str())
                .unwrap_or("file:///tmp");
            send_request(
                &mut transport,
                CALL_TOOL_REQUEST_ID,
                "tools/call",
                json!({ "name": LIST_DIRECTORY_TOOL, "arguments": { "path": test_path } }),
            )?;

            if let Some(call_result) = wait_for_result(
                &mut transport,
                &message_rx,
                &roots,
                CALL_TOOL_REQUEST_ID,
                Duration::from_secs(10),
            )? {
                println!(
                    "Tool call response: {}",
                    call_result.result.as_ref().unwrap_or(&json!({}))
                );
            }
        } else {
            println!("Tool `{LIST_DIRECTORY_TOOL}` not advertised by server");
        }
    }

    transport.close()?;
    Ok(())
}

fn describe_roots(result: &ResultMessage) {
    if let Some(result_body) = &result.result {
        println!(
            "Filesystem service replied to `{}`: {result_body}",
            result.id
        );
        if let Some(roots) = result_body.get("roots") {
            println!("Found root payload: {roots}");
        } else {
            println!("No `roots` field found in response");
        }
    } else if let Some(error) = &result.error {
        eprintln!(
            "Filesystem request `{}` failed: {}",
            result.id, error.message
        );
    } else {
        println!("Filesystem request `{}` returned nothing", result.id);
    }
}

fn drive_initialization_and_roots(
    transport: &mut StdioClientTransport,
    receiver: &mpsc::Receiver<JsonRpcMessage>,
    roots: &[Value],
    timeout: Duration,
) -> Result<Option<ResultMessage>, StdioClientTransportError> {
    let deadline = Instant::now() + timeout;
    let mut init_received = false;
    let mut roots_sent = false;
    let mut roots_result: Option<ResultMessage> = None;

    while Instant::now() < deadline {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .unwrap_or_else(|| Duration::from_secs(0));
        match receiver.recv_timeout(remaining.min(Duration::from_secs(1))) {
            Ok(JsonRpcMessage::Request(request)) if request.method == "roots/list" => {
                println!("Received roots/list request from server, sending roots...");
                let result = ResultMessage::success(request.id.clone(), json!({ "roots": roots }));
                let response = JsonRpcMessage::Result(result.clone());
                transport.send(&response)?;
                roots_sent = true;
                roots_result = Some(result);
            }
            Ok(JsonRpcMessage::Request(request)) => {
                println!("Unexpected server request: {}", request.method);
            }
            Ok(JsonRpcMessage::Result(message))
                if message_id_matches(&message.id, INITIALIZE_REQUEST_ID) =>
            {
                init_received = true;
                println!("Initialize response received, sending notifications...");
                transport.send(&JsonRpcMessage::Notification(NotificationMessage::new(
                    "notifications/initialized",
                    Some(json!({})),
                )))?;
            }
            Ok(JsonRpcMessage::Result(message)) => {
                println!("Ignored result `{}` during wait", message.id);
            }
            Ok(JsonRpcMessage::Notification(notification)) => {
                println!("Notification received: {}", notification.method);
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }

        if init_received && roots_sent {
            return Ok(roots_result);
        }
    }

    Ok(None)
}

fn wait_for_result(
    transport: &mut StdioClientTransport,
    receiver: &mpsc::Receiver<JsonRpcMessage>,
    roots: &[Value],
    request_id: &str,
    timeout: Duration,
) -> Result<Option<ResultMessage>, StdioClientTransportError> {
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .unwrap_or_else(|| Duration::from_secs(0));
        match receiver.recv_timeout(remaining.min(Duration::from_secs(1))) {
            Ok(JsonRpcMessage::Request(request)) if request.method == "roots/list" => {
                println!("Received roots/list request from server, sending roots...");
                let result = ResultMessage::success(request.id.clone(), json!({ "roots": roots }));
                transport.send(&JsonRpcMessage::Result(result))?;
            }
            Ok(JsonRpcMessage::Request(request)) => {
                println!("Unexpected server request: {}", request.method);
            }
            Ok(JsonRpcMessage::Result(message)) if message_id_matches(&message.id, request_id) => {
                return Ok(Some(message));
            }
            Ok(JsonRpcMessage::Result(message)) => {
                println!("Ignored result `{}` during wait", message.id);
            }
            Ok(JsonRpcMessage::Notification(notification)) => {
                println!("Notification received: {}", notification.method);
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(None)
}

fn resolve_filesystem_server_command() -> (String, Vec<String>) {
    let command = env::var("FILESYSTEM_SERVER_COMMAND")
        .unwrap_or_else(|_| FILESYSTEM_DEFAULT_COMMAND.to_string());
    let args = env::var("FILESYSTEM_SERVER_ARGS")
        .map(|value| value.split_whitespace().map(String::from).collect())
        .unwrap_or_else(|_| {
            FILESYSTEM_DEFAULT_ARGS
                .iter()
                .map(|s| s.to_string())
                .collect()
        });
    (command, args)
}

fn send_initialize(transport: &mut StdioClientTransport) -> Result<(), StdioClientTransportError> {
    let params = json!({
        "protocolVersion": LATEST_PROTOCOL_VERSION,
        "capabilities": {
            "roots": {
                "listChanged": false
            }
        },
        "clientInfo": {
            "name": "mcp-rust-examples",
            "version": "0.1.0"
        }
    });
    let request = RequestMessage::new(INITIALIZE_REQUEST_ID, "initialize", params);
    transport.send(&JsonRpcMessage::Request(request))?;
    Ok(())
}

fn send_request(
    transport: &mut StdioClientTransport,
    request_id: &str,
    method: &str,
    params: Value,
) -> Result<(), StdioClientTransportError> {
    let request = RequestMessage::new(request_id, method, params);
    transport.send(&JsonRpcMessage::Request(request))?;
    Ok(())
}

fn message_id_matches(message_id: &MessageId, expected: &str) -> bool {
    message_id.as_str() == Some(expected)
}

fn tool_is_available(result: &ResultMessage, tool_name: &str) -> bool {
    result
        .result
        .as_ref()
        .and_then(|value| value.get("tools"))
        .and_then(|value| value.as_array())
        .map(|tools| {
            tools.iter().any(|tool| {
                tool.get("name")
                    .and_then(|name| name.as_str())
                    .is_some_and(|name| name == tool_name)
            })
        })
        .unwrap_or(false)
}

fn build_root_payloads() -> Result<Vec<Value>, std::io::Error> {
    let default_root = env::current_dir()?;
    let roots = env::var_os("FILESYSTEM_ROOTS")
        .map(|value| env::split_paths(&value).collect::<Vec<PathBuf>>())
        .unwrap_or_else(|| vec![default_root]);

    let mut payloads = Vec::new();
    for root in roots {
        let absolute = if root.is_absolute() {
            root
        } else {
            env::current_dir()?.join(root)
        };
        payloads.push(build_root_payload(&absolute));
    }

    Ok(payloads)
}

fn build_root_payload(path: &Path) -> Value {
    let mut root = serde_json::Map::new();
    root.insert("uri".to_string(), Value::String(path_to_file_uri(path)));
    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
        root.insert("name".to_string(), Value::String(name.to_string()));
    }
    Value::Object(root)
}

fn path_to_file_uri(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if path_str.starts_with('/') {
        format!("file://{path_str}")
    } else {
        format!("file:///{path_str}")
    }
}

fn announce_role(role: Role, config: &CoreConfig) {
    println!(
        "Running {} in {:?} mode (role: {:?})",
        config.service_name, config.environment, role
    );
}
