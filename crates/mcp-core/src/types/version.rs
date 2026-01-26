/// Latest supported MCP protocol version.
pub const LATEST_PROTOCOL_VERSION: &str = "2025-11-25";

/// Default negotiated protocol version used when both sides support it.
pub const DEFAULT_NEGOTIATED_PROTOCOL_VERSION: &str = "2025-03-26";

/// Ordered list of protocol versions supported by this SDK.
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &[
    LATEST_PROTOCOL_VERSION,
    "2025-06-18",
    "2025-03-26",
    "2024-11-05",
    "2024-10-07",
];
