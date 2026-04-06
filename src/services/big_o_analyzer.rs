#![cfg_attr(coverage_nightly, coverage(off))]
//! Big-O Complexity Analyzer - Phase 5 implementation
//!
//! Provides algorithmic complexity analysis for functions using
//! pattern matching and heuristic analysis. This analyzer examines source code
//! to estimate the time and space complexity of functions across multiple
//! programming languages.
//!
//! # Key Features
//!
//! - **Multi-language Support**: Analyzes Rust, JavaScript, TypeScript, Python, Go, Java, C/C++
//! - **Time Complexity Detection**: Identifies O(1), O(log n), O(n), O(n log n), O(n^2), O(n^3), O(2^n)
//! - **Space Complexity Analysis**: Detects dynamic memory allocation patterns
//! - **Pattern Recognition**: Identifies common algorithmic patterns (sorting, searching, recursion)
//! - **Confidence Scoring**: Provides confidence levels for complexity estimates
//!
//! # Example
//!
//! ```no_run
//! use pmat::services::big_o_analyzer::{BigOAnalyzer, BigOAnalysisConfig};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let analyzer = BigOAnalyzer::new();
//!
//! let config = BigOAnalysisConfig {
//!     project_path: PathBuf::from("./src"),
//!     include_patterns: vec!["*.rs".to_string()],
//!     exclude_patterns: vec!["test_*.rs".to_string()],
//!     confidence_threshold: 70,
//!     analyze_space_complexity: true,
//! };
//!
//! let report = analyzer.analyze(config).await?;
//!
//! println!("Analyzed {} functions", report.analyzed_functions);
//! println!("Found {} high-complexity functions", report.high_complexity_functions.len());
//!
//! // Print complexity distribution
//! println!("O(n^2) functions: {}", report.complexity_distribution.quadratic);
//! println!("O(n) functions: {}", report.complexity_distribution.linear);
//!
//! // Get recommendations
//! for recommendation in &report.recommendations {
//!     println!("  {}", recommendation);
//! }
//! # Ok(())
//! # }
//! ```

use crate::models::complexity_bound::{BigOClass, ComplexityBound};

#[cfg(test)]
#[path = "big_o_analyzer_issue54_test.rs"]
mod issue54_tests;
use crate::services::complexity_patterns::{ComplexityAnalysisResult, ComplexityPatternMatcher};
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Big-O complexity analyzer service
pub struct BigOAnalyzer {
    #[allow(dead_code)]
    pattern_matcher: ComplexityPatternMatcher,
}

/// Analysis configuration
#[derive(Debug, Clone)]
pub struct BigOAnalysisConfig {
    pub project_path: PathBuf,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub confidence_threshold: u8,
    pub analyze_space_complexity: bool,
}

/// Big-O analysis report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BigOAnalysisReport {
    pub analyzed_functions: usize,
    pub complexity_distribution: ComplexityDistribution,
    pub high_complexity_functions: Vec<FunctionComplexity>,
    pub pattern_matches: Vec<PatternMatch>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplexityDistribution {
    pub constant: usize,
    pub logarithmic: usize,
    pub linear: usize,
    pub linearithmic: usize,
    pub quadratic: usize,
    pub cubic: usize,
    pub exponential: usize,
    pub unknown: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionComplexity {
    pub file_path: PathBuf,
    pub function_name: String,
    pub line_number: usize,
    pub time_complexity: ComplexityBound,
    pub space_complexity: ComplexityBound,
    pub confidence: u8,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub occurrences: usize,
    pub typical_complexity: BigOClass,
}

impl BigOAnalyzer {
    /// Create new Big-O analyzer
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::big_o_analyzer::BigOAnalyzer;
    ///
    /// let analyzer = BigOAnalyzer::new();
    /// // Analyzer is ready to analyze code complexity
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            pattern_matcher: ComplexityPatternMatcher::new(),
        }
    }
}

impl Default for BigOAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// Include implementation files
include!("big_o_analyzer_detection.rs");
include!("big_o_analyzer_analysis.rs");
include!("big_o_analyzer_report.rs");
include!("big_o_analyzer_inline_tests.rs");
