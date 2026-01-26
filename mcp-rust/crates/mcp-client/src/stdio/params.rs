use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;

/// Parameters used when starting a stdio-based server process.
#[derive(Debug, Clone)]
pub struct StdioServerParameters {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    pub stderr: StdioStream,
    pub cwd: Option<PathBuf>,
}

impl StdioServerParameters {
    /// Create a new set of parameters for the provided command.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: None,
            stderr: StdioStream::Inherit,
            cwd: None,
        }
    }

    /// Replace the argument list for the spawned process.
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(|arg| arg.into()).collect();
        self
    }

    /// Extend the environment variables provided to the child process.
    pub fn env(
        mut self,
        env: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        let map = self.env.get_or_insert_with(HashMap::new);
        for (key, value) in env {
            map.insert(key.into(), value.into());
        }
        self
    }

    /// Set how stderr is handled for the child process.
    pub fn stderr(mut self, stream: StdioStream) -> Self {
        self.stderr = stream;
        self
    }

    /// Set the working directory for the spawned process.
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}

/// Controls how stdio streams are inherited or captured.
#[derive(Debug, Clone, Copy)]
pub enum StdioStream {
    Inherit,
    Pipe,
    Null,
}

impl Default for StdioStream {
    fn default() -> Self {
        Self::Inherit
    }
}

impl StdioStream {
    pub(crate) fn to_stdio(self) -> Stdio {
        match self {
            StdioStream::Inherit => Stdio::inherit(),
            StdioStream::Pipe => Stdio::piped(),
            StdioStream::Null => Stdio::null(),
        }
    }
}
