//! GitLab platform implementation.

use crate::client::{HeaderMap, HttpClient};
use crate::error::{Error, Result};
use crate::model::{Asset, Release};
use crate::platform::Forge;
use serde::Deserialize;

/// A GitLab client for fetching releases.
///
/// Wraps any [`HttpClient`] implementation and optionally stores a
/// personal access token for private repositories.
pub struct GitLab<C: HttpClient> {
    client: C,
    token: Option<String>,
    base_url: String,
}

impl<C: HttpClient> GitLab<C> {
    /// Creates a new GitLab client with the given HTTP backend.
    ///
    /// Defaults to `https://gitlab.com`. Use [`GitLab::with_base_url`] for
    /// self-hosted instances.
    pub fn new(client: C) -> Self {
        Self {
            client,
            token: None,
            base_url: "https://gitlab.com".to_string(),
        }
    }

    /// Sets a custom base URL for the GitLab instance.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    /// Sets a GitLab personal access token for authentication.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Reads a token from the first set environment variable.
    ///
    /// Tries each name in order and uses the first one that is set.
    /// Does not overwrite a token already set via [`GitLab::with_token`].
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
            headers.insert("PRIVATE-TOKEN", token.trim());
        }
        headers
    }
}

#[derive(Deserialize)]
struct GlRelease {
    name: Option<String>,
    tag_name: String,
    #[serde(default)]
    upcoming_release: bool,
    released_at: String,
    description: Option<String>,
    assets: GlAssets,
}

#[derive(Deserialize)]
struct GlAssets {
    links: Vec<GlLink>,
}

#[derive(Deserialize)]
struct GlLink {
    name: String,
    direct_asset_url: String,
}

impl From<GlRelease> for Release {
    fn from(g: GlRelease) -> Self {
        Release {
            name: g.name,
            tag: g.tag_name,
            prerelease: g.upcoming_release,
            published_at: g.released_at,
            body: g.description,
            assets: g.assets.links.into_iter().map(Asset::from).collect(),
        }
    }
}

impl From<GlLink> for Asset {
    fn from(l: GlLink) -> Self {
        Asset {
            name: l.name,
            size: None,
            url: l.direct_asset_url,
        }
    }
}

impl<C: HttpClient> Forge for GitLab<C> {
    /// Fetches releases for the given project.
    ///
    /// `project` can be either `owner/repo` format (will be URL-encoded) or
    /// a numeric project ID.
    fn fetch_releases(&self, project: &str, tag: Option<&str>) -> Result<Vec<Release>> {
        let project_ref = if project.chars().all(|c| c.is_ascii_digit()) {
            project.to_string()
        } else {
            urlencoding::encode(project).into_owned()
        };
        let url = match tag {
            Some(t) => {
                let enc_tag = urlencoding::encode(t);
                format!(
                    "{}/api/v4/projects/{project_ref}/releases/{enc_tag}",
                    self.base_url
                )
            }
            None => format!("{}/api/v4/projects/{project_ref}/releases", self.base_url),
        };

        let resp = self.client.get(&url, &self.auth_headers())?;
        let raw: serde_json::Value = serde_json::from_str(&resp.body)?;

        let releases: Vec<GlRelease> = match raw {
            serde_json::Value::Array(arr) => serde_json::from_value(serde_json::Value::Array(arr))?,
            obj @ serde_json::Value::Object(_) => vec![serde_json::from_value(obj)?],
            _ => return Err(Error::NoReleases),
        };

        Ok(releases.into_iter().map(Release::from).collect())
    }
}
