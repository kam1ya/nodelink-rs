//! Search query helpers and source prefixes.

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// Perform a keyword search using the specified source.
    Search(SearchSource, String),
    /// Load a track or playlist directly via URL.
    Url(String),
}

impl Query {
    /// Constructs a YouTube search query.
    pub fn youtube(query: impl Into<String>) -> Self {
        Self::Search(SearchSource::YouTube, query.into())
    }

    /// Constructs a SoundCloud search query.
    pub fn soundcloud(query: impl Into<String>) -> Self {
        Self::Search(SearchSource::SoundCloud, query.into())
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
