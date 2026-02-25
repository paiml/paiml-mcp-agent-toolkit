#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper functions for name similarity analysis to reduce complexity

use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

use super::{NameInfo, NameSimilarityOutputFormat, NameSimilarityResult, SearchScope};
use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

/// Configuration for JSON results building
pub struct JsonResultsConfig<'a> {
    pub query: &'a str,
    pub all_names_len: usize,
    pub similarities: &'a [NameSimilarityResult],
    pub scope: &'a SearchScope,
    pub threshold: f32,
    pub phonetic: bool,
    pub fuzzy: bool,
    pub case_sensitive: bool,
    pub perf: bool,
    pub analysis_time: std::time::Duration,
    pub analyzed_files_len: usize,
}

/// Configuration for output formatting
pub struct OutputConfig<'a> {
    pub format: NameSimilarityOutputFormat,
    pub query: &'a str,
    pub all_names_len: usize,
    pub similarities: &'a [NameSimilarityResult],
    pub final_results: &'a Value,
    pub perf: bool,
    pub analysis_time: std::time::Duration,
    pub analyzed_files_len: usize,
    pub output: Option<PathBuf>,
}

// --- File discovery and identifier extraction ---
include!("name_similarity_helpers_discovery.rs");

// --- Similarity calculation ---
include!("name_similarity_helpers_similarity.rs");

// --- JSON building, output formatting ---
include!("name_similarity_helpers_output.rs");

// --- Tests ---
include!("name_similarity_helpers_tests.rs");
