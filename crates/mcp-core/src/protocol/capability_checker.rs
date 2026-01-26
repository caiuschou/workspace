use super::ProtocolError;

/// Capability checks for protocol requests and notifications.
pub trait CapabilityChecker: Send + Sync {
    fn assert_request(&self, method: &str) -> Result<(), ProtocolError>;
    fn assert_notification(&self, method: &str) -> Result<(), ProtocolError>;
    fn assert_request_handler(&self, method: &str) -> Result<(), ProtocolError>;
    fn assert_notification_handler(&self, method: &str) -> Result<(), ProtocolError>;
}
