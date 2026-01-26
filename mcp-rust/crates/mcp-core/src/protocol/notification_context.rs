use crate::types::RequestMeta;

/// Context passed to notification handlers.
#[derive(Debug, Clone, Default)]
pub struct NotificationContext {
    pub session_id: Option<String>,
    pub meta: Option<RequestMeta>,
}
