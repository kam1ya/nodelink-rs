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
/// Discord Gateway sends voice initialization across two distinct events:
/// [`VoiceStateUpdate`] (providing the session ID) and [`VoiceServerUpdate`] (providing the voice token and endpoint).
/// This collector correlates them and yields a complete [`VoiceStatePayload`] once both arrive.
#[derive(Default)]
pub struct VoiceUpdateCollector {
    states: DashMap<u64, PendingVoice>,
}

impl VoiceUpdateCollector {
    /// Creates a new empty collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Processes an incoming [`VoiceStateUpdate`] event from Discord Gateway.
    ///
    /// Returns `Some((guild_id, payload))` if the corresponding server update was already received.
    pub fn process_state(&self, event: &VoiceStateUpdate) -> Option<(u64, VoiceStatePayload)> {
        let guild_id = event.guild_id?.get();
        let mut entry = self.states.entry(guild_id).or_default();
        entry.session_id = Some(event.session_id.clone());

        Self::try_complete(guild_id, &entry)
    }

    /// Processes an incoming [`VoiceServerUpdate`] event from Discord Gateway.
    ///
    /// Returns `Some((guild_id, payload))` if the corresponding state update was already received.
    pub fn process_server(&self, event: &VoiceServerUpdate) -> Option<(u64, VoiceStatePayload)> {
        let guild_id = event.guild_id.get();
        let endpoint = event.endpoint.clone()?;

        let mut entry = self.states.entry(guild_id).or_default();
        entry.token = Some(event.token.clone());
        entry.endpoint = Some(endpoint);

        Self::try_complete(guild_id, &entry)
    }

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
