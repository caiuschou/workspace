import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `use std::sync::Arc;
use std::time::Duration;

use mcp_core::protocol::RequestContext;
use mcp_core::types::{
    BaseMetadata, CallToolResult, ContentBlock, Icons, Implementation,
    ServerCapabilities, TextContent, Tool,
};
use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, McpServer, ServerError, 
    ServerOptions, create_router,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Create server info
    let server_info = Implementation {
        base: BaseMetadata {
            name: "mcp-http-server-example".to_string(),
            title: Some("MCP HTTP Server Example".to_string()),
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: Some("Example MCP server with HTTP/SSE transport".to_string()),
    };

    // Configure server capabilities
    let mut server_options = ServerOptions::default();
    server_options.capabilities = Some(ServerCapabilities {
        tools: Some(mcp_core::types::ToolCapabilities {
            list_changed: Some(true),
        }),
        ..Default::default()
    });

    // Create MCP server
    let mut mcp_server = McpServer::new(server_info, server_options);

    // Register tools
    register_tools(&mut mcp_server)?;

    let mcp_server = Arc::new(mcp_server);

    // Configure HTTP handler
    let config = AxumHandlerConfig {
        base_url: Some("http://localhost:8080".to_string()),
        endpoint_path: "/mcp".to_string(),
        keep_alive_interval: Duration::from_secs(30),
        broadcast_capacity: 100,
        enable_cors: true,
        ..Default::default()
    };

    // Create handler state and router
    let state = Arc::new(AxumHandlerState::new(mcp_server, config));
    let app = create_router(state);

    // Start server
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("MCP HTTP Server listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}`;

const toolRegistrationCode = `fn register_tools(server: &mut McpServer) -> Result<(), Box<dyn std::error::Error>> {
    // Register an echo tool
    server.register_tool(
        Tool {
            base: BaseMetadata {
                name: "echo".to_string(),
                title: Some("Echo Tool".to_string()),
            },
            icons: Icons::default(),
            description: Some("Echoes back the input message".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo"
                    }
                },
                "required": ["message"]
            }),
            output_schema: None,
            annotations: None,
            execution: None,
            meta: None,
        },
        |params: Option<serde_json::Value>, _context: RequestContext| {
            Box::pin(async move {
                let message = params
                    .as_ref()
                    .and_then(|p| p.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no message)");

                Ok::<_, ServerError>(CallToolResult {
                    content: vec![ContentBlock::Text(TextContent::new(format!(
                        "Echo: {}",
                        message
                    )))],
                    structured_content: None,
                    is_error: None,
                    meta: None,
                })
            })
        },
    )?;

    Ok(())
}`;

const runCommand = `cargo run -p mcp-http-server`;

const testCommands = `# Initialize the connection
curl -X POST http://localhost:8080/mcp \\
     -H "Content-Type: application/json" \\
     -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}'

# Establish SSE connection for receiving messages
curl -N http://localhost:8080/mcp -H "Accept: text/event-stream"

# List available tools
curl -X POST http://localhost:8080/mcp \\
     -H "Content-Type: application/json" \\
     -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

# Call the echo tool
curl -X POST http://localhost:8080/mcp \\
     -H "Content-Type: application/json" \\
     -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello!"}}}'`;

export default async function HttpServerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.httpServer");
  const common = await getTranslations("common");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Features */}
      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{common("features")}</h2>
        <ul className="list-disc list-inside text-gray-400 space-y-2">
          <li>{t("features.trueSse")}</li>
          <li>{t("features.serverPush")}</li>
          <li>{t("features.lastEventId")}</li>
          <li>{t("features.cors")}</li>
          <li>{t("features.toolRegistration")}</li>
        </ul>
      </section>

      {/* Endpoints */}
      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{t("endpoints")}</h2>
        <div className="overflow-x-auto">
          <table className="w-full text-sm text-gray-400">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left py-2 pr-4">{common("method")}</th>
                <th className="text-left py-2 pr-4">{common("path")}</th>
                <th className="text-left py-2">{common("description")}</th>
              </tr>
            </thead>
            <tbody>
              <tr className="border-b border-gray-800">
                <td className="py-2 pr-4">
                  <code className="text-blue-400">POST</code>
                </td>
                <td className="py-2 pr-4">
                  <code>/mcp</code>
                </td>
                <td className="py-2">Send JSON-RPC messages</td>
              </tr>
              <tr className="border-b border-gray-800">
                <td className="py-2 pr-4">
                  <code className="text-green-400">GET</code>
                </td>
                <td className="py-2 pr-4">
                  <code>/mcp</code>
                </td>
                <td className="py-2">Establish SSE connection</td>
              </tr>
              <tr>
                <td className="py-2 pr-4">
                  <code className="text-red-400">DELETE</code>
                </td>
                <td className="py-2 pr-4">
                  <code>/mcp</code>
                </td>
                <td className="py-2">Close session</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      {/* Main Code */}
      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{t("serverSetup")}</h2>
        <CodeBlock
          code={mainCode}
          language="rust"
          title="examples/http-server/src/main.rs"
        />
      </section>

      {/* Tool Registration */}
      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{t("toolRegistration")}</h2>
        <p className="text-gray-400 mb-4">{t("toolRegistrationDesc")}</p>
        <CodeBlock
          code={toolRegistrationCode}
          language="rust"
          title="Tool Registration"
        />
      </section>

      {/* Running */}
      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{common("running")}</h2>
        <CodeBlock code={runCommand} language="bash" title="Terminal" />
      </section>

      {/* Testing */}
      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{t("testingWithCurl")}</h2>
        <CodeBlock code={testCommands} language="bash" title="Terminal" />
      </section>

      {/* Source Link */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/http-server"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/http-server
          </a>
        </p>
      </section>
    </div>
  );
}
