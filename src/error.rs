//! Error types and Result alias.

/// Errors returned by releasekit operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An HTTP error response was received.
    #[error("HTTP {status} for {url}")]
    Http {
        /// HTTP status code.
        status: u16,
        /// The requested URL.
        url: String,
    },

    /// Failed to parse JSON response.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// A network-level error occurred.
    #[error("Network error: {0}")]
    Network(String),

    /// No releases were found for the given query.
    #[error("No releases found")]
    NoReleases,

    /// No asset matched the filter criteria.
    #[error("No matching asset found")]
    NoMatchingAsset,

    /// The provided URL could not be parsed.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// Alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
