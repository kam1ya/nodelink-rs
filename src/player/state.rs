//! In-memory state tracking for guild players.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};

/// Thread-safe in-memory cache of a player's real-time state.
///
/// Updated automatically in the background when WebSocket `PlayerUpdate` events are received.
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

    /// Updates internal metrics atomically.
    pub(crate) fn update_state(&self, time: u64, position: u64, connected: bool, ping: i64) {
        self.time.store(time, Ordering::Release);
        self.position.store(position, Ordering::Release);
        self.connected.store(connected, Ordering::Release);
        self.ping.store(ping, Ordering::Release);
    }

    /// Sets the pause flag atomically.
    pub(crate) fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Release);
    }

    /// Returns the current playback position in milliseconds.
    pub fn position(&self) -> u64 {
        self.position.load(Ordering::Acquire)
    }

    /// Returns the voice round-trip latency to Discord in milliseconds.
    pub fn ping(&self) -> i64 {
        self.ping.load(Ordering::Acquire)
    }

    /// Returns whether the player is currently connected to a Discord voice server.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    /// Returns whether playback is currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Acquire)
    }

    /// Returns the Unix timestamp (milliseconds) of the last update received from the node.
    pub fn last_updated_time(&self) -> u64 {
        self.time.load(Ordering::Acquire)
    }
}
