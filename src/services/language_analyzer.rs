//! Language-aware analysis dispatcher per SPECIFICATION.md Section 6.2
//!
//! This module provides language-specific analysis capabilities by integrating
//! the language registry with existing analysis services.
//!
//! Split into submodules for file health compliance:
//! - language_analyzer_core.rs: Core analysis, comment detection, metadata
//! - language_analyzer_analyses.rs: Individual analysis implementations

#![cfg_attr(coverage_nightly, coverage(off))]

use super::language_registry::{Language, LanguageRegistry};
use super::service_base::ServiceMetrics;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Language-specific analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalysisRequest {
    pub path: PathBuf,
    pub language: Option<Language>,
    pub analysis_types: Vec<AnalysisType>,
    pub options: AnalysisOptions,
}

/// Available analysis types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    Complexity,
    Satd,
    DeadCode,
    Security,
    Style,
    Documentation,
    Dependencies,
    Metrics,
}

/// Analysis options for language-specific analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub complexity_threshold: u32,
    pub include_comments: bool,
    pub include_tests: bool,
    pub parallel_analysis: bool,
    pub output_format: OutputFormat,
}

pub use crate::contracts::OutputFormat;

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            complexity_threshold: 20,
            include_comments: true,
            include_tests: false,
            parallel_analysis: true,
            output_format: OutputFormat::Json,
        }
    }
}

/// Analysis results for a language-specific file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalysisResult {
    pub path: PathBuf,
    pub language: Language,
    pub analysis_results: Vec<AnalysisResult>,
    pub metadata: FileMetadata,
    pub processing_time_ms: u64,
}

/// Individual analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub analysis_type: AnalysisType,
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub lines_total: usize,
    pub lines_code: usize,
    pub lines_comment: usize,
    pub lines_blank: usize,
    pub file_size_bytes: u64,
    pub detected_language: Language,
    pub confidence: f64,
}

/// Comment style for different languages
#[derive(Debug, Clone, PartialEq)]
enum CommentStyle {
    CStyle,     // //
    Hash,       // #
    Semicolon,  // ;
    Percent,    // %
    DoubleDash, // --
    Xml,        // <!--
    None,       // No comments
}

/// Language-aware analysis service
pub struct LanguageAnalyzer {
    language_registry: LanguageRegistry,
    metrics: Arc<std::sync::Mutex<ServiceMetrics>>,
}

impl Default for LanguageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageAnalyzer {
    /// Create a new language analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            language_registry: LanguageRegistry::new(),
            metrics: Arc::new(std::sync::Mutex::new(ServiceMetrics::default())),
        }
    }
}

// Core analysis methods: analyze_file, supports_analysis, comment detection
include!("language_analyzer_core.rs");

// Individual analysis implementations: complexity, SATD, security, style, etc.
include!("language_analyzer_analyses.rs");
