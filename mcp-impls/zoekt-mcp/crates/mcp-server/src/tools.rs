//! MCP tools for Zoekt: zoekt_search, zoekt_list.
//!
//! Both call the Zoekt JSON API (POST /api/search, POST /api/list) and return
//! the API response as JSON text in the tool result.

use mcp_core::protocol::RequestContext;
use mcp_core::types::{
    BaseMetadata, CallToolResult, ContentBlock, Icons, TextContent, Tool,
};
use mcp_server::{McpServer, ServerError};
use serde_json::{json, Value};

use crate::client::{ZoektClient, ZoektClientError};

/// Registers Zoekt tools: zoekt_search, zoekt_list.
pub fn register_tools(server: &mut McpServer, client: ZoektClient) -> Result<(), ServerError> {
    let client_search = client.clone();
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "zoekt_search".to_string(),
                title: Some("Zoekt code search".to_string()),
            },
            icons: Icons::default(),
            description: Some(
                "Search code using Zoekt. Uses Zoekt query syntax (see query_syntax). \
                 Optional: repo_ids (array of repo IDs), num_context_lines, max_doc_display_count."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "q": {
                        "type": "string",
                        "description": "Zoekt query string (required)"
                    },
                    "repo_ids": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Limit search to these repository IDs"
                    },
                    "num_context_lines": {
                        "type": "integer",
                        "description": "Context lines before/after each match (0-10)"
                    },
                    "max_doc_display_count": {
                        "type": "integer",
                        "description": "Max number of files to return"
                    }
                },
                "required": ["q"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        move |args: Option<Value>, _ctx: RequestContext| {
            let client = client_search.clone();
            Box::pin(async move {
                let args = args.as_ref().and_then(|a| a.as_object());
                let q = args
                    .and_then(|a| a.get("q"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ServerError::Handler("missing q".to_string()))?;

                let repo_ids: Option<Vec<u32>> = args
                    .and_then(|a| a.get("repo_ids"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_u64().map(|u| u as u32))
                            .collect()
                    })
                    .filter(|v: &Vec<u32>| !v.is_empty());

                let mut opts = json!({});
                if let Some(n) = args.and_then(|a| a.get("num_context_lines")).and_then(|v| v.as_i64()) {
                    opts["NumContextLines"] = json!(n);
                }
                if let Some(n) = args
                    .and_then(|a| a.get("max_doc_display_count"))
                    .and_then(|v| v.as_i64())
                {
                    opts["MaxDocDisplayCount"] = json!(n);
                }
                let opts = if opts.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                    None
                } else {
                    Some(opts)
                };

                let result = client.search(q, repo_ids, opts).await;
                tool_result_from_zoekt(result)
            })
        },
    )?;

    let client_list = client.clone();
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "zoekt_list".to_string(),
                title: Some("Zoekt list repositories".to_string()),
            },
            icons: Icons::default(),
            description: Some(
                "List repositories from Zoekt. Use query q to filter (e.g. empty or repo:name).".to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "q": {
                        "type": "string",
                        "description": "Zoekt repo query (e.g. empty for all, or repo:pattern)"
                    }
                },
                "required": ["q"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        move |args: Option<Value>, _ctx: RequestContext| {
            let client = client_list.clone();
            Box::pin(async move {
                let args = args.as_ref().and_then(|a| a.as_object());
                let q = args
                    .and_then(|a| a.get("q"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let result = client.list(q, None).await;
                tool_result_from_zoekt(result)
            })
        },
    )?;

    Ok(())
}

fn tool_result_from_zoekt(
    r: Result<Value, ZoektClientError>,
) -> Result<CallToolResult, ServerError> {
    match r {
        Ok(v) => {
            let text = serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".to_string());
            Ok(CallToolResult {
                content: vec![ContentBlock::Text(TextContent::new(text))],
                structured_content: None,
                is_error: None,
                meta: None,
            })
        }
        Err(e) => Err(ServerError::Handler(e.to_string())),
    }
}
