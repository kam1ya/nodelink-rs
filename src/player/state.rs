//! In-memory state tracking for guild players using atomic operations.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};

/// Thread-safe in-memory cache of a player's real-time state.
///
/// Updated automatically in the background when WebSocket `PlayerUpdate` frames are received.
/// Reads from this structure incur zero lock contention and zero network latency.
#[derive(Debug, Default)]
pub struct PlayerState {
    time: AtomicU64,
    position: AtomicU64,
    ping: AtomicI64,
    connected: AtomicBool,
    paused: AtomicBool,
}

impl PlayerState {
    /// Creates a default, uninitialized player state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Atomically updates playback metrics using Release ordering.
    pub(crate) fn update_state(&self, time: u64, position: u64, connected: bool, ping: i64) {
        self.time.store(time, Ordering::Release);
        self.position.store(position, Ordering::Release);
        self.connected.store(connected, Ordering::Release);
        self.ping.store(ping, Ordering::Release);
    }

    /// Atomically updates the pause state flag.
    pub(crate) fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Release);
    }

    /// Returns the current playback position in milliseconds.
    #[inline]
    pub fn position(&self) -> u64 {
        self.position.load(Ordering::Acquire)
    }

    /// Returns the voice round-trip ping latency to Discord in milliseconds.
    #[inline]
    pub fn ping(&self) -> i64 {
        self.ping.load(Ordering::Acquire)
    }

    /// Returns whether the player is currently connected to a Discord voice server.
    #[inline]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    /// Returns whether playback is currently paused.
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Acquire)
    }

    /// Returns the Unix timestamp (in milliseconds) of the most recent status frame received from the node.
    #[inline]
    pub fn last_updated_time(&self) -> u64 {
        self.time.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_state_and_paused() {
        let state = PlayerState::new();
        state.update_state(1700000000000, 30000, true, 25);
        assert_eq!(state.position(), 30000);
        assert_eq!(state.ping(), 25);
        assert!(state.is_connected());
        assert_eq!(state.last_updated_time(), 1700000000000);

        state.set_paused(true);
        assert!(state.is_paused());
        state.set_paused(false);
        assert!(!state.is_paused());
    }
}
