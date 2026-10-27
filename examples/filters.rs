use nodelink_rs::model::track::LoadResult;
use nodelink_rs::{Client, Config, Query};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::new("127.0.0.1:2333", "youshallnotpass", 123456789012345678);
    let (client, _events) = Client::connect(config).await?;

    // Allow time for WebSocket handshake
    tokio::time::sleep(Duration::from_millis(500)).await;

    let search = client.load_tracks(Query::youtube("Lofi Hip Hop")).await?;
    if let LoadResult::Search(tracks) = search {
        if let Some(track) = tracks.first() {
            let guild_id = 112233445566778899;
            let player = client.player(guild_id);

            // Start playing the track
            player.play(track).await?;

            // 1. Apply Nightcore effect (speed 1.2x + pitch shift)
            println!("Applying Nightcore filter...");
            player.filters().nightcore(true).apply().await?;

            tokio::time::sleep(Duration::from_secs(10)).await;

            // 2. Enable Bass Boost equalizer
            println!("Enabling Bass Boost...");
            player.filters().bass_boost(true).apply().await?;

            tokio::time::sleep(Duration::from_secs(10)).await;

            // 3. Reset all active filters back to normal
            println!("Resetting all filters...");
            player.filters().reset().await?;
        }
    }

    Ok(())
}
