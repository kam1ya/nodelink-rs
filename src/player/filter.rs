//! Fluent audio filter pipeline builder.

use crate::error::NodeResult;
use crate::model::payload::{EqualizerBand, PlayerFilters, TimescaleFilter, UpdatePlayerPayload};
use crate::net::RestClient;
use std::sync::Arc;

/// Fluent builder for applying audio processing filters to a player.
pub struct FilterPipeline {
    guild_id: u64,
    rest: Arc<RestClient>,
    filters: PlayerFilters,
}

impl FilterPipeline {
    pub(crate) fn new(guild_id: u64, rest: Arc<RestClient>) -> Self {
        Self {
            guild_id,
            rest,
            filters: PlayerFilters::default(),
        }
    }

    /// Enables or disables a pre-configured 4-band bass boost preset.
    pub fn bass_boost(mut self, enabled: bool) -> Self {
        if enabled {
            self.filters.equalizer = Some(vec![
                EqualizerBand { band: 0, gain: 0.2 },
                EqualizerBand {
                    band: 1,
                    gain: 0.15,
                },
                EqualizerBand { band: 2, gain: 0.1 },
                EqualizerBand {
                    band: 3,
                    gain: 0.05,
                },
            ]);
        } else {
            self.filters.equalizer = Some(vec![]);
        }
        self
    }

    /// Sets playback speed multiplier (clamped to `0.1..=5.0`).
    pub fn speed(mut self, speed: f64) -> Self {
        let entry = self
            .filters
            .timescale
            .get_or_insert_with(TimescaleFilter::default);
        entry.speed = Some(speed.clamp(0.1, 5.0));
        self
    }

    /// Enables or disables the Nightcore audio preset (1.2x speed + 1.25x pitch).
    pub fn nightcore(mut self, enabled: bool) -> Self {
        if enabled {
            self.filters.timescale = Some(TimescaleFilter {
                speed: Some(1.2),
                pitch: Some(1.25),
                rate: Some(1.0),
            });
        } else {
            self.filters.timescale = None;
        }
        self
    }

    /// Dispatches the accumulated filter payload to the node.
    pub async fn apply(self) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            filters: Some(self.filters),
            ..Default::default()
        };
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }

    /// Resets all active filters on the node back to neutral.
    pub async fn reset(self) -> NodeResult<()> {
        let payload = UpdatePlayerPayload {
            filters: Some(PlayerFilters::default()),
            ..Default::default()
        };
        self.rest
            .update_player(self.guild_id, &payload, false)
            .await
    }
}
