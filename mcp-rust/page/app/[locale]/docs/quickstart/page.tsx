import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const installCode = `[dependencies]
# Core types and protocol
mcp_core = { path = "core" }

# Server with HTTP/SSE (axum integration)
mcp_server = { path = "server", features = ["axum"] }

# Client library
mcp_client = { path = "client" }`;

const serverExample = `use mcp_server::server::McpServer;
use mcp_server::http::create_router;

#[tokio::main]
async fn main() {
    // Create MCP server
    let server = McpServer::new("my-server", "1.0.0");
    
    // Register a tool
    server.register_tool(
        "greet",
        "Greet a user",
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        }),
        |params| async move {
            let name = params["name"].as_str().unwrap_or("World");
            Ok(serde_json::json!({
                "greeting": format!("Hello, {}!", name)
            }))
        },
    );

    // Create HTTP router
    let app = create_router(server);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}`;

const clientExample = `use mcp_client::Client;
use mcp_client::http::{HttpClientTransport, HttpClientConfig};

#[tokio::main]
async fn main() {
    // Create HTTP transport
    let config = HttpClientConfig::new("http://localhost:3000/mcp");
    let transport = HttpClientTransport::new(config);

    // Create client and connect
    let client = Client::new(transport);
    client.connect().await.unwrap();

    // Initialize
    let init_result = client.initialize("my-client", "1.0.0").await.unwrap();
    println!("Connected to: {}", init_result.server_info.name);

    // List available tools
    let tools = client.list_tools().await.unwrap();
    for tool in tools.tools {
        println!("Tool: {} - {}", tool.name, tool.description.unwrap_or_default());
    }

    // Call a tool
    let result = client.call_tool("greet", serde_json::json!({
        "name": "MCP"
    })).await.unwrap();
    
    println!("Result: {:?}", result);
}`;

const buildCommands = `# Build all packages
cargo build

# Run HTTP server example
cargo run -p mcp-http-server

# Run HTTP client example
cargo run -p mcp-http-client

# Run WebSocket server
cargo run -p mcp-websocket-server

# Run all tests
cargo test --workspace

# Run HTTP/SSE integration tests
cargo test -p mcp_server --features axum --test http_sse`;

export default async function QuickStartPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("quickstart");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Prerequisites */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("prerequisites")}</h2>
        <ul className="list-disc list-inside text-gray-400 space-y-2">
          <li>{t("prerequisitesList.rust")}</li>
          <li>{t("prerequisitesList.cargo")}</li>
          <li>{t("prerequisitesList.asyncBasics")}</li>
        </ul>
      </section>

      {/* Installation */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("installation")}</h2>
        <p className="text-gray-400 mb-4">
          {t("installationDesc")}{" "}
          <code className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-300">
            Cargo.toml
          </code>
          :
        </p>
        <CodeBlock code={installCode} language="toml" title="Cargo.toml" />
      </section>

      {/* Server Example */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("createServer")}</h2>
        <p className="text-gray-400 mb-4">{t("createServerDesc")}</p>
        <CodeBlock code={serverExample} language="rust" title="src/main.rs" />
      </section>

      {/* Client Example */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("createClient")}</h2>
        <p className="text-gray-400 mb-4">{t("createClientDesc")}</p>
        <CodeBlock code={clientExample} language="rust" title="src/main.rs" />
      </section>

      {/* Build and Run */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("buildAndRun")}</h2>
        <p className="text-gray-400 mb-4">{t("buildAndRunDesc")}</p>
        <CodeBlock code={buildCommands} language="bash" title="Terminal" />
      </section>

      {/* Next Steps */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">{t("nextSteps")}</h2>
        <ul className="space-y-3">
          <li>
            <a
              href={`/${locale}/docs/architecture`}
              className="text-blue-400 hover:text-blue-300"
            >
              {t("nextStepsLinks.architecture")}
            </a>
            <span className="text-gray-500 ml-2">
              {t("nextStepsLinks.architectureDesc")}
            </span>
          </li>
          <li>
            <a
              href={`/${locale}/docs/http-sse`}
              className="text-blue-400 hover:text-blue-300"
            >
              {t("nextStepsLinks.httpSse")}
            </a>
            <span className="text-gray-500 ml-2">
              {t("nextStepsLinks.httpSseDesc")}
            </span>
          </li>
          <li>
            <a
              href={`/${locale}/docs/auth`}
              className="text-blue-400 hover:text-blue-300"
            >
              {t("nextStepsLinks.auth")}
            </a>
            <span className="text-gray-500 ml-2">
              {t("nextStepsLinks.authDesc")}
            </span>
          </li>
        </ul>
      </section>
    </div>
  );
}
