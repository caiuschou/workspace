use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// General-purpose message used for example logging between runtimes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Message {
    pub sender: String,
    pub recipient: String,
    pub body: String,
}

impl Message {
    /// Creates a new transport-agnostic message.
    pub fn new(
        sender: impl Into<String>,
        recipient: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            sender: sender.into(),
            recipient: recipient.into(),
            body: body.into(),
        }
    }

    /// Creates a short summary for CLI logging.
    pub fn summary(&self) -> String {
        format!("[{} -> {}] {}", self.sender, self.recipient, self.body)
    }
}
