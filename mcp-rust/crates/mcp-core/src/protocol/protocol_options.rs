use std::sync::Arc;

use super::{CapabilityChecker, TaskStore};

/// Configuration for the protocol runtime.
#[derive(Clone, Default)]
pub struct ProtocolOptions {
    pub enforce_strict_capabilities: bool,
    pub capability_checker: Option<Arc<dyn CapabilityChecker>>,
    pub task_store: Option<Arc<dyn TaskStore>>,
}
