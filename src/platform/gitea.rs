//! Gitea platform implementation.
//!
//! Also works with Codeberg and other Gitea-based forges.

use crate::client::{HeaderMap, HttpClient};
use crate::error::{Error, Result};
use crate::model::{Asset, Release};
use crate::platform::Forge;
use serde::Deserialize;

/// A Gitea client for fetching releases.
///
/// Works with any Gitea-based forge including Codeberg. The base URL
/// must be provided since Gitea instances are self-hosted.
///
/// # Example
///
/// ```no_run
/// use releasekit::Forge;
/// use releasekit::client::UreqClient;
/// use releasekit::platform::Gitea;
///
/// // Codeberg
/// let cb = Gitea::new(UreqClient, "https://codeberg.org");
/// let releases = cb.fetch_releases("owner/repo", None).unwrap();
///
/// // Self-hosted Gitea
/// let gt = Gitea::new(UreqClient, "https://gitea.example.com");
/// let releases = gt.fetch_releases("owner/repo", None).unwrap();
/// ```
pub struct Gitea<C: HttpClient> {
    client: C,
    token: Option<String>,
    base_url: String,
}

impl<C: HttpClient> Gitea<C> {
    /// Creates a new Gitea client with the given HTTP backend and instance URL.
    pub fn new(client: C, base_url: impl Into<String>) -> Self {
        Self {
            client,
            token: None,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Sets a Gitea access token for authentication.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Reads a token from the first set environment variable.
    ///
    /// Tries each name in order and uses the first one that is set.
    /// Does not overwrite a token already set via [`Gitea::with_token`].
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
            headers.insert("Authorization", format!("token {}", token.trim()));
        }
        headers
    }
}

#[derive(Deserialize)]
struct GtRelease {
    name: Option<String>,
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
    published_at: String,
    body: Option<String>,
    assets: Vec<GtAsset>,
}

#[derive(Deserialize)]
struct GtAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

impl From<GtRelease> for Release {
    fn from(g: GtRelease) -> Self {
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

impl From<GtAsset> for Asset {
    fn from(a: GtAsset) -> Self {
        Asset {
            name: a.name,
            size: Some(a.size),
            url: a.browser_download_url,
        }
    }
}

impl<C: HttpClient> Forge for Gitea<C> {
    fn fetch_releases(&self, project: &str, tag: Option<&str>) -> Result<Vec<Release>> {
        let url = match tag {
            Some(t) => {
                let enc = urlencoding::encode(t);
                format!(
                    "{}/api/v1/repos/{project}/releases/tags/{enc}",
                    self.base_url
                )
            }
            None => format!("{}/api/v1/repos/{project}/releases?limit=50", self.base_url),
        };

        let resp = self.client.get(&url, &self.auth_headers())?;
        let raw: serde_json::Value = serde_json::from_str(&resp.body)?;

        let releases: Vec<GtRelease> = match raw {
            serde_json::Value::Array(arr) => serde_json::from_value(serde_json::Value::Array(arr))?,
            obj @ serde_json::Value::Object(_) => vec![serde_json::from_value(obj)?],
            _ => return Err(Error::NoReleases),
        };

        Ok(releases.into_iter().map(Release::from).collect())
    }
}
