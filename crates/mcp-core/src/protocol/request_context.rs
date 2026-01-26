use crate::types::{RequestMeta, TaskMetadata};

use super::RequestOptions;

/// Context passed to request handlers.
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub session_id: Option<String>,
    pub options: RequestOptions,
    pub meta: Option<RequestMeta>,
    pub task: Option<TaskMetadata>,
}
