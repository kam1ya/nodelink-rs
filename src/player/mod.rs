//! Guild player controls, in-memory state caching, and audio filter pipeline.

pub mod filter;
pub mod state;

use crate::error::NodeResult;
use crate::model::payload::{UpdatePlayerPayload, VoiceStatePayload};
use crate::model::track::Track;
use crate::net::RestClient;
use crate::player::filter::FilterPipeline;
pub use state::PlayerState;
use std::sync::Arc;

/// High-level control handle for a single Discord guild's audio player.
#[derive(Clone)]
pub struct GuildPlayer {
    /// Target Discord Guild ID.
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
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] if the REST call fails or session is not ready.
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

    /// Sets audio playback volume (clamped to range `0..=1000`, 100 is default).
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
    pub async fn seek(&self, position_ms: u64) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            position: Some(position_ms),
            ..Default::default()
        };
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Stops playback and clears the active track on the node.
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
    pub async fn provide_voice(&self, voice: VoiceStatePayload) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            voice: Some(voice),
            ..Default::default()
        };
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Returns a fluent builder for configuring audio filters (equalizer, timescale, etc.).
    pub fn filters(&self) -> FilterPipeline {
        FilterPipeline::new(self.guild_id, self.rest.clone())
    }

    /// Reads the cached playback position in milliseconds with zero latency overhead.
    #[inline]
    pub fn position(&self) -> u64 {
        self.state.position()
    }

    /// Reads the cached voice ping in milliseconds with zero latency overhead.
    #[inline]
    pub fn ping(&self) -> i64 {
        self.state.ping()
    }

    /// Destroys this player on the remote node and frees node-side resources.
    pub async fn destroy(&self) -> NodeResult<()> {
        self.rest.destroy_player(self.guild_id).await
    }
}
