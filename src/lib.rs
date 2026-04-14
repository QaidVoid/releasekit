#![deny(missing_docs)]

//! releasekit — client-agnostic library for fetching releases from git forges.
//!
//! # Quick start
//!
//! ```no_run
//! use releasekit::platform::GitHub;
//! use releasekit::client::UreqClient;
//! use releasekit::Forge;
//!
//! let gh = GitHub::new(UreqClient);
//! let releases = gh.fetch_releases("owner/repo", None).unwrap();
//! for r in &releases {
//!     println!("{} — {} assets", r.tag(), r.assets().len());
//! }
//! ```

pub mod client;
pub mod error;
pub mod filter;
pub mod model;
pub mod platform;
pub mod url;

pub use error::Error;
pub use filter::Filter;
pub use model::{Asset, Release};
pub use platform::Forge;
pub use platform::GitHub;
pub use platform::GitLab;
pub use url::PlatformUrl;
