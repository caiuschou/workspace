use async_trait::async_trait;
use serde_json::Value;

use crate::types::{Cursor, ErrorObject, MessageId, RequestMessage, Task, TaskMetadata};

use super::ProtocolError;

/// Storage backend for task-augmented requests.
#[async_trait]
pub trait TaskStore: Send + Sync + 'static {
    async fn create_task(
        &self,
        params: TaskMetadata,
        request_id: MessageId,
        request: RequestMessage,
    ) -> Result<Task, ProtocolError>;

    async fn set_task_result(
        &self,
        task_id: &str,
        result: Result<Value, ErrorObject>,
    ) -> Result<(), ProtocolError>;

    async fn get_task(&self, task_id: &str) -> Result<Option<Task>, ProtocolError>;

    async fn list_tasks(
        &self,
        cursor: Option<Cursor>,
    ) -> Result<(Vec<Task>, Option<Cursor>), ProtocolError>;

    async fn get_task_result(
        &self,
        task_id: &str,
    ) -> Result<Option<Result<Value, ErrorObject>>, ProtocolError>;

    async fn cancel_task(&self, task_id: &str) -> Result<Option<Task>, ProtocolError>;
}
