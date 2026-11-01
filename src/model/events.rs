//! WebSocket event payloads sent by Lavalink v4 and NodeLink instances.

use super::track::Track;
use serde::Deserialize;
use std::time::Duration;

/// Root container for all unsolicited WebSocket messages dispatched by the node.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum NodeEvent {
    /// Emitted immediately after successful WebSocket handshake.
    #[serde(rename_all = "camelCase")]
    Ready {
        /// Whether the previous session was successfully recovered via Resuming.
        resumed: bool,
        /// Unique session token assigned by the node to this client connection.
        session_id: String,
    },
    /// Periodic status frame reporting current track position and connection quality.
    #[serde(rename_all = "camelCase")]
    PlayerUpdate {
        /// Target Discord guild snowflake ID as string.
        guild_id: String,
        /// Real-time playback status metrics.
        state: PlayerStateUpdate,
    },
    /// Node resource consumption metrics (CPU, RAM, active players).
    Stats(NodeStats),
    /// Audio player lifecycle event (track started, track finished, exceptions).
    Event(TrackEventPayload),
}

impl NodeEvent {
    /// Returns the target guild snowflake ID if the event is guild-specific.
    pub fn guild_id(&self) -> Option<&str> {
        match self {
            Self::PlayerUpdate { guild_id, .. } => Some(guild_id),
            Self::Event(ev) => Some(ev.guild_id()),
            Self::Ready { .. } | Self::Stats(_) => None,
        }
    }

    /// Returns `true` if this event is [`NodeEvent::Ready`].
    #[inline]
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}

/// Real-time playback status update reported inside [`NodeEvent::PlayerUpdate`].
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStateUpdate {
    /// Unix timestamp (milliseconds) representing when this state was measured.
    pub time: u64,
    /// Current playback position in milliseconds.
    pub position: u64,
    /// Indicates whether the node has an active UDP connection to Discord voice servers.
    pub connected: bool,
    /// Round-trip ping latency to Discord voice servers in milliseconds.
    pub ping: i64,
}

impl PlayerStateUpdate {
    /// Returns current playback position as a [`Duration`].
    #[inline]
    pub fn position_duration(&self) -> Duration {
        Duration::from_millis(self.position)
    }
}

/// Specific audio playback events dispatched under [`NodeEvent::Event`].
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum TrackEventPayload {
    /// Dispatched when an audio track begins active playback on a player.
    #[serde(rename_all = "camelCase")]
    TrackStartEvent {
        /// Target Discord guild snowflake ID.
        guild_id: String,
        /// The track that started playing.
        track: Track,
    },
    /// Dispatched when an audio track finishes playing, fails, or is stopped.
    #[serde(rename_all = "camelCase")]
    TrackEndEvent {
        /// Target Discord guild snowflake ID.
        guild_id: String,
        /// The track that finished or was interrupted.
        track: Track,
        /// The explicit reason why playback stopped.
        reason: TrackEndReason,
    },
    /// Dispatched when track playback crashes due to an upstream decoding or network error.
    #[serde(rename_all = "camelCase")]
    TrackExceptionEvent {
        /// Target Discord guild snowflake ID.
        guild_id: String,
        /// The track that triggered the exception.
        track: Track,
        /// Node error details describing the failure.
        exception: serde_json::Value,
    },
    /// Dispatched when a track stalls and fails to deliver frames within a timeout window.
    #[serde(rename_all = "camelCase")]
    TrackStuckEvent {
        /// Target Discord guild snowflake ID.
        guild_id: String,
        /// The stalled track.
        track: Track,
        /// Duration threshold in milliseconds before the event was fired.
        threshold_ms: u64,
    },
    /// Dispatched when the underlying Discord voice WebSocket connection is severed.
    #[serde(rename_all = "camelCase")]
    WebSocketClosedEvent {
        /// Target Discord guild snowflake ID.
        guild_id: String,
        /// Discord Gateway close code (e.g. 4014, 4006).
        code: u16,
        /// Text reason provided by the voice gateway.
        reason: String,
        /// Whether the connection was closed by Discord's remote end.
        by_remote: bool,
    },
}

impl TrackEventPayload {
    /// Returns the target guild snowflake ID associated with this event.
    pub fn guild_id(&self) -> &str {
        match self {
            Self::TrackStartEvent { guild_id, .. }
            | Self::TrackEndEvent { guild_id, .. }
            | Self::TrackExceptionEvent { guild_id, .. }
            | Self::TrackStuckEvent { guild_id, .. }
            | Self::WebSocketClosedEvent { guild_id, .. } => guild_id,
        }
    }

    /// Returns a reference to the associated track, if applicable.
    pub fn track(&self) -> Option<&Track> {
        match self {
            Self::TrackStartEvent { track, .. }
            | Self::TrackEndEvent { track, .. }
            | Self::TrackExceptionEvent { track, .. }
            | Self::TrackStuckEvent { track, .. } => Some(track),
            Self::WebSocketClosedEvent { .. } => None,
        }
    }
}

/// Termination reason supplied by [`TrackEventPayload::TrackEndEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrackEndReason {
    /// Track played to its natural conclusion without interruptions.
    Finished,
    /// Track encountered a fatal error during decoding or playback start.
    LoadFailed,
    /// Track was explicitly halted via a stop request.
    Stopped,
    /// Track was interrupted by a new incoming track request.
    Replaced,
    /// Player was destroyed or garbage-collected due to an idle channel timeout.
    Cleanup,
}

impl TrackEndReason {
    /// Returns `true` if the queue pipeline should proceed to play the next track.
    ///
    /// Evaluates to `true` for [`TrackEndReason::Finished`] and [`TrackEndReason::LoadFailed`].
    #[inline]
    pub fn may_start_next(self) -> bool {
        matches!(self, Self::Finished | Self::LoadFailed)
    }
}

/// Health and load statistics emitted periodically by the node instance.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeStats {
    /// Total number of guild players allocated on the node.
    pub players: u32,
    /// Number of guild players actively streaming audio frames.
    pub playing_players: u32,
    /// Total node server uptime in milliseconds.
    pub uptime: u64,
    /// Java Virtual Machine (or Node.js process) heap memory usage.
    pub memory: MemoryStats,
    /// Machine and process CPU utilization rates.
    pub cpu: CpuStats,
}

impl NodeStats {
    /// Returns the node uptime as a [`Duration`].
    #[inline]
    pub fn uptime_duration(&self) -> Duration {
        Duration::from_millis(self.uptime)
    }
}

/// Memory metrics reported by the audio node.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MemoryStats {
    /// Free memory available to the node in bytes.
    pub free: u64,
    /// Memory actively consumed by the node in bytes.
    pub used: u64,
    /// Total committed memory allocated to the node process in bytes.
    pub allocated: u64,
    /// Total reservable physical memory on the host in bytes.
    pub reservable: u64,
}

impl MemoryStats {
    /// Memory actively consumed by the node, in megabytes.
    #[inline]
    pub fn used_mb(&self) -> u64 {
        self.used / (1024 * 1024)
    }

    /// Total committed memory allocated to the node process, in megabytes.
    #[inline]
    pub fn allocated_mb(&self) -> u64 {
        self.allocated / (1024 * 1024)
    }
}

/// CPU utilization metrics reported by the audio node.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CpuStats {
    /// Number of physical or virtual CPU cores available to the node host.
    pub cores: u32,
    /// System-wide load average across all processes (`0.0` to `1.0`).
    pub system_load: f64,
    /// Audio node process CPU utilization (`0.0` to `1.0`).
    pub lavalink_load: f64,
}
