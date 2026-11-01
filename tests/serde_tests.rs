use nodelink_rs::model::events::{NodeEvent, TrackEndReason, TrackEventPayload};
use nodelink_rs::model::query::{Query, SearchSource};
use nodelink_rs::model::track::LoadResult;
use nodelink_rs::player::PlayerState;
use std::time::Duration;

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
        LoadResult::Track(ref track) => {
            assert_eq!(track.info.title, "Never Gonna Give You Up");
            assert_eq!(track.info.author, "Rick Astley");
            assert_eq!(track.info.source_name, "youtube");
            assert_eq!(track.info.length, 213000);
            assert_eq!(track.title(), "Never Gonna Give You Up");
            assert_eq!(track.author(), "Rick Astley");
            assert_eq!(track.duration(), Duration::from_millis(213000));
        }
        _ => panic!("Expected LoadResult::Track"),
    }

    assert_eq!(res.tracks().len(), 1);
    assert!(!res.is_empty());
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
        LoadResult::Search(ref tracks) => {
            assert_eq!(tracks.len(), 1);
            assert_eq!(tracks[0].info.title, "Test Search Result");
            assert_eq!(tracks[0].title(), "Test Search Result");
        }
        _ => panic!("Expected LoadResult::Search"),
    }

    assert_eq!(res.tracks().len(), 1);
    assert!(!res.is_empty());
}

#[test]
fn test_deserialize_ready_event() {
    let raw = r#"{
        "op": "ready",
        "resumed": false,
        "sessionId": "b4737d2f-1a13-4ef1-8f5a-e7be94017f8a"
    }"#;

    let event: NodeEvent = serde_json::from_str(raw).expect("Failed to deserialize Ready event");
    assert!(event.is_ready());
    assert_eq!(event.guild_id(), None);

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
    assert_eq!(event.guild_id(), Some("123456789012345678"));

    match event {
        NodeEvent::Event(TrackEventPayload::TrackEndEvent {
            guild_id,
            reason,
            track,
        }) => {
            assert_eq!(guild_id, "123456789012345678");
            assert_eq!(reason, TrackEndReason::Finished);
            assert!(reason.may_start_next());
            assert_eq!(track.title(), "Never Gonna Give You Up");
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
    assert_eq!(event.guild_id(), Some("987654321"));

    match event {
        NodeEvent::PlayerUpdate { guild_id, state } => {
            assert_eq!(guild_id, "987654321");
            assert_eq!(state.position, 45120);
            assert_eq!(state.position_duration(), Duration::from_millis(45120));
            assert_eq!(state.ping, 18);
            assert!(state.connected);
        }
        _ => panic!("Expected PlayerUpdate"),
    }
}

#[test]
fn test_query_constructors() {
    let q_yt = Query::youtube("Never Gonna Give You Up");
    assert_eq!(
        q_yt.into_identifier(),
        "ytsearch:Never Gonna Give You Up"
    );

    let q_ytm = Query::youtube_music("Daft Punk");
    assert_eq!(q_ytm.into_identifier(), "ytmsearch:Daft Punk");

    let q_sc = Query::soundcloud("Lofi");
    assert_eq!(q_sc.into_identifier(), "scsearch:Lofi");

    let q_sp = Query::spotify("Taylor Swift");
    assert_eq!(q_sp.into_identifier(), "spsearch:Taylor Swift");

    let q_url = Query::url("https://example.com/audio.mp3");
    assert_eq!(q_url.into_identifier(), "https://example.com/audio.mp3");

    // From<&str>
    let from_url: Query = "https://youtube.com/watch?v=123".into();
    assert_eq!(
        from_url,
        Query::Url("https://youtube.com/watch?v=123".into())
    );

    let from_text: Query = "Bohemian Rhapsody".into();
    assert_eq!(
        from_text,
        Query::Search(SearchSource::YouTube, "Bohemian Rhapsody".into())
    );
}

#[test]
fn test_player_state_defaults() {
    let state = PlayerState::new();
    assert_eq!(state.position(), 0);
    assert_eq!(state.ping(), 0);
    assert!(!state.is_connected());
    assert!(!state.is_paused());
    assert_eq!(state.last_updated_time(), 0);
}
