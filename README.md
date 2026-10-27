# nodelink-rs

[![Crates.io](https://img.shields.io/crates/v/nodelink-rs.svg)](https://crates.io/crates/nodelink-rs)
[![Documentation](https://docs.rs/nodelink-rs/badge.svg)](https://docs.rs/nodelink-rs)

High-performance client for [Lavalink v4](https://lavalink.dev/) and [NodeLink](https://github.com/PerformanC/NodeLink) audio nodes.

Designed for production Discord bots: zero-allocation deserialization, automatic session recovery, in-memory player state tracking, and a fluent filter pipeline.

---

## Features

* **Full Protocol Compatibility:** Strict DTOs for Lavalink v4 / NodeLink REST endpoints and WebSocket events.
* **Session Resuming:** Automatic session recovery on reconnection to avoid interrupting active playback during network drops.
* **In-Memory State Caching:** Instant reads for `player.position()` and `player.ping()` without REST roundtrips.
* **Audio Filter Pipeline:** Builder API for volume, speed, pitch, nightcore, equalizers, and karaoke.
* **Framework Agnostic:** Seamless integration with Twilight, Serenity, Poise, or distributed microservice architectures.

---

## Installation

Add `nodelink-rs` to your `Cargo.toml`:

```toml
[dependencies]
nodelink-rs = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Cargo Feature Flags

* `rustls-tls` *(default)*: Uses `rustls` for secure HTTPS and WSS connections.
* `native-tls`: Uses system-native TLS backends.
* `twilight`: Enables Discord voice update collection helpers (`twilight-model`).

---

## Quick Start

```rust
use std::time::Duration;
use nodelink_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configure node credentials and connection details
    let config = Config::new("127.0.0.1:2333", "youshallnotpass", 123456789012345678);
    let (client, mut events) = Client::connect(config).await?;

    // 2. Spawn a background listener for node lifecycle and playback events
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                NodeEvent::Ready { session_id, resumed } => {
                    println!("Node ready! Session ID: {session_id} (resumed: {resumed})");
                }
                NodeEvent::PlayerUpdate { guild_id, state } => {
                    println!(
                        "Guild {guild_id}: position={}ms ping={}ms connected={}",
                        state.position, state.ping, state.connected
                    );
                }
                NodeEvent::Event(event) => {
                    println!("Player event received: {event:?}");
                }
                _ => {}
            }
        }
    });

    // 3. Search and load tracks using the ergonomic Query helper
    let result = client
        .load_tracks(Query::youtube("Rick Astley - Never Gonna Give You Up"))
        .await?;

    if let LoadResult::Search(tracks) = result {
        if let Some(track) = tracks.first() {
            let guild_id = 112233445566778899;
            let player = client.player(guild_id);

            // 4. Play track and apply audio filters
            player.play(track).await?;
            player.set_volume(80).await?;
            player.filters().bass_boost(true).apply().await?;

            println!("Playback started. Cached latency: {}ms", player.ping());
        }
    }

    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok(())
}
```

---

## Audio Filters

Apply real-time DSP audio manipulation via the fluent builder:

```rust
use nodelink_rs::Client;

async fn apply_custom_filters(client: &Client, guild_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    let player = client.player(guild_id);

    // Chain filter configurations
    player.filters()
        .speed(1.15)
        .nightcore(true)
        .bass_boost(true)
        .apply()
        .await?;

    // Reset all filters back to neutral
    player.filters().reset().await?;

    Ok(())
}
```

---

## Twilight Integration

Enable the `twilight` feature flag to correlate Discord voice handshake events:

```rust
use nodelink_rs::integration::VoiceUpdateCollector;
use nodelink_rs::Client;
use twilight_model::gateway::payload::incoming::{VoiceServerUpdate, VoiceStateUpdate};

pub async fn handle_gateway_events(
    client: &Client,
    collector: &VoiceUpdateCollector,
    voice_state: Option<&VoiceStateUpdate>,
    voice_server: Option<&VoiceServerUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(state_update) = voice_state {
        if let Some((guild_id, payload)) = collector.process_state(state_update) {
            client.player(guild_id).provide_voice(payload).await?;
        }
    }

    if let Some(server_update) = voice_server {
        if let Some((guild_id, payload)) = collector.process_server(server_update) {
            client.player(guild_id).provide_voice(payload).await?;
        }
    }

    Ok(())
}
```