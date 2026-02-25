#![cfg_attr(coverage_nightly, coverage(off))]
//! Advanced code similarity detection with entropy analysis
//!
//! Implements multiple algorithms for detecting code clones and similarities:
//! - Winnowing for fingerprinting
//! - AST-based structural similarity
//! - Token-based semantic similarity
//! - Shannon entropy analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Configuration for similarity detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityConfig {
    pub min_lines: usize,
    pub min_tokens: usize,
    pub similarity_threshold: f64,
    pub enable_entropy: bool,
    pub enable_ast: bool,
    pub enable_semantic: bool,
    pub window_size: usize,
    pub k_gram_size: usize,
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            min_lines: 6,
            min_tokens: 50,
            similarity_threshold: 0.7,
            enable_entropy: true,
            enable_ast: true,
            enable_semantic: true,
            window_size: 40,
            k_gram_size: 15,
        }
    }
}

/// Types of code clones
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CloneType {
    Type1, // Exact clones
    Type2, // Renamed clones
    Type3, // Modified clones
    Type4, // Semantic clones
}

/// A duplicate or similar code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarBlock {
    pub id: String,
    pub locations: Vec<Location>,
    pub similarity: f64,
    pub clone_type: CloneType,
    pub lines: usize,
    pub tokens: usize,
    pub content_preview: String,
}

/// Location of a code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: Option<usize>,
    pub end_column: Option<usize>,
}

/// Entropy analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyReport {
    pub average_entropy: f64,
    pub high_entropy_blocks: Vec<EntropyBlock>,
    pub low_entropy_patterns: Vec<EntropyBlock>,
    pub recommendations: Vec<String>,
}

/// A code block with entropy measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyBlock {
    pub location: Location,
    pub entropy: f64,
    pub category: String,
    pub suggestion: String,
}

/// Refactoring hint for similar code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringHint {
    pub locations: Vec<Location>,
    pub pattern: String,
    pub suggestion: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// Comprehensive analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveReport {
    pub exact_duplicates: Vec<SimilarBlock>,
    pub structural_similarities: Vec<SimilarBlock>,
    pub semantic_similarities: Vec<SimilarBlock>,
    pub entropy_analysis: Option<EntropyReport>,
    pub refactoring_opportunities: Vec<RefactoringHint>,
    pub metrics: Metrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub duplication_percentage: f64,
    pub average_entropy: f64,
    pub total_clones: usize,
}

/// Main similarity detector
pub struct SimilarityDetector {
    config: SimilarityConfig,
    #[allow(dead_code)] // Will be used in future winnowing implementation
    winnower: Winnowing,
    token_analyzer: TokenAnalyzer,
    entropy_calculator: EntropyCalculator,
}

struct CodeBlock {
    start_line: usize,
    end_line: usize,
    content: String,
}

/// Winnowing algorithm for fingerprinting
pub struct Winnowing {
    window_size: usize,
    k_gram_size: usize,
}

/// Token-based analysis for semantic similarity
struct TokenAnalyzer;

type TokenVector = HashMap<String, f64>;

/// Shannon entropy calculator
struct EntropyCalculator;

// SimilarityDetector: public API and private helper methods
include!("similarity_detection.rs");

// Winnowing: fingerprinting algorithm implementation
include!("similarity_winnowing.rs");

// TokenAnalyzer and EntropyCalculator: token-based and entropy analysis
include!("similarity_analyzers.rs");

// Tests extracted to similarity_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "similarity_tests.rs"]
mod tests;
