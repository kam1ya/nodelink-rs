use nodelink_rs::model::events::NodeEvent;
use nodelink_rs::model::track::LoadResult;
use nodelink_rs::{Client, Config, Query};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging connection events and heartbeats
    tracing_subscriber::fmt::init();

    // 1. Configure the connection to your Lavalink v4 or NodeLink instance
    let config = Config::new("127.0.0.1:2333", "youshallnotpass", 123456789012345678);
    let (client, mut events) = Client::connect(config).await?;

    // 2. Spawn a background listener for node lifecycle and playback events
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                NodeEvent::Ready {
                    session_id,
                    resumed,
                } => {
                    println!("Node ready! Session ID: {session_id} (resumed: {resumed})");
                }
                NodeEvent::PlayerUpdate { guild_id, state } => {
                    println!(
                        "Guild {guild_id} update -> position: {}ms, ping: {}ms, connected: {}",
                        state.position, state.ping, state.connected
                    );
                }
                NodeEvent::Stats(stats) => {
                    println!(
                        "Node stats -> players: {}/{} playing, memory used: {}MB",
                        stats.playing_players,
                        stats.players,
                        stats.memory.used / (1024 * 1024)
                    );
                }
                NodeEvent::Event(event) => {
                    println!("Received player event: {event:?}");
                }
            }
        }
    });

    // Give the WebSocket a brief moment to receive the initial Ready event
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 3. Search for a track using YouTube provider
    println!("Searching for track...");
    let search = client
        .load_tracks(Query::youtube("Rick Astley - Never Gonna Give You Up"))
        .await?;

    if let LoadResult::Search(tracks) = search {
        if let Some(track) = tracks.first() {
            println!("Found track: {} by {}", track.info.title, track.info.author);

            let guild_id = 112233445566778899;
            let player = client.player(guild_id);

            // Initiate playback and configure initial volume
            player.play(track).await?;
            player.set_volume(80).await?;

            // Read state instantly from in-memory cache without making REST requests
            println!("Current cached position: {}ms", player.position());
            println!("Current voice latency ping: {}ms", player.ping());
        }
    }

    // Keep the runtime alive for demonstration purposes
    tokio::time::sleep(Duration::from_secs(5)).await;
    Ok(())
}
