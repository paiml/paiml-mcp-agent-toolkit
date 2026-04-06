#![cfg_attr(coverage_nightly, coverage(off))]
//! Concrete implementations of defect analyzers for various quality issues.
//!
//! This module provides specific analyzers that implement the `DefectAnalyzer`
//! trait to detect different types of code quality issues. Each analyzer focuses
//! on a specific category of defects and can be composed to provide comprehensive
//! code quality assessment.
//!
//! # Analyzer Types
//!
//! - **`ComplexityDefectAnalyzer`**: Detects overly complex code using TDG metrics
//! - **`DeadCodeDefectAnalyzer`**: Identifies unreachable and unused code
//! - **`DuplicateDefectAnalyzer`**: Finds duplicated code blocks and patterns
//! - **`PerformanceDefectAnalyzer`**: Detects performance anti-patterns
//! - **`SecurityDefectAnalyzer`**: Identifies potential security vulnerabilities
//! - **`TechnicalDebtAnalyzer`**: Tracks self-admitted technical debt (SATD)

use crate::models::defect_report::{Defect, DefectCategory, Severity};
use crate::models::tdg::{TDGScore, TDGSeverity};
use crate::services::{
    big_o_analyzer::FunctionComplexity,
    dead_code_analyzer::{DeadCodeItem, UnreachableBlock},
    defect_analyzer::{AnalyzerConfig, DefectAnalyzer},
    duplicate_detector::{CloneGroup, CloneType},
    satd_detector::{DebtCategory, TechnicalDebt},
};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Helper function to discover source files
async fn discover_source_files(path: &Path) -> Result<Vec<PathBuf>> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use walkdir::WalkDir;

    let mut files = Vec::new();
    let extensions = ["rs", "js", "ts", "py", "java", "cpp", "c", "go"];

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext) {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

// --- Struct and config definitions ---

/// Adapter for complexity analysis (TDG-based)
pub struct ComplexityDefectAnalyzer;

#[derive(Clone)]
pub struct ComplexityConfig {
    pub max_tdg_score: f64,
    pub high_threshold: f64,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            max_tdg_score: 2.0,
            high_threshold: 1.5,
        }
    }
}

impl AnalyzerConfig for ComplexityConfig {}

#[derive(Clone, Default)]
pub struct SATDConfig {
    pub include_test_files: bool,
}

impl AnalyzerConfig for SATDConfig {}

/// Adapter for SATD (Self-Admitted Technical Debt) detection
pub struct SATDDefectAnalyzer {
    detector: crate::services::satd_detector::SATDDetector,
}

/// Adapter for dead code detection
pub struct DeadCodeDefectAnalyzer;

#[derive(Clone)]
pub struct DeadCodeConfig {
    pub min_confidence: f64,
}

impl Default for DeadCodeConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
        }
    }
}

impl AnalyzerConfig for DeadCodeConfig {}

/// Adapter for code duplication detection
pub struct DuplicationDefectAnalyzer {
    detector: crate::services::duplicate_detector::DuplicateDetectionEngine,
}

#[derive(Clone)]
pub struct DuplicationConfig {
    pub min_similarity: f64,
}

impl Default for DuplicationConfig {
    fn default() -> Self {
        Self {
            min_similarity: 0.8,
        }
    }
}

impl AnalyzerConfig for DuplicationConfig {}

/// Adapter for performance analysis (Big-O)
pub struct PerformanceDefectAnalyzer {
    analyzer: crate::services::big_o_analyzer::BigOAnalyzer,
}

#[derive(Clone, Default)]
pub struct PerformanceConfig {
    pub include_nlogn: bool,
}

impl AnalyzerConfig for PerformanceConfig {}

/// Adapter for architecture issues detection
pub struct ArchitectureDefectAnalyzer;

#[derive(Clone)]
pub struct ArchitectureConfig {
    pub max_coupling: usize,
}

impl Default for ArchitectureConfig {
    fn default() -> Self {
        Self { max_coupling: 10 }
    }
}

impl AnalyzerConfig for ArchitectureConfig {}

// --- Include files with trait impls and helper methods ---

// ComplexityDefectAnalyzer trait impl and TDG-based complexity detection helpers
include!("defect_analyzers_complexity.rs");

// SATDDefectAnalyzer constructors, trait impl, and technical debt detection helpers
include!("defect_analyzers_debt.rs");

// DeadCodeDefectAnalyzer constructors, trait impl, and dead/unreachable code detection helpers
include!("defect_analyzers_dead_code.rs");

// DuplicationDefectAnalyzer constructors, trait impl, and code clone detection helpers
include!("defect_analyzers_duplication.rs");

// PerformanceDefectAnalyzer and ArchitectureDefectAnalyzer constructors, trait impls, and helpers
include!("defect_analyzers_performance.rs");

// Fixed: TDGComponents dead_code field added to all initializers
#[cfg(test)]
#[path = "defect_analyzers_tests.rs"]
mod tests;
