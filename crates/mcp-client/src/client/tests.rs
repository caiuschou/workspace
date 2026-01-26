use super::*;
use std::cell::RefCell;
use std::rc::Rc;

use mcp_core::stdio::{JsonRpcMessage, Transport};
use mcp_core::types::{NotificationMessage, ResultMessage};

#[derive(Debug)]
enum MockError {}

struct MockTransport {
    history: Rc<RefCell<Vec<JsonRpcMessage>>>,
    started: bool,
    closed: bool,
}

impl MockTransport {
    fn new(history: Rc<RefCell<Vec<JsonRpcMessage>>>) -> Self {
        Self {
            history,
            started: false,
            closed: false,
        }
    }
}

impl Transport for MockTransport {
    type Message = JsonRpcMessage;
    type Error = MockError;

    fn start(&mut self) -> Result<(), Self::Error> {
        self.started = true;
        Ok(())
    }

    fn send(&mut self, message: &Self::Message) -> Result<(), Self::Error> {
        self.history.borrow_mut().push(message.clone());
        Ok(())
    }

    fn close(&mut self) -> Result<(), Self::Error> {
        self.closed = true;
        Ok(())
    }
}

#[test]
fn initialize_is_sent_on_connect() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));
    let mut client = Client::new(transport, ClientOptions::new("rust-client"));

    client.connect().unwrap();

    let sent = history.borrow();
    assert_eq!(sent.len(), 1);
    if let JsonRpcMessage::Request(req) = &sent[0] {
        assert_eq!(req.method, "initialize");
        assert_eq!(req.params["clientInfo"]["name"], "rust-client");
    } else {
        panic!("expected a request");
    }
}

#[test]
fn initialize_response_sends_initialized_notification() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));
    let mut client = Client::new(transport, ClientOptions::new("rust-client"));

    client.connect().unwrap();

    let init_id = match history.borrow().get(0) {
        Some(JsonRpcMessage::Request(req)) => req.id.clone(),
        _ => panic!("expected initialize request"),
    };

    let response = ResultMessage::success(
        init_id,
        serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "rust-server", "version": "1.2.3" },
            "instructions": "hello"
        }),
    );

    client
        .handle_message(JsonRpcMessage::Result(response))
        .unwrap();

    let sent = history.borrow();
    assert_eq!(sent.len(), 2);
    if let JsonRpcMessage::Notification(note) = &sent[1] {
        assert_eq!(note.method, "notifications/initialized");
    } else {
        panic!("expected a notification");
    }

    let server = client.get_server_version().expect("server info");
    assert_eq!(server.name, "rust-server");
}

#[test]
fn list_tools_caches_task_support() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));
    let mut client = Client::new(transport, ClientOptions::new("rust-client"));

    client.connect().unwrap();
    let init_id = match history.borrow().get(0) {
        Some(JsonRpcMessage::Request(req)) => req.id.clone(),
        _ => panic!("expected initialize request"),
    };
    let response = ResultMessage::success(
        init_id,
        serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "rust-server" }
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(response))
        .unwrap();

    let list_id = client.list_tools().unwrap();
    let list_response = ResultMessage::success(
        list_id.clone(),
        serde_json::json!({
            "tools": [
                {
                    "name": "required-tool",
                    "outputSchema": { "type": "object" },
                    "execution": { "taskSupport": "required" }
                }
            ]
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(list_response))
        .unwrap();

    assert!(client.is_tool_task_required("required-tool"));
}

#[test]
fn tool_call_requires_structured_content_when_schema_exists() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));
    let mut client = Client::new(transport, ClientOptions::new("rust-client"));

    client.connect().unwrap();
    let init_id = match history.borrow().get(0) {
        Some(JsonRpcMessage::Request(req)) => req.id.clone(),
        _ => panic!("expected initialize request"),
    };
    let response = ResultMessage::success(
        init_id,
        serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "rust-server" }
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(response))
        .unwrap();

    let list_id = client.list_tools().unwrap();
    let list_response = ResultMessage::success(
        list_id.clone(),
        serde_json::json!({
            "tools": [
                {
                    "name": "schema-tool",
                    "outputSchema": { "type": "object" }
                }
            ]
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(list_response))
        .unwrap();

    let call_id = client
        .call_tool("schema-tool", serde_json::json!({}))
        .unwrap();
    let call_response = ResultMessage::success(
        call_id,
        serde_json::json!({
            "isError": false
        }),
    );
    let err = client
        .handle_message(JsonRpcMessage::Result(call_response))
        .expect_err("missing structuredContent should error");
    assert!(matches!(err, ClientError::Validation(_)));
}

#[test]
fn list_changed_debounce_delays_refresh() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));

    let mut handlers = ListChangedHandlers::default();
    handlers.tools = Some(ListChangedOptions::new(|_result| {}).with_debounce_ms(20));

    let mut client = Client::new(
        transport,
        ClientOptions::new("rust-client").with_list_changed(handlers),
    );

    client.connect().unwrap();
    let init_id = match history.borrow().get(0) {
        Some(JsonRpcMessage::Request(req)) => req.id.clone(),
        _ => panic!("expected initialize request"),
    };
    let response = ResultMessage::success(
        init_id,
        serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": { "tools": { "listChanged": true } },
            "serverInfo": { "name": "rust-server" }
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(response))
        .unwrap();

    client
        .handle_message(JsonRpcMessage::Notification(NotificationMessage::new(
            "notifications/tools/list_changed",
            None,
        )))
        .unwrap();

    assert_eq!(history.borrow().len(), 2);

    std::thread::sleep(std::time::Duration::from_millis(25));

    client
        .handle_message(JsonRpcMessage::Notification(NotificationMessage::new(
            "notifications/initialized",
            None,
        )))
        .unwrap();

    let sent = history.borrow();
    assert_eq!(sent.len(), 3);
    if let JsonRpcMessage::Request(req) = &sent[2] {
        assert_eq!(req.method, "tools/list");
    } else {
        panic!("expected tools/list request");
    }
}

#[test]
fn prompts_list_changed_triggers_handler() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));

    let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let received_clone = std::sync::Arc::clone(&received);

    let mut handlers = ListChangedHandlers::default();
    handlers.prompts = Some(ListChangedOptions::new(move |result| {
        if let Ok(Some(items)) = result {
            if let Ok(mut guard) = received_clone.lock() {
                *guard = items;
            }
        }
    }));

    let mut client = Client::new(
        transport,
        ClientOptions::new("rust-client").with_list_changed(handlers),
    );

    client.connect().unwrap();
    let init_id = match history.borrow().get(0) {
        Some(JsonRpcMessage::Request(req)) => req.id.clone(),
        _ => panic!("expected initialize request"),
    };
    let response = ResultMessage::success(
        init_id,
        serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": { "prompts": { "listChanged": true } },
            "serverInfo": { "name": "rust-server" }
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(response))
        .unwrap();

    client
        .handle_message(JsonRpcMessage::Notification(NotificationMessage::new(
            "notifications/prompts/list_changed",
            None,
        )))
        .unwrap();

    let list_id = match history.borrow().iter().find_map(|msg| {
        if let JsonRpcMessage::Request(req) = msg {
            if req.method == "prompts/list" {
                return Some(req.id.clone());
            }
        }
        None
    }) {
        Some(id) => id,
        None => panic!("expected prompts/list request"),
    };

    let list_response = ResultMessage::success(
        list_id,
        serde_json::json!({
            "prompts": [
                { "name": "welcome", "description": "hello" }
            ]
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(list_response))
        .unwrap();

    let captured = received.lock().expect("received lock");
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0]["name"], "welcome");
}

#[test]
fn request_stream_returns_result() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));
    let mut client = Client::new(transport, ClientOptions::new("rust-client"));

    client.connect().unwrap();
    let init_id = match history.borrow().get(0) {
        Some(JsonRpcMessage::Request(req)) => req.id.clone(),
        _ => panic!("expected initialize request"),
    };
    let response = ResultMessage::success(
        init_id,
        serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "rust-server" }
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(response))
        .unwrap();

    let stream = client
        .request_stream("tools/list", serde_json::json!({}))
        .unwrap();

    let request_id = match history.borrow().iter().find_map(|msg| {
        if let JsonRpcMessage::Request(req) = msg {
            if req.method == "tools/list" {
                return Some(req.id.clone());
            }
        }
        None
    }) {
        Some(id) => id,
        None => panic!("expected tools/list request"),
    };

    let list_response = ResultMessage::success(
        request_id,
        serde_json::json!({
            "tools": []
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(list_response))
        .unwrap();

    let mut stream = stream;
    match stream.next() {
        Some(ResponseMessage::Result(value)) => {
            assert!(value.get("tools").is_some());
        }
        other => panic!("unexpected stream message: {other:?}"),
    }
}

#[test]
fn request_stream_emits_task_notifications() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));
    let mut client = Client::new(transport, ClientOptions::new("rust-client"));

    client.connect().unwrap();
    let init_id = match history.borrow().get(0) {
        Some(JsonRpcMessage::Request(req)) => req.id.clone(),
        _ => panic!("expected initialize request"),
    };
    let response = ResultMessage::success(
        init_id,
        serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "rust-server" }
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(response))
        .unwrap();

    let mut stream = client
        .request_stream("tools/list", serde_json::json!({}))
        .unwrap();

    client
        .handle_message(JsonRpcMessage::Notification(NotificationMessage::new(
            "notifications/tasks/created",
            Some(serde_json::json!({
                "taskId": "task-1",
                "status": "running"
            })),
        )))
        .unwrap();

    match stream.next() {
        Some(ResponseMessage::TaskCreated(task)) => {
            assert_eq!(task.task_id, "task-1");
            assert_eq!(task.status.as_deref(), Some("running"));
        }
        other => panic!("unexpected stream message: {other:?}"),
    }
}

#[test]
fn task_requests_require_tasks_capability() {
    let history = Rc::new(RefCell::new(Vec::new()));
    let transport = MockTransport::new(Rc::clone(&history));
    let mut client = Client::new(transport, ClientOptions::new("rust-client"));

    client.connect().unwrap();
    let init_id = match history.borrow().get(0) {
        Some(JsonRpcMessage::Request(req)) => req.id.clone(),
        _ => panic!("expected initialize request"),
    };
    let response = ResultMessage::success(
        init_id,
        serde_json::json!({
            "protocolVersion": "0.1.0",
            "capabilities": {},
            "serverInfo": { "name": "rust-server" }
        }),
    );
    client
        .handle_message(JsonRpcMessage::Result(response))
        .unwrap();

    let err = client
        .get_task("task-1")
        .expect_err("tasks should be unsupported");
    assert!(matches!(err, ClientError::Capability(_)));
}
