mod support;

use futures::executor::block_on;
use serde_json::json;

use mcp_core::types::{
    BaseMetadata, ContentBlock, GetPromptRequestParams, GetPromptResult, Icons, Prompt,
    PromptMessage, RequestMessage, RequestParams, Role, TextContent,
};
use mcp_server::{McpServer, ServerOptions};

#[test]
fn prompts_list_and_get_work() {
    let server_info = support::implementation("prompt-server");
    let mut server = McpServer::new(server_info, ServerOptions::default());

    let prompt = Prompt {
        base: BaseMetadata {
            name: "welcome".to_string(),
            title: None,
        },
        icons: Icons { icons: None },
        description: Some("welcome prompt".to_string()),
        arguments: None,
        meta: None,
    };

    server
        .register_prompt(
            prompt,
            |_args, _ctx: mcp_core::protocol::RequestContext| async move {
                Ok(GetPromptResult {
                    description: Some("welcome prompt".to_string()),
                    messages: vec![PromptMessage {
                        role: Role::Assistant,
                        content: ContentBlock::Text(TextContent::new("hi")),
                    }],
                    meta: None,
                })
            },
        )
        .expect("register prompt");

    let list_request = RequestMessage::new("1", "prompts/list", json!({}));
    let list_response = block_on(server.server().handle_request(list_request, None))
        .expect("prompts/list response");
    let list_result: mcp_core::types::ListPromptsResult =
        serde_json::from_value(list_response.result.unwrap()).unwrap();
    assert_eq!(list_result.prompts.len(), 1);

    let get_params = GetPromptRequestParams {
        base: RequestParams { meta: None },
        name: "welcome".to_string(),
        arguments: None,
    };
    let get_request = RequestMessage::new(
        "2",
        "prompts/get",
        serde_json::to_value(get_params).unwrap(),
    );
    let get_response =
        block_on(server.server().handle_request(get_request, None)).expect("prompts/get response");
    let get_result: GetPromptResult = serde_json::from_value(get_response.result.unwrap()).unwrap();
    assert_eq!(get_result.messages.len(), 1);

    let notification = server.prompt_list_changed_notification();
    assert_eq!(notification.method, "notifications/prompts/list_changed");
}
