import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const workspaceStructure = `mcp-rust/
├── core/                 # Core library (mcp_core)
├── server/               # Server library (mcp_server)
├── client/               # Client library (mcp_client)
├── examples/             # Example code
│   ├── http-server/      # HTTP server example
│   ├── http-client/      # HTTP client example
│   ├── websocket-server/ # WebSocket server example
│   ├── sse-compat-server/# Compat server (new + legacy)
│   ├── mcp-filesystem-server/
│   └── mcp-filesystem-client/
└── docs/                 # Documentation`;

const coreStructure = `core/src/
├── http/                 # HTTP transport types
│   ├── mod.rs           # Module entry
│   ├── session.rs       # SessionId, ResumptionToken
│   ├── sse.rs           # SseEvent, SseParser
│   ├── transport.rs     # ConnectionState, AsyncTransport trait
│   └── error.rs         # HTTP transport errors
├── protocol/            # MCP protocol
├── stdio/               # Stdio transport
├── types/               # Type definitions
└── lib.rs`;

const serverStructure = `server/src/
├── http/                 # HTTP transport
│   ├── mod.rs           # Module entry
│   ├── handler.rs       # HttpServerHandler (framework-agnostic)
│   ├── axum_handler.rs  # axum integration (feature = "axum")
│   ├── legacy_sse.rs    # Legacy SSE compat transport
│   ├── broadcast.rs     # SseBroadcaster, EventBuffer
│   ├── session_manager.rs  # SessionManager
│   ├── sse_writer.rs    # SseWriter, SseResponseBuilder
│   └── error.rs         # HTTP server errors
├── websocket/           # WebSocket transport
│   ├── mod.rs           # Module entry
│   └── axum_handler.rs  # axum WebSocket integration
├── server/              # MCP server core
│   ├── mcp_server.rs    # McpServer main struct
│   ├── server.rs        # Server trait
│   ├── handlers/        # Request handlers
│   └── registries/      # Tool/Resource/Prompt registries
└── lib.rs`;

const clientStructure = `client/src/
├── http/                 # HTTP transport
│   ├── mod.rs           # Module entry
│   ├── transport.rs     # HttpClientTransport
│   ├── config.rs        # HttpClientConfig
│   ├── legacy_sse.rs    # Legacy SSE client transport
│   ├── reconnect.rs     # ReconnectOptions, ReconnectState
│   ├── sse_reader.rs    # SseReader
│   └── error.rs         # HTTP client errors
├── websocket/           # WebSocket transport
│   ├── mod.rs           # Module entry
│   ├── transport.rs     # WebSocketClientTransport
│   └── error.rs         # WebSocket client errors
├── stdio/               # Stdio transport
│   ├── mod.rs
│   ├── transport.rs     # StdioClientTransport
│   └── params.rs        # StdioServerParameters
├── client/              # MCP client core
│   ├── client.rs        # Client main struct
│   └── ...
└── lib.rs`;

const coreTypes = [
  { type: "SessionId", description: "Session identifier" },
  { type: "ResumptionToken", description: "Reconnection token" },
  {
    type: "SseEvent",
    description: "SSE event (Message, Ping, SessionReady, Endpoint)",
  },
  { type: "SseParser", description: "SSE stream incremental parser" },
  { type: "ConnectionState", description: "Connection state machine" },
  { type: "JsonRpcMessage", description: "JSON-RPC message" },
];

const serverTypes = [
  { type: "McpServer", description: "MCP server main struct" },
  { type: "HttpServerHandler", description: "Framework-agnostic HTTP handler" },
  { type: "AxumHandlerState", description: "axum integration state" },
  { type: "SessionManager", description: "Session manager" },
  { type: "SseBroadcaster", description: "SSE message broadcaster" },
  {
    type: "EventBuffer",
    description: "Event buffer (Last-Event-ID replay)",
  },
  { type: "WebSocketState", description: "WebSocket connection management" },
  { type: "LegacySseState", description: "Legacy SSE state management" },
];

const clientTypes = [
  { type: "Client", description: "MCP client main struct" },
  { type: "HttpClientTransport", description: "HTTP transport layer" },
  { type: "HttpClientConfig", description: "HTTP configuration" },
  { type: "ReconnectOptions", description: "Reconnection strategy config" },
  { type: "SseReader", description: "SSE stream reader" },
  {
    type: "WebSocketClientTransport",
    description: "WebSocket transport layer",
  },
  { type: "LegacySseClientTransport", description: "Legacy SSE transport layer" },
  { type: "StdioClientTransport", description: "Stdio transport layer" },
];

const featureFlags = [
  { feature: "axum", description: "Enable axum framework integration (HTTP/SSE)" },
  { feature: "websocket", description: "Enable WebSocket support (includes axum)" },
  { feature: "tokio", description: "Enable tokio runtime support" },
];

export default async function ArchitecturePage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("architecture");
  const common = await getTranslations("common");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      {/* Workspace Structure */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("workspaceStructure")}</h2>
        <CodeBlock code={workspaceStructure} language="text" />
      </section>

      {/* Core Library */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("coreLibrary")}</h2>
        <p className="text-gray-400 mb-4">{t("coreLibraryDesc")}</p>
        <CodeBlock code={coreStructure} language="text" />

        <h3 className="text-xl font-semibold mt-6 mb-4">{t("keyTypes")}</h3>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("type")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {coreTypes.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3">
                    <code className="text-blue-400">{item.type}</code>
                  </td>
                  <td className="px-4 py-3 text-gray-400">{item.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Server Library */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("serverLibrary")}</h2>
        <p className="text-gray-400 mb-4">{t("serverLibraryDesc")}</p>
        <CodeBlock code={serverStructure} language="text" />

        <h3 className="text-xl font-semibold mt-6 mb-4">{t("keyTypes")}</h3>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("type")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {serverTypes.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3">
                    <code className="text-blue-400">{item.type}</code>
                  </td>
                  <td className="px-4 py-3 text-gray-400">{item.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <h3 className="text-xl font-semibold mt-6 mb-4">{t("featureFlags")}</h3>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("feature")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {featureFlags.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3">
                    <code className="text-green-400">{item.feature}</code>
                  </td>
                  <td className="px-4 py-3 text-gray-400">{item.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Client Library */}
      <section className="mb-12">
        <h2 className="text-2xl font-semibold mb-4">{t("clientLibrary")}</h2>
        <p className="text-gray-400 mb-4">{t("clientLibraryDesc")}</p>
        <CodeBlock code={clientStructure} language="text" />

        <h3 className="text-xl font-semibold mt-6 mb-4">{t("keyTypes")}</h3>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("type")}
                </th>
                <th className="px-4 py-3 text-left font-medium text-gray-300">
                  {common("description")}
                </th>
              </tr>
            </thead>
            <tbody>
              {clientTypes.map((item, index) => (
                <tr key={index} className="border-b border-gray-800">
                  <td className="px-4 py-3">
                    <code className="text-blue-400">{item.type}</code>
                  </td>
                  <td className="px-4 py-3 text-gray-400">{item.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Data Flow */}
      <section>
        <h2 className="text-2xl font-semibold mb-4">{t("dataFlow")}</h2>

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {t("requestResponseFlow")}
        </h3>
        <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-300 overflow-x-auto">
          <pre>{`Client                          Server
  │                               │
  │  POST /mcp (JSON-RPC Request) │
  │──────────────────────────────▶│
  │                               │
  │                               ├──▶ handle_request()
  │                               │
  │  JSON-RPC Response            │
  │◀──────────────────────────────│`}</pre>
        </div>

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {t("sseStreamingFlow")}
        </h3>
        <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-300 overflow-x-auto">
          <pre>{`Client                          Server
  │                               │
  │  GET /mcp (Accept: SSE)       │
  │──────────────────────────────▶│
  │                               │
  │  event: session               │
  │◀──────────────────────────────│
  │                               │
  │  event: message (push)        │
  │◀──────────────────────────────│
  │                               │
  │  :ping                        │
  │◀──────────────────────────────│`}</pre>
        </div>

        <h3 className="text-xl font-semibold mt-6 mb-4">
          {t("reconnectionFlow")}
        </h3>
        <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-300 overflow-x-auto">
          <pre>{`Client                          Server
  │                               │
  │  Connection lost              │
  │  ✕ ✕ ✕ ✕ ✕ ✕ ✕ ✕ ✕ ✕ ✕ ✕ ✕  │
  │                               │
  │  GET /mcp                     │
  │  Last-Event-ID: xxx-42        │
  │──────────────────────────────▶│
  │                               │
  │                               ├──▶ EventBuffer.events_after("xxx-42")
  │                               │
  │  Replay missed events         │
  │◀──────────────────────────────│
  │                               │
  │  Continue streaming           │
  │◀──────────────────────────────│`}</pre>
        </div>
      </section>
    </div>
  );
}
