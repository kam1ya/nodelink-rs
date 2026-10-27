use nodelink_rs::model::events::{NodeEvent, TrackEndReason, TrackEventPayload};
use nodelink_rs::model::track::LoadResult;

#[test]
fn test_deserialize_load_tracks_track() {
    let raw = r#"{
        "loadType": "track",
        "data": {
            "encoded": "QAAAjQIAJVJpY2sgQXN0bGV5IC0gTmV2ZXIgR29ubmEgR2l2ZSBZb3UgVXA...",
            "info": {
                "identifier": "dQw4w9WgXcQ",
                "isSeekable": true,
                "author": "Rick Astley",
                "length": 213000,
                "isStream": false,
                "position": 0,
                "title": "Never Gonna Give You Up",
                "uri": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
                "artworkUrl": "https://i.ytimg.com/vi/dQw4w9WgXcQ/maxresdefault.jpg",
                "isrc": null,
                "sourceName": "youtube"
            },
            "pluginInfo": {},
            "userData": {}
        }
    }"#;

    let res: LoadResult =
        serde_json::from_str(raw).expect("Failed to deserialize Track LoadResult");
    match res {
        LoadResult::Track(track) => {
            assert_eq!(track.info.title, "Never Gonna Give You Up");
            assert_eq!(track.info.author, "Rick Astley");
            assert_eq!(track.info.source_name, "youtube");
            assert_eq!(track.info.length, 213000);
        }
        _ => panic!("Expected LoadResult::Track"),
    }
}

#[test]
fn test_deserialize_load_tracks_search() {
    let raw = r#"{
        "loadType": "search",
        "data": [
            {
                "encoded": "QAAAjQIA...",
                "info": {
                    "identifier": "abc12345",
                    "isSeekable": true,
                    "author": "Artist",
                    "length": 180000,
                    "isStream": false,
                    "position": 0,
                    "title": "Test Search Result",
                    "uri": "https://youtube.com/watch?v=abc12345",
                    "artworkUrl": null,
                    "isrc": null,
                    "sourceName": "youtube"
                },
                "pluginInfo": {},
                "userData": {}
            }
        ]
    }"#;

    let res: LoadResult =
        serde_json::from_str(raw).expect("Failed to deserialize Search LoadResult");
    match res {
        LoadResult::Search(tracks) => {
            assert_eq!(tracks.len(), 1);
            assert_eq!(tracks[0].info.title, "Test Search Result");
        }
        _ => panic!("Expected LoadResult::Search"),
    }
}

#[test]
fn test_deserialize_ready_event() {
    let raw = r#"{
        "op": "ready",
        "resumed": false,
        "sessionId": "b4737d2f-1a13-4ef1-8f5a-e7be94017f8a"
    }"#;

    let event: NodeEvent = serde_json::from_str(raw).expect("Failed to deserialize Ready event");
    match event {
        NodeEvent::Ready {
            resumed,
            session_id,
        } => {
            assert!(!resumed);
            assert_eq!(session_id, "b4737d2f-1a13-4ef1-8f5a-e7be94017f8a");
        }
        _ => panic!("Expected NodeEvent::Ready"),
    }
}

#[test]
fn test_deserialize_track_end_event() {
    let raw = r#"{
        "op": "event",
        "type": "TrackEndEvent",
        "guildId": "123456789012345678",
        "track": {
            "encoded": "QAAAjQIA...",
            "info": {
                "identifier": "dQw4w9WgXcQ",
                "isSeekable": true,
                "author": "Rick Astley",
                "length": 213000,
                "isStream": false,
                "position": 0,
                "title": "Never Gonna Give You Up",
                "uri": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
                "artworkUrl": null,
                "isrc": null,
                "sourceName": "youtube"
            },
            "pluginInfo": {},
            "userData": {}
        },
        "reason": "finished"
    }"#;

    let event: NodeEvent = serde_json::from_str(raw).expect("Failed to deserialize TrackEndEvent");
    match event {
        NodeEvent::Event(TrackEventPayload::TrackEndEvent {
            guild_id,
            reason,
            track,
        }) => {
            assert_eq!(guild_id, "123456789012345678");
            assert_eq!(reason, TrackEndReason::Finished);
            assert!(reason.may_start_next());
            assert_eq!(track.info.title, "Never Gonna Give You Up");
        }
        _ => panic!("Expected TrackEndEvent"),
    }
}

#[test]
fn test_deserialize_player_update() {
    let raw = r#"{
        "op": "playerUpdate",
        "guildId": "987654321",
        "state": {
            "time": 1700000000000,
            "position": 45120,
            "connected": true,
            "ping": 18
        }
    }"#;

    let event: NodeEvent = serde_json::from_str(raw).expect("Failed to deserialize PlayerUpdate");
    match event {
        NodeEvent::PlayerUpdate { guild_id, state } => {
            assert_eq!(guild_id, "987654321");
            assert_eq!(state.position, 45120);
            assert_eq!(state.ping, 18);
            assert!(state.connected);
        }
        _ => panic!("Expected PlayerUpdate"),
    }
}
