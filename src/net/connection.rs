use crate::model::events::NodeEvent;
use crate::model::payload::UpdateSessionPayload;
use crate::model::{TrackEndReason, TrackEventPayload};
use crate::net::RestClient;
use crate::player::state::PlayerState;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Instant};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

pub(crate) struct Connection;

impl Connection {
    pub fn spawn(
        ws_url: String,
        password: String,
        user_id: u64,
        session_id: Arc<RwLock<Option<String>>>,
        events_tx: broadcast::Sender<NodeEvent>,
        states: Arc<DashMap<u64, Arc<PlayerState>>>,
        rest: Arc<RestClient>,
    ) {
        tokio::spawn(async move {
            let mut backoff = Duration::from_millis(500);

            loop {
                let start_time = Instant::now();
                let res = Self::connect_and_listen(
                    &ws_url,
                    &password,
                    user_id,
                    &session_id,
                    &events_tx,
                    &states,
                    &rest,
                )
                .await;

                if let Err(err) = res {
                    warn!(error = %err, retry_in = ?backoff, "WS interrupted, retrying...");
                    tokio::time::sleep(backoff).await;
                    backoff = if start_time.elapsed() > Duration::from_secs(30) {
                        Duration::from_millis(500)
                    } else {
                        (backoff * 2).min(Duration::from_secs(15))
                    }
                } else {
                    info!("WS connection closed cleanly. Reconnecting...");
                    backoff = Duration::from_millis(500);
                }
            }
        });
    }

    async fn build_request(
        ws_url: &str,
        password: &str,
        user_id: u64,
        session_id: &Arc<RwLock<Option<String>>>,
    ) -> Result<Request, Box<dyn std::error::Error + Send + Sync>> {
        let mut req = ws_url.into_client_request()?;
        let headers = req.headers_mut();

        headers.insert(AUTHORIZATION, HeaderValue::from_str(password)?);
        headers.insert("User-Id", HeaderValue::from_str(&user_id.to_string())?);
        headers.insert("Client-Name", HeaderValue::from_static("nodelink-rs/0.1.0"));

        if let Some(ref session) = *session_id.read().await {
            if let Ok(val) = HeaderValue::from_str(session) {
                headers.insert("Session", val);
            }
        }

        Ok(req)
    }

    async fn connect_and_listen(
        ws_url: &str,
        password: &str,
        user_id: u64,
        session_id: &Arc<RwLock<Option<String>>>,
        events_tx: &broadcast::Sender<NodeEvent>,
        states: &Arc<DashMap<u64, Arc<PlayerState>>>,
        rest: &Arc<RestClient>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = Self::build_request(ws_url, password, user_id, session_id).await?;
        let (ws_stream, _) = connect_async(request).await?;
        let (mut writer, mut reader) = ws_stream.split();

        let mut heartbeat = interval(Duration::from_secs(20));
        heartbeat.tick().await;

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    if writer.send(Message::Ping(vec![].into())).await.is_err() {
                        break;
                    }
                }
                msg = reader.next() => {
                    let Some(msg) = msg else { break };
                    let msg = msg?;

                    match msg {
                        Message::Text(raw) => {
                            Self::handle_text_payload(
                                raw.as_bytes(),
                                session_id,
                                events_tx,
                                states,
                                rest
                            ).await;
                        }
                        Message::Pong(payload) => {
                            let _ = writer.send(Message::Pong(payload)).await;
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_text_payload(
        bytes: &[u8],
        session_id: &Arc<RwLock<Option<String>>>,
        event_tx: &broadcast::Sender<NodeEvent>,
        states: &Arc<DashMap<u64, Arc<PlayerState>>>,
        rest: &Arc<RestClient>,
    ) {
        let event = match serde_json::from_slice::<NodeEvent>(bytes) {
            Ok(ev) => ev,
            Err(e) => {
                error!(error = %e, "Failed to parse Lavalink event");
                return;
            }
        };

        match &event {
            NodeEvent::Ready {
                session_id: sid, ..
            } => {
                Self::on_ready(sid, session_id, rest).await;
            }
            NodeEvent::PlayerUpdate { guild_id, state } => {
                if let Ok(gid) = guild_id.parse::<u64>() {
                    let entry = states
                        .entry(gid)
                        .or_insert_with(|| Arc::new(PlayerState::new()));
                    entry.update_state(state.time, state.position, state.connected, state.ping);
                }
            }
            NodeEvent::Event(TrackEventPayload::TrackEndEvent {
                guild_id, reason, ..
            }) => {
                if matches!(reason, TrackEndReason::Cleanup | TrackEndReason::Stopped) {
                    if let Ok(gid) = guild_id.parse::<u64>() {
                        states.remove(&gid);
                    }
                }
            }
            _ => {}
        }

        let _ = event_tx.send(event);
    }

    async fn on_ready(sid: &str, session_id: &Arc<RwLock<Option<String>>>, rest: &Arc<RestClient>) {
        let changed = {
            let current = session_id.read().await;
            current.as_deref() != Some(sid)
        };

        if changed {
            *session_id.write().await = Some(sid.to_string());
            info!(session_id = %sid, "Active session established");
        }

        let rest = rest.clone();
        tokio::spawn(async move {
            let payload = UpdateSessionPayload {
                resuming: Some(true),
                timeout: Some(60),
            };
            if let Err(e) = rest.update_session(&payload).await {
                warn!(error = %e, "Failed to set resuming on node");
            }
        });
    }
}
