//! Reconnection strategy for HTTP client transport.

use std::time::Duration;

/// Options for reconnection behavior.
#[derive(Debug, Clone)]
pub struct ReconnectOptions {
    /// Initial delay before first reconnection attempt.
    pub initial_delay: Duration,

    /// Maximum delay between reconnection attempts.
    pub max_delay: Duration,

    /// Multiplier for exponential backoff.
    pub backoff_multiplier: f64,

    /// Maximum number of reconnection attempts (None for unlimited).
    pub max_attempts: Option<u32>,

    /// Jitter factor (0.0 to 1.0) to add randomness to delays.
    pub jitter: f64,
}

impl Default for ReconnectOptions {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            max_attempts: Some(10),
            jitter: 0.1,
        }
    }
}

impl ReconnectOptions {
    /// Create options for aggressive reconnection.
    pub fn aggressive() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 1.5,
            max_attempts: Some(20),
            jitter: 0.1,
        }
    }

    /// Create options for relaxed reconnection.
    pub fn relaxed() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            max_attempts: Some(5),
            jitter: 0.2,
        }
    }

    /// Create options that never give up.
    pub fn persistent() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            max_attempts: None,
            jitter: 0.2,
        }
    }
}

/// State machine for managing reconnection attempts.
#[derive(Debug)]
pub struct ReconnectState {
    options: ReconnectOptions,
    attempt: u32,
    current_delay: Duration,
}

impl ReconnectState {
    /// Create a new reconnection state with the given options.
    pub fn new(options: ReconnectOptions) -> Self {
        let initial_delay = options.initial_delay;
        Self {
            options,
            attempt: 0,
            current_delay: initial_delay,
        }
    }

    /// Check if another reconnection attempt should be made.
    pub fn should_retry(&self) -> bool {
        match self.options.max_attempts {
            Some(max) => self.attempt < max,
            None => true,
        }
    }

    /// Get the delay before the next reconnection attempt.
    pub fn next_delay(&mut self) -> Option<Duration> {
        if !self.should_retry() {
            return None;
        }

        let delay = self.current_delay;
        self.attempt += 1;

        // Calculate next delay with exponential backoff
        let next = Duration::from_secs_f64(
            self.current_delay.as_secs_f64() * self.options.backoff_multiplier,
        );
        self.current_delay = next.min(self.options.max_delay);

        // Apply jitter
        let jittered = if self.options.jitter > 0.0 {
            let jitter_range = delay.as_secs_f64() * self.options.jitter;
            let jitter = (rand_simple() * 2.0 - 1.0) * jitter_range;
            Duration::from_secs_f64((delay.as_secs_f64() + jitter).max(0.0))
        } else {
            delay
        };

        Some(jittered)
    }

    /// Reset the state for a new connection cycle.
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.current_delay = self.options.initial_delay;
    }

    /// Get the current attempt number.
    pub fn attempt(&self) -> u32 {
        self.attempt
    }
}

/// Simple pseudo-random number generator (not cryptographically secure).
fn rand_simple() -> f64 {
    use std::cell::Cell;
    use std::time::SystemTime;

    thread_local! {
        static SEED: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        );
    }

    SEED.with(|seed| {
        // Simple xorshift64
        let mut s = seed.get();
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        seed.set(s);
        (s as f64) / (u64::MAX as f64)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = ReconnectOptions::default();
        assert_eq!(options.initial_delay, Duration::from_millis(500));
        assert_eq!(options.max_attempts, Some(10));
    }

    #[test]
    fn test_reconnect_state_basic() {
        let options = ReconnectOptions {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            max_attempts: Some(3),
            jitter: 0.0,
        };

        let mut state = ReconnectState::new(options);

        assert!(state.should_retry());
        assert_eq!(state.next_delay(), Some(Duration::from_millis(100)));
        assert_eq!(state.attempt(), 1);

        assert!(state.should_retry());
        assert_eq!(state.next_delay(), Some(Duration::from_millis(200)));
        assert_eq!(state.attempt(), 2);

        assert!(state.should_retry());
        assert_eq!(state.next_delay(), Some(Duration::from_millis(400)));
        assert_eq!(state.attempt(), 3);

        // Max attempts reached
        assert!(!state.should_retry());
        assert_eq!(state.next_delay(), None);
    }

    #[test]
    fn test_reconnect_state_reset() {
        let options = ReconnectOptions {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            max_attempts: Some(3),
            jitter: 0.0,
        };

        let mut state = ReconnectState::new(options);

        state.next_delay();
        state.next_delay();
        assert_eq!(state.attempt(), 2);

        state.reset();
        assert_eq!(state.attempt(), 0);
        assert!(state.should_retry());
    }

    #[test]
    fn test_max_delay_cap() {
        let options = ReconnectOptions {
            initial_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 3.0,
            max_attempts: Some(5),
            jitter: 0.0,
        };

        let mut state = ReconnectState::new(options);

        assert_eq!(state.next_delay(), Some(Duration::from_secs(5)));
        assert_eq!(state.next_delay(), Some(Duration::from_secs(10))); // Capped
        assert_eq!(state.next_delay(), Some(Duration::from_secs(10))); // Still capped
    }
}
