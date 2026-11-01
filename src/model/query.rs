//! Search query helpers, source prefixes, and identifier formatting.

use std::fmt;

/// Search engine providers supported by Lavalink v4 and NodeLink plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchSource {
    /// YouTube search (`ytsearch:`).
    YouTube,
    /// YouTube Music search (`ytmsearch:`).
    YouTubeMusic,
    /// SoundCloud search (`scsearch:`).
    SoundCloud,
    /// Spotify search (`spsearch:`).
    Spotify,
}

impl SearchSource {
    /// Returns the protocol prefix associated with this source.
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::YouTube => "ytsearch:",
            Self::YouTubeMusic => "ytmsearch:",
            Self::SoundCloud => "scsearch:",
            Self::Spotify => "spsearch:",
        }
    }
}

/// A structured query specifying search providers or direct URLs.
///
/// Supports automatic conversion from string slices: URLs starting with `http://`
/// or `https://` are treated as direct URLs, while other strings default to YouTube search.
///
/// # Examples
///
/// ```
/// use nodelink_rs::Query;
///
/// // Direct constructors
/// let q1 = Query::youtube("Rick Astley - Never Gonna Give You Up");
/// let q2 = Query::url("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
/// let q3 = Query::spotify("Daft Punk - Get Lucky");
///
/// // Automatic conversion from string
/// let from_str: Query = "Rick Astley".into();
/// assert_eq!(from_str, Query::youtube("Rick Astley"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// Perform a keyword search using the specified source.
    Search(SearchSource, String),
    /// Load a track or playlist directly via URL.
    Url(String),
}

impl Query {
    /// Constructs a YouTube search query (`ytsearch:<query>`).
    pub fn youtube(query: impl Into<String>) -> Self {
        Self::Search(SearchSource::YouTube, query.into())
    }

    /// Constructs a YouTube Music search query (`ytmsearch:<query>`).
    pub fn youtube_music(query: impl Into<String>) -> Self {
        Self::Search(SearchSource::YouTubeMusic, query.into())
    }

    /// Constructs a SoundCloud search query (`scsearch:<query>`).
    pub fn soundcloud(query: impl Into<String>) -> Self {
        Self::Search(SearchSource::SoundCloud, query.into())
    }

    /// Constructs a Spotify search query (`spsearch:<query>`).
    pub fn spotify(query: impl Into<String>) -> Self {
        Self::Search(SearchSource::Spotify, query.into())
    }

    /// Constructs a search query using an explicit [`SearchSource`].
    pub fn search(source: SearchSource, query: impl Into<String>) -> Self {
        Self::Search(source, query.into())
    }

    /// Constructs a direct URL query.
    pub fn url(url: impl Into<String>) -> Self {
        Self::Url(url.into())
    }

    /// Formats the query into the raw identifier string expected by `/v4/loadtracks`.
    pub fn into_identifier(self) -> String {
        match self {
            Self::Search(source, q) => format!("{}{}", source.prefix(), q),
            Self::Url(url) => url,
        }
    }
}

impl From<Query> for String {
    fn from(query: Query) -> Self {
        query.into_identifier()
    }
}

impl From<&str> for Query {
    fn from(s: &str) -> Self {
        if s.starts_with("http://") || s.starts_with("https://") {
            Self::Url(s.to_string())
        } else {
            Self::youtube(s)
        }
    }
}

impl From<String> for Query {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Search(source, q) => write!(f, "{}{}", source.prefix(), q),
            Self::Url(url) => write!(f, "{}", url),
        }
    }
}
