//! Client abstraction for HTTP backends.

use crate::error::Result;

/// A lightweight HTTP response.
pub struct Response {
    /// HTTP status code.
    pub status: u16,
    /// Response body as a string.
    pub body: String,
}

/// A simple key-value header store.
///
/// Avoids pulling in the `http` crate. We only ever set 1-2 headers
/// (User-Agent + Authorization).
#[derive(Default, Clone)]
pub struct HeaderMap {
    entries: Vec<(String, String)>,
}

impl HeaderMap {
    /// Creates an empty `HeaderMap`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a header key-value pair.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.push((key.into(), value.into()));
    }

    /// Returns an iterator over all header entries.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// Trait for synchronous HTTP clients.
///
/// Implement this trait to bring your own HTTP backend.
/// The `Clone` bound allows platforms to own and share the client.
pub trait HttpClient: Clone {
    /// Performs a GET request with the given headers.
    fn get(&self, url: &str, headers: &HeaderMap) -> Result<Response>;
}
