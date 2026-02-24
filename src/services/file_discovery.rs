#![cfg_attr(coverage_nightly, coverage(off))]
//! Intelligent file discovery service with multi-level filtering
//!
//! Provides file discovery capabilities that respect project boundaries,
//! ignore patterns, and categorize files for optimal analysis. Implements
//! a multi-stage filtering pipeline with gitignore integration, external
//! repository detection, smart categorization, and parallel traversal.

use anyhow::Result;
use ignore::{DirEntry, WalkBuilder, WalkState};
use lazy_static::lazy_static;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::{debug, trace};

use crate::services::file_classifier::FileClassifier;

/// File categorization for deep context analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileCategory {
    SourceCode,      // .rs, .ts, .py - full AST analysis
    EssentialDoc,    // README.md - compress and include
    BuildConfig,     // Makefile, Cargo.toml - compress and include
    GeneratedOutput, // *deep_context*.md - exclude
    DevelopmentDoc,  // docs/*.md - exclude from defect analysis
    TestArtifact,    // test_*.md - exclude
}

lazy_static! {
    /// Patterns for detecting external repository clones
    static ref EXTERNAL_REPO_PATTERNS: RegexSet = RegexSet::new([
        r"https?___",                    // Cloned external repos
        r".*___github_com_.*",           // GitHub clones
        r".*___gitlab_com_.*",           // GitLab clones
        r".*___bitbucket_org_.*",        // Bitbucket clones
        r".*\$\$external\$\$.*",         // Other external markers
        r".*/external_deps/.*",          // External dependencies directory
        r".*/third_party_repos/.*",      // Third party repos
    ]).expect("Invalid regex patterns");

    /// Additional ignore patterns beyond .gitignore
    static ref ADDITIONAL_IGNORE_PATTERNS: Vec<&'static str> = vec![
        "/.cargo/registry/",
        "/.cargo/git/",
        "/.rustup/",
        "/site-packages/",
        "/.venv/",
        "/venv/",
        "/.tox/",
        "/__pycache__/",
        "/.mypy_cache/",
        "/.pytest_cache/",
        "/.gradle/",
        "/gradle/",
        "/.m2/",
        "/.ivy2/",
        "/.sbt/",
        "/.coursier/",
        "/bazel-*/",
        "/.ccache/",
        "/.cache/",
        // Minified and bundled files
        "*.min.js",
        "*.min.css",
        "*.bundle.js",
        "*-bundle.js",
        "*.production.js",
        "*.prod.js",
        "*-min.js",
        "*.packed.js",
        "*.dist.js",
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiscoveryConfig {
    /// Maximum depth to traverse
    pub max_depth: Option<usize>,
    /// Whether to follow symlinks
    pub follow_links: bool,
    /// Whether to respect .gitignore files
    pub respect_gitignore: bool,
    /// Whether to filter external repositories
    pub filter_external_repos: bool,
    /// Additional ignore patterns
    pub custom_ignore_patterns: Vec<String>,
    /// Maximum number of files to discover
    pub max_files: Option<usize>,
}

impl Default for FileDiscoveryConfig {
    fn default() -> Self {
        Self {
            max_depth: Some(15),
            follow_links: false,
            respect_gitignore: true,
            filter_external_repos: true,
            custom_ignore_patterns: vec![],
            max_files: Some(50_000), // Safety limit
        }
    }
}

pub struct ProjectFileDiscovery {
    root: PathBuf,
    config: FileDiscoveryConfig,
    classifier: Arc<FileClassifier>,
}

impl ProjectFileDiscovery {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            config: FileDiscoveryConfig::default(),
            classifier: Arc::new(FileClassifier::default()),
        }
    }

    #[must_use]
    pub fn with_config(mut self, config: FileDiscoveryConfig) -> Self {
        self.config = config;
        self
    }

    #[must_use]
    pub fn with_classifier(mut self, classifier: Arc<FileClassifier>) -> Self {
        self.classifier = classifier;
        self
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DiscoveryStats {
    pub total_files: usize,
    pub files_by_extension: std::collections::HashMap<String, usize>,
    pub files_by_category: std::collections::HashMap<String, usize>,
    pub discovered_paths: Vec<PathBuf>,
}

/// External repository filter for precise detection
pub struct ExternalRepoFilter {
    patterns: RegexSet,
}

impl ExternalRepoFilter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            patterns: EXTERNAL_REPO_PATTERNS.clone(),
        }
    }

    #[must_use]
    pub fn is_external_dependency(&self, entry: &DirEntry) -> bool {
        let path_str = entry.path().to_string_lossy();
        self.patterns.is_match(&path_str)
    }
}

impl Default for ExternalRepoFilter {
    fn default() -> Self {
        Self::new()
    }
}

// Include implementation files
include!("file_discovery_walker.rs");
include!("file_discovery_filters.rs");
include!("file_discovery_categorization.rs");
include!("file_discovery_tests.rs");
