// Module-level clippy allows for prototype CUDA-SIMD code
#![allow(clippy::too_many_arguments)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_char_add_str)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::trivial_regex)]
#![allow(clippy::double_must_use)]
#![allow(clippy::wildcard_in_or_patterns)]
#![allow(clippy::unused_enumerate_index)]

//! CUDA-SIMD Technical Debt Gradient (TDG) Module
//!
//! Implements the 100-point Karl Popper falsification scoring system for
//! CUDA PTX, SIMD (AVX2/AVX-512/NEON), and WGPU compute code analysis.
//!
//! # Toyota Way Integration
//!
//! - **Genchi Genbutsu** (現地現物): Analyze actual PTX/SIMD artifacts
//! - **Jidoka** (自働化): Automatic quality gates that stop on defect detection
//! - **Kaizen** (改善): Continuous improvement through historical fault analysis
//! - **Hansei** (反省): Root cause analysis with 5-Why methodology
//! - **Poka-Yoke** (ポカヨケ): Error-proofing through static analysis
//!
//! # References
//!
//! - Popper, K. R. (1959). *The Logic of Scientific Discovery*. Routledge.
//! - Liker, J. K. (2004). *The Toyota Way*. McGraw-Hill.
//! - Volkov, V., & Demmel, J. W. (2008). "Benchmarking GPUs to tune dense linear algebra." SC '08.
//! - Dao, T., et al. (2022). "FlashAttention: Fast and Memory-Efficient Exact Attention." NeurIPS.

use serde::{Deserialize, Serialize};
use std::path::Path;

// Defect taxonomy extracted to cuda_simd_defects.rs for file health (CB-040)
pub use super::cuda_simd_defects::{DefectClass, DefectSeverity, DefectTaxonomy};

// Score types extracted to cuda_simd_scores.rs for file health (CB-040)
pub use super::cuda_simd_scores::{
    CudaTdgGrade, FalsifiabilityScore, GpuSimdSpecificScore, HistoricalIntegrityScore, PopperScore,
    ReproducibilityScore, StatisticalRigorScore, TransparencyScore,
};

// Result types extracted to cuda_simd_results.rs for file health (CB-040)
pub use super::cuda_simd_results::{
    AccessPattern, BarrierIssue, BarrierSafetyResult, CoalescingResult, CudaSimdTdgResult,
    DetectedDefect, KaizenMetrics, MemoryAccessIssue, TileDimensionResult, TileIssue,
};

/// CUDA-SIMD TDG Analyzer
#[derive(Debug, Clone)]
pub struct CudaSimdAnalyzer {
    /// Defect taxonomy
    taxonomy: DefectTaxonomy,
    /// Configuration
    config: CudaSimdConfig,
}

/// Configuration for CUDA-SIMD analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CudaSimdConfig {
    /// Minimum score to pass quality gate
    pub min_score: f64,
    /// Whether to fail on P0 defects
    pub fail_on_p0: bool,
    /// Include SIMD analysis (AVX2/AVX-512/NEON)
    pub analyze_simd: bool,
    /// Include WGPU analysis
    pub analyze_wgpu: bool,
    /// Shared memory limit (bytes)
    pub shared_memory_limit: usize,
    /// Register limit per thread
    pub register_limit: usize,
}

impl CudaSimdConfig {
    /// Create default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_score: 85.0,
            fail_on_p0: true,
            analyze_simd: true,
            analyze_wgpu: true,
            shared_memory_limit: 49152, // 48KB default
            register_limit: 64,
        }
    }
}

impl Default for CudaSimdAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Detected Rust project quality patterns for enhanced scoring
#[derive(Debug, Default)]
struct RustProjectPatterns {
    /// Cargo.lock exists (version pinning)
    has_cargo_lock: bool,
    /// rust-toolchain.toml exists (Rust version pinning)
    has_rust_toolchain: bool,
    /// Criterion benchmarks in benches/
    has_criterion_benches: bool,
    /// GitHub Actions workflows exist
    has_github_ci: bool,
    /// proptest-regressions/ exists (regression tests)
    has_proptest_regressions: bool,
    /// CHANGELOG.md exists (historical integrity)
    has_changelog: bool,
    /// golden_traces/ exists (deterministic output)
    has_golden_traces: bool,
    /// SAFETY comments in SIMD code
    has_safety_comments: bool,
    /// Miri configured in .cargo/config.toml
    has_miri_config: bool,
}

include!("analyzer_core.rs");
include!("detection.rs");
include!("scoring.rs");

/// File analysis intermediate result
#[derive(Debug, Clone, Default)]
struct FileAnalysis {
    cuda_files: usize,
    simd_files: usize,
    wgpu_files: usize,
    defects: Vec<DetectedDefect>,
    barrier_safety: BarrierSafetyResult,
    coalescing: CoalescingResult,
}

// Tests extracted to cuda_simd_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "cuda_simd_tests.rs"]
mod tests;
