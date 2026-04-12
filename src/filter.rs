//! Asset name filtering with glob, include, and exclude patterns.

use fast_glob::glob_match;

/// Filter for matching asset names against glob patterns and keywords.
///
/// All matching is case-insensitive by default. Use [`Filter::case_sensitive`]
/// to opt into case-sensitive matching.
///
/// # Example
///
/// ```
/// use releasekit::filter::Filter;
///
/// let filter = Filter::new()
///     .glob("*.AppImage")
///     .include("x86_64")
///     .exclude("debug");
///
/// assert!(filter.matches("app-x86_64.AppImage"));
/// assert!(!filter.matches("app-x86_64-debug.AppImage"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Filter {
    globs: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    case_sensitive: bool,
}

impl Filter {
    /// Creates a new empty filter that matches everything.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a glob pattern that asset names must match.
    ///
    /// Multiple glob patterns are OR-ed: a name must match at least one.
    pub fn glob(mut self, pattern: impl Into<String>) -> Self {
        self.globs.push(pattern.into());
        self
    }

    /// Adds a keyword that asset names must contain.
    ///
    /// Multiple include keywords are OR-ed: a name must contain at least one.
    pub fn include(mut self, keyword: impl Into<String>) -> Self {
        self.include.push(keyword.into());
        self
    }

    /// Adds a keyword that asset names must not contain.
    ///
    /// Multiple exclude keywords are OR-ed: a name must not contain any of them.
    pub fn exclude(mut self, keyword: impl Into<String>) -> Self {
        self.exclude.push(keyword.into());
        self
    }

    /// Sets whether matching is case-sensitive.
    pub fn case_sensitive(mut self, yes: bool) -> Self {
        self.case_sensitive = yes;
        self
    }

    /// Returns `true` if the given name passes all filter rules.
    pub fn matches(&self, name: &str) -> bool {
        let name_lower = if self.case_sensitive {
            None
        } else {
            Some(name.to_lowercase())
        };

        let name_for_check = name_lower.as_deref().unwrap_or(name);

        if !self.globs.is_empty() {
            let glob_matches = self.globs.iter().any(|g| {
                let pattern = if self.case_sensitive {
                    g.clone()
                } else {
                    g.to_lowercase()
                };
                glob_match(&pattern, name_for_check)
            });
            if !glob_matches {
                return false;
            }
        }

        if !self.include.is_empty() {
            let has_include = self.include.iter().any(|k| {
                let keyword = if self.case_sensitive {
                    k.as_str()
                } else {
                    &k.to_lowercase()
                };
                name_for_check.contains(keyword)
            });
            if !has_include {
                return false;
            }
        }

        if !self.exclude.is_empty() {
            let has_exclude = self.exclude.iter().any(|k| {
                let keyword = if self.case_sensitive {
                    k.as_str()
                } else {
                    &k.to_lowercase()
                };
                name_for_check.contains(keyword)
            });
            if has_exclude {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_match() {
        let filter = Filter::new().glob("*.AppImage");
        assert!(filter.matches("app-1.0.AppImage"));
        assert!(!filter.matches("app-1.0.tar.gz"));
    }

    #[test]
    fn include_keyword() {
        let filter = Filter::new().include("x86_64");
        assert!(filter.matches("app-x86_64.AppImage"));
        assert!(!filter.matches("app-arm64.AppImage"));
    }

    #[test]
    fn exclude_keyword() {
        let filter = Filter::new().exclude("debug");
        assert!(!filter.matches("app-debug.AppImage"));
        assert!(filter.matches("app-1.0.AppImage"));
    }

    #[test]
    fn combined_filters() {
        let filter = Filter::new()
            .glob("*.AppImage")
            .include("x86_64")
            .exclude("debug");
        assert!(filter.matches("app-x86_64.AppImage"));
        assert!(!filter.matches("app-x86_64-debug.AppImage"));
        assert!(!filter.matches("app-arm64.AppImage"));
    }

    #[test]
    fn empty_filter_matches_all() {
        let filter = Filter::new();
        assert!(filter.matches("anything"));
    }

    #[test]
    fn case_insensitive_by_default() {
        let filter = Filter::new().include("x86_64");
        assert!(filter.matches("APP-X86_64.AppImage"));
    }

    #[test]
    fn case_sensitive_mode() {
        let filter = Filter::new().case_sensitive(true).include("x86_64");
        assert!(filter.matches("app-x86_64.AppImage"));
        assert!(!filter.matches("APP-X86_64.AppImage"));
    }
}
