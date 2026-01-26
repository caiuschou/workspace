use super::*;
use mcp_core::types::RequestMessage;
use serde_json::json;
use std::sync::mpsc::channel;
use std::time::Duration;

#[cfg(unix)]
#[test]
fn transport_roundtrip_via_cat() {
    let params = StdioServerParameters::new("cat");
    let mut transport = StdioClientTransport::new(params);
    let (tx, rx) = channel();

    transport.on_error(|err| panic!("echo transport errored: {err}"));
    transport.on_message(move |message| {
        let _ = tx.send(message);
    });

    transport.start().expect("should start cat");

    let request =
        JsonRpcMessage::Request(RequestMessage::new("1", "echo", json!({ "text": "hello" })));

    transport
        .send(&request)
        .expect("should be able to send to cat");

    let received = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("cat should echo");
    assert_eq!(received, request);

    transport.close().expect("should close cleanly");
}

#[cfg(unix)]
#[test]
fn start_then_close_triggers_on_close() {
    let params = StdioServerParameters::new("cat");
    let mut transport = StdioClientTransport::new(params);
    let (tx, rx) = channel();

    transport.on_close(move || {
        let _ = tx.send(());
    });

    transport.start().expect("should start cat");
    assert!(rx.try_recv().is_err());
    transport.close().expect("should close cleanly");
    assert!(rx.try_recv().is_ok());
}

#[cfg(unix)]
#[test]
fn reads_multiple_messages() {
    let params = StdioServerParameters::new("cat");
    let mut transport = StdioClientTransport::new(params);
    let (tx, rx) = channel();

    let messages = vec![
        JsonRpcMessage::Request(RequestMessage::new("1", "echo", json!({ "text": "one" }))),
        JsonRpcMessage::Request(RequestMessage::new("2", "echo", json!({ "text": "two" }))),
    ];

    transport.on_message(move |message| {
        let _ = tx.send(message);
    });

    transport.start().expect("should start cat");

    for message in &messages {
        transport
            .send(message)
            .expect("should be able to send to cat");
    }

    let mut received = Vec::new();
    for _ in 0..messages.len() {
        received.push(
            rx.recv_timeout(Duration::from_secs(1))
                .expect("cat should echo"),
        );
    }

    assert_eq!(received, messages);

    transport.close().expect("should close cleanly");
}
