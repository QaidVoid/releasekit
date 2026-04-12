//! URL parser for git forge URLs.

/// A parsed git forge URL.
///
/// Extracts the project identifier and optional tag from URLs like
/// `https://github.com/owner/repo` or `https://github.com/owner/repo/releases/tag/v1.0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformUrl {
    /// A GitHub repository URL.
    GitHub {
        /// The project in `owner/repo` format.
        project: String,
        /// An optional tag name.
        tag: Option<String>,
    },
    /// A GitLab repository URL.
    GitLab {
        /// The project in `owner/repo` format.
        project: String,
        /// An optional tag name.
        tag: Option<String>,
    },
    /// A Codeberg repository URL.
    Codeberg {
        /// The project in `owner/repo` format.
        project: String,
        /// An optional tag name.
        tag: Option<String>,
    },
    /// A Gitea repository URL.
    Gitea {
        /// The Gitea instance hostname.
        instance: String,
        /// The project in `owner/repo` format.
        project: String,
        /// An optional tag name.
        tag: Option<String>,
    },
}

impl PlatformUrl {
    /// Parses a URL string into a `PlatformUrl`.
    ///
    /// Returns `None` if the input is empty or doesn't match any known forge.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        if let Some(rest) = input
            .strip_prefix("https://github.com/")
            .or_else(|| input.strip_prefix("http://github.com/"))
        {
            return Self::parse_github(rest);
        }

        if let Some(rest) = input
            .strip_prefix("https://gitlab.com/")
            .or_else(|| input.strip_prefix("http://gitlab.com/"))
        {
            return Self::parse_gitlab(rest);
        }

        if let Some(rest) = input
            .strip_prefix("https://codeberg.org/")
            .or_else(|| input.strip_prefix("http://codeberg.org/"))
        {
            return Self::parse_codeberg(rest);
        }

        if input.starts_with("http://") || input.starts_with("https://") {
            return Self::parse_gitea(input);
        }

        None
    }

    fn parse_project_and_tag(path: &str) -> Option<(String, Option<String>)> {
        let path = path.trim_end_matches('/');

        let tag_prefixes = ["/releases/tag/", "/releases/", "/tags/", "/tag/"];

        for prefix in tag_prefixes {
            if let Some(idx) = path.find(prefix) {
                let project = &path[..idx];
                let tag = &path[idx + prefix.len()..];
                if !tag.is_empty() {
                    let proj = Self::validate_project(project)?;
                    return Some((proj, Some(tag.to_string())));
                }
            }
        }

        let project = Self::validate_project(path)?;
        Some((project, None))
    }

    fn validate_project(project: &str) -> Option<String> {
        let parts: Vec<&str> = project.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        if parts[0].is_empty() || parts[1].is_empty() {
            return None;
        }
        Some(project.to_string())
    }

    fn parse_github(rest: &str) -> Option<Self> {
        let (project, tag) = Self::parse_project_and_tag(rest)?;
        Some(Self::GitHub { project, tag })
    }

    fn parse_gitlab(rest: &str) -> Option<Self> {
        let (project, tag) = Self::parse_project_and_tag(rest)?;
        Some(Self::GitLab { project, tag })
    }

    fn parse_codeberg(rest: &str) -> Option<Self> {
        let (project, tag) = Self::parse_project_and_tag(rest)?;
        Some(Self::Codeberg { project, tag })
    }

    fn parse_gitea(input: &str) -> Option<Self> {
        let without_scheme = input
            .strip_prefix("https://")
            .or_else(|| input.strip_prefix("http://"))?;

        let (host, path) = without_scheme.split_once('/')?;
        let (project, tag) = Self::parse_project_and_tag(path)?;

        Some(Self::Gitea {
            instance: host.to_string(),
            project,
            tag,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_basic() {
        let url = PlatformUrl::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(
            url,
            PlatformUrl::GitHub {
                project: "owner/repo".to_string(),
                tag: None,
            }
        );
    }

    #[test]
    fn github_with_tag() {
        let url = PlatformUrl::parse("https://github.com/owner/repo/releases/tag/v1.0").unwrap();
        assert_eq!(
            url,
            PlatformUrl::GitHub {
                project: "owner/repo".to_string(),
                tag: Some("v1.0".to_string()),
            }
        );
    }

    #[test]
    fn gitlab_basic() {
        let url = PlatformUrl::parse("https://gitlab.com/owner/repo").unwrap();
        assert_eq!(
            url,
            PlatformUrl::GitLab {
                project: "owner/repo".to_string(),
                tag: None,
            }
        );
    }

    #[test]
    fn codeberg_basic() {
        let url = PlatformUrl::parse("https://codeberg.org/owner/repo").unwrap();
        assert_eq!(
            url,
            PlatformUrl::Codeberg {
                project: "owner/repo".to_string(),
                tag: None,
            }
        );
    }

    #[test]
    fn gitea_basic() {
        let url = PlatformUrl::parse("https://gitea.example.com/owner/repo").unwrap();
        assert_eq!(
            url,
            PlatformUrl::Gitea {
                instance: "gitea.example.com".to_string(),
                project: "owner/repo".to_string(),
                tag: None,
            }
        );
    }

    #[test]
    fn gitea_with_tag() {
        let url =
            PlatformUrl::parse("https://gitea.example.com/owner/repo/releases/tag/v2.0").unwrap();
        assert_eq!(
            url,
            PlatformUrl::Gitea {
                instance: "gitea.example.com".to_string(),
                project: "owner/repo".to_string(),
                tag: Some("v2.0".to_string()),
            }
        );
    }

    #[test]
    fn empty_input() {
        assert!(PlatformUrl::parse("").is_none());
        assert!(PlatformUrl::parse("   ").is_none());
    }

    #[test]
    fn invalid_project() {
        assert!(PlatformUrl::parse("https://github.com/owner").is_none());
        assert!(PlatformUrl::parse("https://github.com/owner/").is_none());
    }

    #[test]
    fn trailing_slash() {
        let url = PlatformUrl::parse("https://github.com/owner/repo/").unwrap();
        assert_eq!(
            url,
            PlatformUrl::GitHub {
                project: "owner/repo".to_string(),
                tag: None,
            }
        );
    }
}
