use async_trait::async_trait;
use futures::executor::block_on;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use mcp_core::{
    JsonSchemaValidator, Protocol, ProtocolError, RequestContext, RequestHandler, RequestMessage,
};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct EchoParams {
    text: String,
}

struct EchoHandler;

#[async_trait]
impl RequestHandler for EchoHandler {
    async fn handle(
        &self,
        request: &RequestMessage,
        _context: &RequestContext,
    ) -> Result<serde_json::Value, ProtocolError> {
        let params: EchoParams = serde_json::from_value(request.params.clone())?;
        Ok(json!({ "echo": params.text }))
    }
}

#[test]
fn handles_valid_request() {
    let mut protocol = Protocol::new(JsonSchemaValidator::default());
    protocol.register_handler(
        "echo",
        JsonSchemaValidator::schema_for::<EchoParams>(),
        EchoHandler,
    );

    let request = RequestMessage::new("1", "echo", json!({ "text": "hello" }));
    let response = block_on(protocol.handle_request(request)).expect("valid response");
    assert!(response.error.is_none());
    assert_eq!(response.result.unwrap()["echo"], "hello");
}

#[test]
fn rejects_unknown_method() {
    let protocol = Protocol::new(JsonSchemaValidator::default());
    let request = RequestMessage::new("1", "missing", json!({}));
    let err = block_on(protocol.handle_request(request)).expect_err("should error");
    assert!(matches!(err, ProtocolError::UnknownMethod(method) if method == "missing"));
}
