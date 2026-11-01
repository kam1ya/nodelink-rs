//! # nodelink-rs
//!
//! High-performance, ergonomic asynchronous client for [Lavalink v4](https://lavalink.dev/)
//! and [NodeLink](https://github.com/PerformanC/NodeLink) audio nodes in Rust.
//!
//! ## Key Architectural Features
//!
//! - **Strict Protocol DTOs**: Comprehensive types for Lavalink v4 / NodeLink REST endpoints and WebSocket events.
//! - **Automatic Session Recovery**: Background WebSocket supervisor transparently resumes sessions upon reconnect.
//! - **Lock-Free State Caching**: Read playback position, ping, and connection state with atomic operations and zero network overhead.
//! - **Fluent Audio Filters**: Chain equalizer presets, speed, pitch, nightcore, rotation (8D), and distortion DSP filters.
//! - **Framework Agnostic**: Clean abstractions with optional native helpers for [Twilight](https://twilight.rs).
//!
//! ## Cargo Features
//!
//! | Feature | Default | Description |
//! | :--- | :---: | :--- |
//! | `rustls-tls` | **Yes** | Uses `rustls` with platform-native certificates for TLS connections. |
//! | `native-tls` | No | Uses system-native TLS implementation (Schannel on Windows, OpenSSL on Linux). |
//! | `twilight` | No | Enables [`integration::VoiceUpdateCollector`] for Twilight gateway voice events. |
//!
//! ## Quick Start
//!
//! ```no_run
//! use nodelink_rs::prelude::*;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Initialize client configuration
//!     let config = Config::new("127.0.0.1:2333", "youshallnotpass", 123456789012345678);
//!     let (client, mut events) = Client::connect(config).await?;
//!
//!     // 2. Spawn a background listener for node lifecycle and playback events
//!     tokio::spawn(async move {
//!         while let Ok(event) = events.recv().await {
//!             match event {
//!                 NodeEvent::Ready { session_id, resumed } => {
//!                     println!("Connected! Session: {session_id} (resumed: {resumed})");
//!                 }
//!                 NodeEvent::PlayerUpdate { guild_id, state } => {
//!                     println!("Guild {guild_id}: pos={}ms ping={}ms", state.position, state.ping);
//!                 }
//!                 NodeEvent::Event(event) => {
//!                     println!("Player event: {event:?}");
//!                 }
//!                 _ => {}
//!             }
//!         }
//!     });
//!
//!     // 3. Search for tracks via YouTube provider
//!     let result = client
//!         .load_tracks(Query::youtube("Rick Astley - Never Gonna Give You Up"))
//!         .await?;
//!
//!     if let LoadResult::Search(tracks) = result {
//!         if let Some(track) = tracks.first() {
//!             let guild_id = 112233445566778899;
//!             let player = client.player(guild_id);
//!
//!             // 4. Play track, configure volume, and apply audio filters
//!             player.play(track).await?;
//!             player.set_volume(80).await?;
//!             player.filters().bass_boost(true).apply().await?;
//!
//!             // Read cached state with zero REST latency
//!             println!("Current latency ping: {}ms", player.ping());
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod error;

#[cfg(feature = "twilight")]
pub mod integration;

pub mod model;

pub(crate) mod net;

pub mod player;

use dashmap::DashMap;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub use error::{NodeLinkError, NodeResult};
pub use model::events::NodeEvent;
pub use model::query::{Query, SearchSource};
pub use model::track::LoadResult;
pub use player::{FilterPipeline, GuildPlayer, PlayerState};

use crate::net::{Connection, RestClient};

/// Connection parameters required to connect to a Lavalink v4 or NodeLink instance.
#[derive(Debug, Clone)]
pub struct Config {
    /// Host address of the node including port (e.g., `"127.0.0.1:2333"` or `"lavalink.example.com"`).
    pub host: String,
    /// Authorization password configured on the node.
    pub password: String,
    /// Discord Application / Bot User ID.
    pub user_id: u64,
    /// Whether to use secure protocols (`https://` and `wss://`).
    pub secure: bool,
}

impl Config {
    /// Creates a new configuration with standard non-TLS protocols (`http://` and `ws://`).
    pub fn new(host: impl Into<String>, password: impl Into<String>, user_id: u64) -> Self {
        Self {
            host: host.into(),
            password: password.into(),
            user_id,
            secure: false,
        }
    }

    /// Toggles TLS support (`https://` and `wss://`).
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }
}

/// The main client coordinating REST calls, WebSocket lifecycles, and guild players.
pub struct Client {
    rest: Arc<RestClient>,
    events_tx: broadcast::Sender<NodeEvent>,
    states: Arc<DashMap<u64, Arc<PlayerState>>>,
}

impl Client {
    /// Establishes an asynchronous connection to the audio node.
    ///
    /// Spawns a resilient background task that performs WebSocket handshakes,
    /// enables session resuming, maintains periodic heartbeats, and routes events.
    ///
    /// # Errors
    ///
    /// Returns [`NodeLinkError::Api`] if the authorization header contains invalid characters,
    /// or [`NodeLinkError::Http`] if the HTTP client builder fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use nodelink_rs::{Client, Config};
    /// # async fn doc_example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::new("127.0.0.1:2333", "youshallnotpass", 123456789012345678);
    /// let (client, mut events) = Client::connect(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(
        config: Config,
    ) -> NodeResult<(Arc<Self>, broadcast::Receiver<NodeEvent>)> {
        let (events_tx, events_rx) = broadcast::channel(512);
        let session_id = Arc::new(RwLock::new(None));
        let states = Arc::new(DashMap::new());

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&config.password).map_err(|_| NodeLinkError::Api {
                status: 400,
                message: "Invalid characters in password".into(),
            })?,
        );

        let raw_http = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let http_proto = if config.secure { "https" } else { "http" };
        let base_url = format!("{}://{}", http_proto, config.host);

        let rest = Arc::new(RestClient::new(base_url, raw_http, session_id.clone()));

        let ws_proto = if config.secure { "wss" } else { "ws" };
        let ws_url = format!("{}://{}/v4/websocket", ws_proto, config.host);

        Connection::spawn(
            ws_url,
            config.password,
            config.user_id,
            session_id,
            events_tx.clone(),
            states.clone(),
            rest.clone(),
        );

        let client = Arc::new(Self {
            rest,
            events_tx,
            states,
        });
        Ok((client, events_rx))
    }

    /// Obtains a [`GuildPlayer`] handle for dispatching playback commands to a specific Discord guild.
    ///
    /// If no player state exists in memory for this guild, an uninitialized state container is created.
    pub fn player(&self, guild_id: u64) -> GuildPlayer {
        let state = self
            .states
            .entry(guild_id)
            .or_insert_with(|| Arc::new(PlayerState::new()))
            .clone();

        GuildPlayer::new(guild_id, self.rest.clone(), state)
    }

    /// Resolves tracks, playlists, or search terms using the node's `/v4/loadtracks` endpoint.
    ///
    /// # Arguments
    ///
    /// * `query` - Anything converting into [`Query`] (`&str`, `String`, or explicit [`Query`] variants).
    ///
    /// # Errors
    ///
    /// Returns [`NodeLinkError::Http`] on transport errors or [`NodeLinkError::Api`] if the node returns a non-2xx status.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use nodelink_rs::{Client, Query};
    /// # async fn doc_example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let tracks = client.load_tracks(Query::youtube("Bohemian Rhapsody")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_tracks(&self, query: impl Into<Query>) -> NodeResult<LoadResult> {
        let identifier = query.into().into_identifier();
        self.rest.load_tracks(&identifier).await
    }

    /// Subscribes to the broadcast channel of node events.
    ///
    /// Can be called multiple times to spawn independent event listener tasks.
    pub fn subscribe(&self) -> broadcast::Receiver<NodeEvent> {
        self.events_tx.subscribe()
    }
}

/// Convenient re-exports of primary types.
pub mod prelude {
    pub use crate::error::{NodeLinkError, NodeResult};
    pub use crate::model::events::NodeEvent;
    pub use crate::model::query::{Query, SearchSource};
    pub use crate::model::track::{LoadResult, Track, TrackInfo};
    pub use crate::player::{FilterPipeline, GuildPlayer, PlayerState};
    pub use crate::{Client, Config};
}
