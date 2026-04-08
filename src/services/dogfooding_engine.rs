//! Dogfooding Engine for Self-Analysis Artifacts
//!
//! This module implements the self-bootstrapping artifact generation system
//! that deterministically produces dogfooding artifacts by analyzing the
//! codebase's own AST and git history.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::models::error::TemplateError;
use crate::services::git_analysis::GitAnalysisService;
use crate::services::unified_ast_engine::{AstForest, ProjectMetrics, UnifiedAstEngine};
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Engine for generating self-analysis dogfooding artifacts
pub struct DogfoodingEngine {
    ast_engine: UnifiedAstEngine,
}

/// Context information extracted from a single file
#[derive(Debug, Clone)]
pub struct FileContext {
    pub path: PathBuf,
    pub functions: usize,
    pub structs: usize,
    pub traits: usize,
    pub max_complexity: u32,
    pub lines: usize,
}

/// Git churn metrics for the project
#[derive(Debug, Clone)]
pub struct ChurnMetrics {
    pub files_changed: usize,
    pub commit_count: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub hotspots: Vec<FileHotspot>,
}

#[derive(Debug, Clone)]
/// Hotspot identified in file analysis.
pub struct FileHotspot {
    pub path: PathBuf,
    pub change_count: usize,
    pub complexity_score: u32,
    pub risk_score: f64,
}

/// DAG metrics for dependency analysis
#[derive(Debug, Clone)]
pub struct DagMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub diameter: usize,
    pub clustering: f64,
    pub strongly_connected_components: usize,
}

impl DogfoodingEngine {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            ast_engine: UnifiedAstEngine::new(),
        }
    }
}

impl Default for DogfoodingEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Generator methods: generate_ast_context, generate_combined_metrics,
// generate_complexity_analysis
include!("dogfooding_engine_generators.rs");

// Analysis and helper methods: get_churn_metrics, generate_churn_analysis,
// generate_server_info, analyze_all_files, analyze_single_file,
// compute_dag_metrics, compute_metrics_hash
include!("dogfooding_engine_analysis.rs");

// Unit tests and property tests
include!("dogfooding_engine_tests.rs");
