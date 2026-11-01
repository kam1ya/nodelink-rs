//! REST payload DTOs for updating node sessions, players, voice states, and audio filters.

use serde::{Deserialize, Serialize};

/// Payload sent to update the current node session configuration.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSessionPayload {
    /// Whether session resuming is enabled for the current connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resuming: Option<bool>,
    /// Timeout in seconds before the session is automatically closed on the node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

/// Parameters for updating player playback, filters, and voice connection state.
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerPayload {
    /// Base64-encoded audio track buffer to play, or `Some(None)` to stop/clear playback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded_track: Option<Option<String>>,
    /// Track playback position to seek to, in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    /// Track position in milliseconds where playback should terminate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    /// Output volume level in range `0..=1000` (100 is default / 100%).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u16>,
    /// Whether playback is paused.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    /// Audio digital signal processing (DSP) filter configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<PlayerFilters>,
    /// Discord voice server and state connection credentials.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceStatePayload>,
}

/// Discord voice server connection credentials required by the node's audio engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceStatePayload {
    /// Voice channel authorization token provided by Discord (`VOICE_SERVER_UPDATE`).
    pub token: String,
    /// Discord voice server endpoint hostname (e.g. `us-east-1.discord.gg`).
    pub endpoint: String,
    /// Discord voice session ID provided by Discord (`VOICE_STATE_UPDATE`).
    pub session_id: String,
}

/// Collection of audio DSP filters applied during track playback.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerFilters {
    /// Master volume scaling multiplier (`0.0` to `5.0`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f32>,
    /// 15-band frequency equalizer adjustments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equalizer: Option<Vec<EqualizerBand>>,
    /// Vocal suppression or isolation filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub karaoke: Option<KaraokeFilter>,
    /// Speed, pitch, and rate transformations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timescale: Option<TimescaleFilter>,
    /// Volume oscillation effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tremolo: Option<TremoloFilter>,
    /// Pitch oscillation effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vibrato: Option<VibratoFilter>,
    /// Audio rotation and 8D panning effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<RotationFilter>,
    /// Sine, cosine, and tangent audio distortion waveshaping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distortion: Option<DistortionFilter>,
    /// Channel matrix mapping for stereo channel mixing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_mix: Option<ChannelMixFilter>,
    /// Low-pass filter to smooth or dampen high frequency sounds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_pass: Option<LowPassFilter>,
}

/// Gain setting for a single equalizer frequency band.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerBand {
    /// Band index in range `0..=14` (corresponding to frequencies from 25Hz to 16kHz).
    pub band: u8,
    /// Gain multiplier in range `-0.25..=1.0` (where `0.0` is neutral).
    pub gain: f64,
}

/// Filter configuration for vocal suppression or isolation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KaraokeFilter {
    /// Level of vocal suppression (`0.0` is neutral, `1.0` is maximum suppression).
    pub level: Option<f32>,
    /// Mono level of the processed audio (`0.0` to `1.0`).
    pub mono_level: Option<f32>,
    /// Target frequency band center in Hertz.
    pub filter_band: Option<f32>,
    /// Frequency width of the vocal filter band.
    pub filter_width: Option<f32>,
}

/// Timescale filter modifying playback speed, pitch, and playback rate independently.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimescaleFilter {
    /// Playback speed factor (`0.1` to `5.0`, where `1.0` is normal speed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// Pitch shift multiplier (`0.1` to `5.0`, where `1.0` is original pitch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,
    /// Playback rate factor (`0.1` to `5.0`, where `1.0` is original rate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
}

/// Tremolo filter producing a pulsating volume oscillation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TremoloFilter {
    /// Oscillation frequency in Hertz (must be greater than `0.0`).
    pub frequency: Option<f32>,
    /// Modulation depth (`0.0` to `1.0`, where `0.0` is no effect).
    pub depth: Option<f32>,
}

/// Vibrato filter producing an oscillating pitch modulation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VibratoFilter {
    /// Oscillation frequency in Hertz (`0.0` to `14.0`).
    pub frequency: Option<f32>,
    /// Modulation depth (`0.0` to `1.0`, where `0.0` is no effect).
    pub depth: Option<f32>,
}

/// Audio rotation filter producing panning and 8D surround audio effects.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RotationFilter {
    /// Frequency of rotation in Hertz (e.g. `0.2` for one complete rotation every 5 seconds).
    pub rotation_hz: Option<f64>,
}

/// Audio distortion filter using mathematical function waveshaping.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DistortionFilter {
    /// Sine wave offset.
    pub sin_offset: Option<f32>,
    /// Sine wave scaling factor.
    pub sin_scale: Option<f32>,
    /// Cosine wave offset.
    pub cos_offset: Option<f32>,
    /// Cosine wave scaling factor.
    pub cos_scale: Option<f32>,
    /// Tangent wave offset.
    pub tan_offset: Option<f32>,
    /// Tangent wave scaling factor.
    pub tan_scale: Option<f32>,
    /// Overall output offset.
    pub offset: Option<f32>,
    /// Overall output scaling factor.
    pub scale: Option<f32>,
}

/// Stereo channel mixer matrix allowing redistribution of left and right audio channels.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMixFilter {
    /// Portion of left channel passed to left output (`0.0` to `1.0`).
    pub left_to_left: Option<f32>,
    /// Portion of left channel passed to right output (`0.0` to `1.0`).
    pub left_to_right: Option<f32>,
    /// Portion of right channel passed to left output (`0.0` to `1.0`).
    pub right_to_left: Option<f32>,
    /// Portion of right channel passed to right output (`0.0` to `1.0`).
    pub right_to_right: Option<f32>,
}

/// Low-pass filter attenuating higher audio frequencies.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LowPassFilter {
    /// Frequency smoothing factor (must be greater than `1.0`).
    pub smoothing: Option<f32>,
}
