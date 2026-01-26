use std::collections::HashMap;
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

use async_trait::async_trait;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;

use mcp_core::protocol::{ProtocolError, TaskStore};
use mcp_core::types::{
    Cursor, ErrorObject, MessageId, RequestMessage, Task, TaskMetadata, TaskStatus,
};

/// Simple in-memory TaskStore implementation.
pub struct InMemoryTaskStore {
    counter: AtomicU64,
    tasks: Mutex<HashMap<String, Task>>,
    results: Mutex<HashMap<String, Result<Value, ErrorObject>>>,
}

impl InMemoryTaskStore {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
            tasks: Mutex::new(HashMap::new()),
            results: Mutex::new(HashMap::new()),
        }
    }

    fn now_timestamp() -> String {
        time::OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
    }
}

impl Default for InMemoryTaskStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskStore for InMemoryTaskStore {
    async fn create_task(
        &self,
        params: TaskMetadata,
        _request_id: MessageId,
        _request: RequestMessage,
    ) -> Result<Task, ProtocolError> {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let task_id = format!("task-{id}");
        let now = Self::now_timestamp();
        let task = Task {
            task_id: task_id.clone(),
            status: TaskStatus::Working,
            ttl: params.ttl,
            created_at: now.clone(),
            last_updated_at: now,
            poll_interval: None,
            status_message: None,
            meta: None,
        };
        self.tasks
            .lock()
            .expect("task mutex")
            .insert(task_id, task.clone());
        Ok(task)
    }

    async fn set_task_result(
        &self,
        task_id: &str,
        result: Result<Value, ErrorObject>,
    ) -> Result<(), ProtocolError> {
        let mut tasks = self.tasks.lock().expect("task mutex");
        if let Some(task) = tasks.get_mut(task_id) {
            match &result {
                Ok(_) => task.status = TaskStatus::Completed,
                Err(err) => {
                    task.status = TaskStatus::Failed;
                    task.status_message = Some(err.message.clone());
                }
            }
            task.last_updated_at = Self::now_timestamp();
        }
        self.results
            .lock()
            .expect("result mutex")
            .insert(task_id.to_string(), result);
        Ok(())
    }

    async fn get_task(&self, task_id: &str) -> Result<Option<Task>, ProtocolError> {
        Ok(self.tasks.lock().expect("task mutex").get(task_id).cloned())
    }

    async fn list_tasks(
        &self,
        _cursor: Option<Cursor>,
    ) -> Result<(Vec<Task>, Option<Cursor>), ProtocolError> {
        let tasks = self
            .tasks
            .lock()
            .expect("task mutex")
            .values()
            .cloned()
            .collect();
        Ok((tasks, None))
    }

    async fn get_task_result(
        &self,
        task_id: &str,
    ) -> Result<Option<Result<Value, ErrorObject>>, ProtocolError> {
        Ok(self
            .results
            .lock()
            .expect("result mutex")
            .get(task_id)
            .cloned())
    }

    async fn cancel_task(&self, task_id: &str) -> Result<Option<Task>, ProtocolError> {
        let mut tasks = self.tasks.lock().expect("task mutex");
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;
            task.last_updated_at = Self::now_timestamp();
            return Ok(Some(task.clone()));
        }
        Ok(None)
    }
}
