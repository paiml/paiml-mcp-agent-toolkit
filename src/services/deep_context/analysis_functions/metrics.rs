// Shared metrics computation (complexity, churn, dead code, duplicate detection,
// SATD, provability, DAG, Big-O)
// Extracted for file health (CB-040)

use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::services::complexity::{ComplexityReport, FileComplexityMetrics};
use crate::services::satd_detector::SATDAnalysisResult;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::path::PathBuf;
use tracing::{info, warn};

use super::super::DagType;

// Re-export thread-local caches so sibling submodules can access them
// These are defined here because metrics.rs owns the complexity cache logic

thread_local! {
    pub static RUST_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static TYPESCRIPT_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static PYTHON_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static GO_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static WASM_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
    pub static BASH_UNIFIED_CACHE: RefCell<FxHashMap<PathBuf, FileComplexityMetrics>> = RefCell::new(FxHashMap::default());
}

// Language detection and complexity analysis (detect_language, analyze_complexity, Lua helpers)
include!("metrics_complexity.rs");

// Churn, duplicate code, SATD, provability, DAG, Big-O analyses
include!("metrics_analyses.rs");

// Dead code analysis core (analyze_dead_code, Rust/TypeScript dead code helpers)
include!("metrics_dead_code.rs");

// Dead code helpers (Python dead code, name extractors, usage checks)
include!("metrics_dead_code_helpers.rs");
