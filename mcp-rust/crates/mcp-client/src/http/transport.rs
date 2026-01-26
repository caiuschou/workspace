//! HTTP client transport implementation.

use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};

use mcp_core::http::{headers, ConnectionState, SessionId, SseEvent, SseParser};
use mcp_core::stdio::{serialize_message, JsonRpcMessage};

use super::config::HttpClientConfig;
use super::error::HttpClientError;
use super::reconnect::ReconnectState;

type MessageHandler = Arc<dyn Fn(JsonRpcMessage) + Send + Sync>;
type ErrorHandler = Arc<dyn Fn(HttpClientError) + Send + Sync>;
type CloseHandler = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct EventHandlers {
    message: Option<MessageHandler>,
    error: Option<ErrorHandler>,
    close: Option<CloseHandler>,
}

/// HTTP client transport for MCP communication.
///
/// This transport uses HTTP POST for sending messages and SSE for receiving.
pub struct HttpClientTransport {
    config: HttpClientConfig,
    session_id: Arc<RwLock<Option<SessionId>>>,
    state: Arc<RwLock<ConnectionState>>,
    handlers: Arc<Mutex<EventHandlers>>,
    sse_handle: Option<JoinHandle<()>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    last_event_id: Arc<RwLock<Option<String>>>,
}

impl HttpClientTransport {
    /// Create a new HTTP client transport with the given configuration.
    pub fn new(config: HttpClientConfig) -> Self {
        Self {
            config,
            session_id: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            handlers: Arc::new(Mutex::new(EventHandlers::default())),
            sse_handle: None,
            shutdown: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            last_event_id: Arc::new(RwLock::new(None)),
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

    /// Get the current session ID.
    pub fn session_id(&self) -> Option<String> {
        self.session_id.read().ok()?.clone().map(|s| s.to_string())
    }

    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        *self.state.read().unwrap()
    }

    /// Start the transport and establish an SSE connection.
    pub fn start(&mut self) -> Result<(), HttpClientError> {
        if self.state() != ConnectionState::Disconnected {
            return Err(HttpClientError::AlreadyStarted);
        }

        self.set_state(ConnectionState::Connecting);
        self.shutdown
            .store(false, std::sync::atomic::Ordering::SeqCst);

        // Spawn SSE reader thread
        let config = self.config.clone();
        let session_id = Arc::clone(&self.session_id);
        let state = Arc::clone(&self.state);
        let handlers = Arc::clone(&self.handlers);
        let shutdown = Arc::clone(&self.shutdown);
        let last_event_id = Arc::clone(&self.last_event_id);

        let handle = thread::spawn(move || {
            run_sse_loop(config, session_id, state, handlers, shutdown, last_event_id);
        });

        self.sse_handle = Some(handle);
        Ok(())
    }

    /// Send a JSON-RPC message via HTTP POST.
    pub fn send(&self, message: &JsonRpcMessage) -> Result<(), HttpClientError> {
        if self.state() != ConnectionState::Connected {
            return Err(HttpClientError::NotConnected);
        }

        let url = self.config.endpoint_url();
        let payload = serialize_message(message)?;

        // Build request
        let session_id = self.session_id.read().unwrap();
        let mut request = ureq::post(&url)
            .set("Content-Type", headers::CONTENT_TYPE_JSON)
            .set("Accept", headers::CONTENT_TYPE_JSON);

        if let Some(ref sid) = *session_id {
            request = request.set(headers::MCP_SESSION_ID, sid.as_str());
        }

        for (name, value) in &self.config.custom_headers {
            request = request.set(name, value);
        }

        // Send request
        let response = request
            .send_string(&payload)
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

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

        // Try to send DELETE request to close session
        if let Some(ref sid) = *self.session_id.read().unwrap() {
            let url = self.config.endpoint_url();
            let _ = ureq::delete(&url)
                .set(headers::MCP_SESSION_ID, sid.as_str())
                .call();
        }

        // Wait for SSE thread to finish
        if let Some(handle) = self.sse_handle.take() {
            let _ = handle.join();
        }

        // Dispatch close event
        dispatch_close(&self.handlers);

        Ok(())
    }

    fn set_state(&self, new_state: ConnectionState) {
        let mut state = self.state.write().unwrap();
        *state = new_state;
    }
}

impl Drop for HttpClientTransport {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn run_sse_loop(
    config: HttpClientConfig,
    session_id: Arc<RwLock<Option<SessionId>>>,
    state: Arc<RwLock<ConnectionState>>,
    handlers: Arc<Mutex<EventHandlers>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    last_event_id: Arc<RwLock<Option<String>>>,
) {
    let mut reconnect_state = ReconnectState::new(config.reconnect_options.clone());

    loop {
        if shutdown.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        // Try to connect
        match connect_sse(&config, &session_id, &last_event_id) {
            Ok(reader) => {
                // Connection successful
                reconnect_state.reset();
                {
                    let mut s = state.write().unwrap();
                    *s = ConnectionState::Connected;
                }

                // Process SSE events
                if let Err(e) = process_sse_stream(reader, &handlers, &session_id, &last_event_id, &shutdown)
                {
                    dispatch_error(&handlers, e);
                }

                // Check if we should reconnect
                if shutdown.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                if !config.auto_reconnect {
                    break;
                }

                {
                    let mut s = state.write().unwrap();
                    *s = ConnectionState::Reconnecting;
                }
            }
            Err(e) => {
                dispatch_error(&handlers, e);

                if !config.auto_reconnect {
                    break;
                }

                // Get next retry delay
                match reconnect_state.next_delay() {
                    Some(delay) => {
                        thread::sleep(delay);
                    }
                    None => {
                        // Max retries exceeded
                        dispatch_error(&handlers, HttpClientError::ReconnectionExhausted);
                        break;
                    }
                }
            }
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

fn connect_sse(
    config: &HttpClientConfig,
    session_id: &Arc<RwLock<Option<SessionId>>>,
    last_event_id: &Arc<RwLock<Option<String>>>,
) -> Result<ureq::Response, HttpClientError> {
    let url = config.endpoint_url();

    let mut request = ureq::get(&url).set("Accept", headers::ACCEPT_SSE);

    // Add session ID if we have one
    if let Some(ref sid) = *session_id.read().unwrap() {
        request = request.set(headers::MCP_SESSION_ID, sid.as_str());
    }

    // Add Last-Event-ID for reconnection
    if let Some(ref id) = *last_event_id.read().unwrap() {
        request = request.set(headers::LAST_EVENT_ID, id);
    }

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

    // Extract session ID from response headers
    if let Some(sid) = response.header(headers::MCP_SESSION_ID) {
        let mut guard = session_id.write().unwrap();
        *guard = Some(SessionId::from_string(sid));
    }

    Ok(response)
}

fn process_sse_stream(
    response: ureq::Response,
    handlers: &Arc<Mutex<EventHandlers>>,
    session_id: &Arc<RwLock<Option<SessionId>>>,
    last_event_id: &Arc<RwLock<Option<String>>>,
    shutdown: &Arc<std::sync::atomic::AtomicBool>,
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
                    // Update last event ID
                    if let Some(ref id) = parsed.id {
                        let mut guard = last_event_id.write().unwrap();
                        *guard = Some(id.clone());
                    }

                    // Convert to MCP event
                    match parsed.to_mcp_event() {
                        Ok(event) => {
                            handle_sse_event(event, handlers, session_id);
                        }
                        Err(e) => {
                            return Err(HttpClientError::Sse(e.to_string()));
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

fn handle_sse_event(
    event: SseEvent,
    handlers: &Arc<Mutex<EventHandlers>>,
    session_id: &Arc<RwLock<Option<SessionId>>>,
) {
    match event {
        SseEvent::Message { data, .. } => {
            dispatch_message(handlers, data);
        }
        SseEvent::SessionReady {
            session_id: new_sid,
        } => {
            let mut guard = session_id.write().unwrap();
            *guard = Some(new_sid);
        }
        SseEvent::Endpoint { .. } => {
            // Could update endpoint URL if needed
        }
        SseEvent::Ping => {
            // Keep-alive, nothing to do
        }
    }
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

fn dispatch_close(handlers: &Arc<Mutex<EventHandlers>>) {
    let handler = handlers.lock().unwrap().close.clone();
    if let Some(handler) = handler {
        handler();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_creation() {
        let config = HttpClientConfig::new("http://localhost:8080");
        let transport = HttpClientTransport::new(config);

        assert_eq!(transport.state(), ConnectionState::Disconnected);
        assert!(transport.session_id().is_none());
    }
}
