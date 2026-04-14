//! GitHub platform implementation.

use crate::client::{HeaderMap, HttpClient};
use crate::error::{Error, Result};
use crate::model::{Asset, Release};
use crate::platform::Forge;
use serde::Deserialize;

/// A GitHub client for fetching releases.
///
/// Wraps any [`HttpClient`] implementation and optionally stores an
/// authentication token for private repositories or higher rate limits.
pub struct GitHub<C: HttpClient> {
    client: C,
    token: Option<String>,
    base_url: String,
}

impl<C: HttpClient> GitHub<C> {
    /// Creates a new GitHub client with the given HTTP backend.
    pub fn new(client: C) -> Self {
        Self {
            client,
            token: None,
            base_url: "https://api.github.com".to_string(),
        }
    }

    /// Sets a custom base URL for the GitHub API.
    ///
    /// Useful for GitHub Enterprise or API proxies. Trailing slashes are
    /// stripped automatically.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    /// Sets a GitHub personal access token for authentication.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Reads a token from the first set environment variable.
    ///
    /// Tries each name in order and uses the first one that is set.
    /// Does not overwrite a token already set via [`GitHub::with_token`].
    pub fn with_token_from_env(mut self, names: &[&str]) -> Self {
        if self.token.is_none() {
            for name in names {
                if let Ok(val) = std::env::var(name) {
                    let trimmed = val.trim().to_string();
                    if !trimmed.is_empty() {
                        self.token = Some(trimmed);
                        break;
                    }
                }
            }
        }
        self
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(ref token) = self.token {
            headers.insert("Authorization", format!("Bearer {}", token.trim()));
        }
        headers
    }
}

#[derive(Deserialize)]
struct GhRelease {
    name: Option<String>,
    tag_name: String,
    prerelease: bool,
    published_at: String,
    body: Option<String>,
    assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

impl From<GhRelease> for Release {
    fn from(g: GhRelease) -> Self {
        Release {
            name: g.name,
            tag: g.tag_name,
            prerelease: g.prerelease,
            published_at: g.published_at,
            body: g.body,
            assets: g.assets.into_iter().map(Asset::from).collect(),
        }
    }
}

impl From<GhAsset> for Asset {
    fn from(g: GhAsset) -> Self {
        Asset {
            name: g.name,
            size: Some(g.size),
            url: g.browser_download_url,
        }
    }
}

impl<C: HttpClient> Forge for GitHub<C> {
    fn fetch_releases(&self, project: &str, tag: Option<&str>) -> Result<Vec<Release>> {
        let url = match tag {
            Some(t) => {
                let enc = urlencoding::encode(t);
                format!("{}/repos/{project}/releases/tags/{enc}", self.base_url)
            }
            None => format!("{}/repos/{project}/releases?per_page=100", self.base_url),
        };

        let resp = self.client.get(&url, &self.auth_headers())?;

        let raw: serde_json::Value = serde_json::from_str(&resp.body)?;

        let releases: Vec<GhRelease> = match raw {
            serde_json::Value::Array(arr) => serde_json::from_value(serde_json::Value::Array(arr))?,
            obj @ serde_json::Value::Object(_) => vec![serde_json::from_value(obj)?],
            _ => return Err(Error::NoReleases),
        };

        Ok(releases.into_iter().map(Release::from).collect())
    }
}
