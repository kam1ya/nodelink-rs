//! Optional integration helpers for the `twilight` ecosystem.

use crate::model::payload::VoiceStatePayload;
use dashmap::DashMap;
use twilight_model::gateway::payload::incoming::{VoiceServerUpdate, VoiceStateUpdate};

#[derive(Default)]
struct PendingVoice {
    token: Option<String>,
    endpoint: Option<String>,
    session_id: Option<String>,
}

/// A lightweight, lock-free collector for Discord's split voice handshake events.
///
/// Discord Gateway dispatches voice connection credentials across two distinct events:
/// 1. [`VoiceStateUpdate`] (providing the voice session ID)
/// 2. [`VoiceServerUpdate`] (providing the voice token and regional endpoint)
///
/// This collector correlates both updates across arbitrary arrival order and produces
/// a ready-to-dispatch [`VoiceStatePayload`] once both halves are present.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "twilight")]
/// # {
/// use nodelink_rs::integration::VoiceUpdateCollector;
/// use nodelink_rs::Client;
/// use twilight_model::gateway::payload::incoming::{VoiceServerUpdate, VoiceStateUpdate};
///
/// async fn handle_voice_server(
///     client: &Client,
///     collector: &VoiceUpdateCollector,
///     server: &VoiceServerUpdate,
/// ) -> Result<(), Box<dyn std::error::Error>> {
///     if let Some((guild_id, payload)) = collector.process_server(server) {
///         client.player(guild_id).provide_voice(payload).await?;
///     }
///     Ok(())
/// }
/// # }
/// ```
#[derive(Default)]
pub struct VoiceUpdateCollector {
    states: DashMap<u64, PendingVoice>,
}

impl VoiceUpdateCollector {
    /// Creates a new empty voice update collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Processes an incoming [`VoiceStateUpdate`] event from Discord Gateway.
    ///
    /// Stores the session ID for the associated guild. If a corresponding
    /// [`VoiceServerUpdate`] has already been recorded, returns `Some((guild_id, payload))`.
    pub fn process_state(&self, event: &VoiceStateUpdate) -> Option<(u64, VoiceStatePayload)> {
        let guild_id = event.guild_id?.get();
        let mut entry = self.states.entry(guild_id).or_default();
        entry.session_id = Some(event.session_id.clone());

        Self::try_complete(guild_id, &entry)
    }

    /// Processes an incoming [`VoiceServerUpdate`] event from Discord Gateway.
    ///
    /// Stores the authentication token and gateway endpoint for the guild.
    /// If a corresponding [`VoiceStateUpdate`] has already been recorded, returns `Some((guild_id, payload))`.
    pub fn process_server(&self, event: &VoiceServerUpdate) -> Option<(u64, VoiceStatePayload)> {
        let guild_id = event.guild_id.get();
        let endpoint = event.endpoint.clone()?;

        let mut entry = self.states.entry(guild_id).or_default();
        entry.token = Some(event.token.clone());
        entry.endpoint = Some(endpoint);

        Self::try_complete(guild_id, &entry)
    }

    /// Checks whether all three required fields (token, endpoint, session_id) are present.
    fn try_complete(guild_id: u64, pending: &PendingVoice) -> Option<(u64, VoiceStatePayload)> {
        let (Some(token), Some(endpoint), Some(session_id)) =
            (&pending.token, &pending.endpoint, &pending.session_id)
        else {
            return None;
        };

        Some((
            guild_id,
            VoiceStatePayload {
                token: token.clone(),
                endpoint: endpoint.clone(),
                session_id: session_id.clone(),
            },
        ))
    }

    /// Clears any cached handshake state for the specified guild.
    pub fn remove(&self, guild_id: u64) {
        self.states.remove(&guild_id);
    }
}
