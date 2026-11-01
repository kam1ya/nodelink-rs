//! Specialized error types and result aliases for `nodelink-rs`.

use thiserror::Error;

/// All errors that can occur during communication with Lavalink v4 / NodeLink nodes.
#[derive(Debug, Error)]
pub enum NodeLinkError {
    /// Network or protocol error produced by the underlying HTTP client.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Transport error produced by the WebSocket stream.
    #[error("WebSocket transport error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// JSON serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// URL parsing error when constructing target endpoints.
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// The remote node responded with an HTTP error status code.
    #[error("Node returned HTTP error {status}: {message}")]
    Api {
        /// HTTP status code (e.g. 400, 404, 500).
        status: u16,
        /// Response body or descriptive error message returned by the node.
        message: String,
    },

    /// An operation requiring a valid session was called before receiving the `Ready` event.
    #[error("Session is not initialized (no Ready event received yet)")]
    SessionNotReady,
}

/// Convenience alias for `Result<T, NodeLinkError>`.
pub type NodeResult<T> = Result<T, NodeLinkError>;
