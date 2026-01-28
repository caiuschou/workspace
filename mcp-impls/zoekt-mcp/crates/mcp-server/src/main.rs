//! Zoekt MCP server: stdio, SSE (Streamable HTTP), and WebSocket transports.
//!
//! - `zoekt-mcp` or `zoekt-mcp --stdio`: stdio transport (for subprocess/pipe)
//! - `zoekt-mcp --http`: HTTP server with Streamable HTTP/SSE and WebSocket
//!
//! Zoekt base URL from env `ZOEKT_BASE_URL` (default `http://127.0.0.1:6070`).

mod client;
mod config;
mod tools;

use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;

use clap::Parser;
use futures::executor::block_on;
use mcp_core::stdio::{serialize_message, JsonRpcMessage, ReadBuffer};
use mcp_core::types::{
    BaseMetadata, Icons, Implementation, ServerCapabilities, ToolCapabilities,
};
use mcp_server::{
    AxumHandlerConfig, AxumHandlerState, McpServer, ServerOptions, WebSocketConfig, WebSocketState,
    create_router, create_websocket_router,
};

use client::ZoektClient;
use tools::register_tools;

#[derive(Parser)]
#[command(name = "zoekt-mcp")]
#[command(about = "Zoekt MCP server (stdio, SSE, WebSocket)")]
struct Args {
    /// Use stdio transport (default when --http is not passed)
    #[arg(long)]
    stdio: bool,

    /// Use HTTP transport: Streamable HTTP/SSE at /mcp and WebSocket at /ws. Optional bind address.
    #[arg(long, num_args = 0..=1, default_missing_value = "0.0.0.0:8080")]
    http: Option<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("zoekt-mcp error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let use_stdio = args.http.is_none() || args.stdio;
    let http_bind = args.http.as_deref();

    let (server_info, server_options) = server_meta();
    let client = ZoektClient::from_env();
    let mut server = McpServer::new(server_info.clone(), server_options);
    register_tools(&mut server, client)?;

    if use_stdio {
        run_stdio(server)?;
    } else if let Some(addr) = http_bind {
        run_http(server, addr)?;
    }

    Ok(())
}

fn server_meta() -> (Implementation, ServerOptions) {
    let info = Implementation {
        base: BaseMetadata {
            name: "zoekt-mcp".to_string(),
            title: Some("Zoekt MCP Server".to_string()),
        },
        icons: Icons::default(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        website_url: Some("https://github.com/sourcegraph/zoekt".to_string()),
        description: Some(
            "MCP server for Zoekt code search. Tools: zoekt_search, zoekt_list. \
             Set ZOEKT_BASE_URL to point at zoekt-webserver (default http://127.0.0.1:6070)."
                .to_string(),
        ),
    };
    let mut opts = ServerOptions::default();
    opts.capabilities = Some(ServerCapabilities {
        tools: Some(ToolCapabilities { list_changed: None }),
        ..Default::default()
    });
    opts.instructions = Some(
        "Zoekt MCP: use zoekt_search to search code, zoekt_list to list repos. \
         Query syntax: see Zoekt query_syntax. ZOEKT_BASE_URL defaults to http://127.0.0.1:6070."
            .to_string(),
    );
    (info, opts)
}

fn run_stdio(server: McpServer) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut buffer = String::new();
    let mut read_buffer = ReadBuffer::default();

    loop {
        buffer.clear();
        let n = reader.read_line(&mut buffer)?;
        if n == 0 {
            break;
        }
        read_buffer.append(buffer.as_bytes());
        while let Ok(Some(msg)) = read_buffer.read_message() {
            match msg {
                JsonRpcMessage::Request(req) => {
                    let resp = block_on(server.server().handle_request(req, None))?;
                    let out = serialize_message(&JsonRpcMessage::Result(resp))?;
                    stdout.write_all(out.as_bytes())?;
                    stdout.flush()?;
                }
                JsonRpcMessage::Notification(notif) => {
                    block_on(server.server().handle_notification(notif, None))?;
                }
                JsonRpcMessage::Result(_) => {}
            }
        }
    }
    Ok(())
}

fn run_http(server: McpServer, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(server);

    let sse_config = AxumHandlerConfig {
        endpoint_path: "/mcp".to_string(),
        enable_cors: true,
        ..Default::default()
    };
    let sse_state = Arc::new(AxumHandlerState::new(Arc::clone(&server), sse_config));
    let sse_router = create_router(sse_state);

    let ws_config = WebSocketConfig {
        endpoint_path: "/ws".to_string(),
        enable_cors: true,
        channel_buffer_size: 100,
    };
    let ws_state = Arc::new(WebSocketState::new(Arc::clone(&server), ws_config));
    let ws_router = create_websocket_router(ws_state);

    let app = sse_router.merge(ws_router);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        eprintln!("zoekt-mcp HTTP listening on http://{}", addr);
        eprintln!("  Streamable HTTP/SSE: POST/GET/DELETE http://{}/mcp", addr);
        eprintln!("  WebSocket:           ws://{}/ws", addr);
        eprintln!("  ZOEKT_BASE_URL: {}", config::zoekt_base_url());
        axum::serve(listener, app).await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })?;
    Ok(())
}
