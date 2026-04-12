//! Data model for releases and assets.

/// A release from a git forge.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Release {
    pub(crate) name: Option<String>,
    pub(crate) tag: String,
    pub(crate) prerelease: bool,
    pub(crate) published_at: String,
    pub(crate) body: Option<String>,
    pub(crate) assets: Vec<Asset>,
}

impl Release {
    /// The release name, if set.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The git tag this release points to.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Whether this is a prerelease.
    pub fn is_prerelease(&self) -> bool {
        self.prerelease
    }

    /// The ISO 8601 timestamp when the release was published.
    pub fn published_at(&self) -> &str {
        &self.published_at
    }

    /// The release body / description, if set.
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    /// The assets attached to this release.
    pub fn assets(&self) -> &[Asset] {
        &self.assets
    }
}

/// A downloadable asset attached to a release.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    pub(crate) name: String,
    pub(crate) size: Option<u64>,
    pub(crate) url: String,
}

impl Asset {
    /// The asset filename.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The asset size in bytes, if reported by the forge.
    pub fn size(&self) -> Option<u64> {
        self.size
    }

    /// The download URL for this asset.
    pub fn url(&self) -> &str {
        &self.url
    }
}
