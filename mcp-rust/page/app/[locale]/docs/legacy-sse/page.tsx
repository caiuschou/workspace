import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const serverDeps = `[dependencies]
mcp_server = { path = "server", features = ["axum"] }
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"`;

const legacyOnlyExample = `use std::sync::Arc;
use mcp_server::{
    McpServer, ServerOptions, LegacySseConfig, LegacySseState, create_legacy_sse_router,
};

let mcp_server = Arc::new(McpServer::new(server_info, server_options));

// Configure legacy SSE
let config = LegacySseConfig {
    endpoint_path: "/sse".to_string(),
    message_path: "/message".to_string(),
};

let state = Arc::new(LegacySseState::new(mcp_server, config));
let app = create_legacy_sse_router(state);

// Start server
let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
axum::serve(listener, app).await?;`;

const compatExample = `use std::sync::Arc;
use mcp_server::{
    McpServer, ServerOptions,
    AxumHandlerConfig, AxumHandlerState, create_router,
    LegacySseConfig, LegacySseState, create_legacy_sse_router,
};

let mcp_server = Arc::new(McpServer::new(server_info, server_options));

// Create Streamable HTTP router
let streamable_config = AxumHandlerConfig {
    endpoint_path: "/mcp".to_string(),
    ..Default::default()
};
let streamable_state = Arc::new(AxumHandlerState::new(Arc::clone(&mcp_server), streamable_config));
let streamable_router = create_router(streamable_state);

// Create legacy SSE router
let legacy_config = LegacySseConfig {
    endpoint_path: "/sse".to_string(),
    message_path: "/message".to_string(),
};
let legacy_state = Arc::new(LegacySseState::new(Arc::clone(&mcp_server), legacy_config));
let legacy_router = create_legacy_sse_router(legacy_state);

// Merge routers
let app = streamable_router.merge(legacy_router);

// Start server
let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
axum::serve(listener, app).await?;`;

const clientExample = `use mcp_client::http::{LegacySseClientConfig, LegacySseClientTransport};

// Create configuration
let config = LegacySseClientConfig::new("http://localhost:8080")
    .sse_path("/sse");

// Create transport
let mut transport = LegacySseClientTransport::new(config);

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

const testCommands = `# Run compat server example
cargo run -p mcp-sse-compat-server

# Test legacy SSE connection
curl -N http://localhost:8080/sse -H "Accept: text/event-stream"

# Output will be like:
# event: endpoint
# data: /message?sessionId=abc123`;

const comparison = [
  {
    feature: "Session ID",
    legacy: "URL query param ?sessionId=xxx",
    streamable: "Mcp-Session-Id header",
  },
  {
    feature: "Endpoint Event",
    legacy: "Sends endpoint event with POST address",
    streamable: "Not needed",
  },
  {
    feature: "POST Response",
    legacy: "Returns 202 Accepted (no content)",
    streamable: "Returns JSON or SSE stream",
  },
  {
    feature: "Reconnection",
    legacy: "No Last-Event-ID support",
    streamable: "Last-Event-ID replay",
  },
];

const endpoints = [
  {
    method: "GET",
    path: "/sse",
    description: "Establish SSE connection, receive endpoint event",
  },
  {
    method: "POST",
    path: "/message?sessionId=xxx",
    description: "Send JSON-RPC message",
  },
];

export default async function LegacySsePage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("legacySse");
  const common = await getTranslations("common");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Deprecation Notice */}
      <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-4 mb-8">
        <p className="text-yellow-400">
          <strong>Note:</strong> {t("deprecationNotice")}
        </p>
      </div>

      {/* Comparison */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">
          {t("comparisonWithStreamable")}
        </h2>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("feature")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  Legacy SSE (2024-11-05)
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  Streamable HTTP (2025-03-26)
                </th>
              </tr>
            </thead>
            <tbody>
              {comparison.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3 text-gray-400">{item.feature}</td>
                  <td className="px-4 py-3 text-gray-400">{item.legacy}</td>
                  <td className="px-4 py-3 text-gray-400">{item.streamable}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Protocol Flow */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("protocolFlow")}</h2>
        <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-300 overflow-x-auto">
          <pre>{`Client                              Server
  │                                   │
  │  GET /sse                         │
  │  Accept: text/event-stream        │
  │──────────────────────────────────▶│
  │                                   │
  │  event: endpoint                  │
  │  data: /message?sessionId=xxx     │
  │◀──────────────────────────────────│
  │                                   │
  │  POST /message?sessionId=xxx      │
  │  (JSON-RPC Request)               │
  │──────────────────────────────────▶│
  │                                   │
  │  202 Accepted                     │
  │◀──────────────────────────────────│
  │                                   │
  │  event: message                   │
  │  data: (JSON-RPC Response)        │
  │◀──────────────────────────────────│`}</pre>
        </div>
      </section>

      {/* Server Usage */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{common("serverUsage")}</h2>

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {common("dependencies")}
        </h3>
        <CodeBlock code={serverDeps} language="toml" title="Cargo.toml" />

        <h3 className="text-xl font-semibold mt-6 mb-4">{t("legacySseOnly")}</h3>
        <CodeBlock code={legacyOnlyExample} language="rust" />

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {t("supportBothProtocols")}
        </h3>
        <p className="text-gray-400 mb-4">{t("supportBothProtocolsDesc")}</p>
        <CodeBlock code={compatExample} language="rust" />
      </section>

      {/* Client Usage */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{common("clientUsage")}</h2>
        <CodeBlock code={clientExample} language="rust" />
      </section>

      {/* Endpoints */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("endpoints")}</h2>
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
      </section>

      {/* Testing */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{common("testing")}</h2>
        <CodeBlock code={testCommands} language="bash" title="Terminal" />
      </section>

      {/* Compat Server Example */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">
          {t("compatServerExample")}
        </h2>
        <p className="text-gray-400 mb-4">{t("compatServerDesc")}</p>
        <CodeBlock code="cargo run -p mcp-sse-compat-server" language="bash" />
        <p className="text-gray-400 mt-4">{t("availableEndpoints")}</p>
        <ul className="list-disc list-inside text-gray-400 space-y-2 mt-2">
          <li>
            <code className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-300">
              POST/GET/DELETE /mcp
            </code>{" "}
            - Streamable HTTP
          </li>
          <li>
            <code className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-300">
              GET /sse
            </code>{" "}
            +{" "}
            <code className="px-1.5 py-0.5 rounded bg-gray-800 text-gray-300">
              POST /message
            </code>{" "}
            - Legacy SSE
          </li>
        </ul>
      </section>
    </div>
  );
}
