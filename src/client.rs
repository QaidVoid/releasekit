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

/// Ureq-based HTTP client (available with the `ureq` feature).
#[cfg(feature = "ureq")]
#[derive(Clone)]
pub struct UreqClient;

#[cfg(feature = "ureq")]
impl HttpClient for UreqClient {
    fn get(&self, url: &str, headers: &HeaderMap) -> Result<Response> {
        use crate::error::Error;

        let mut req = ureq::get(url).header("User-Agent", "releasekit");
        for (k, v) in headers.iter() {
            req = req.header(k, v);
        }
        let resp = req.call().map_err(|e| Error::Network(e.to_string()))?;
        let status = resp.status().as_u16();
        if status >= 400 {
            return Err(Error::Http {
                status,
                url: url.to_string(),
            });
        }
        let body = resp
            .into_body()
            .read_to_string()
            .map_err(|e| Error::Network(e.to_string()))?;
        Ok(Response { status, body })
    }
}
