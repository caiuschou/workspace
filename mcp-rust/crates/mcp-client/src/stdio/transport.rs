use std::{
    io::{Read, Write},
    process::{Child, ChildStderr, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use mcp_core::stdio::{JsonRpcMessage, ReadBuffer, ReadBufferError, Transport, serialize_message};

use crate::stdio::{
    env::get_default_environment, error::StdioClientTransportError, params::StdioServerParameters,
};

type MessageHandler = Arc<dyn Fn(JsonRpcMessage) + Send + Sync>;
type ErrorHandler = Arc<dyn Fn(StdioClientTransportError) + Send + Sync>;
type CloseHandler = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct EventHandlers {
    message: Option<MessageHandler>,
    error: Option<ErrorHandler>,
    close: Option<CloseHandler>,
}

/// Client transport that talks to a child process over stdin/stdout.
pub struct StdioClientTransport {
    server_params: StdioServerParameters,
    child: Option<Child>,
    stderr_handle: Option<ChildStderr>,
    reader_handle: Option<JoinHandle<()>>,
    handlers: Arc<Mutex<EventHandlers>>,
}

impl StdioClientTransport {
    /// Create a new transport targeting the provided command.
    pub fn new(server_params: StdioServerParameters) -> Self {
        Self {
            server_params,
            child: None,
            stderr_handle: None,
            reader_handle: None,
            handlers: Arc::new(Mutex::new(EventHandlers::default())),
        }
    }

    /// Register a handler triggered for every decoded JSON-RPC message.
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

    /// Register a handler invoked when the transport encounters an error.
    pub fn on_error(
        &mut self,
        handler: impl Fn(StdioClientTransportError) + Send + Sync + 'static,
    ) -> &mut Self {
        {
            let mut guard = self.handlers.lock().unwrap();
            guard.error = Some(Arc::new(handler));
        }
        self
    }

    /// Register a handler invoked when the child process closes its stdout.
    pub fn on_close(&mut self, handler: impl Fn() + Send + Sync + 'static) -> &mut Self {
        {
            let mut guard = self.handlers.lock().unwrap();
            guard.close = Some(Arc::new(handler));
        }
        self
    }

    /// Start the child process and begin listening for messages on stdout.
    pub fn start(&mut self) -> Result<(), StdioClientTransportError> {
        if self.child.is_some() {
            return Err(StdioClientTransportError::AlreadyStarted);
        }

        let mut command = Command::new(&self.server_params.command);
        command.args(&self.server_params.args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(self.server_params.stderr.to_stdio());
        command.env_clear();
        command.envs(get_default_environment().into_iter());
        if let Some(env) = &self.server_params.env {
            command.envs(env);
        }
        if let Some(cwd) = &self.server_params.cwd {
            command.current_dir(cwd);
        }

        let mut child = command
            .spawn()
            .map_err(|source| StdioClientTransportError::Spawn {
                command: self.server_params.command.clone(),
                source,
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or(StdioClientTransportError::NotConnected)?;
        self.stderr_handle = child.stderr.take();

        let handlers = Arc::clone(&self.handlers);
        let reader_handle = spawn_reader(stdout, handlers);

        self.child = Some(child);
        self.reader_handle = Some(reader_handle);
        Ok(())
    }

    /// Send a JSON-RPC message over stdin.
    pub fn send(&mut self, message: &JsonRpcMessage) -> Result<(), StdioClientTransportError> {
        let child = self
            .child
            .as_mut()
            .ok_or(StdioClientTransportError::NotConnected)?;
        let stdin = child
            .stdin
            .as_mut()
            .ok_or(StdioClientTransportError::NotConnected)?;

        let payload = serialize_message(message)?;
        stdin.write_all(payload.as_bytes())?;
        stdin.flush()?;
        Ok(())
    }

    /// Close the transport and wait for the child to exit.
    pub fn close(&mut self) -> Result<(), StdioClientTransportError> {
        if let Some(mut child) = self.child.take() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.flush();
            }

            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline {
                match child.try_wait()? {
                    Some(_) => break,
                    None => thread::sleep(Duration::from_millis(50)),
                }
            }

            if child.try_wait()?.is_none() {
                let _ = child.kill();
            }
        }

        self.stderr_handle = None;
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }

        Ok(())
    }

    /// The stderr handle of the child process, if available.
    pub fn stderr(&mut self) -> Option<&mut ChildStderr> {
        self.stderr_handle.as_mut()
    }

    /// The process ID of the spawned child, if running.
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(|child| child.id())
    }
}

impl Drop for StdioClientTransport {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl From<ReadBufferError> for StdioClientTransportError {
    fn from(err: ReadBufferError) -> Self {
        match err {
            ReadBufferError::Utf8(utf8) => StdioClientTransportError::Utf8(utf8),
            ReadBufferError::Json(json) => StdioClientTransportError::Serialization(json),
        }
    }
}

impl Transport for StdioClientTransport {
    type Message = JsonRpcMessage;
    type Error = StdioClientTransportError;

    fn start(&mut self) -> Result<(), Self::Error> {
        StdioClientTransport::start(self)
    }

    fn send(&mut self, message: &Self::Message) -> Result<(), Self::Error> {
        StdioClientTransport::send(self, message)
    }

    fn close(&mut self) -> Result<(), Self::Error> {
        StdioClientTransport::close(self)
    }
}

fn spawn_reader(stdout: ChildStdout, handlers: Arc<Mutex<EventHandlers>>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut stdout = stdout;
        let mut buffer = ReadBuffer::default();
        let mut temp = [0u8; 4096];

        'outer: loop {
            match stdout.read(&mut temp) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.append(&temp[..n]);
                    loop {
                        match buffer.read_message() {
                            Ok(Some(message)) => dispatch_message(&handlers, message),
                            Ok(None) => break,
                            Err(err) => {
                                dispatch_error(&handlers, err.into());
                                break 'outer;
                            }
                        }
                    }
                }
                Err(err) => {
                    dispatch_error(&handlers, StdioClientTransportError::Io(err));
                    break;
                }
            }
        }

        dispatch_close(&handlers);
    })
}

fn dispatch_message(handlers: &Arc<Mutex<EventHandlers>>, message: JsonRpcMessage) {
    let handler = handlers.lock().unwrap().message.clone();
    if let Some(handler) = handler {
        handler(message);
    }
}

fn dispatch_error(handlers: &Arc<Mutex<EventHandlers>>, error: StdioClientTransportError) {
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
mod tests;
