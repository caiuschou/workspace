mod support;

use futures::executor::block_on;
use serde_json::json;

use mcp_core::types::{
    BaseMetadata, Icons, ReadResourceResult, RequestMessage, RequestParams, Resource,
    ResourceContents, ResourceContentsBase, ResourceRequestParams, ResourceTemplate,
    TextResourceContents,
};
use mcp_server::{McpServer, ServerOptions};

#[test]
fn resources_list_templates_and_read_work() {
    let server_info = support::implementation("resource-server");
    let mut server = McpServer::new(server_info, ServerOptions::default());

    let resource = Resource {
        base: BaseMetadata {
            name: "test".to_string(),
            title: None,
        },
        icons: Icons { icons: None },
        uri: "file:///test.txt".to_string(),
        description: Some("test file".to_string()),
        mime_type: Some("text/plain".to_string()),
        annotations: None,
        meta: None,
    };

    server
        .register_resource(
            resource,
            |uri: String, _ctx: mcp_core::protocol::RequestContext| async move {
                let contents = ResourceContents::Text(TextResourceContents {
                    base: ResourceContentsBase {
                        uri: uri.to_string(),
                        mime_type: Some("text/plain".to_string()),
                        meta: None,
                    },
                    text: "hello".to_string(),
                });
                Ok(ReadResourceResult {
                    contents: vec![contents],
                    meta: None,
                })
            },
        )
        .expect("register resource");

    let template = ResourceTemplate {
        base: BaseMetadata {
            name: "template".to_string(),
            title: None,
        },
        icons: Icons { icons: None },
        uri_template: "file:///{path}".to_string(),
        description: None,
        mime_type: Some("text/plain".to_string()),
        annotations: None,
        meta: None,
    };

    server
        .register_resource_template(template)
        .expect("register template");

    let list_request = RequestMessage::new("1", "resources/list", json!({}));
    let list_response = block_on(server.server().handle_request(list_request, None))
        .expect("resources/list response");
    let list_result: mcp_core::types::ListResourcesResult =
        serde_json::from_value(list_response.result.unwrap()).unwrap();
    assert_eq!(list_result.resources.len(), 1);

    let templates_request = RequestMessage::new("2", "resources/templates/list", json!({}));
    let templates_response = block_on(server.server().handle_request(templates_request, None))
        .expect("resources/templates/list response");
    let templates_result: mcp_core::types::ListResourceTemplatesResult =
        serde_json::from_value(templates_response.result.unwrap()).unwrap();
    assert_eq!(templates_result.resource_templates.len(), 1);

    let read_params = ResourceRequestParams {
        base: RequestParams { meta: None },
        uri: "file:///test.txt".to_string(),
    };
    let read_request = RequestMessage::new(
        "3",
        "resources/read",
        serde_json::to_value(read_params).unwrap(),
    );
    let read_response = block_on(server.server().handle_request(read_request, None))
        .expect("resources/read response");
    let read_result: ReadResourceResult =
        serde_json::from_value(read_response.result.unwrap()).unwrap();
    assert_eq!(read_result.contents.len(), 1);

    let notification = server.resource_list_changed_notification();
    assert_eq!(notification.method, "notifications/resources/list_changed");
}
