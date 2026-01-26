import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const serverDeps = `[dependencies]
mcp_server = { path = "server", features = ["websocket"] }
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"`;

const serverExample = `use std::sync::Arc;
use mcp_core::types::{BaseMetadata, Icons, Implementation, ServerCapabilities};
use mcp_server::{
    McpServer, ServerOptions, WebSocketConfig, WebSocketState, create_websocket_router,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server info
    let server_info = Implementation {
        base: BaseMetadata {
            name: "my-mcp-server".to_string(),
            title: Some("My MCP Server".to_string()),
        },
        icons: Icons::default(),
        version: "0.1.0".to_string(),
        website_url: None,
        description: Some("My MCP server description".to_string()),
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
    let mcp_server = Arc::new(McpServer::new(server_info, server_options));

    // Configure WebSocket handler
    let config = WebSocketConfig {
        endpoint_path: "/ws".to_string(),
        enable_cors: true,
        channel_buffer_size: 100,
    };

    // Create router
    let state = Arc::new(WebSocketState::new(mcp_server, config));
    let app = create_websocket_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on ws://0.0.0.0:8080/ws");
    axum::serve(listener, app).await?;

    Ok(())
}`;

const configOptions = `pub struct WebSocketConfig {
    /// Endpoint path (default: "/ws")
    pub endpoint_path: String,
    /// Enable CORS
    pub enable_cors: bool,
    /// Message channel buffer size per connection
    pub channel_buffer_size: usize,
}`;

const serverPush = `// Get handler state
let state: Arc<WebSocketState> = /* ... */;

// Push message to specific connection
let message = JsonRpcMessage::Notification(/* ... */);
state.send_to_connection("connection-id", message).await?;

// Broadcast to all connections
state.broadcast(message).await;`;

const clientDeps = `[dependencies]
mcp_client = { path = "client", features = ["websocket"] }
tokio = { version = "1.0", features = ["full"] }`;

const clientExample = `use mcp_client::websocket::WebSocketClientTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create transport
    let mut transport = WebSocketClientTransport::new("ws://localhost:8080/ws");

    // Register event handlers
    transport
        .on_message(|msg| {
            println!("Received: {:?}", msg);
        })
        .on_error(|err| {
            eprintln!("Error: {:?}", err);
        })
        .on_close(|| {
            println!("Connection closed");
        });

    // Start connection
    transport.start().await?;

    // Send message
    let request = /* build JSON-RPC request */;
    transport.send(&request).await?;

    // Close connection
    transport.close().await?;

    Ok(())
}`;

const testCommands = `# Run unit tests
cargo test -p mcp_server --features websocket
cargo test -p mcp_client --features websocket

# Start example server
cargo run -p mcp-websocket-server

# Test with websocat
websocat ws://localhost:8080/ws -H "Sec-WebSocket-Protocol: mcp"`;

const exampleRequests = `// Initialize
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}

// List tools
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}

// Call tool
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello!"}}}`;

const comparison = [
  {
    feature: "Communication",
    websocket: "Full-duplex",
    httpsse: "Half-duplex (POST + SSE)",
  },
  { feature: "Connections", websocket: "1", httpsse: "2 (POST + SSE)" },
  {
    feature: "Session Management",
    websocket: "Connection = Session",
    httpsse: "Requires Session ID",
  },
  {
    feature: "Reconnection",
    websocket: "Manual implementation",
    httpsse: "Last-Event-ID replay",
  },
  {
    feature: "Browser Support",
    websocket: "Native",
    httpsse: "SSE has limitations",
  },
];

export default async function WebSocketPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("websocket");
  const common = await getTranslations("common");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Features */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{common("features")}</h2>
        <ul className="list-disc list-inside text-gray-400 space-y-2">
          <li>{t("features.fullDuplex")}</li>
          <li>
            {t("features.subprotocol")} (
            <code className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-300">
              Sec-WebSocket-Protocol: mcp
            </code>
            )
          </li>
          <li>{t("features.pingPong")}</li>
          <li>{t("features.cors")}</li>
          <li>{t("features.axum")}</li>
        </ul>
      </section>

      {/* Use Cases */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("useCases")}</h2>
        <ul className="list-disc list-inside text-gray-400 space-y-2">
          <li>{t("useCasesList.browser")}</li>
          <li>{t("useCasesList.realtime")}</li>
          <li>{t("useCasesList.simplified")}</li>
        </ul>
      </section>

      {/* Comparison with HTTP/SSE */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">
          {t("comparisonWithHttpSse")}
        </h2>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("feature")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  WebSocket
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  HTTP/SSE
                </th>
              </tr>
            </thead>
            <tbody>
              {comparison.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3 text-gray-400">{item.feature}</td>
                  <td className="px-4 py-3 text-gray-400">{item.websocket}</td>
                  <td className="px-4 py-3 text-gray-400">{item.httpsse}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Server Usage */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{common("serverUsage")}</h2>

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {common("dependencies")}
        </h3>
        <CodeBlock code={serverDeps} language="toml" title="Cargo.toml" />

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {common("basicExample")}
        </h3>
        <CodeBlock code={serverExample} language="rust" title="src/main.rs" />

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {common("configurationOptions")}
        </h3>
        <CodeBlock code={configOptions} language="rust" />

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {t("serverInitiatedPush")}
        </h3>
        <CodeBlock code={serverPush} language="rust" />
      </section>

      {/* Client Usage */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{common("clientUsage")}</h2>

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {common("dependencies")}
        </h3>
        <CodeBlock code={clientDeps} language="toml" title="Cargo.toml" />

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {common("basicExample")}
        </h3>
        <CodeBlock code={clientExample} language="rust" title="src/main.rs" />
      </section>

      {/* Protocol Details */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("protocolDetails")}</h2>

        <h3 className="text-xl font-semibold mt-6 mb-4">{t("subprotocol")}</h3>
        <p className="text-gray-400 mb-4">{t("subprotocolDesc")}</p>
        <CodeBlock code="Sec-WebSocket-Protocol: mcp" language="text" />

        <h3 className="text-xl font-semibold mt-6 mb-4">{t("messageFormat")}</h3>
        <p className="text-gray-400 mb-4">{t("messageFormatDesc")}</p>
      </section>

      {/* Example Requests */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("exampleRequests")}</h2>
        <CodeBlock code={exampleRequests} language="json" />
      </section>

      {/* Testing */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("testing")}</h2>
        <CodeBlock code={testCommands} language="bash" title="Terminal" />
      </section>
    </div>
  );
}
