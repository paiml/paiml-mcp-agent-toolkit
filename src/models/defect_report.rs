#![cfg_attr(coverage_nightly, coverage(off))]
//! Unified defect report model for aggregating all quality issues
//!
//! This module defines the core structures for the comprehensive defect
//! reporting system that consolidates results from all analysis commands.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

/// Unique identifier for a defect
pub type DefectId = String;

/// Comprehensive defect report containing all quality issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectReport {
    /// Metadata about the report generation
    pub metadata: ReportMetadata,
    /// All defects found across the codebase
    pub defects: Vec<Defect>,
    /// Aggregated summary statistics
    pub summary: DefectSummary,
    /// Index mapping files to their defects
    pub file_index: BTreeMap<PathBuf, Vec<DefectId>>,
}

/// Metadata about the report generation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Tool name and version
    pub tool: String,
    /// Version of the tool
    pub version: String,
    /// When the report was generated
    pub generated_at: DateTime<Utc>,
    /// Root directory of the analyzed project
    pub project_root: PathBuf,
    /// Total number of files analyzed
    pub total_files_analyzed: usize,
    /// Time taken to generate the report in milliseconds
    pub analysis_duration_ms: u64,
}

/// Individual defect found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defect {
    /// Unique identifier for this defect
    pub id: DefectId,
    /// Severity level of the defect
    pub severity: Severity,
    /// Category of the defect
    pub category: DefectCategory,
    /// File path relative to project root
    pub file_path: PathBuf,
    /// Starting line number
    pub line_start: u32,
    /// Ending line number (if applicable)
    pub line_end: Option<u32>,
    /// Starting column number (if applicable)
    pub column_start: Option<u32>,
    /// Ending column number (if applicable)
    pub column_end: Option<u32>,
    /// Human-readable description of the defect
    pub message: String,
    /// Rule identifier that triggered this defect
    pub rule_id: String,
    /// Suggested fix or refactoring (if available)
    pub fix_suggestion: Option<String>,
    /// Additional metrics associated with the defect
    pub metrics: HashMap<String, f64>,
}

/// Severity levels for defects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Low impact issues
    Low,
    /// Medium impact issues
    Medium,
    /// High impact issues requiring attention
    High,
    /// Critical issues requiring immediate attention
    Critical,
}

/// Categories of defects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefectCategory {
    /// Cyclomatic/cognitive complexity violations
    Complexity,
    /// Self-admitted technical debt markers
    TechnicalDebt,
    /// Unreachable or unused code
    DeadCode,
    /// Code duplication
    Duplication,
    /// Performance issues (O(n^2) or worse)
    Performance,
    /// Architecture/coupling issues
    Architecture,
    /// Insufficient test coverage
    TestCoverage,
}

/// Summary statistics for the defect report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectSummary {
    /// Total number of defects found
    pub total_defects: usize,
    /// Breakdown by severity
    pub by_severity: BTreeMap<String, usize>,
    /// Breakdown by category
    pub by_category: BTreeMap<String, usize>,
    /// Top files by defect count
    pub hotspot_files: Vec<FileHotspot>,
}

/// File with high defect density
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHotspot {
    /// File path relative to project root
    pub path: PathBuf,
    /// Number of defects in this file
    pub defect_count: usize,
    /// Weighted severity score
    pub severity_score: f64,
}

/// Configuration for file ranking
#[derive(Debug, Clone)]
pub struct FileRankingConfig {
    /// Whether to include severity in scoring
    pub use_severity: bool,
    /// Whether to include defect count in scoring
    pub use_count: bool,
    /// Custom weights for different categories
    pub category_weights: HashMap<DefectCategory, f64>,
}

/// Result of file ranking operation
#[derive(Debug, Clone)]
pub struct RankedFile {
    /// Rank position (1-based)
    pub rank: usize,
    /// Computed score
    pub score: f64,
    /// File path
    pub path: PathBuf,
    /// Defects in this file
    pub defects: Vec<Defect>,
}

// --- Implementation blocks ---
include!("defect_report_impls.rs");

// --- Unit tests ---
include!("defect_report_tests.rs");

// --- Serialization, clone, and debug tests ---
include!("defect_report_serialization_tests.rs");

// --- Property tests ---
include!("defect_report_property_tests.rs");
