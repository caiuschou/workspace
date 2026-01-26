import { getTranslations, setRequestLocale } from "next-intl/server";
import { CodeBlock } from "../../../components";

const mainCode = `use std::sync::Arc;
use mcp_server::{McpServer, ServerOptions, WebSocketConfig, WebSocketState, create_websocket_router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mcp_server = Arc::new(McpServer::new(server_info, server_options));
    
    let config = WebSocketConfig {
        endpoint_path: "/ws".to_string(),
        enable_cors: true,
        channel_buffer_size: 100,
    };
    
    let state = Arc::new(WebSocketState::new(mcp_server, config));
    let app = create_websocket_router(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("WebSocket server listening on ws://0.0.0.0:8080/ws");
    axum::serve(listener, app).await?;
    
    Ok(())
}`;

export default async function WebSocketServerPage({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;
  setRequestLocale(locale);
  const t = await getTranslations("examplePages.websocketServer");
  const common = await getTranslations("common");

  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">{t("title")}</h1>
      <p className="text-gray-400 text-lg mb-8">{t("subtitle")}</p>

      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{common("basicExample")}</h2>
        <CodeBlock code={mainCode} language="rust" title="src/main.rs" />
      </section>

      <section className="mb-8">
        <h2 className="text-2xl font-semibold mb-4">{common("running")}</h2>
        <CodeBlock code="cargo run -p mcp-websocket-server" language="bash" title="Terminal" />
      </section>

      <section>
        <h2 className="text-2xl font-semibold mb-4">{common("sourceCode")}</h2>
        <p className="text-gray-400">
          {common("viewSource")}{" "}
          <a
            href="https://github.com/anthropics/mcp-rust/tree/main/examples/websocket-server"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            examples/websocket-server
          </a>
        </p>
      </section>
    </div>
  );
}
