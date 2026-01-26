use serde_json::Value;

use crate::client::TaskInfo;

/// Message emitted by request_stream.
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseMessage {
    Result(Value),
    Error(String),
    TaskCreated(TaskInfo),
    TaskStatus(TaskInfo),
}
