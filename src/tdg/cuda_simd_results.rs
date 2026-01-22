//! CUDA-SIMD Result Types
//!
//! Extracted from cuda_simd.rs for file health compliance (CB-040).
//! Contains analysis result structures used by the CUDA-SIMD TDG analyzer.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::cuda_simd_defects::{DefectClass, DefectSeverity};
use super::cuda_simd_scores::PopperScore;

/// Detected defect in analyzed code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedDefect {
    /// Defect class from taxonomy
    pub defect_class: DefectClass,
    /// File where defect was found
    pub file_path: PathBuf,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Code snippet showing the issue
    pub snippet: Option<String>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Barrier safety analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BarrierSafetyResult {
    /// Total barrier synchronization points found
    pub total_barriers: usize,
    /// Barriers with guaranteed convergence
    pub safe_barriers: usize,
    /// Potentially unsafe barriers (early exit possible)
    pub unsafe_barriers: Vec<BarrierIssue>,
    /// Safety score (0.0 - 1.0)
    pub safety_score: f64,
}

/// Issue with a specific barrier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarrierIssue {
    /// Line number of barrier
    pub line: usize,
    /// Type of barrier (bar.sync, bar.arrive, etc.)
    pub barrier_type: String,
    /// Description of the issue
    pub issue: String,
    /// Paths that can exit before barrier
    pub exit_paths: Vec<String>,
}

/// Memory coalescing analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoalescingResult {
    /// Memory coalescing efficiency (0.0 - 1.0)
    pub efficiency: f64,
    /// Total memory operations analyzed
    pub total_operations: usize,
    /// Fully coalesced operations
    pub coalesced_operations: usize,
    /// Problematic access patterns
    pub problematic_accesses: Vec<MemoryAccessIssue>,
}

/// Issue with memory access pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessIssue {
    /// Line number
    pub line: usize,
    /// Access pattern type
    pub pattern: AccessPattern,
    /// Estimated performance impact
    pub impact: String,
}

/// Memory access pattern classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPattern {
    /// Contiguous access (stride 1) - optimal
    Contiguous,
    /// Strided access - partial coalescing
    Strided { stride: usize },
    /// Random access - no coalescing
    Random,
    /// Bank conflict in shared memory
    BankConflict { conflicting_banks: Vec<usize> },
}

/// Tile dimension validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileDimensionResult {
    /// Whether tile dimensions are valid
    pub valid: bool,
    /// Tile K dimension
    pub tile_k: Option<usize>,
    /// Tile KV dimension (for attention)
    pub tile_kv: Option<usize>,
    /// Head dimension
    pub head_dim: Option<usize>,
    /// Shared memory required
    pub shared_memory_required: Option<usize>,
    /// Shared memory available
    pub shared_memory_available: Option<usize>,
    /// Issues found
    pub issues: Vec<TileIssue>,
}

/// Issue with tile dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileIssue {
    /// Issue description
    pub description: String,
    /// Related ticket
    pub ticket: Option<String>,
    /// Severity
    pub severity: DefectSeverity,
}

/// Kaizen metrics for continuous improvement tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KaizenMetrics {
    /// Number of PARITY/PAR tickets resolved
    pub tickets_resolved: u32,
    /// Mean time to detect defects (hours)
    pub mttd: f64,
    /// Mean time to fix defects (hours)
    pub mttf: f64,
    /// Defect escape rate (defects found in production / total defects)
    pub escape_rate: f64,
    /// Regression rate (% of fixes that regressed)
    pub regression_rate: f64,
    /// Historical ticket references found in tests
    pub ticket_references: Vec<String>,
}

/// Complete CUDA-SIMD TDG analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaSimdTdgResult {
    /// File or directory analyzed
    pub path: PathBuf,
    /// 100-point Popper falsification score
    pub score: PopperScore,
    /// Detected defects
    pub defects: Vec<DetectedDefect>,
    /// Barrier safety analysis
    pub barrier_safety: BarrierSafetyResult,
    /// Memory coalescing analysis
    pub coalescing: CoalescingResult,
    /// Tile dimension validation
    pub tile_dimensions: TileDimensionResult,
    /// Kaizen continuous improvement metrics
    pub kaizen: KaizenMetrics,
    /// Analysis timestamp
    pub timestamp: String,
    /// Files analyzed
    pub files_analyzed: usize,
    /// CUDA files found
    pub cuda_files: usize,
    /// SIMD files found
    pub simd_files: usize,
    /// WGPU files found
    pub wgpu_files: usize,
}
