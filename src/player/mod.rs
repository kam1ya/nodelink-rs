//! Guild player controls, in-memory state caching, and audio filter pipeline.

pub mod filter;
pub mod state;

use crate::error::NodeResult;
use crate::model::payload::{UpdatePlayerPayload, VoiceStatePayload};
use crate::model::track::Track;
use crate::net::RestClient;
pub use filter::FilterPipeline;
pub use state::PlayerState;
use std::sync::Arc;

/// High-level control handle for a single Discord guild's audio player.
///
/// Provides ergonomic async methods for playback control, volume adjustment,
/// position seeking, DSP filter configuration, and cached state querying.
#[derive(Clone)]
pub struct GuildPlayer {
    /// Target Discord Guild snowflake ID.
    pub guild_id: u64,
    rest: Arc<RestClient>,
    /// Thread-safe in-memory cache of this player's metrics.
    pub state: Arc<PlayerState>,
}

impl GuildPlayer {
    pub(crate) fn new(guild_id: u64, rest: Arc<RestClient>, state: Arc<PlayerState>) -> Self {
        Self {
            guild_id,
            rest,
            state,
        }
    }

    /// Starts playback of the given [`Track`].
    ///
    /// Updates the player's internal pause flag to `false` and dispatches
    /// an `update_player` REST request with the base64-encoded track buffer.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError::SessionNotReady`] if no active WebSocket session exists,
    /// or [`crate::error::NodeLinkError::Api`] if the node returns an error response.
    pub async fn play(&self, track: &Track) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            encoded_track: Some(Some(track.encoded.clone())),
            ..Default::default()
        };
        self.state.set_paused(false);
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Sets playback pause state.
    ///
    /// Updates the cached atomic pause state and informs the remote audio node.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] on network or API failures.
    pub async fn set_pause(&self, paused: bool) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            paused: Some(paused),
            ..Default::default()
        };
        self.state.set_paused(paused);
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Sets audio playback volume.
    ///
    /// The value is clamped to the range `0..=1000`, where `100` represents normal 100% volume.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] on network or API failures.
    pub async fn set_volume(&self, volume: u16) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            volume: Some(volume.clamp(0, 1000)),
            ..Default::default()
        };
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Seeks to a specific track position in milliseconds.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] on network or API failures.
    pub async fn seek(&self, position_ms: u64) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            position: Some(position_ms),
            ..Default::default()
        };
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Stops playback and clears the currently loaded track on the node.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] on network or API failures.
    pub async fn stop(&self) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            encoded_track: Some(None),
            ..Default::default()
        };
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Forwards Discord voice connection tokens and endpoint credentials to the node.
    ///
    /// This establishes the audio pipe between the node and Discord voice servers.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] on network or API failures.
    pub async fn provide_voice(&self, voice: VoiceStatePayload) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            voice: Some(voice),
            ..Default::default()
        };
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Returns a fluent builder for configuring and applying audio DSP filters.
    #[inline]
    pub fn filters(&self) -> FilterPipeline {
        FilterPipeline::new(self.guild_id, self.rest.clone())
    }

    /// Reads the cached playback position in milliseconds with zero network latency overhead.
    #[inline]
    pub fn position(&self) -> u64 {
        self.state.position()
    }

    /// Reads the cached voice round-trip ping in milliseconds with zero network latency overhead.
    #[inline]
    pub fn ping(&self) -> i64 {
        self.state.ping()
    }

    /// Destroys this player on the remote node and frees node-side resources.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] on network or API failures.
    pub async fn destroy(&self) -> NodeResult<()> {
        self.rest.destroy_player(self.guild_id).await
    }
}
