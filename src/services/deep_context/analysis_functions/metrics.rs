// Shared metrics computation (complexity, churn, dead code, duplicate detection,
// SATD, provability, DAG, Big-O)
// Extracted for file health (CB-040)

use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::services::complexity::{ComplexityReport, FileComplexityMetrics};
use crate::services::satd_detector::SATDAnalysisResult;
use dashmap::DashMap;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::LazyLock;
use tracing::{info, warn};

use super::super::DagType;

// Process-global caches shared across all tokio tasks.
// Previously these were thread_local! RefCell<FxHashMap> which couldn't share
// across the parallel analysis phases (AST, Complexity, Provability, DAG).
// DashMap provides lock-free concurrent reads/writes across threads.

pub(super) static RUST_UNIFIED_CACHE: LazyLock<DashMap<PathBuf, FileComplexityMetrics>> =
    LazyLock::new(|| DashMap::with_capacity(1024));
pub(super) static TYPESCRIPT_UNIFIED_CACHE: LazyLock<DashMap<PathBuf, FileComplexityMetrics>> =
    LazyLock::new(|| DashMap::with_capacity(256));
pub(super) static PYTHON_UNIFIED_CACHE: LazyLock<DashMap<PathBuf, FileComplexityMetrics>> =
    LazyLock::new(|| DashMap::with_capacity(256));
pub(super) static GO_UNIFIED_CACHE: LazyLock<DashMap<PathBuf, FileComplexityMetrics>> =
    LazyLock::new(|| DashMap::with_capacity(256));
pub(super) static WASM_UNIFIED_CACHE: LazyLock<DashMap<PathBuf, FileComplexityMetrics>> =
    LazyLock::new(|| DashMap::with_capacity(64));
pub(super) static BASH_UNIFIED_CACHE: LazyLock<DashMap<PathBuf, FileComplexityMetrics>> =
    LazyLock::new(|| DashMap::with_capacity(64));

// Language detection and complexity analysis (detect_language, analyze_complexity, Lua helpers)
include!("metrics_complexity.rs");

// Churn, duplicate code, SATD, provability, DAG, Big-O analyses
include!("metrics_analyses.rs");

// Dead code analysis core (analyze_dead_code, Rust/TypeScript dead code helpers)
include!("metrics_dead_code.rs");

// Dead code helpers (Python dead code, name extractors, usage checks)
include!("metrics_dead_code_helpers.rs");
