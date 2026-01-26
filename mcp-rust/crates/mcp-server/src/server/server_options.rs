use mcp_core::protocol::ProtocolOptions;
use mcp_core::types::ServerCapabilities;

/// Configuration options for an MCP server.
#[derive(Clone, Default)]
pub struct ServerOptions {
    pub capabilities: Option<ServerCapabilities>,
    pub instructions: Option<String>,
    pub protocol_options: Option<ProtocolOptions>,
}
