#![cfg_attr(coverage_nightly, coverage(off))]
//! Concurrent Deep Context Analysis with World-Class Performance
//! Uses proper parallel processing with tokio::join! and rayon

use crate::services::deep_context::*;
use crate::services::progress::{MultiProgress, ProgressBar, ProgressStyle};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Number of parallel analyses executed during deep context generation
/// (complexity, provability, satd, churn, dag, tdg, big_o, dead_code)
const ANALYSIS_COUNT: u64 = 8;

// ─── Re-exports ──────────────────────────────────────────────────────────────

pub use crate::services::deep_context::{ChurnAnalysis, DependencyGraph};
pub use crate::services::lightweight_provability_analyzer::{FunctionId, ProofSummary};
pub use crate::services::satd_detector::SATDAnalysisResult;

// ─── Core types ──────────────────────────────────────────────────────────────

/// Enhanced Deep Context Analyzer with proper concurrency
pub struct ConcurrentDeepContextAnalyzer {
    config: DeepContextConfig,
    progress: MultiProgress,
}

impl ConcurrentDeepContextAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(config: DeepContextConfig) -> Self {
        Self {
            config,
            progress: MultiProgress::new(),
        }
    }
}

// ─── Supporting types ────────────────────────────────────────────────────────

/// Ast cache.
pub struct AstCache {
    data: std::collections::HashMap<std::path::PathBuf, ParsedAst>,
}

impl AstCache {
    fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    fn insert(&mut self, path: std::path::PathBuf, ast: ParsedAst) {
        self.data.insert(path, ast);
    }

    fn files(&self) -> &std::collections::HashMap<std::path::PathBuf, ParsedAst> {
        &self.data
    }
}

#[derive(Default)]
/// Parsed ast.
pub struct ParsedAst {
    // AST representation
}

#[derive(Default)]
/// Result of complexity operation.
pub struct ComplexityResult {
    // Complexity metrics
}

#[derive(Default)]
/// Complexity results.
pub struct ComplexityResults {
    // Combined complexity results
}

impl ComplexityResults {
    fn combine(_results: Vec<ComplexityResult>) -> Self {
        Self::default()
    }
}

#[derive(Default)]
/// Big o results.
pub struct BigOResults {
    // Big-O analysis results
}

#[derive(Default)]
/// T d g results.
pub struct TDGResults {
    // Technical debt gradient results
}

#[derive(Default)]
/// Dead code results.
pub struct DeadCodeResults {
    // Dead code detection results
}

/// Combined analyses.
pub struct CombinedAnalyses {
    pub complexity: ComplexityResults,
    pub provability: Vec<ProofSummary>,
    pub satd: SATDAnalysisResult,
    pub churn: ChurnAnalysis,
    pub dag: DependencyGraph,
    pub tdg: TDGResults,
    pub big_o: BigOResults,
    pub dead_code: DeadCodeResults,
}

/// Result of deep analysis operation.
pub struct DeepAnalysisResult {
    pub analyses: CombinedAnalyses,
    pub timestamp: std::time::SystemTime,
}

// ─── Included implementation files ───────────────────────────────────────────

include!("deep_context_concurrent_analysis.rs");
include!("deep_context_concurrent_tests.rs");
