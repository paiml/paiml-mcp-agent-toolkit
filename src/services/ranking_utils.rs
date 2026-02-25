#![cfg_attr(coverage_nightly, coverage(off))]
//! Ranking utilities for prioritizing analysis results
//!
//! This module provides utilities for ranking and prioritizing files based on
//! analysis results such as complexity metrics, defect counts, and code quality
//! indicators. It helps developers focus on the most problematic areas of code.
//!
//! # Features
//!
//! - **Flexible Ranking**: Configure top-N files and minimum score thresholds
//! - **Metric-based Scoring**: Uses complexity, coverage, and defect metrics
//! - **Severity Mapping**: Automatically maps metrics to severity levels
//! - **Result Builder**: Fluent API for creating analysis results
//! - **Table Formatting**: Pretty-print ranked results for CLI display
//!
//! # Ranking Strategy
//!
//! Files are ranked based on:
//! 1. **Complexity Metrics**: Cyclomatic, cognitive complexity
//! 2. **Defect Severity**: Critical > High > Medium > Low
//! 3. **Defect Count**: Total number of issues found
//! 4. **Coverage Metrics**: Lower coverage increases priority
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::ranking_utils::{RankingConfig, apply_file_ranking, AnalysisResultBuilder};
//! use pmat::services::defect_analyzer::MetricValue;
//! use std::path::PathBuf;
//!
//! # fn example() {
//! // Configure ranking
//! let config = RankingConfig {
//!     top_files: 10,
//!     min_score: Some(5.0),
//! };
//!
//! // Create analysis results
//! let results = vec![
//!     AnalysisResultBuilder::new(PathBuf::from("complex.rs"))
//!         .add_metric_int("cyclomatic", 25)
//!         .with_entity("calculate", "function")
//!         .build(),
//!     AnalysisResultBuilder::new(PathBuf::from("simple.rs"))
//!         .add_metric_int("cyclomatic", 3)
//!         .with_entity("helper", "function")
//!         .build(),
//! ];
//!
//! // Apply ranking
//! let ranked = apply_file_ranking(results, &config, |r| r.clone());
//!
//! // Top file should be complex.rs
//! assert_eq!(ranked[0].1, 1); // rank 1
//! # }
//! ```

use crate::models::defect_report::{Defect, DefectCategory, Severity};
use crate::services::defect_analyzer::{
    AnalysisContext, AnalysisResult, FileRankingEngine, LineInfo, LineRange, MetricValue,
    RankedFile, SimpleScorer,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Configuration for file ranking
#[derive(Default)]
pub struct RankingConfig {
    /// Maximum number of files to return (0 = all files)
    pub top_files: usize,
    /// Minimum score threshold (files below this are excluded)
    pub min_score: Option<f64>,
}

/// Helper to create analysis results from common patterns
pub struct AnalysisResultBuilder {
    file_path: PathBuf,
    absolute_path: PathBuf,
    line_start: u32,
    line_end: Option<u32>,
    column_start: u32,
    column_end: Option<u32>,
    metrics: BTreeMap<String, MetricValue>,
    description: String,
    entity_name: Option<String>,
    entity_type: Option<String>,
}

// --- Included submodules ---

// Scoring: apply_file_ranking, result_to_defect, compute_severity_from_metrics
include!("ranking_utils_scoring.rs");

// Builder: impl AnalysisResultBuilder methods
include!("ranking_utils_builder.rs");

// Formatting: format_ranked_files_table
include!("ranking_utils_formatting.rs");

// Tests: unit tests and property tests
include!("ranking_utils_tests.rs");
