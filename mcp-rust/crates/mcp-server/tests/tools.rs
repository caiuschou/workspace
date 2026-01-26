mod support;

use futures::executor::block_on;
use serde_json::json;

use mcp_core::types::{
    BaseMetadata, CallToolRequestParams, CallToolResult, ContentBlock, Icons, RequestMessage,
    RequestParams, TextContent, Tool,
};
use mcp_server::{McpServer, ServerOptions};

#[test]
fn tools_list_and_call_work() {
    let server_info = support::implementation("tool-server");
    let mut server = McpServer::new(server_info, ServerOptions::default());

    let tool = Tool {
        base: BaseMetadata {
            name: "echo".to_string(),
            title: None,
        },
        icons: Icons { icons: None },
        description: Some("echo tool".to_string()),
        input_schema: json!({ "type": "object" }),
        output_schema: None,
        annotations: None,
        execution: None,
        meta: None,
    };

    server
        .register_tool(
            tool,
            |_args, _ctx: mcp_core::protocol::RequestContext| async move {
                Ok(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new("ok"))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
        )
        .expect("register tool");

    let list_request = RequestMessage::new("1", "tools/list", json!({}));
    let list_response =
        block_on(server.server().handle_request(list_request, None)).expect("tools/list response");
    let list_result: mcp_core::types::ListToolsResult =
        serde_json::from_value(list_response.result.unwrap()).unwrap();
    assert_eq!(list_result.tools.len(), 1);

    let call_params = CallToolRequestParams {
        base: RequestParams { meta: None },
        name: "echo".to_string(),
        arguments: Some(json!({ "value": "hello" })),
        task: None,
    };
    let call_request = RequestMessage::new(
        "2",
        "tools/call",
        serde_json::to_value(call_params).unwrap(),
    );
    let call_response =
        block_on(server.server().handle_request(call_request, None)).expect("tools/call response");
    let call_result: CallToolResult =
        serde_json::from_value(call_response.result.unwrap()).unwrap();
    assert!(!call_result.content.is_empty());

    let notification = server.tool_list_changed_notification();
    assert_eq!(notification.method, "notifications/tools/list_changed");
}
