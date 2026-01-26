//! Broadcast channel for SSE message delivery.
//!
//! This module provides message broadcasting capabilities for SSE connections,
//! including event buffering for reconnection support with Last-Event-ID.

use std::collections::VecDeque;

use mcp_core::http::SseEvent;
#[cfg(feature = "tokio")]
use mcp_core::stdio::JsonRpcMessage;

/// Configuration for the event buffer.
#[derive(Debug, Clone)]
pub struct EventBufferConfig {
    /// Maximum number of events to retain for replay.
    pub max_events: usize,
    /// Maximum age of events to retain (in seconds).
    pub max_age_secs: u64,
}

impl Default for EventBufferConfig {
    fn default() -> Self {
        Self {
            max_events: 100,
            max_age_secs: 300, // 5 minutes
        }
    }
}

/// A buffered SSE event with metadata for replay.
#[derive(Debug, Clone)]
pub struct BufferedEvent {
    /// The event ID.
    pub id: String,
    /// The SSE event.
    pub event: SseEvent,
    /// When the event was created (Unix timestamp in milliseconds).
    pub timestamp_ms: u64,
}

impl BufferedEvent {
    /// Create a new buffered event.
    pub fn new(id: String, event: SseEvent) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            id,
            event,
            timestamp_ms,
        }
    }

    /// Check if the event has expired.
    pub fn is_expired(&self, max_age_secs: u64) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let age_ms = now_ms.saturating_sub(self.timestamp_ms);
        age_ms > max_age_secs * 1000
    }
}

/// Event buffer for storing recent events for replay.
#[derive(Debug)]
pub struct EventBuffer {
    events: VecDeque<BufferedEvent>,
    config: EventBufferConfig,
}

impl EventBuffer {
    /// Create a new event buffer with the given configuration.
    pub fn new(config: EventBufferConfig) -> Self {
        Self {
            events: VecDeque::new(),
            config,
        }
    }

    /// Add an event to the buffer.
    pub fn push(&mut self, event: BufferedEvent) {
        // Remove expired events
        self.cleanup_expired();

        // Remove oldest if at capacity
        while self.events.len() >= self.config.max_events {
            self.events.pop_front();
        }

        self.events.push_back(event);
    }

    /// Get all events after the given event ID.
    pub fn events_after(&self, last_event_id: &str) -> Vec<BufferedEvent> {
        // Find the position of the last event ID
        let start_pos = self
            .events
            .iter()
            .position(|e| e.id == last_event_id)
            .map(|pos| pos + 1)
            .unwrap_or(0);

        self.events
            .iter()
            .skip(start_pos)
            .filter(|e| !e.is_expired(self.config.max_age_secs))
            .cloned()
            .collect()
    }

    /// Get all buffered events.
    pub fn all_events(&self) -> Vec<BufferedEvent> {
        self.events
            .iter()
            .filter(|e| !e.is_expired(self.config.max_age_secs))
            .cloned()
            .collect()
    }

    /// Remove expired events.
    fn cleanup_expired(&mut self) {
        let max_age = self.config.max_age_secs;
        self.events.retain(|e| !e.is_expired(max_age));
    }

    /// Get the number of buffered events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new(EventBufferConfig::default())
    }
}

/// Tokio-based broadcast channel wrapper for SSE events.
#[cfg(feature = "tokio")]
pub mod async_broadcast {
    use super::*;
    use std::sync::RwLock;
    use tokio::sync::broadcast;

    /// A broadcast sender for SSE events.
    #[derive(Debug)]
    pub struct SseBroadcaster {
        sender: broadcast::Sender<SseEvent>,
        buffer: RwLock<EventBuffer>,
        event_counter: std::sync::atomic::AtomicU64,
        session_id: String,
    }

    impl SseBroadcaster {
        /// Create a new broadcaster with the given capacity.
        pub fn new(session_id: String, capacity: usize) -> Self {
            let (sender, _) = broadcast::channel(capacity);
            Self {
                sender,
                buffer: RwLock::new(EventBuffer::default()),
                event_counter: std::sync::atomic::AtomicU64::new(0),
                session_id,
            }
        }

        /// Create a new broadcaster with custom buffer configuration.
        pub fn with_buffer_config(
            session_id: String,
            capacity: usize,
            buffer_config: EventBufferConfig,
        ) -> Self {
            let (sender, _) = broadcast::channel(capacity);
            Self {
                sender,
                buffer: RwLock::new(EventBuffer::new(buffer_config)),
                event_counter: std::sync::atomic::AtomicU64::new(0),
                session_id,
            }
        }

        /// Subscribe to the broadcast.
        pub fn subscribe(&self) -> broadcast::Receiver<SseEvent> {
            self.sender.subscribe()
        }

        /// Get the number of active subscribers.
        pub fn receiver_count(&self) -> usize {
            self.sender.receiver_count()
        }

        /// Generate the next event ID.
        fn next_event_id(&self) -> String {
            let counter = self
                .event_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                + 1;
            format!("{}-{}", self.session_id, counter)
        }

        /// Broadcast a JSON-RPC message.
        pub fn send_message(&self, message: JsonRpcMessage) -> Result<String, broadcast::error::SendError<SseEvent>> {
            let event_id = self.next_event_id();
            let event = SseEvent::Message {
                id: Some(event_id.clone()),
                data: message,
            };

            // Buffer the event for replay
            {
                let mut buffer = self.buffer.write().unwrap();
                buffer.push(BufferedEvent::new(event_id.clone(), event.clone()));
            }

            self.sender.send(event)?;
            Ok(event_id)
        }

        /// Broadcast a ping event.
        pub fn send_ping(&self) -> Result<(), broadcast::error::SendError<SseEvent>> {
            self.sender.send(SseEvent::Ping)?;
            Ok(())
        }

        /// Broadcast a raw SSE event.
        pub fn send_event(&self, event: SseEvent) -> Result<(), broadcast::error::SendError<SseEvent>> {
            // Buffer message events for replay
            if let SseEvent::Message { ref id, .. } = event {
                if let Some(event_id) = id {
                    let mut buffer = self.buffer.write().unwrap();
                    buffer.push(BufferedEvent::new(event_id.clone(), event.clone()));
                }
            }

            self.sender.send(event)?;
            Ok(())
        }

        /// Get events after the given Last-Event-ID for replay.
        pub fn get_replay_events(&self, last_event_id: &str) -> Vec<BufferedEvent> {
            let buffer = self.buffer.read().unwrap();
            buffer.events_after(last_event_id)
        }

        /// Get all buffered events.
        pub fn get_all_buffered_events(&self) -> Vec<BufferedEvent> {
            let buffer = self.buffer.read().unwrap();
            buffer.all_events()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_buffer_push() {
        let mut buffer = EventBuffer::default();
        let event = BufferedEvent::new(
            "test-1".to_string(),
            SseEvent::Ping,
        );
        buffer.push(event);
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_event_buffer_capacity() {
        let config = EventBufferConfig {
            max_events: 3,
            max_age_secs: 300,
        };
        let mut buffer = EventBuffer::new(config);

        for i in 0..5 {
            buffer.push(BufferedEvent::new(
                format!("event-{}", i),
                SseEvent::Ping,
            ));
        }

        assert_eq!(buffer.len(), 3);
        // Should have events 2, 3, 4 (oldest removed)
        let events = buffer.all_events();
        assert_eq!(events[0].id, "event-2");
        assert_eq!(events[2].id, "event-4");
    }

    #[test]
    fn test_events_after() {
        let mut buffer = EventBuffer::default();

        for i in 0..5 {
            buffer.push(BufferedEvent::new(
                format!("event-{}", i),
                SseEvent::Ping,
            ));
        }

        let after = buffer.events_after("event-2");
        assert_eq!(after.len(), 2);
        assert_eq!(after[0].id, "event-3");
        assert_eq!(after[1].id, "event-4");
    }

    #[test]
    fn test_events_after_not_found() {
        let mut buffer = EventBuffer::default();

        for i in 0..3 {
            buffer.push(BufferedEvent::new(
                format!("event-{}", i),
                SseEvent::Ping,
            ));
        }

        // If event ID not found, return all events
        let after = buffer.events_after("nonexistent");
        assert_eq!(after.len(), 3);
    }
}
