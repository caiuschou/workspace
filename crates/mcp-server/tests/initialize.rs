mod support;

use futures::executor::block_on;
use mcp_core::types::{
    ClientCapabilities, InitializeRequestParams, InitializeResult, LATEST_PROTOCOL_VERSION,
    RequestMessage, RequestParams,
};
use mcp_server::{Server, ServerOptions};

#[test]
fn initialize_returns_version_and_sets_client_info() {
    let server_info = support::implementation("test-server");
    let client_info = support::implementation("test-client");
    let server = Server::new(server_info, ServerOptions::default());

    let params = InitializeRequestParams {
        base: RequestParams { meta: None },
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        capabilities: ClientCapabilities::default(),
        client_info,
    };

    let request = RequestMessage::new("1", "initialize", serde_json::to_value(params).unwrap());
    let response = block_on(server.handle_request(request, None)).expect("initialize response");
    let result: InitializeResult = serde_json::from_value(response.result.unwrap()).unwrap();

    assert_eq!(result.protocol_version, LATEST_PROTOCOL_VERSION);
    assert!(server.get_client_capabilities().is_some());
    assert!(server.get_client_info().is_some());
}
