//! Forge trait for fetching releases from git hosting platforms.

use crate::error::Result;
use crate::model::Release;

/// A git forge that can fetch releases.
///
/// Implementations wrap an [`HttpClient`](crate::client::HttpClient) and
/// translate forge-specific APIs into the common [`Release`] type.
pub trait Forge {
    /// Fetches releases for the given project.
    ///
    /// `project` is in `owner/repo` format. When `tag` is `Some`, fetches
    /// only the release for that specific tag.
    fn fetch_releases(&self, project: &str, tag: Option<&str>) -> Result<Vec<Release>>;
}
