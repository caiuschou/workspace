import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const serverDeps = `[dependencies]
mcp_server = { path = "server", features = ["axum"] }
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"`;

const serverExample = `use std::sync::Arc;
use mcp_core::types::{BaseMetadata, Icons, Implementation, ServerCapabilities};
use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, McpServer, ServerOptions, create_router,
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

    // Configure HTTP handler
    let config = AxumHandlerConfig {
        base_url: Some("http://localhost:8080".to_string()),
        endpoint_path: "/mcp".to_string(),
        keep_alive_interval: std::time::Duration::from_secs(30),
        broadcast_capacity: 100,
        enable_cors: true,
        ..Default::default()
    };

    // Create router
    let state = Arc::new(AxumHandlerState::new(mcp_server, config));
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await?;

    Ok(())
}`;

const configOptions = `pub struct AxumHandlerConfig {
    /// Session configuration
    pub session_config: SessionConfig,
    /// Event buffer config (for Last-Event-ID replay)
    pub event_buffer_config: EventBufferConfig,
    /// Server base URL
    pub base_url: Option<String>,
    /// Endpoint path (default: "/mcp")
    pub endpoint_path: String,
    /// SSE keep-alive interval
    pub keep_alive_interval: Duration,
    /// Broadcast channel capacity per session
    pub broadcast_capacity: usize,
    /// Enable CORS
    pub enable_cors: bool,
}`;

const serverPush = `// Get handler state
let state: Arc<AxumHandlerState> = /* ... */;

// Push message to specific session
let message = JsonRpcMessage::Notification(/* ... */);
state.broadcast_to_session("session-id", message).await?;`;

const clientExample = `use mcp_client::http::{HttpClientConfig, HttpClientTransport};

// Create configuration
let config = HttpClientConfig::new("http://localhost:8080/mcp")
    .auto_reconnect(true)
    .custom_header("Authorization", "Bearer token");

// Create transport
let mut transport = HttpClientTransport::new(config);

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
transport.start()?;

// Send message
transport.send(&JsonRpcMessage::Request(/* ... */))?;

// Close connection
transport.close()?;`;

const reconnectOptions = `pub struct ReconnectOptions {
    /// Initial reconnection delay
    pub initial_delay: Duration,
    /// Maximum reconnection delay
    pub max_delay: Duration,
    /// Delay multiplier
    pub backoff_factor: f64,
    /// Max retry attempts (None = infinite)
    pub max_attempts: Option<u32>,
}`;

const testCommands = `# Run integration tests
cargo test -p mcp_server --features axum --test http_sse

# Start example server
cargo run -p mcp-http-server

# Test POST request
curl -X POST http://localhost:8080/mcp \\
     -H "Content-Type: application/json" \\
     -d '{"jsonrpc":"2.0","id":1,"method":"initialize",...}'

# Test SSE connection
curl -N http://localhost:8080/mcp -H "Accept: text/event-stream"`;

const endpoints = [
  { method: "POST", path: "/mcp", description: "Send JSON-RPC message" },
  { method: "GET", path: "/mcp", description: "Establish SSE connection" },
  { method: "DELETE", path: "/mcp", description: "Close session" },
];

const headers = [
  {
    header: "Content-Type",
    description: "Must be application/json for POST requests",
  },
  {
    header: "Accept",
    description: "Must include text/event-stream for GET requests",
  },
  {
    header: "Mcp-Session-Id",
    description: "Session ID (optional, server returns in response)",
  },
  { header: "Last-Event-ID", description: "For reconnection replay (optional)" },
];

export default async function HttpSsePage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("httpSse");
  const common = await getTranslations("common");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Features */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{common("features")}</h2>
        <ul className="list-disc list-inside text-gray-400 space-y-2">
          <li>{t("features.trueSse")}</li>
          <li>{t("features.bidirectional")}</li>
          <li>{t("features.lastEventId")}</li>
          <li>{t("features.cors")}</li>
          <li>{t("features.axum")}</li>
        </ul>
      </section>

      {/* Architecture */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("architecture")}</h2>
        <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-300 overflow-x-auto">
          <pre>{`┌─────────────────┐                    ┌─────────────────┐
│   HTTP Client   │                    │   HTTP Server   │
│                 │                    │                 │
│ ┌─────────────┐ │   POST /mcp        │ ┌─────────────┐ │
│ │  Transport  │─┼───────────────────▶│ │   Handler   │ │
│ └─────────────┘ │                    │ └─────────────┘ │
│                 │   GET /mcp (SSE)   │        │        │
│ ┌─────────────┐ │◀──────────────────┼│ ┌─────────────┐ │
│ │  SseReader  │ │                    │ │ Broadcaster │ │
│ └─────────────┘ │                    │ └─────────────┘ │
│                 │                    │        │        │
│ ┌─────────────┐ │                    │ ┌─────────────┐ │
│ │  Reconnect  │ │   Last-Event-ID    │ │EventBuffer  │ │
│ └─────────────┘ │───────────────────▶│ └─────────────┘ │
└─────────────────┘                    └─────────────────┘`}</pre>
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
        <CodeBlock code={clientExample} language="rust" title="src/main.rs" />
      </section>

      {/* API Endpoints */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{common("apiEndpoints")}</h2>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("method")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("path")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {endpoints.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3">
                    <code className="text-green-400">{item.method}</code>
                  </td>
                  <td className="px-4 py-3">
                    <code className="text-blue-400">{item.path}</code>
                  </td>
                  <td className="px-4 py-3 text-gray-400">{item.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {t("requestHeaders")}
        </h3>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("header")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {headers.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3">
                    <code className="text-blue-400">{item.header}</code>
                  </td>
                  <td className="px-4 py-3 text-gray-400">{item.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Reconnection */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("reconnection")}</h2>
        <p className="text-gray-400 mb-4">{t("reconnectionDesc")}</p>
        <CodeBlock code={reconnectOptions} language="rust" />
      </section>

      {/* Testing */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("testing")}</h2>
        <CodeBlock code={testCommands} language="bash" title="Terminal" />
      </section>
    </div>
  );
}
