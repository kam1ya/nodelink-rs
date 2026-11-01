//! Track and playlist models conforming to the Lavalink v4 / NodeLink specification.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Detailed metadata describing an individual audio track.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    /// Unique provider-specific identifier (e.g. YouTube video ID).
    pub identifier: String,
    /// Whether the audio track supports position seeking.
    pub is_seekable: bool,
    /// Track artist or channel name.
    pub author: String,
    /// Total duration in milliseconds (0 for endless livestreams).
    pub length: u64,
    /// Indicates whether the track is an active livestream.
    pub is_stream: bool,
    /// Starting playback position in milliseconds.
    pub position: u64,
    /// Title of the audio track.
    pub title: String,
    /// Canonical web URL pointing to the source track, if available.
    #[serde(default)]
    pub uri: Option<String>,
    /// Direct link to the track's album cover or thumbnail art.
    #[serde(default)]
    pub artwork_url: Option<String>,
    /// International Standard Recording Code (ISRC), if resolved.
    #[serde(default)]
    pub isrc: Option<String>,
    /// Name of the resolving source plugin (e.g., `"youtube"`, `"soundcloud"`).
    pub source_name: String,
}

impl TrackInfo {
    /// Returns the track duration as a [`Duration`].
    #[inline]
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.length)
    }
}

/// A playable audio track containing its base64 bytecode payload and metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    /// Base64-encoded audio track buffer used by the node's audio engine.
    pub encoded: String,
    /// Metadata parameters associated with this track.
    pub info: TrackInfo,
    /// Arbitrary plugin payload attached to this track instance.
    #[serde(default)]
    pub plugin_info: serde_json::Value,
    /// Client-defined custom metadata preserved across node operations.
    #[serde(default)]
    pub user_data: serde_json::Value,
}

impl Track {
    /// Convenient accessor for the track's title.
    #[inline]
    pub fn title(&self) -> &str {
        &self.info.title
    }

    /// Convenient accessor for the track's author or channel name.
    #[inline]
    pub fn author(&self) -> &str {
        &self.info.author
    }

    /// Convenient accessor for the track's duration.
    #[inline]
    pub fn duration(&self) -> Duration {
        self.info.duration()
    }
}

/// The result returned by `/v4/loadtracks` representing resolved audio resources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "loadType", content = "data", rename_all = "camelCase")]
pub enum LoadResult {
    /// A single track was matched and returned directly.
    Track(Track),
    /// A full playlist or album was loaded.
    Playlist {
        /// Metadata describing the loaded playlist.
        info: PlaylistInfo,
        /// Optional plugin-specific data attached to the playlist.
        #[serde(default)]
        plugin_info: serde_json::Value,
        /// Complete list of tracks contained in this playlist.
        tracks: Vec<Track>,
    },
    /// A list of candidate tracks matching the search query.
    Search(Vec<Track>),
    /// The search yielded no matching results on the remote node.
    Empty,
    /// An error occurred during track resolution on the remote node.
    Error(LoadError),
}

impl LoadResult {
    /// Returns a slice of tracks if the result contains any tracks, or an empty slice otherwise.
    pub fn tracks(&self) -> &[Track] {
        match self {
            Self::Track(track) => std::slice::from_ref(track),
            Self::Playlist { tracks, .. } => tracks.as_slice(),
            Self::Search(tracks) => tracks.as_slice(),
            Self::Empty | Self::Error(_) => &[],
        }
    }

    /// Returns `true` if the result yielded no tracks (either [`LoadResult::Empty`] or an empty search/playlist).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tracks().is_empty()
    }
}

/// Metadata describing a loaded playlist or collection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    /// Name or title of the playlist.
    pub name: String,
    /// 0-indexed position of the track requested by the user, if applicable.
    #[serde(default)]
    pub selected_track: Option<i64>,
}

/// Detailed error payload emitted when track resolution fails on the node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoadError {
    /// Descriptive error message explaining the cause of failure.
    pub message: String,
    /// Severity level reported by the node engine (e.g. `"fault"`, `"common"`).
    pub severity: String,
    /// Internal reason or upstream exception description.
    pub cause: String,
}
