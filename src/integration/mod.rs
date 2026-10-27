//! Framework-specific integrations and voice event collectors.
#[cfg(feature = "twilight")]
pub mod twilight;

#[cfg(feature = "twilight")]
pub use twilight::VoiceUpdateCollector;
