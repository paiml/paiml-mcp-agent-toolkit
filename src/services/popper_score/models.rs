#![cfg_attr(coverage_nightly, coverage(off))]
//! Core data models for Popper Falsifiability Score v1.1
//!
//! Defines types for the 100-point normalized scoring system
//! with 6 categories following Karl Popper's falsifiability criterion.
//! Academic Foundation: 31 peer-reviewed papers (2022-2025)

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Comprehensive Popper Falsifiability Score (v1.1)
///
/// Total: 100 points normalized across 6 categories with gateway logic.
/// If Category A (Falsifiability) scores below 15/25 (60%), the total
/// score is 0 regardless of other categories (Popper's demarcation criterion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperScore {
    pub raw_score: f64,
    pub max_available: f64,
    pub normalized_score: f64,
    pub grade: PopperGrade,
    pub gateway_passed: bool,
    pub categories: PopperCategoryScores,
    pub recommendations: Vec<PopperRecommendation>,
    pub metadata: PopperMetadata,
    pub analysis: PopperAnalysis,
}

/// Letter grade based on normalized percentage.
/// Special case: InsufficientFalsifiability when gateway fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopperGrade {
    APlus,
    A,
    AMinus,
    BPlus,
    B,
    C,
    D,
    F,
    InsufficientFalsifiability,
}

/// Six scoring categories (100 points total when all applicable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperCategoryScores {
    /// A. Falsifiability & Testability (25pts) - GATEWAY
    pub falsifiability: PopperCategoryScore,
    /// B. Reproducibility Infrastructure (25pts)
    pub reproducibility: PopperCategoryScore,
    /// C. Transparency & Openness (20pts)
    pub transparency: PopperCategoryScore,
    /// D. Statistical Rigor (15pts)
    pub statistical_rigor: PopperCategoryScore,
    /// E. Historical Integrity (10pts)
    pub historical_integrity: PopperCategoryScore,
    /// F. ML/AI Reproducibility (5pts or N/A)
    pub ml_reproducibility: PopperCategoryScore,
}

/// Score for an individual category with sub-scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperCategoryScore {
    pub name: String,
    pub earned: f64,
    pub max: f64,
    pub is_applicable: bool,
    pub is_not_applicable: bool,
    pub sub_scores: Vec<PopperSubScore>,
    pub findings: Vec<PopperFinding>,
}

/// Sub-score within a category (e.g., A1, A2, A3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperSubScore {
    pub id: String,
    pub name: String,
    pub earned: f64,
    pub max: f64,
    pub description: String,
}

/// Severity level for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Positive,
    Info,
    Warning,
    Critical,
}

/// Finding discovered during analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperFinding {
    pub severity: FindingSeverity,
    pub message: String,
    pub location: Option<PathBuf>,
    pub impact: f64,
}

/// Priority level for recommendations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Actionable recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperRecommendation {
    pub category: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub potential_percent: f64,
    pub command: Option<String>,
}

/// Summary of Popperian analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopperAnalysis {
    pub falsifiability_status: AnalysisStatus,
    pub reproducibility_status: AnalysisStatus,
    pub scrutiny_status: AnalysisStatus,
    pub methodology_status: AnalysisStatus,
    pub validation_status: AnalysisStatus,
    pub verdict: String,
}

/// Status for each analysis dimension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnalysisStatus {
    Pass,
    Partial,
    #[default]
    Fail,
}

/// Metadata about the scoring analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperMetadata {
    pub timestamp: String,
    pub project_name: String,
    pub version: String,
    pub project_path: Option<PathBuf>,
}

// Include files: impl blocks and tests
include!("models_impls.rs");
include!("models_findings.rs");
include!("models_tests.rs");
