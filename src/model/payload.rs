use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSessionPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resuming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoded_track: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<PlayerFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceStatePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceStatePayload {
    pub token: String,
    pub endpoint: String,
    pub session_id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equalizer: Option<Vec<EqualizerBand>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub karaoke: Option<KaraokeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timescale: Option<TimescaleFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tremolo: Option<TremoloFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vibrato: Option<VibratoFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<RotationFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distortion: Option<DistortionFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_mix: Option<ChannelMixFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_pass: Option<LowPassFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerBand {
    pub band: u8,
    pub gain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KaraokeFilter {
    pub level: Option<f32>,
    pub mono_level: Option<f32>,
    pub filter_band: Option<f32>,
    pub filter_width: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimescaleFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TremoloFilter {
    pub frequency: Option<f32>,
    pub depth: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VibratoFilter {
    pub frequency: Option<f32>,
    pub depth: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RotationFilter {
    pub rotation_hz: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DistortionFilter {
    pub sin_offset: Option<f32>,
    pub sin_scale: Option<f32>,
    pub cos_offset: Option<f32>,
    pub cos_scale: Option<f32>,
    pub tan_offset: Option<f32>,
    pub tan_scale: Option<f32>,
    pub offset: Option<f32>,
    pub scale: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMixFilter {
    pub left_to_left: Option<f32>,
    pub left_to_right: Option<f32>,
    pub right_to_left: Option<f32>,
    pub right_to_right: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LowPassFilter {
    pub smoothing: Option<f32>,
}
