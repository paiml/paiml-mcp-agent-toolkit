//! File filtering utilities for include/exclude patterns
//!
//! Provides glob-based filtering for analysis commands.

use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};
use anyhow::Result;

/// File filter that applies include/exclude patterns
#[derive(Debug, Clone)]
pub struct FileFilter {
    include_set: Option<GlobSet>,
    exclude_set: Option<GlobSet>,
}

impl FileFilter {
    /// Create a new file filter from include/exclude patterns
    pub fn new(include_patterns: Vec<String>, exclude_patterns: Vec<String>) -> Result<Self> {
        let include_set = if !include_patterns.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in include_patterns {
                builder.add(Glob::new(&pattern)?);
            }
            Some(builder.build()?)
        } else {
            None
        };

        let exclude_set = if !exclude_patterns.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in exclude_patterns {
                builder.add(Glob::new(&pattern)?);
            }
            Some(builder.build()?)
        } else {
            None
        };

        Ok(Self {
            include_set,
            exclude_set,
        })
    }

    /// Check if a file path should be included based on the filters
    pub fn should_include(&self, path: &Path) -> bool {
        // If exclude patterns are specified and the path matches, exclude it
        if let Some(ref exclude_set) = self.exclude_set {
            if exclude_set.is_match(path) {
                return false;
            }
        }

        // If include patterns are specified, only include if path matches
        if let Some(ref include_set) = self.include_set {
            return include_set.is_match(path);
        }

        // No include patterns specified, include by default
        true
    }

    /// Filter a list of paths based on include/exclude patterns
    pub fn filter_paths(&self, paths: Vec<PathBuf>) -> Vec<PathBuf> {
        paths
            .into_iter()
            .filter(|path| self.should_include(path))
            .collect()
    }

    /// Check if any filters are active
    pub fn has_filters(&self) -> bool {
        self.include_set.is_some() || self.exclude_set.is_some()
    }
}

impl Default for FileFilter {
    fn default() -> Self {
        Self {
            include_set: None,
            exclude_set: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_patterns() {
        let filter = FileFilter::new(
            vec!["src/**/*.rs".to_string()],
            vec![],
        ).unwrap();

        assert!(filter.should_include(Path::new("src/main.rs")));
        assert!(filter.should_include(Path::new("src/lib/mod.rs")));
        assert!(!filter.should_include(Path::new("tests/test.rs")));
        assert!(!filter.should_include(Path::new("src/main.toml")));
    }

    #[test]
    fn test_exclude_patterns() {
        let filter = FileFilter::new(
            vec![],
            vec!["tests/**".to_string(), "*.tmp".to_string()],
        ).unwrap();

        assert!(filter.should_include(Path::new("src/main.rs")));
        assert!(!filter.should_include(Path::new("tests/test.rs")));
        assert!(!filter.should_include(Path::new("file.tmp")));
    }

    #[test]
    fn test_combined_patterns() {
        let filter = FileFilter::new(
            vec!["**/*.rs".to_string()],
            vec!["tests/**".to_string()],
        ).unwrap();

        assert!(filter.should_include(Path::new("src/main.rs")));
        assert!(!filter.should_include(Path::new("tests/test.rs")));
        assert!(!filter.should_include(Path::new("src/config.toml")));
    }

    #[test]
    fn test_filter_paths() {
        let filter = FileFilter::new(
            vec!["src/**/*.rs".to_string()],
            vec!["src/generated/**".to_string()],
        ).unwrap();

        let paths = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
            PathBuf::from("src/generated/code.rs"),
            PathBuf::from("tests/test.rs"),
            PathBuf::from("README.md"),
        ];

        let filtered = filter.filter_paths(paths);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&PathBuf::from("src/main.rs")));
        assert!(filtered.contains(&PathBuf::from("src/lib.rs")));
    }

    #[test]
    fn test_no_filters() {
        let filter = FileFilter::default();
        assert!(!filter.has_filters());
        assert!(filter.should_include(Path::new("any/path.rs")));
    }
}