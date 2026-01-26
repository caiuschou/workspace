mod support;

use futures::executor::block_on;

use mcp_core::types::{
    CapabilityFlag, ClientCapabilities, InitializeRequestParams, LATEST_PROTOCOL_VERSION,
    RequestMessage, RequestParams, ServerCapabilities, ToolCapabilities,
};
use mcp_server::{Server, ServerError, ServerOptions};

#[test]
fn register_capabilities_merges_and_locks() {
    let server_info = support::implementation("test-server");
    let mut server = Server::new(server_info, ServerOptions::default());

    server
        .register_capabilities(ServerCapabilities {
            logging: Some(CapabilityFlag::default()),
            ..Default::default()
        })
        .expect("register logging");

    server
        .register_capabilities(ServerCapabilities {
            tools: Some(ToolCapabilities {
                list_changed: Some(true),
            }),
            ..Default::default()
        })
        .expect("register tools");

    let merged = server.get_capabilities();
    assert!(merged.logging.is_some());
    assert_eq!(merged.tools.unwrap().list_changed, Some(true));

    let params = InitializeRequestParams {
        base: RequestParams { meta: None },
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: support::implementation("client"),
    };
    let request = RequestMessage::new("1", "initialize", serde_json::to_value(params).unwrap());
    let _ = block_on(server.handle_request(request, None)).expect("initialize response");

    let err = server
        .register_capabilities(ServerCapabilities::default())
        .expect_err("capabilities locked");
    assert!(matches!(err, ServerError::CapabilitiesLocked));
}
