use serde_json::Value;

use crate::client::ListChangedOptions;

/// Handlers for list changed notifications.
#[derive(Clone, Default)]
pub struct ListChangedHandlers {
    pub tools: Option<ListChangedOptions<Value>>,
    pub prompts: Option<ListChangedOptions<Value>>,
    pub resources: Option<ListChangedOptions<Value>>,
}
