//! Fluent audio filter pipeline builder.

use crate::error::NodeResult;
use crate::model::payload::{
    ChannelMixFilter, DistortionFilter, EqualizerBand, KaraokeFilter, LowPassFilter, PlayerFilters,
    RotationFilter, TimescaleFilter, TremoloFilter, UpdatePlayerPayload, VibratoFilter,
};
use crate::net::RestClient;
use std::sync::Arc;

pub use crate::model::payload::{
    ChannelMixFilter as ChannelMix, DistortionFilter as Distortion, EqualizerBand as EqBand,
    KaraokeFilter as Karaoke, LowPassFilter as LowPass, PlayerFilters as Filters,
    RotationFilter as Rotation, TimescaleFilter as Timescale, TremoloFilter as Tremolo,
    VibratoFilter as Vibrato,
};

/// Fluent builder for constructing and applying audio processing filters to a [`crate::GuildPlayer`].
///
/// # Example
///
/// ```no_run
/// # use nodelink_rs::Client;
/// # async fn doc_example(client: &Client, guild_id: u64) -> Result<(), Box<dyn std::error::Error>> {
/// let player = client.player(guild_id);
///
/// // Chain filter configurations and dispatch to the node
/// player.filters()
///     .volume(1.2)
///     .speed(1.15)
///     .bass_boost(true)
///     .apply()
///     .await?;
///
/// // Reset filters back to default
/// player.filters().reset().await?;
/// # Ok(())
/// # }
/// ```
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

    /// Sets the master volume multiplier (`0.0` to `5.0`).
    pub fn volume(mut self, volume: f32) -> Self {
        self.filters.volume = Some(volume.clamp(0.0, 5.0));
        self
    }

    /// Sets custom equalizer frequency bands (`0..=14`, gain `-0.25..=1.0`).
    pub fn equalizer(mut self, bands: Vec<EqualizerBand>) -> Self {
        self.filters.equalizer = Some(bands);
        self
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

    /// Sets pitch shift multiplier (clamped to `0.1..=5.0`).
    pub fn pitch(mut self, pitch: f64) -> Self {
        let entry = self
            .filters
            .timescale
            .get_or_insert_with(TimescaleFilter::default);
        entry.pitch = Some(pitch.clamp(0.1, 5.0));
        self
    }

    /// Sets playback rate factor (clamped to `0.1..=5.0`).
    pub fn rate(mut self, rate: f64) -> Self {
        let entry = self
            .filters
            .timescale
            .get_or_insert_with(TimescaleFilter::default);
        entry.rate = Some(rate.clamp(0.1, 5.0));
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

    /// Applies karaoke vocal suppression parameters.
    pub fn karaoke(mut self, karaoke: KaraokeFilter) -> Self {
        self.filters.karaoke = Some(karaoke);
        self
    }

    /// Configures tremolo volume oscillation effect.
    pub fn tremolo(mut self, frequency: f32, depth: f32) -> Self {
        self.filters.tremolo = Some(TremoloFilter {
            frequency: Some(frequency),
            depth: Some(depth.clamp(0.0, 1.0)),
        });
        self
    }

    /// Configures vibrato pitch oscillation effect.
    pub fn vibrato(mut self, frequency: f32, depth: f32) -> Self {
        self.filters.vibrato = Some(VibratoFilter {
            frequency: Some(frequency.clamp(0.0, 14.0)),
            depth: Some(depth.clamp(0.0, 1.0)),
        });
        self
    }

    /// Configures 8D audio / panning rotation frequency in Hertz.
    pub fn rotation(mut self, rotation_hz: f64) -> Self {
        self.filters.rotation = Some(RotationFilter {
            rotation_hz: Some(rotation_hz),
        });
        self
    }

    /// Configures custom mathematical distortion waveshaping.
    pub fn distortion(mut self, distortion: DistortionFilter) -> Self {
        self.filters.distortion = Some(distortion);
        self
    }

    /// Configures stereo channel mixing matrix.
    pub fn channel_mix(mut self, mix: ChannelMixFilter) -> Self {
        self.filters.channel_mix = Some(mix);
        self
    }

    /// Configures low-pass frequency filter.
    pub fn low_pass(mut self, smoothing: f32) -> Self {
        self.filters.low_pass = Some(LowPassFilter {
            smoothing: Some(smoothing),
        });
        self
    }

    /// Directly overrides the underlying filter configuration.
    pub fn custom(mut self, filters: PlayerFilters) -> Self {
        self.filters = filters;
        self
    }

    /// Dispatches the accumulated filter payload to the node.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] if the REST request fails or the session is not ready.
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
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::NodeLinkError`] if the REST request fails or the session is not ready.
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
