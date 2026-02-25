#![cfg_attr(coverage_nightly, coverage(off))]
// Toyota Way: Unified Duplicate Detection Strategy

use super::{
    DetectionConfig, DetectionInput, DetectionOutput, Detector, DetectorCapabilities,
    DetectorSpecificConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Duplicate detection strategy using the existing duplicate detector
pub struct DuplicateDetector;

impl Default for DuplicateDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DuplicateDetector {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Duplicate detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateConfig {
    pub similarity_threshold: f64,
    pub min_lines: usize,
    pub hash_count: usize,
    pub ignore_whitespace: bool,
    pub cross_language: bool,
}

impl Default for DuplicateConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.8,
            min_lines: 3,
            hash_count: 128,
            ignore_whitespace: true,
            cross_language: true,
        }
    }
}

/// Duplicate detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetectionResult {
    pub duplicates: Vec<DuplicateGroup>,
    pub summary: DuplicateSummary,
}

/// Group of duplicate code fragments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub id: String,
    pub similarity: f64,
    pub fragments: Vec<CodeFragment>,
    pub clone_type: CloneType,
}

/// Individual code fragment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFragment {
    pub file: std::path::PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub hash: String,
}

/// Summary of duplicate detection analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateSummary {
    pub total_groups: usize,
    pub total_duplicates: usize,
    pub files_analyzed: usize,
    pub time_saved_hours: f64,
}

/// Type of code clone detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloneType {
    /// Exact clones (modulo whitespace)
    Type1 { similarity: f64 },
    /// Parametric clones (identifiers/literals differ)
    Type2 { similarity: f64, normalized: bool },
    /// Structural clones (statements added/removed)
    Type3 { similarity: f64, ast_distance: f64 },
}

// Detector trait implementation and DuplicateDetector methods
include!("duplicates_detection.rs");

// Property tests and data type unit tests (config, result, group, fragment, summary)
include!("duplicates_tests.rs");

// CloneType tests, Detector trait integration tests, and DetectorCapabilities tests
include!("duplicates_tests_detector.rs");
