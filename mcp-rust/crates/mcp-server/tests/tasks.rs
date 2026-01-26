mod support;

use std::sync::Arc;

use futures::executor::block_on;
use serde_json::json;

use mcp_core::protocol::ProtocolOptions;
use mcp_core::types::{
    BaseMetadata, CallToolRequestParams, CallToolResult, ContentBlock, CreateTaskResult,
    GetTaskPayloadRequestParams, GetTaskRequestParams, GetTaskResult, Icons, RequestMessage,
    RequestParams, TaskMetadata, TextContent, Tool,
};
use mcp_server::{InMemoryTaskStore, McpServer, ServerOptions};

#[test]
fn tasks_flow_returns_result() {
    let server_info = support::implementation("task-server");
    let task_store = Arc::new(InMemoryTaskStore::default());

    let options = ServerOptions {
        protocol_options: Some(ProtocolOptions {
            task_store: Some(task_store),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut server = McpServer::new(server_info, options);

    let tool = Tool {
        base: BaseMetadata {
            name: "echo".to_string(),
            title: None,
        },
        icons: Icons { icons: None },
        description: None,
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
                    content: vec![ContentBlock::Text(TextContent::new("done"))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            },
        )
        .expect("register tool");

    let call_params = CallToolRequestParams {
        base: RequestParams { meta: None },
        name: "echo".to_string(),
        arguments: None,
        task: Some(TaskMetadata { ttl: Some(1000) }),
    };

    let call_request = RequestMessage::new(
        "1",
        "tools/call",
        serde_json::to_value(call_params).unwrap(),
    );
    let call_response =
        block_on(server.server().handle_request(call_request, None)).expect("tools/call response");
    let create_task: CreateTaskResult =
        serde_json::from_value(call_response.result.unwrap()).unwrap();
    let task_id = create_task.task.task_id.clone();

    let get_params = GetTaskRequestParams {
        base: RequestParams { meta: None },
        task_id: task_id.clone(),
    };
    let get_request =
        RequestMessage::new("2", "tasks/get", serde_json::to_value(get_params).unwrap());
    let get_response =
        block_on(server.server().handle_request(get_request, None)).expect("tasks/get response");
    let get_result: GetTaskResult = serde_json::from_value(get_response.result.unwrap()).unwrap();
    assert_eq!(get_result.task.task_id, task_id);

    let result_params = GetTaskPayloadRequestParams {
        base: RequestParams { meta: None },
        task_id: task_id.clone(),
    };
    let result_request = RequestMessage::new(
        "3",
        "tasks/result",
        serde_json::to_value(result_params).unwrap(),
    );
    let result_response = block_on(server.server().handle_request(result_request, None))
        .expect("tasks/result response");
    let call_result: CallToolResult =
        serde_json::from_value(result_response.result.unwrap()).unwrap();
    assert!(!call_result.content.is_empty());

    let list_request = RequestMessage::new("4", "tasks/list", json!({}));
    let list_response =
        block_on(server.server().handle_request(list_request, None)).expect("tasks/list response");
    let list_result: mcp_core::types::ListTasksResult =
        serde_json::from_value(list_response.result.unwrap()).unwrap();
    assert!(!list_result.tasks.is_empty());
}
