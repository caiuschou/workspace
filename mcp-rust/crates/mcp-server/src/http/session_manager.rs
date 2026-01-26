//! Session management for HTTP server.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use mcp_core::http::{ResumptionToken, SessionId};

use super::error::HttpServerError;

/// Configuration for session management.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum number of concurrent sessions.
    pub max_sessions: usize,
    /// Session timeout (how long a session can be idle).
    pub session_timeout: Duration,
    /// How often to clean up expired sessions.
    pub cleanup_interval: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 1000,
            session_timeout: Duration::from_secs(30 * 60), // 30 minutes
            cleanup_interval: Duration::from_secs(60),      // 1 minute
        }
    }
}

/// State of a single session.
#[derive(Debug, Clone)]
pub struct SessionState {
    /// The session ID.
    pub session_id: SessionId,
    /// When the session was created.
    pub created_at: Instant,
    /// When the session was last active.
    pub last_activity: Instant,
    /// Whether the session has been initialized.
    pub initialized: bool,
    /// Counter for SSE event IDs.
    pub event_counter: u64,
    /// Custom data associated with the session.
    pub data: HashMap<String, serde_json::Value>,
}

impl SessionState {
    /// Create a new session state.
    fn new(session_id: SessionId) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            created_at: now,
            last_activity: now,
            initialized: false,
            event_counter: 0,
            data: HashMap::new(),
        }
    }

    /// Check if the session has expired.
    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// Update the last activity timestamp.
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Generate the next event ID.
    pub fn next_event_id(&mut self) -> String {
        self.event_counter += 1;
        format!("{}-{}", self.session_id.as_str(), self.event_counter)
    }

    /// Create a resumption token for this session.
    pub fn resumption_token(&self, last_event_id: Option<String>) -> ResumptionToken {
        ResumptionToken::new(self.session_id.clone(), last_event_id)
    }
}

/// Thread-safe session manager.
#[derive(Debug)]
pub struct SessionManager {
    config: SessionConfig,
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session.
    pub fn create_session(&self) -> Result<SessionState, HttpServerError> {
        let mut sessions = self.sessions.write().unwrap();

        // Check session limit
        if sessions.len() >= self.config.max_sessions {
            return Err(HttpServerError::SessionLimitReached {
                max: self.config.max_sessions,
            });
        }

        let session_id = SessionId::new();
        let state = SessionState::new(session_id.clone());
        sessions.insert(session_id.to_string(), state.clone());

        Ok(state)
    }

    /// Get a session by ID.
    pub fn get_session(&self, session_id: &str) -> Option<SessionState> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }

    /// Get a session by ID, updating its last activity timestamp.
    pub fn touch_session(&self, session_id: &str) -> Option<SessionState> {
        let mut sessions = self.sessions.write().unwrap();
        if let Some(state) = sessions.get_mut(session_id) {
            state.touch();
            Some(state.clone())
        } else {
            None
        }
    }

    /// Update a session's state.
    pub fn update_session<F>(&self, session_id: &str, f: F) -> Option<SessionState>
    where
        F: FnOnce(&mut SessionState),
    {
        let mut sessions = self.sessions.write().unwrap();
        if let Some(state) = sessions.get_mut(session_id) {
            f(state);
            state.touch();
            Some(state.clone())
        } else {
            None
        }
    }

    /// Mark a session as initialized.
    pub fn mark_initialized(&self, session_id: &str) -> Option<SessionState> {
        self.update_session(session_id, |state| {
            state.initialized = true;
        })
    }

    /// Remove a session.
    pub fn remove_session(&self, session_id: &str) -> Option<SessionState> {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(session_id)
    }

    /// Validate a session ID and return the session if valid.
    pub fn validate_session(&self, session_id: &str) -> Result<SessionState, HttpServerError> {
        let sessions = self.sessions.read().unwrap();

        let state = sessions
            .get(session_id)
            .ok_or_else(|| HttpServerError::SessionNotFound(session_id.to_string()))?;

        if state.is_expired(self.config.session_timeout) {
            return Err(HttpServerError::SessionExpired(session_id.to_string()));
        }

        Ok(state.clone())
    }

    /// Clean up expired sessions.
    pub fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().unwrap();
        let timeout = self.config.session_timeout;

        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, state)| state.is_expired(timeout))
            .map(|(id, _)| id.clone())
            .collect();

        let count = expired.len();
        for id in expired {
            sessions.remove(&id);
        }

        count
    }

    /// Get the number of active sessions.
    pub fn session_count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.len()
    }

    /// Get all session IDs.
    pub fn session_ids(&self) -> Vec<String> {
        let sessions = self.sessions.read().unwrap();
        sessions.keys().cloned().collect()
    }

    /// Try to resume a session from a resumption token.
    pub fn resume_session(&self, token: &ResumptionToken) -> Result<SessionState, HttpServerError> {
        let session_id = token.session_id.as_str();

        // Try to get and validate the session
        match self.validate_session(session_id) {
            Ok(state) => {
                // Touch the session to update last activity
                self.touch_session(session_id);
                Ok(state)
            }
            Err(HttpServerError::SessionExpired(_)) => {
                // Session expired, remove it
                self.remove_session(session_id);
                Err(HttpServerError::SessionExpired(session_id.to_string()))
            }
            Err(e) => Err(e),
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let manager = SessionManager::default();
        let session = manager.create_session().unwrap();

        assert!(!session.initialized);
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_get_session() {
        let manager = SessionManager::default();
        let session = manager.create_session().unwrap();
        let session_id = session.session_id.to_string();

        let retrieved = manager.get_session(&session_id).unwrap();
        assert_eq!(retrieved.session_id, session.session_id);
    }

    #[test]
    fn test_remove_session() {
        let manager = SessionManager::default();
        let session = manager.create_session().unwrap();
        let session_id = session.session_id.to_string();

        assert_eq!(manager.session_count(), 1);
        manager.remove_session(&session_id);
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_limit() {
        let config = SessionConfig {
            max_sessions: 2,
            ..Default::default()
        };
        let manager = SessionManager::new(config);

        manager.create_session().unwrap();
        manager.create_session().unwrap();

        let result = manager.create_session();
        assert!(matches!(
            result,
            Err(HttpServerError::SessionLimitReached { max: 2 })
        ));
    }

    #[test]
    fn test_mark_initialized() {
        let manager = SessionManager::default();
        let session = manager.create_session().unwrap();
        let session_id = session.session_id.to_string();

        assert!(!session.initialized);

        let updated = manager.mark_initialized(&session_id).unwrap();
        assert!(updated.initialized);
    }

    #[test]
    fn test_next_event_id() {
        let manager = SessionManager::default();
        let session = manager.create_session().unwrap();
        let session_id = session.session_id.to_string();

        let id1 = manager
            .update_session(&session_id, |s| {
                s.next_event_id();
            })
            .unwrap();

        let state = manager.get_session(&session_id).unwrap();
        assert_eq!(state.event_counter, 1);
    }
}
