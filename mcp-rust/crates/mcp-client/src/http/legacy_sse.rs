//! Legacy SSE client transport for backwards compatibility.
//!
//! This module implements the deprecated HTTP+SSE transport (protocol version 2024-11-05)
//! for connecting to older MCP servers.
//!
//! ## Differences from Streamable HTTP
//!
//! | Feature | Legacy SSE | Streamable HTTP |
//! |---------|-----------|-----------------|
//! | Session ID | URL query param `?sessionId=xxx` | `Mcp-Session-Id` header |
//! | Endpoint | Receives `endpoint` event with POST URL | Not needed |
//! | POST response | Expects 202 Accepted | Expects JSON or SSE stream |
//!
//! ## Usage
//!
//! This transport is deprecated and should only be used to connect to older servers.
//! New implementations should use Streamable HTTP.

use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};

use mcp_core::http::{headers, ConnectionState, SseParser};
use mcp_core::stdio::{serialize_message, JsonRpcMessage};

use super::error::HttpClientError;

type MessageHandler = Arc<dyn Fn(JsonRpcMessage) + Send + Sync>;
type ErrorHandler = Arc<dyn Fn(HttpClientError) + Send + Sync>;
type CloseHandler = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct EventHandlers {
    message: Option<MessageHandler>,
    error: Option<ErrorHandler>,
    close: Option<CloseHandler>,
}

/// Configuration for legacy SSE client transport.
#[derive(Debug, Clone)]
pub struct LegacySseClientConfig {
    /// Base URL of the server (e.g., "http://localhost:8080")
    base_url: String,
    /// SSE endpoint path (default: "/sse")
    pub sse_path: String,
    /// Custom headers to include in requests
    pub custom_headers: Vec<(String, String)>,
}

impl LegacySseClientConfig {
    /// Create a new configuration with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            sse_path: "/sse".to_string(),
            custom_headers: Vec::new(),
        }
    }

    /// Set the SSE endpoint path.
    pub fn sse_path(mut self, path: impl Into<String>) -> Self {
        self.sse_path = path.into();
        self
    }

    /// Add a custom header.
    pub fn custom_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_headers.push((name.into(), value.into()));
        self
    }

    /// Get the full SSE endpoint URL.
    pub fn sse_url(&self) -> String {
        format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            self.sse_path
        )
    }
}

/// Legacy SSE client transport for MCP communication.
///
/// This transport connects to servers using the deprecated HTTP+SSE protocol.
/// It receives messages via SSE and sends messages via POST requests.
///
/// ## Protocol Flow
///
/// 1. Client connects to SSE endpoint (GET /sse)
/// 2. Server sends `endpoint` event with POST URL containing session ID
/// 3. Client sends messages via POST to the endpoint URL
/// 4. Server responds with 202 Accepted
/// 5. Server sends response messages via SSE
pub struct LegacySseClientTransport {
    config: LegacySseClientConfig,
    state: Arc<RwLock<ConnectionState>>,
    handlers: Arc<Mutex<EventHandlers>>,
    /// POST endpoint URL (received from server via `endpoint` event)
    post_endpoint: Arc<RwLock<Option<String>>>,
    sse_handle: Option<JoinHandle<()>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl LegacySseClientTransport {
    /// Create a new legacy SSE client transport.
    pub fn new(config: LegacySseClientConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            handlers: Arc::new(Mutex::new(EventHandlers::default())),
            post_endpoint: Arc::new(RwLock::new(None)),
            sse_handle: None,
            shutdown: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Register a handler for incoming JSON-RPC messages.
    pub fn on_message(
        &mut self,
        handler: impl Fn(JsonRpcMessage) + Send + Sync + 'static,
    ) -> &mut Self {
        {
            let mut guard = self.handlers.lock().unwrap();
            guard.message = Some(Arc::new(handler));
        }
        self
    }

    /// Register a handler for transport errors.
    pub fn on_error(
        &mut self,
        handler: impl Fn(HttpClientError) + Send + Sync + 'static,
    ) -> &mut Self {
        {
            let mut guard = self.handlers.lock().unwrap();
            guard.error = Some(Arc::new(handler));
        }
        self
    }

    /// Register a handler for connection close events.
    pub fn on_close(&mut self, handler: impl Fn() + Send + Sync + 'static) -> &mut Self {
        {
            let mut guard = self.handlers.lock().unwrap();
            guard.close = Some(Arc::new(handler));
        }
        self
    }

    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        *self.state.read().unwrap()
    }

    /// Start the transport and establish SSE connection.
    pub fn start(&mut self) -> Result<(), HttpClientError> {
        if self.state() != ConnectionState::Disconnected {
            return Err(HttpClientError::AlreadyStarted);
        }

        self.set_state(ConnectionState::Connecting);
        self.shutdown
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Spawn SSE reader thread
        let config = self.config.clone();
        let state = Arc::clone(&self.state);
        let handlers = Arc::clone(&self.handlers);
        let shutdown = Arc::clone(&self.shutdown);
        let post_endpoint = Arc::clone(&self.post_endpoint);

        let handle = thread::spawn(move || {
            run_sse_loop(config, state, handlers, shutdown, post_endpoint);
        });

        self.sse_handle = Some(handle);
        Ok(())
    }

    /// Send a JSON-RPC message via HTTP POST.
    pub fn send(&self, message: &JsonRpcMessage) -> Result<(), HttpClientError> {
        if self.state() != ConnectionState::Connected {
            return Err(HttpClientError::NotConnected);
        }

        let endpoint = self
            .post_endpoint
            .read()
            .unwrap()
            .clone()
            .ok_or(HttpClientError::NotConnected)?;

        let payload = serialize_message(message)?;

        // Build request
        let mut request = ureq::post(&endpoint).set("Content-Type", headers::CONTENT_TYPE_JSON);

        for (name, value) in &self.config.custom_headers {
            request = request.set(name, value);
        }

        // Send request
        let response = request
            .send_string(&payload)
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

        // Legacy protocol expects 202 Accepted
        if response.status() >= 400 {
            return Err(HttpClientError::HttpStatus {
                status: response.status(),
                body: response.into_string().ok(),
            });
        }

        Ok(())
    }

    /// Close the transport.
    pub fn close(&mut self) -> Result<(), HttpClientError> {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.set_state(ConnectionState::Closed);

        // Wait for SSE thread to finish
        if let Some(handle) = self.sse_handle.take() {
            let _ = handle.join();
        }

        // Dispatch close event
        let handler = self.handlers.lock().unwrap().close.clone();
        if let Some(handler) = handler {
            handler();
        }

        Ok(())
    }

    fn set_state(&self, new_state: ConnectionState) {
        let mut state = self.state.write().unwrap();
        *state = new_state;
    }
}

impl Drop for LegacySseClientTransport {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn run_sse_loop(
    config: LegacySseClientConfig,
    state: Arc<RwLock<ConnectionState>>,
    handlers: Arc<Mutex<EventHandlers>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    post_endpoint: Arc<RwLock<Option<String>>>,
) {
    // Try to connect
    match connect_sse(&config) {
        Ok(reader) => {
            // Process SSE events
            if let Err(e) = process_sse_stream(
                reader,
                &config,
                &handlers,
                &state,
                &shutdown,
                &post_endpoint,
            ) {
                dispatch_error(&handlers, e);
            }
        }
        Err(e) => {
            dispatch_error(&handlers, e);
        }
    }

    // Final state update
    {
        let mut s = state.write().unwrap();
        if *s != ConnectionState::Closed {
            *s = ConnectionState::Disconnected;
        }
    }
}

fn connect_sse(config: &LegacySseClientConfig) -> Result<ureq::Response, HttpClientError> {
    let url = config.sse_url();

    let mut request = ureq::get(&url).set("Accept", headers::ACCEPT_SSE);

    // Add custom headers
    for (name, value) in &config.custom_headers {
        request = request.set(name, value);
    }

    let response = request
        .call()
        .map_err(|e| HttpClientError::Request(e.to_string()))?;

    if response.status() >= 400 {
        return Err(HttpClientError::HttpStatus {
            status: response.status(),
            body: None,
        });
    }

    Ok(response)
}

fn process_sse_stream(
    response: ureq::Response,
    config: &LegacySseClientConfig,
    handlers: &Arc<Mutex<EventHandlers>>,
    state: &Arc<RwLock<ConnectionState>>,
    shutdown: &Arc<std::sync::atomic::AtomicBool>,
    post_endpoint: &Arc<RwLock<Option<String>>>,
) -> Result<(), HttpClientError> {
    let reader = response.into_reader();
    let mut buf_reader = std::io::BufReader::new(reader);
    let mut parser = SseParser::new();
    let mut line = String::new();

    loop {
        if shutdown.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        line.clear();
        match std::io::BufRead::read_line(&mut buf_reader, &mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                parser.append(&line);

                while let Some(parsed) = parser.next_event() {
                    // Handle legacy protocol events
                    match parsed.event.as_deref() {
                        Some("endpoint") => {
                            // Received POST endpoint URL
                            let endpoint_path = parsed.data.trim();
                            let full_url = format!(
                                "{}{}",
                                config.base_url.trim_end_matches('/'),
                                endpoint_path
                            );

                            {
                                let mut guard = post_endpoint.write().unwrap();
                                *guard = Some(full_url);
                            }

                            // Now connected
                            {
                                let mut s = state.write().unwrap();
                                *s = ConnectionState::Connected;
                            }
                        }
                        Some("message") | None => {
                            // Regular message
                            match parsed.to_mcp_event() {
                                Ok(event) => {
                                    if let mcp_core::http::SseEvent::Message { data, .. } = event {
                                        dispatch_message(handlers, data);
                                    }
                                }
                                Err(e) => {
                                    return Err(HttpClientError::Sse(e.to_string()));
                                }
                            }
                        }
                        _ => {
                            // Ignore other events
                        }
                    }
                }
            }
            Err(e) => {
                return Err(HttpClientError::Io(e));
            }
        }
    }

    Ok(())
}

fn dispatch_message(handlers: &Arc<Mutex<EventHandlers>>, message: JsonRpcMessage) {
    let handler = handlers.lock().unwrap().message.clone();
    if let Some(handler) = handler {
        handler(message);
    }
}

fn dispatch_error(handlers: &Arc<Mutex<EventHandlers>>, error: HttpClientError) {
    let handler = handlers.lock().unwrap().error.clone();
    if let Some(handler) = handler {
        handler(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = LegacySseClientConfig::new("http://localhost:8080");
        assert_eq!(config.sse_url(), "http://localhost:8080/sse");
    }

    #[test]
    fn test_config_custom_path() {
        let config = LegacySseClientConfig::new("http://localhost:8080").sse_path("/custom/sse");
        assert_eq!(config.sse_url(), "http://localhost:8080/custom/sse");
    }

    #[test]
    fn test_transport_creation() {
        let config = LegacySseClientConfig::new("http://localhost:8080");
        let transport = LegacySseClientTransport::new(config);
        assert_eq!(transport.state(), ConnectionState::Disconnected);
    }
}
