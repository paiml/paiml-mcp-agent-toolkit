#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/deep_context/analyzer_formatting.rs
//!
//! Tests the pure formatting methods of `DeepContextAnalyzer` which produce
//! markdown, JSON, and SARIF output from `DeepContext` structures.
//! All methods under test are pure (no filesystem access).

use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use crate::models::dead_code::{DeadCodeAnalysisConfig, DeadCodeRankingResult, DeadCodeSummary};
use crate::models::project_meta::{BuildInfo, ProjectOverview};
use crate::services::complexity::{
    ComplexityMetrics, ComplexityReport, ComplexitySummary, FileComplexityMetrics,
    FunctionComplexity,
};
use crate::services::context::{AstItem, FileContext};
use crate::services::deep_context::{
    AnnotatedFileTree, AnnotatedNode, CrossLangReference, CrossLangReferenceType, DeepContext,
    DeepContextAnalyzer, DeepContextConfig, DefectAnnotations, DefectHotspot, DefectSummary,
    EnhancedFileContext, FileLocation, Impact, NodeAnnotations, NodeType,
    PrioritizedRecommendation, Priority, QualityScorecard, RefactoringEstimate,
};
use crate::services::satd_detector::{DebtCategory, SATDAnalysisResult, SATDSummary, Severity};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helper constructors
// ---------------------------------------------------------------------------
include!("coverage_boost_analyzer_formatting_helpers.rs");

// ---------------------------------------------------------------------------
// Comprehensive markdown tests
// ---------------------------------------------------------------------------
include!("coverage_boost_analyzer_formatting_comprehensive_markdown.rs");

// ---------------------------------------------------------------------------
// Legacy markdown tests
// ---------------------------------------------------------------------------
include!("coverage_boost_analyzer_formatting_legacy_markdown.rs");

// ---------------------------------------------------------------------------
// Legacy analysis JSON tests
// ---------------------------------------------------------------------------
include!("coverage_boost_analyzer_formatting_legacy_analysis_json.rs");

// ---------------------------------------------------------------------------
// SARIF format tests
// ---------------------------------------------------------------------------
include!("coverage_boost_analyzer_formatting_sarif.rs");

// ---------------------------------------------------------------------------
// Enhanced AST section tests
// ---------------------------------------------------------------------------
include!("coverage_boost_analyzer_formatting_enhanced_ast.rs");

// ---------------------------------------------------------------------------
// Edge case tests
// ---------------------------------------------------------------------------
include!("coverage_boost_analyzer_formatting_edge_cases.rs");
