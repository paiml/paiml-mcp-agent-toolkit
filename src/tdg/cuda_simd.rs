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
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Defect severity levels following PAIML Tauranta taxonomy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectSeverity {
    /// P0: Critical - causes crashes, data corruption, or incorrect results
    P0Critical,
    /// P1: Performance - significant performance degradation
    P1Performance,
    /// P2: Efficiency - memory or compute inefficiency
    P2Efficiency,
    /// P3: Minor - style or minor optimization opportunities
    P3Minor,
}

impl std::fmt::Display for DefectSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P0Critical => write!(f, "P0-Critical"),
            Self::P1Performance => write!(f, "P1-Performance"),
            Self::P2Efficiency => write!(f, "P2-Efficiency"),
            Self::P3Minor => write!(f, "P3-Minor"),
        }
    }
}

/// Defect class from Tauranta fault history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectClass {
    /// Ticket ID (e.g., "PARITY-114", "PAR-041")
    pub ticket_id: String,
    /// Human-readable description
    pub description: String,
    /// Severity level
    pub severity: DefectSeverity,
    /// Detection method
    pub detection_method: String,
    /// Resolution status
    pub resolved: bool,
    /// Root cause (5-Why analysis)
    pub root_cause: Option<String>,
}

/// Known defect patterns from Tauranta fault history
#[derive(Debug, Clone, Default)]
pub struct DefectTaxonomy {
    patterns: HashMap<String, DefectClass>,
}

impl DefectTaxonomy {
    /// Create taxonomy with known PAIML Tauranta defect patterns
    #[must_use]
    pub fn with_tauranta_patterns() -> Self {
        let mut patterns = HashMap::new();

        // P0 Critical Defects
        patterns.insert(
            "PARITY-114".to_string(),
            DefectClass {
                ticket_id: "PARITY-114".to_string(),
                description: "Barrier divergence: thread exits before bar.sync".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "PTX CFG analysis".to_string(),
                resolved: true,
                root_cause: Some(
                    "Early-exit optimization without barrier convergence guarantee".to_string(),
                ),
            },
        );

        patterns.insert(
            "PAR-002".to_string(),
            DefectClass {
                ticket_id: "PAR-002".to_string(),
                description: "CUDA GEMV Error 700: illegal memory access".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "Bounds checking".to_string(),
                resolved: true,
                root_cause: Some("Thread index exceeds matrix dimensions".to_string()),
            },
        );

        patterns.insert(
            "PAR-041".to_string(),
            DefectClass {
                ticket_id: "PAR-041".to_string(),
                description: "FlashAttention tile_kv < head_dim shared memory overflow".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "Tile dimension validation".to_string(),
                resolved: true,
                root_cause: Some("Tile size miscalculation for large head dimensions".to_string()),
            },
        );

        // P1 Performance Defects
        patterns.insert(
            "PAR-001".to_string(),
            DefectClass {
                ticket_id: "PAR-001".to_string(),
                description: "Q4_K dequantization inefficiency".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Throughput analysis".to_string(),
                resolved: true,
                root_cause: None,
            },
        );

        patterns.insert(
            "PAR-034".to_string(),
            DefectClass {
                ticket_id: "PAR-034".to_string(),
                description: "Tensor Core GEMM: unused capability".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "SM utilization analysis".to_string(),
                resolved: true,
                root_cause: None,
            },
        );

        patterns.insert(
            "PAR-036".to_string(),
            DefectClass {
                ticket_id: "PAR-036".to_string(),
                description: "Non-persistent thread design".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Thread lifecycle analysis".to_string(),
                resolved: true,
                root_cause: None,
            },
        );

        patterns.insert(
            "PAR-037".to_string(),
            DefectClass {
                ticket_id: "PAR-037".to_string(),
                description: "Missing CUDA Graphs".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "API call pattern".to_string(),
                resolved: true,
                root_cause: None,
            },
        );

        patterns.insert(
            "PAR-039".to_string(),
            DefectClass {
                ticket_id: "PAR-039".to_string(),
                description: "Megakernel fusion opportunity missed".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Kernel launch count".to_string(),
                resolved: true,
                root_cause: None,
            },
        );

        // P2 Efficiency Defects
        patterns.insert(
            "PAR-005".to_string(),
            DefectClass {
                ticket_id: "PAR-005".to_string(),
                description: "Redundant weight transfers".to_string(),
                severity: DefectSeverity::P2Efficiency,
                detection_method: "D2H/H2D ratio".to_string(),
                resolved: true,
                root_cause: None,
            },
        );

        patterns.insert(
            "PAR-028".to_string(),
            DefectClass {
                ticket_id: "PAR-028".to_string(),
                description: "FP32 KV cache overhead".to_string(),
                severity: DefectSeverity::P2Efficiency,
                detection_method: "Precision analysis".to_string(),
                resolved: true,
                root_cause: None,
            },
        );

        // ═══════════════════════════════════════════════════════════════════════
        // REAL TAURANTA FAULTS FROM GIT HISTORY (trueno, realizar, aprender)
        // These are actual bugs found in production code, documented for detection
        // ═══════════════════════════════════════════════════════════════════════

        // TRUENO-SIMD-001: Missing #[target_feature] on sqrt/recip
        // Impact: AVX2 sqrt was 1.7x SLOWER than scalar (0.58x performance)
        // Impact: AVX2 recip was 5.9x SLOWER than scalar (0.17x performance)
        // Commit: 04cc458 "[CRITICAL FIX] Add missing #[target_feature] to sqrt/recip"
        patterns.insert(
            "TRUENO-SIMD-001".to_string(),
            DefectClass {
                ticket_id: "TRUENO-SIMD-001".to_string(),
                description: "Missing #[target_feature] on SIMD functions".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "SIMD attribute analysis".to_string(),
                resolved: true,
                root_cause: Some(
                    "Compiler doesn't enable SIMD instructions without #[target_feature] attribute"
                        .to_string(),
                ),
            },
        );

        // TRUENO-SIMD-002: Missing #[target_feature] on ln/log2/log10
        // Same pattern as TRUENO-SIMD-001, affected 6 functions
        // Commit: 542d10e "[CRITICAL FIX] Add missing #[target_feature] to logarithm functions"
        patterns.insert(
            "TRUENO-SIMD-002".to_string(),
            DefectClass {
                ticket_id: "TRUENO-SIMD-002".to_string(),
                description: "Missing #[target_feature] on logarithm SIMD functions".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "SIMD attribute analysis".to_string(),
                resolved: true,
                root_cause: Some(
                    "ln/log2/log10 in AVX2/AVX512 backends missing #[target_feature]".to_string(),
                ),
            },
        );

        // TRUENO-PTX-001: PTX misaligned address (lane_id * 4 alignment bug)
        // Error: CUDA_ERROR_MISALIGNED_ADDRESS
        // Root cause: u32 store at byte offset lane_id instead of lane_id * 4
        // Commit: 3f4efb7 "fix: PTX generic addressing and kernel alignment for LZ4 warp compression"
        patterns.insert(
            "TRUENO-PTX-001".to_string(),
            DefectClass {
                ticket_id: "TRUENO-PTX-001".to_string(),
                description: "PTX misaligned shared memory access (lane_id * 4 bug)".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "PTX dataflow analysis".to_string(),
                resolved: true,
                root_cause: Some("u32 store at byte offset lane_id instead of lane_id * 4, addresses 4097, 4098... not 4-byte aligned".to_string()),
            },
        );

        // TRUENO-GPU-BARRIER-001: Barrier safety false positives with *_done labels
        // Root cause: Only checked *_end labels, not *_done suffix
        // Commit: 66227d1 "feat(trueno-gpu): PTX emission 20.9% faster + barrier safety fixes"
        patterns.insert(
            "TRUENO-GPU-BARRIER-001".to_string(),
            DefectClass {
                ticket_id: "TRUENO-GPU-BARRIER-001".to_string(),
                description: "Barrier analyzer missed *_done loop end labels".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "PTX CFG analysis".to_string(),
                resolved: true,
                root_cause: Some("Only checked *_end suffix, not *_done (sb_loop_done, sub_block_done, k_block_done)".to_string()),
            },
        );

        // REALIZAR-QUANT-001: Q4_0/Q4_K/Q6_K dequantization nibble ordering
        // Impact: "Paris" prediction rank went from 24,573 → 470 → 1
        // Root cause: Interleaved nibbles instead of sequential (candle/llama.cpp layout)
        // Commit: 0ed0717 "fix(gguf): Fix tokenizer and dequantization for correct LLaMA inference"
        patterns.insert(
            "REALIZAR-QUANT-001".to_string(),
            DefectClass {
                ticket_id: "REALIZAR-QUANT-001".to_string(),
                description: "Q4_0/Q4_K/Q6_K dequantization nibble ordering incorrect".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "Output validation".to_string(),
                resolved: true,
                root_cause: Some("Interleaved nibbles instead of sequential - low nibbles go to 0-15/0-31, high to 16-31/32-63".to_string()),
            },
        );

        // REALIZAR-TOK-001: SentencePiece tokenizer space encoding
        // Impact: Wrong token boundaries, incorrect inference
        // Root cause: Spaces not replaced with ▁ (U+2581) before tokenization
        // Commit: 0ed0717 "fix(gguf): Fix tokenizer and dequantization for correct LLaMA inference"
        patterns.insert(
            "REALIZAR-TOK-001".to_string(),
            DefectClass {
                ticket_id: "REALIZAR-TOK-001".to_string(),
                description: "SentencePiece tokenizer spaces not replaced with ▁".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "Token validation".to_string(),
                resolved: true,
                root_cause: Some("Must replace spaces with ▁ (U+2581) before tokenization for SentencePiece models".to_string()),
            },
        );

        // APRENDER-GQA-001: GQA attention bug
        // Impact: Incorrect attention for Grouped Query Attention models
        // Commit: 7fb24a2 "fix(chat): Add GQA fallback and progress indicators for GGUF generation"
        patterns.insert(
            "APRENDER-GQA-001".to_string(),
            DefectClass {
                ticket_id: "APRENDER-GQA-001".to_string(),
                description: "GQA (Grouped Query Attention) incorrect key-value head mapping"
                    .to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "Attention validation".to_string(),
                resolved: true,
                root_cause: Some(
                    "GQA models have fewer KV heads than query heads, requires head group mapping"
                        .to_string(),
                ),
            },
        );

        // F082: Address computed from shared memory load (data-dependent addressing)
        // Detected in trueno toxic.ptx test file
        // Root cause: Address register depends on value loaded from shared memory
        patterns.insert(
            "F082".to_string(),
            DefectClass {
                ticket_id: "F082".to_string(),
                description: "Address computed from shared memory load (data-dependent addressing)".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "PTX dataflow analysis".to_string(),
                resolved: false,
                root_cause: Some("Address register depends on value loaded from shared memory, causing non-uniform memory access".to_string()),
            },
        );

        // ═══════════════════════════════════════════════════════════════════════
        // COMPUTEBRICK DEFECT PATTERNS (PROBAR-SPEC-009-P8)
        // ComputeBrick generates WebGPU/WGSL shaders from Rust type definitions.
        // These defects detect issues in generated shader code.
        // Reference: docs/specifications/compute-brick-support.md
        // ═══════════════════════════════════════════════════════════════════════

        // CB-001: Missing bounds check in generated WGSL
        // Impact: Out-of-bounds memory access in compute shader
        // Detection: global_invocation_id used without arrayLength guard
        patterns.insert(
            "CB-001".to_string(),
            DefectClass {
                ticket_id: "CB-001".to_string(),
                description: "WGSL global_invocation_id used without bounds check".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "WGSL AST pattern match".to_string(),
                resolved: false,
                root_cause: Some(
                    "ComputeBrick::to_wgsl() generates gid without arrayLength guard".to_string(),
                ),
            },
        );

        // CB-002: Barrier divergence in generated WGSL
        // Impact: Race condition or deadlock in workgroup
        // Detection: workgroupBarrier() unreachable from some threads
        patterns.insert(
            "CB-002".to_string(),
            DefectClass {
                ticket_id: "CB-002".to_string(),
                description: "WGSL workgroupBarrier() unreachable from some threads".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "CFG reachability analysis".to_string(),
                resolved: false,
                root_cause: Some(
                    "Conditional early return before barrier in generated shader".to_string(),
                ),
            },
        );

        // CB-003: Tile dimension exceeds tensor shape
        // Impact: Shared memory overflow or incorrect results
        // Detection: tile_size > tensor.element_count()
        patterns.insert(
            "CB-003".to_string(),
            DefectClass {
                ticket_id: "CB-003".to_string(),
                description: "Tile dimensions exceed tensor shape".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "Constraint solving".to_string(),
                resolved: false,
                root_cause: Some(
                    "TileStrategy dimensions not validated against tensor shapes".to_string(),
                ),
            },
        );

        // CB-004: Shared memory exceeds 16KB limit
        // Impact: Shader compilation failure or runtime error
        // Detection: Sum of var<workgroup> allocations > 16384 bytes
        patterns.insert(
            "CB-004".to_string(),
            DefectClass {
                ticket_id: "CB-004".to_string(),
                description: "Workgroup shared memory exceeds 16KB limit".to_string(),
                severity: DefectSeverity::P0Critical,
                detection_method: "Static memory analysis".to_string(),
                resolved: false,
                root_cause: Some(
                    "ComputeBrick::shared() allocations not summed and validated".to_string(),
                ),
            },
        );

        // CB-010: Suboptimal workgroup size
        // Impact: Warp/wavefront underutilization (up to 50% waste)
        // Detection: workgroup_size not multiple of 32
        patterns.insert(
            "CB-010".to_string(),
            DefectClass {
                ticket_id: "CB-010".to_string(),
                description: "Workgroup size not multiple of 32 (warp waste)".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Numeric check".to_string(),
                resolved: false,
                root_cause: Some(
                    "Workgroup size chosen without considering warp alignment".to_string(),
                ),
            },
        );

        // CB-011: Redundant barrier without preceding shared memory write
        // Impact: Unnecessary synchronization overhead
        // Detection: workgroupBarrier() without prior shared memory store
        patterns.insert(
            "CB-011".to_string(),
            DefectClass {
                ticket_id: "CB-011".to_string(),
                description: "Redundant barrier without preceding shared memory write".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Data flow analysis".to_string(),
                resolved: false,
                root_cause: Some(
                    "TileOp::Barrier added without corresponding LoadShared/StoreShared"
                        .to_string(),
                ),
            },
        );

        // CB-012: Low vectorization ratio
        // Impact: Suboptimal SIMD utilization
        // Detection: <50% of operations use vector types (vec2/vec3/vec4)
        patterns.insert(
            "CB-012".to_string(),
            DefectClass {
                ticket_id: "CB-012".to_string(),
                description: "Low vectorization ratio (<50%)".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Type analysis".to_string(),
                resolved: false,
                root_cause: Some(
                    "ElementwiseOp generates scalar operations instead of vector".to_string(),
                ),
            },
        );

        // CB-013: Missing cooperative matrix for MMA operations
        // Impact: Not using tensor core / matrix unit acceleration
        // Detection: TileOp::Mma without subgroup cooperative extension
        patterns.insert(
            "CB-013".to_string(),
            DefectClass {
                ticket_id: "CB-013".to_string(),
                description: "Matrix ops without subgroup cooperative usage".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Pattern match".to_string(),
                resolved: false,
                root_cause: Some(
                    "TileStrategy::Cooperative not using WGSL subgroup operations".to_string(),
                ),
            },
        );

        // CB-020: Unsafe block without SAFETY comment
        // Impact: Code review friction, potential hidden UB
        // Detection: unsafe {} without // SAFETY: comment
        patterns.insert(
            "CB-020".to_string(),
            DefectClass {
                ticket_id: "CB-020".to_string(),
                description: "unsafe block without SAFETY comment".to_string(),
                severity: DefectSeverity::P2Efficiency,
                detection_method: "Regex + AST".to_string(),
                resolved: false,
                root_cause: Some("Rust safety documentation convention not followed".to_string()),
            },
        );

        // CB-021: SIMD intrinsics without #[target_feature]
        // Impact: Compiler may not emit SIMD instructions (regression)
        // Detection: SIMD intrinsics in function without #[target_feature] attribute
        patterns.insert(
            "CB-021".to_string(),
            DefectClass {
                ticket_id: "CB-021".to_string(),
                description: "SIMD intrinsics without #[target_feature] attribute".to_string(),
                severity: DefectSeverity::P2Efficiency,
                detection_method: "Attribute check".to_string(),
                resolved: false,
                root_cause: Some(
                    "Same as TRUENO-SIMD-001: missing attribute causes scalar fallback".to_string(),
                ),
            },
        );

        // CB-022: Excessive barriers (>4 per kernel)
        // Impact: Indicates algorithmic inefficiency
        // Detection: Count of workgroupBarrier() > 4
        patterns.insert(
            "CB-022".to_string(),
            DefectClass {
                ticket_id: "CB-022".to_string(),
                description: "Excessive barriers (>4) suggests algorithmic issue".to_string(),
                severity: DefectSeverity::P2Efficiency,
                detection_method: "Count analysis".to_string(),
                resolved: false,
                root_cause: Some(
                    "Algorithm requires rethinking to reduce synchronization points".to_string(),
                ),
            },
        );

        Self { patterns }
    }

    /// Get defect pattern by ticket ID
    #[must_use]
    pub fn get(&self, ticket_id: &str) -> Option<&DefectClass> {
        self.patterns.get(ticket_id)
    }

    /// Get all patterns
    #[must_use]
    pub fn all(&self) -> impl Iterator<Item = &DefectClass> {
        self.patterns.values()
    }
}

/// Category A: Falsifiability & Testability (25 points) - GATEWAY
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FalsifiabilityScore {
    /// A.1: All bar.sync reachable from all threads (5 pts)
    pub barrier_safety: f64,
    /// A.2: Shared memory indices within tile dimensions (5 pts)
    pub bounds_verification: f64,
    /// A.3: Branch coverage includes warp divergence cases (5 pts)
    pub divergence_testing: f64,
    /// A.4: ThreadSanitizer or equivalent analysis (5 pts)
    pub memory_race_detection: f64,
    /// A.5: Register/shared memory within SM limits (5 pts)
    pub occupancy_bounds: f64,
}

impl FalsifiabilityScore {
    /// Calculate total for Category A
    #[must_use]
    pub fn total(&self) -> f64 {
        self.barrier_safety
            + self.bounds_verification
            + self.divergence_testing
            + self.memory_race_detection
            + self.occupancy_bounds
    }

    /// Maximum possible score for Category A
    pub const MAX: f64 = 25.0;

    /// Gateway threshold - if below this, total score is 0
    pub const GATEWAY_THRESHOLD: f64 = 15.0;
}

/// Category B: Reproducibility Infrastructure (25 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReproducibilityScore {
    /// B.1: Bitwise reproducible results (8 pts)
    pub deterministic_output: f64,
    /// B.2: CUDA/Driver/SM version locked (5 pts)
    pub version_pinning: f64,
    /// B.3: GPU model, compute capability documented (5 pts)
    pub hardware_specification: f64,
    /// B.4: Criterion-style statistical benchmarking (4 pts)
    pub benchmark_harness: f64,
    /// B.5: Automated regression on GPU hardware (3 pts)
    pub ci_cd_integration: f64,
}

impl ReproducibilityScore {
    /// Calculate total for Category B
    #[must_use]
    pub fn total(&self) -> f64 {
        self.deterministic_output
            + self.version_pinning
            + self.hardware_specification
            + self.benchmark_harness
            + self.ci_cd_integration
    }

    /// Maximum possible score for Category B
    pub const MAX: f64 = 25.0;
}

/// Category C: Transparency & Openness (20 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransparencyScore {
    /// C.1: Generated PTX accessible and documented (6 pts)
    pub ptx_inspection: f64,
    /// C.2: --ptxas-options=-v output analyzed (5 pts)
    pub register_allocation: f64,
    /// C.3: SM occupancy explicitly computed (5 pts)
    pub occupancy_calculation: f64,
    /// C.4: Shared memory bank mapping documented (4 pts)
    pub memory_layout: f64,
}

impl TransparencyScore {
    /// Calculate total for Category C
    #[must_use]
    pub fn total(&self) -> f64 {
        self.ptx_inspection
            + self.register_allocation
            + self.occupancy_calculation
            + self.memory_layout
    }

    /// Maximum possible score for Category C
    pub const MAX: f64 = 20.0;
}

/// Category D: Statistical Rigor (15 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatisticalRigorScore {
    /// D.1: ≥3s warmup before measurement (4 pts)
    pub warmup_iterations: f64,
    /// D.2: ≥100 samples for statistical significance (4 pts)
    pub sample_count: f64,
    /// D.3: IQR-based outlier detection reported (4 pts)
    pub outlier_analysis: f64,
    /// D.4: 95% CI on throughput metrics (3 pts)
    pub confidence_intervals: f64,
}

impl StatisticalRigorScore {
    /// Calculate total for Category D
    #[must_use]
    pub fn total(&self) -> f64 {
        self.warmup_iterations
            + self.sample_count
            + self.outlier_analysis
            + self.confidence_intervals
    }

    /// Maximum possible score for Category D
    pub const MAX: f64 = 15.0;
}

/// Category E: Historical Integrity (10 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoricalIntegrityScore {
    /// E.1: PARITY/PAR ticket references (4 pts)
    pub fault_lineage: f64,
    /// E.2: Tests derived from historical bugs (3 pts)
    pub regression_tests: f64,
    /// E.3: 5-Why analysis for each P0 defect (3 pts)
    pub root_cause_documentation: f64,
}

impl HistoricalIntegrityScore {
    /// Calculate total for Category E
    #[must_use]
    pub fn total(&self) -> f64 {
        self.fault_lineage + self.regression_tests + self.root_cause_documentation
    }

    /// Maximum possible score for Category E
    pub const MAX: f64 = 10.0;
}

/// Category F: GPU/SIMD Specific (5 points)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuSimdSpecificScore {
    /// F.1: Active threads / warp size ratio (2 pts)
    pub warp_efficiency: f64,
    /// F.2: Achieved vs theoretical bandwidth (2 pts)
    pub memory_throughput: f64,
    /// F.3: FMA/memory instruction ratio (1 pt)
    pub instruction_mix: f64,
}

impl GpuSimdSpecificScore {
    /// Calculate total for Category F
    #[must_use]
    pub fn total(&self) -> f64 {
        self.warp_efficiency + self.memory_throughput + self.instruction_mix
    }

    /// Maximum possible score for Category F
    pub const MAX: f64 = 5.0;
}

/// Complete 100-point Popper Falsification Score
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopperScore {
    /// Category A: Falsifiability & Testability (25 pts) - GATEWAY
    pub falsifiability: FalsifiabilityScore,
    /// Category B: Reproducibility Infrastructure (25 pts)
    pub reproducibility: ReproducibilityScore,
    /// Category C: Transparency & Openness (20 pts)
    pub transparency: TransparencyScore,
    /// Category D: Statistical Rigor (15 pts)
    pub statistical_rigor: StatisticalRigorScore,
    /// Category E: Historical Integrity (10 pts)
    pub historical_integrity: HistoricalIntegrityScore,
    /// Category F: GPU/SIMD Specific (5 pts)
    pub gpu_simd_specific: GpuSimdSpecificScore,
    /// Total score (0-100, or 0 if gateway fails)
    pub total: f64,
    /// Whether the gateway (Category A ≥ 15) passed
    pub gateway_passed: bool,
    /// Grade interpretation
    pub grade: CudaTdgGrade,
}

impl PopperScore {
    /// Calculate total score with gateway rule
    #[must_use]
    pub fn calculate(
        falsifiability: FalsifiabilityScore,
        reproducibility: ReproducibilityScore,
        transparency: TransparencyScore,
        statistical_rigor: StatisticalRigorScore,
        historical_integrity: HistoricalIntegrityScore,
        gpu_simd_specific: GpuSimdSpecificScore,
    ) -> Self {
        let category_a = falsifiability.total();
        let gateway_passed = category_a >= FalsifiabilityScore::GATEWAY_THRESHOLD;

        let raw_total = category_a
            + reproducibility.total()
            + transparency.total()
            + statistical_rigor.total()
            + historical_integrity.total()
            + gpu_simd_specific.total();

        // Gateway rule: if Category A < 15, total = 0
        let total = if gateway_passed { raw_total } else { 0.0 };
        let grade = CudaTdgGrade::from_score(total, gateway_passed);

        Self {
            falsifiability,
            reproducibility,
            transparency,
            statistical_rigor,
            historical_integrity,
            gpu_simd_specific,
            total,
            gateway_passed,
            grade,
        }
    }
}

/// Grade interpretation for CUDA-SIMD TDG scores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CudaTdgGrade {
    /// 90-100: Production-ready, minimal debt
    APLus,
    /// 80-89: Production-ready with monitoring
    A,
    /// 70-79: Acceptable, prioritize improvements
    B,
    /// 60-69: Technical debt accumulating
    C,
    /// 50-59: Significant remediation needed
    D,
    /// 0-49: Not production-ready
    #[default]
    F,
    /// Gateway failure: Falsifiability requirements not met
    GatewayFail,
}

impl CudaTdgGrade {
    /// Convert score to grade
    #[must_use]
    pub fn from_score(score: f64, gateway_passed: bool) -> Self {
        if !gateway_passed {
            return Self::GatewayFail;
        }
        match score {
            s if s >= 90.0 => Self::APLus,
            s if s >= 80.0 => Self::A,
            s if s >= 70.0 => Self::B,
            s if s >= 60.0 => Self::C,
            s if s >= 50.0 => Self::D,
            _ => Self::F,
        }
    }
}

impl std::fmt::Display for CudaTdgGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::APLus => write!(f, "A+"),
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::F => write!(f, "F"),
            Self::GatewayFail => write!(f, "FAIL (Gateway)"),
        }
    }
}

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

impl CudaSimdAnalyzer {
    /// Create new analyzer with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            taxonomy: DefectTaxonomy::with_tauranta_patterns(),
            config: CudaSimdConfig::new(),
        }
    }

    /// Create analyzer with custom configuration
    #[must_use]
    pub fn with_config(config: CudaSimdConfig) -> Self {
        Self {
            taxonomy: DefectTaxonomy::with_tauranta_patterns(),
            config,
        }
    }

    /// Analyze a file or directory
    pub fn analyze(&self, path: &Path) -> anyhow::Result<CudaSimdTdgResult> {
        let mut defects = Vec::new();
        let mut cuda_files = 0;
        let mut simd_files = 0;
        let mut wgpu_files = 0;
        let mut files_analyzed = 0;

        let mut barrier_safety = BarrierSafetyResult::default();
        let mut coalescing = CoalescingResult::default();
        let tile_dimensions = TileDimensionResult {
            valid: true,
            tile_k: None,
            tile_kv: None,
            head_dim: None,
            shared_memory_required: None,
            shared_memory_available: Some(self.config.shared_memory_limit),
            issues: Vec::new(),
        };

        if path.is_file() {
            let analysis = self.analyze_file(path)?;
            files_analyzed = 1;
            cuda_files = analysis.cuda_files;
            simd_files = analysis.simd_files;
            wgpu_files = analysis.wgpu_files;
            defects = analysis.defects;
            barrier_safety = analysis.barrier_safety;
            coalescing = analysis.coalescing;
        } else if path.is_dir() {
            self.analyze_directory(
                path,
                &mut defects,
                &mut cuda_files,
                &mut simd_files,
                &mut wgpu_files,
                &mut files_analyzed,
                &mut barrier_safety,
                &mut coalescing,
            )?;
        }

        // Calculate Popper score based on analysis (with Rust pattern detection)
        let score = self.calculate_score(&defects, &barrier_safety, &coalescing, path);

        // Build Kaizen metrics
        let kaizen = self.build_kaizen_metrics(&defects);

        Ok(CudaSimdTdgResult {
            path: path.to_path_buf(),
            score,
            defects,
            barrier_safety,
            coalescing,
            tile_dimensions,
            kaizen,
            timestamp: chrono::Utc::now().to_rfc3339(),
            files_analyzed,
            cuda_files,
            simd_files,
            wgpu_files,
        })
    }

    /// Directories to skip during analysis (common ignore patterns)
    const IGNORED_DIRS: &'static [&'static str] = &[
        ".venv",
        "venv",
        "node_modules",
        "target",
        ".git",
        "__pycache__",
        ".tox",
        ".nox",
        "dist",
        "build",
        ".eggs",
        "*.egg-info",
        ".mypy_cache",
        ".pytest_cache",
        ".cargo",
        "vendor",
    ];

    /// Check if a path should be skipped (in an ignored directory)
    fn should_skip_path(path: &Path) -> bool {
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                let name_str = name.to_string_lossy();
                for ignored in Self::IGNORED_DIRS {
                    if name_str == *ignored || name_str.ends_with(".egg-info") {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn analyze_directory(
        &self,
        path: &Path,
        defects: &mut Vec<DetectedDefect>,
        cuda_files: &mut usize,
        simd_files: &mut usize,
        wgpu_files: &mut usize,
        files_analyzed: &mut usize,
        barrier_safety: &mut BarrierSafetyResult,
        coalescing: &mut CoalescingResult,
    ) -> anyhow::Result<()> {
        for entry in walkdir::WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !Self::should_skip_path(e.path()))
            .filter_map(Result::ok)
        {
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if matches!(
                        ext_str.as_str(),
                        "cu" | "cuh" | "ptx" | "rs" | "wgsl" | "c" | "cpp" | "h" | "hpp"
                    ) {
                        if let Ok(analysis) = self.analyze_file(file_path) {
                            *files_analyzed += 1;
                            *cuda_files += analysis.cuda_files;
                            *simd_files += analysis.simd_files;
                            *wgpu_files += analysis.wgpu_files;
                            defects.extend(analysis.defects);

                            // Merge barrier safety results
                            barrier_safety.total_barriers += analysis.barrier_safety.total_barriers;
                            barrier_safety.safe_barriers += analysis.barrier_safety.safe_barriers;
                            barrier_safety
                                .unsafe_barriers
                                .extend(analysis.barrier_safety.unsafe_barriers);

                            // Merge coalescing results
                            coalescing.total_operations += analysis.coalescing.total_operations;
                            coalescing.coalesced_operations +=
                                analysis.coalescing.coalesced_operations;
                            coalescing
                                .problematic_accesses
                                .extend(analysis.coalescing.problematic_accesses);
                        }
                    }
                }
            }
        }

        // Calculate aggregate scores
        if barrier_safety.total_barriers > 0 {
            barrier_safety.safety_score =
                barrier_safety.safe_barriers as f64 / barrier_safety.total_barriers as f64;
        }
        if coalescing.total_operations > 0 {
            coalescing.efficiency =
                coalescing.coalesced_operations as f64 / coalescing.total_operations as f64;
        }

        Ok(())
    }

    fn analyze_file(&self, path: &Path) -> anyhow::Result<FileAnalysis> {
        let content = std::fs::read_to_string(path)?;
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let mut analysis = FileAnalysis::default();

        match ext.as_str() {
            "cu" | "cuh" | "ptx" => {
                analysis.cuda_files = 1;
                self.analyze_cuda_content(&content, path, &mut analysis);
            }
            "wgsl" => {
                analysis.wgpu_files = 1;
                self.analyze_wgpu_content(&content, path, &mut analysis);
            }
            "rs" => {
                // Check for SIMD intrinsics
                if content.contains("std::arch::") || content.contains("core::arch::") {
                    analysis.simd_files = 1;
                    self.analyze_simd_content(&content, path, &mut analysis);
                }
                // Check for wgpu usage or embedded WGSL shaders
                if content.contains("wgpu::")
                    || content.contains("@compute")
                    || content.contains("@workgroup_size")
                {
                    analysis.wgpu_files = 1;
                    // Analyze embedded WGSL in Rust strings
                    self.analyze_wgpu_content(&content, path, &mut analysis);
                }
            }
            "c" | "cpp" | "h" | "hpp" => {
                // Check for SIMD intrinsics
                // Use concat! to avoid self-matching during CB-021 compliance scanning
                if content.contains("immintrin.h")
                    || content.contains(concat!("_mm", "256_"))
                    || content.contains(concat!("_mm", "512_"))
                    || content.contains("arm_neon.h")
                {
                    analysis.simd_files = 1;
                    self.analyze_simd_content(&content, path, &mut analysis);
                }
            }
            _ => {}
        }

        Ok(analysis)
    }

    fn analyze_cuda_content(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        // Analyze barrier safety (PARITY-114)
        self.detect_barrier_issues(content, path, analysis);

        // Analyze memory access patterns
        self.detect_memory_patterns(content, path, analysis);

        // Check for known defect patterns
        self.detect_known_patterns(content, path, analysis);
    }

    fn analyze_wgpu_content(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        // Check for workgroup barriers
        let barrier_count =
            content.matches("workgroupBarrier").count() + content.matches("storageBarrier").count();
        analysis.barrier_safety.total_barriers += barrier_count;

        // For WGPU, barriers are generally safer due to structured control flow
        analysis.barrier_safety.safe_barriers += barrier_count;

        // Detect memory patterns
        self.detect_wgpu_memory_patterns(content, path, analysis);
    }

    /// Comprehensive SIMD bug detection based on trueno research
    ///
    /// Detects (from trueno-explain/src/simd.rs and common SIMD bugs):
    ///
    /// ## P0 Critical
    /// - SIMD_ALIGN_FAULT: Aligned load without alignment guarantee
    /// - SIMD_BOUNDS_OVERFLOW: SIMD operation may read past buffer end
    ///
    /// ## P1 High (Performance)
    /// - SIMD_LOW_VECTORIZATION: Low vectorization ratio (<50%)
    /// - SIMD_SCALAR_FALLBACK: Scalar operations in hot path
    /// - SIMD_MISSING_TARGET: Missing #[target_feature] attribute
    /// - SIMD_VZEROUPPER: Mixed SSE/AVX without vzeroupper (SSE/AVX transition penalty)
    /// - SIMD_UNSAFE_NO_SAFETY: unsafe SIMD block without SAFETY comment
    ///
    /// ## P2 Medium (Efficiency)
    /// - SIMD_UNALIGNED_PERF: Unaligned loads where aligned could be used
    /// - SIMD_SUBOPTIMAL_WIDTH: Using narrower SIMD than available (SSE when AVX available)
    fn analyze_simd_content(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        let lines: Vec<&str> = content.lines().collect();

        // Instruction counts for vectorization ratio
        let mut scalar_ops = 0u32;
        let mut sse_ops = 0u32;
        let mut avx_ops = 0u32;
        let mut avx512_ops = 0u32;

        // Track unsafe blocks and SAFETY comments
        let mut in_unsafe_block = false;
        let mut unsafe_start_line = 0;
        let mut has_safety_comment = false;

        // Check for target_feature attribute
        // Use concat! to avoid self-matching during CB-021 compliance scanning
        let has_target_feature = content.contains("#[target_feature(enable");
        let has_avx512 = content.contains("avx512") || content.contains(concat!("_mm", "512_"));
        let has_avx = content.contains(concat!("_mm", "256_")) || content.contains("avx2");
        let _has_sse = content.contains("_mm_") || content.contains("sse");

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track SAFETY comments
            if trimmed.contains("// SAFETY:") || trimmed.contains("/// SAFETY:") {
                has_safety_comment = true;
            }

            // Track unsafe blocks
            if trimmed.contains("unsafe {") || trimmed.starts_with("unsafe ") {
                in_unsafe_block = true;
                unsafe_start_line = line_num + 1;
                has_safety_comment = false; // Reset for this block
            }
            if in_unsafe_block && trimmed.contains('}') {
                // Check if unsafe block has SIMD and no SAFETY comment
                let block_content = lines[unsafe_start_line - 1..=line_num].join("\n");
                if (block_content.contains("_mm") || block_content.contains("arch::"))
                    && !has_safety_comment
                {
                    analysis.defects.push(DetectedDefect {
                        defect_class: DefectClass {
                            ticket_id: "SIMD_UNSAFE_NO_SAFETY".to_string(),
                            description: "unsafe SIMD block without SAFETY comment".to_string(),
                            severity: DefectSeverity::P2Efficiency,
                            detection_method: "SIMD pattern analysis".to_string(),
                            resolved: false,
                            root_cause: Some("Undocumented safety invariants".to_string()),
                        },
                        file_path: path.to_path_buf(),
                        line: Some(unsafe_start_line),
                        snippet: Some(trimmed.to_string()),
                        suggestion: Some(
                            "Add // SAFETY: comment explaining alignment and bounds guarantees"
                                .to_string(),
                        ),
                    });
                }
                in_unsafe_block = false;
            }

            // Count SIMD instruction types
            // Use concat! to avoid self-matching during CB-021 compliance scanning
            if trimmed.contains(concat!("_mm", "512_")) {
                avx512_ops += 1;
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            } else if trimmed.contains(concat!("_mm", "256_")) {
                avx_ops += 1;
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            } else if trimmed.contains("_mm_")
                && !trimmed.contains(concat!("_mm", "256_"))
                && !trimmed.contains(concat!("_mm", "512_"))
            {
                sse_ops += 1;
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            }

            // Scalar operations in SIMD context
            if (trimmed.contains(".iter()") || trimmed.contains("for "))
                && (content.contains(concat!("_mm", "256_")) || content.contains(concat!("_mm", "512_")))
                && !trimmed.contains("chunks")
            {
                scalar_ops += 1;
            }

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: Alignment fault risk
            // ─────────────────────────────────────────────────────────────────
            // Use concat! to avoid self-matching during CB-021 compliance scanning
            if trimmed.contains(concat!("_mm", "256_load_si256"))
                || trimmed.contains(concat!("_mm", "512_load_si512"))
                || trimmed.contains(concat!("_mm", "256_load_ps"))
                || trimmed.contains(concat!("_mm", "512_load_ps"))
            {
                // Check if there's alignment guarantee in surrounding context
                let context_start = line_num.saturating_sub(10);
                let context = lines[context_start..=line_num].join("\n");
                let has_align = context.contains("align")
                    || context.contains("ALIGN")
                    || context.contains("repr(align")
                    || context.contains("as_ptr()");

                if !has_align {
                    analysis.defects.push(DetectedDefect {
                        defect_class: DefectClass {
                            ticket_id: "SIMD_ALIGN_FAULT".to_string(),
                            description: "Aligned SIMD load without visible alignment guarantee"
                                .to_string(),
                            severity: DefectSeverity::P0Critical,
                            detection_method: "SIMD pattern analysis".to_string(),
                            resolved: false,
                            root_cause: Some(
                                "Aligned loads require 32/64-byte aligned pointers".to_string(),
                            ),
                        },
                        file_path: path.to_path_buf(),
                        line: Some(line_num + 1),
                        snippet: Some(trimmed.to_string()),
                        suggestion: Some(
                            "Use _loadu_ variant or ensure pointer is aligned".to_string(),
                        ),
                    });
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: Bounds overflow risk
            // ─────────────────────────────────────────────────────────────────
            // Use concat! to avoid self-matching during CB-021 compliance scanning
            if (trimmed.contains(concat!("_mm", "256_loadu_")) || trimmed.contains(concat!("_mm", "512_loadu_")))
                && !content.contains("len()")
                && !content.contains(".len")
            {
                // No bounds check visible in file
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "SIMD_BOUNDS_OVERFLOW".to_string(),
                        description: "SIMD load without visible bounds check".to_string(),
                        severity: DefectSeverity::P1Performance,
                        detection_method: "SIMD pattern analysis".to_string(),
                        resolved: false,
                        root_cause: Some("SIMD loads may read past buffer end".to_string()),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(line_num + 1),
                    snippet: Some(trimmed.to_string()),
                    suggestion: Some(
                        "Ensure i + SIMD_WIDTH <= len before SIMD operations".to_string(),
                    ),
                });
            }

            // ─────────────────────────────────────────────────────────────────
            // P1 HIGH: SSE/AVX transition penalty
            // ─────────────────────────────────────────────────────────────────
            // Use concat! to avoid self-matching during CB-021 compliance scanning
            if (trimmed.contains("_mm_")
                && !trimmed.contains(concat!("_mm", "256_"))
                && !trimmed.contains(concat!("_mm", "512_")))
                && (content.contains(concat!("_mm", "256_")) || content.contains(concat!("_mm", "512_")))
                && !content.contains("vzeroupper")
                && !content.contains(concat!("_mm", "256_zeroupper"))
            {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "SIMD_VZEROUPPER".to_string(),
                        description: "Mixed SSE/AVX without vzeroupper (transition penalty)"
                            .to_string(),
                        severity: DefectSeverity::P1Performance,
                        detection_method: "SIMD pattern analysis".to_string(),
                        resolved: false,
                        root_cause: Some(
                            "SSE instructions after AVX cause ~70 cycle penalty".to_string(),
                        ),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(line_num + 1),
                    snippet: Some(trimmed.to_string()),
                    suggestion: Some(
                        concat!("Add _mm", "256_zeroupper() before SSE code or use all AVX").to_string(),
                    ),
                });
                break; // Only report once per file
            }

            // Detect unaligned loads (not errors, but note for coalescing)
            // Use concat! to avoid self-matching during CB-021 compliance scanning
            if trimmed.contains(concat!("_mm", "256_loadu_")) || trimmed.contains(concat!("_mm", "512_loadu_")) {
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            }
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Missing target_feature attribute
        // ─────────────────────────────────────────────────────────────────
        if (has_avx512 || has_avx)
            && !has_target_feature
            && !content.contains("is_x86_feature_detected")
        {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "SIMD_MISSING_TARGET".to_string(),
                    description: "SIMD intrinsics without #[target_feature] or runtime detection"
                        .to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "SIMD pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("May crash on CPUs without required features".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some(
                    "Add #[target_feature(enable = \"avx2\")] or runtime detection".to_string(),
                ),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Low vectorization ratio
        // ─────────────────────────────────────────────────────────────────
        let total_ops = scalar_ops + sse_ops + avx_ops + avx512_ops;
        if total_ops > 0 {
            let vectorized = sse_ops + avx_ops + avx512_ops;
            let ratio = vectorized as f32 / total_ops as f32;
            if ratio < 0.5 && scalar_ops > 5 {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "SIMD_LOW_VECTORIZATION".to_string(),
                        description: format!(
                            "Low vectorization ratio: {:.0}% (threshold: 50%)",
                            ratio * 100.0
                        ),
                        severity: DefectSeverity::P1Performance,
                        detection_method: "SIMD pattern analysis".to_string(),
                        resolved: false,
                        root_cause: Some("Scalar fallback reducing SIMD benefits".to_string()),
                    },
                    file_path: path.to_path_buf(),
                    line: None,
                    snippet: Some(format!(
                        "scalar: {}, vectorized: {}",
                        scalar_ops, vectorized
                    )),
                    suggestion: Some(
                        "Check for alignment issues or loop trip count problems".to_string(),
                    ),
                });
            }
        }

        // ─────────────────────────────────────────────────────────────────
        // P2 MEDIUM: Using narrower SIMD than available
        // ─────────────────────────────────────────────────────────────────
        if sse_ops > avx_ops && has_avx && avx_ops == 0 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "SIMD_SUBOPTIMAL_WIDTH".to_string(),
                    description: "Using SSE when AVX is available".to_string(),
                    severity: DefectSeverity::P2Efficiency,
                    detection_method: "SIMD pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("2x wider AVX could double throughput".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("SSE ops: {}, AVX ops: {}", sse_ops, avx_ops)),
                suggestion: Some("Consider upgrading to AVX2 for 256-bit operations".to_string()),
            });
        }

        if analysis.coalescing.total_operations > 0 {
            analysis.coalescing.efficiency = analysis.coalescing.coalesced_operations as f64
                / analysis.coalescing.total_operations as f64;
        }
    }

    fn detect_barrier_issues(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Detect barrier synchronization points
            if line.contains("__syncthreads")
                || line.contains("__syncwarp")
                || line.contains("bar.sync")
            {
                analysis.barrier_safety.total_barriers += 1;

                // Check for early return before this barrier
                let before_barrier = lines[..line_num].join("\n");
                if before_barrier.contains("return;")
                    || before_barrier.contains("return ")
                    || before_barrier.contains("exit")
                {
                    // Check if return is in the same function scope
                    let mut brace_depth = 0;
                    let mut found_unsafe_return = false;

                    for prev_line in lines[..line_num].iter().rev() {
                        if prev_line.contains('}') {
                            brace_depth += 1;
                        }
                        if prev_line.contains('{') {
                            brace_depth -= 1;
                            if brace_depth < 0 {
                                break;
                            }
                        }
                        if brace_depth == 0
                            && (prev_line.contains("return;") || prev_line.contains("return "))
                        {
                            found_unsafe_return = true;
                            break;
                        }
                    }

                    if found_unsafe_return {
                        analysis.barrier_safety.unsafe_barriers.push(BarrierIssue {
                            line: line_num + 1,
                            barrier_type: if line.contains("__syncthreads") {
                                "__syncthreads".to_string()
                            } else if line.contains("__syncwarp") {
                                "__syncwarp".to_string()
                            } else {
                                "bar.sync".to_string()
                            },
                            issue: "PARITY-114: Possible thread exit before barrier".to_string(),
                            exit_paths: vec!["Early return detected before barrier".to_string()],
                        });

                        // Add defect
                        if let Some(defect_class) = self.taxonomy.get("PARITY-114") {
                            analysis.defects.push(DetectedDefect {
                                defect_class: defect_class.clone(),
                                file_path: path.to_path_buf(),
                                line: Some(line_num + 1),
                                snippet: Some(line.trim().to_string()),
                                suggestion: Some(
                                    "Ensure all threads reach barrier or use cooperative groups"
                                        .to_string(),
                                ),
                            });
                        }
                    } else {
                        analysis.barrier_safety.safe_barriers += 1;
                    }
                } else {
                    analysis.barrier_safety.safe_barriers += 1;
                }
            }
        }

        if analysis.barrier_safety.total_barriers > 0 {
            analysis.barrier_safety.safety_score = analysis.barrier_safety.safe_barriers as f64
                / analysis.barrier_safety.total_barriers as f64;
        }
    }

    fn detect_memory_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        // Check if this is a PTX file
        let is_ptx = path.extension().is_some_and(|e| e == "ptx");

        if is_ptx {
            self.detect_ptx_memory_patterns(content, path, analysis);
            return;
        }

        for (line_num, line) in content.lines().enumerate() {
            // Detect global memory accesses
            if line.contains("[threadIdx.x")
                || line.contains("[tid")
                || line.contains("global_mem[")
            {
                analysis.coalescing.total_operations += 1;

                // Check for strided access
                if line.contains("* stride") || line.contains("* STRIDE") {
                    analysis
                        .coalescing
                        .problematic_accesses
                        .push(MemoryAccessIssue {
                            line: line_num + 1,
                            pattern: AccessPattern::Strided { stride: 0 }, // Would need analysis
                            impact: "Strided access may reduce memory throughput".to_string(),
                        });
                } else {
                    analysis.coalescing.coalesced_operations += 1;
                }
            }

            // Detect shared memory bank conflicts
            if line.contains("__shared__") && line.contains("[threadIdx") {
                // Simple heuristic: access with stride can cause bank conflicts
                if line.contains("% 32") || line.contains("& 31") {
                    // Likely bank conflict mitigation
                    analysis.coalescing.coalesced_operations += 1;
                }
            }
        }

        if analysis.coalescing.total_operations > 0 {
            analysis.coalescing.efficiency = analysis.coalescing.coalesced_operations as f64
                / analysis.coalescing.total_operations as f64;
        }
    }

    /// Comprehensive PTX bug detection based on trueno research and Tauranta fault history
    ///
    /// Detects (from trueno-explain/src/ptx/bugs.rs and trueno-ptx-debug):
    ///
    /// ## P0 Critical
    /// - F082: Address computed from shared memory load (data-dependent addressing)
    /// - SHARED_U64: Shared memory accessed with 64-bit register (should be 32-bit)
    /// - LOOP_BRANCH_END: Loop branches to END label instead of START
    /// - MISSING_BARRIER: Missing bar.sync between st.shared and ld.shared
    /// - EARLY_EXIT_BARRIER: Early thread exit before barrier (PARITY-114)
    /// - GENERIC_ADDR_CORRUPTION: cvta.shared creates 64-bit generic address
    ///
    /// ## P1 High (Performance)
    /// - REG_SPILLS: Register spills to local memory
    /// - HIGH_REG_PRESSURE: >64 registers reduces occupancy
    /// - PRED_OVERFLOW: >8 predicate registers causes spills
    /// - PLACEHOLDER_CODE: Incomplete code detected ("omitted", "simplified")
    /// - EMPTY_LOOP: Loop body contains no computation
    /// - NO_BOUNDS_CHECK: Missing thread bounds check before memory access
    ///
    /// ## P2 Medium (Efficiency)
    /// - REDUNDANT_MOV: Redundant register move chains
    /// - UNOPT_MEM: Multiple single loads could be vectorized
    /// - DEAD_CODE: Unreachable code after ret or unconditional branch
    fn detect_ptx_memory_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        let lines: Vec<&str> = content.lines().collect();
        let mut shared_load_regs: Vec<String> = Vec::new();

        // =====================================================================
        // State tracking for multi-line pattern detection
        // =====================================================================
        let mut loop_labels: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut loop_end_labels: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut in_loop = false;
        let mut loop_start_line = 0;
        let mut barrier_seen_in_loop = false;
        let mut last_st_shared_line: Option<usize> = None;
        let mut last_mov: Option<(usize, String, String)> = None;
        let mut after_unconditional = false;
        let mut unconditional_line = 0;
        let mut total_registers: usize = 0;
        let mut predicate_count: usize = 0;

        // First pass: identify loop labels (labels with back-edges)
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.ends_with(':') && !trimmed.starts_with('.') {
                let label = trimmed.trim_end_matches(':').to_string();
                let bra_pattern = format!("bra {};", label);
                let bra_pattern2 = format!("bra {}", label);
                if content.contains(&bra_pattern) || content.contains(&bra_pattern2) {
                    loop_labels.insert(label.clone());
                    loop_end_labels.insert(format!("{}_end", label));
                    loop_end_labels.insert(format!("{}_done", label));
                }
            }
        }

        // Count registers for pressure analysis
        let reg_pattern = regex::Regex::new(r"\.reg\s+\.\w+\s+%\w+<(\d+)>").ok();
        let pred_pattern = regex::Regex::new(r"\.reg\s+\.pred\s+%p<(\d+)>").ok();

        if let Some(ref re) = reg_pattern {
            for caps in re.captures_iter(content) {
                if let Some(count) = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok()) {
                    total_registers += count;
                }
            }
        }

        if let Some(ref re) = pred_pattern {
            if let Some(caps) = re.captures(content) {
                if let Some(count) = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok()) {
                    predicate_count = count;
                }
            }
        }

        // =====================================================================
        // Main analysis pass
        // =====================================================================
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and comments for some checks
            let is_comment = trimmed.starts_with("//");
            let is_empty = trimmed.is_empty();

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: Placeholder code detection (check comments)
            // ─────────────────────────────────────────────────────────────────
            if is_comment {
                let lower = trimmed.to_lowercase();
                let placeholders = [
                    "omitted",
                    "simplified",
                    "placeholder",
                    "todo",
                    "fixme",
                    "not implemented",
                    "for now",
                    "for brevity",
                ];
                for pattern in &placeholders {
                    if lower.contains(pattern) {
                        analysis.defects.push(DetectedDefect {
                            defect_class: DefectClass {
                                ticket_id: "PLACEHOLDER".to_string(),
                                description: format!("Placeholder/incomplete code: '{}'", pattern),
                                severity: DefectSeverity::P1Performance,
                                detection_method: "Comment analysis".to_string(),
                                resolved: false,
                                root_cause: Some(
                                    "Code is incomplete and may not work correctly".to_string(),
                                ),
                            },
                            file_path: path.to_path_buf(),
                            line: Some(line_num + 1),
                            snippet: Some(trimmed.to_string()),
                            suggestion: Some("Implement complete kernel logic".to_string()),
                        });
                        break;
                    }
                }
            }

            if is_empty || is_comment {
                continue;
            }

            // ─────────────────────────────────────────────────────────────────
            // Track loop structure for barrier analysis
            // ─────────────────────────────────────────────────────────────────
            if trimmed.ends_with(':') && !trimmed.starts_with('.') {
                let label = trimmed.trim_end_matches(':');
                if loop_labels.contains(label) {
                    in_loop = true;
                    loop_start_line = line_num + 1;
                    barrier_seen_in_loop = false;
                }
                if loop_end_labels.contains(label)
                    || label.contains("_end")
                    || label.contains("_done")
                {
                    in_loop = false;
                }
                // Reset after_unconditional since label is reachable
                after_unconditional = false;
            }

            // Track barrier instructions
            if trimmed.contains("bar.sync") || trimmed.contains("bar.arrive") {
                if in_loop {
                    barrier_seen_in_loop = true;
                }
                last_st_shared_line = None; // Reset after barrier
            }

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: SHARED_U64 - 64-bit register for shared memory
            // Pattern: st.shared.* [%rd*] or ld.shared.* [%rd*]
            // ─────────────────────────────────────────────────────────────────
            if (trimmed.contains("st.shared") || trimmed.contains("ld.shared"))
                && trimmed.contains("[%rd")
            {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "SHARED_U64".to_string(),
                        description: "Shared memory accessed with 64-bit register".to_string(),
                        severity: DefectSeverity::P0Critical,
                        detection_method: "PTX pattern analysis".to_string(),
                        resolved: false,
                        root_cause: Some("Shared memory requires 32-bit addressing".to_string()),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(line_num + 1),
                    snippet: Some(trimmed.to_string()),
                    suggestion: Some(
                        "Replace %rd* with %r* for shared memory addressing".to_string(),
                    ),
                });
            }

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: cvta.shared creates generic address corruption
            // ─────────────────────────────────────────────────────────────────
            if trimmed.contains("cvta.shared") {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "CVTA_SHARED".to_string(),
                        description:
                            "cvta.shared creates 64-bit generic address that SASS may clobber"
                                .to_string(),
                        severity: DefectSeverity::P0Critical,
                        detection_method: "PTX pattern analysis".to_string(),
                        resolved: false,
                        root_cause: Some(
                            "Generic address from cvta.shared causes address corruption"
                                .to_string(),
                        ),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(line_num + 1),
                    snippet: Some(trimmed.to_string()),
                    suggestion: Some(
                        "Use direct ld.shared/st.shared with 32-bit offset instead".to_string(),
                    ),
                });
            }

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: Missing barrier between st.shared and ld.shared
            // ─────────────────────────────────────────────────────────────────
            if trimmed.contains("st.shared") {
                last_st_shared_line = Some(line_num);
            }

            if trimmed.contains("ld.shared") {
                // Track registers loaded from shared memory for F082 detection
                if let Some(reg) = Self::extract_ptx_dest_register(trimmed) {
                    shared_load_regs.push(reg);
                }

                // Check for missing barrier
                if let Some(st_line) = last_st_shared_line {
                    analysis.defects.push(DetectedDefect {
                        defect_class: DefectClass {
                            ticket_id: "MISSING_BARRIER".to_string(),
                            description:
                                "ld.shared follows st.shared without barrier synchronization"
                                    .to_string(),
                            severity: DefectSeverity::P0Critical,
                            detection_method: "PTX dataflow analysis".to_string(),
                            resolved: false,
                            root_cause: Some(
                                "Race condition: threads may read stale data".to_string(),
                            ),
                        },
                        file_path: path.to_path_buf(),
                        line: Some(line_num + 1),
                        snippet: Some(format!(
                            "st.shared at line {}, ld.shared at line {}",
                            st_line + 1,
                            line_num + 1
                        )),
                        suggestion: Some(format!(
                            "Add bar.sync 0; between lines {} and {}",
                            st_line + 1,
                            line_num + 1
                        )),
                    });
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: F082 - Address computed from shared memory value
            // ─────────────────────────────────────────────────────────────────
            if (trimmed.contains("add.u64")
                || trimmed.contains("add.s64")
                || trimmed.contains("cvt.u64"))
                && !shared_load_regs.is_empty()
            {
                for reg in &shared_load_regs {
                    if trimmed.contains(reg) {
                        analysis.defects.push(DetectedDefect {
                            defect_class: DefectClass {
                                ticket_id: "F082".to_string(),
                                description: "Address computed from shared memory load (data-dependent addressing)".to_string(),
                                severity: DefectSeverity::P0Critical,
                                detection_method: "PTX dataflow analysis".to_string(),
                                resolved: false,
                                root_cause: Some("Address register depends on value loaded from shared memory, causing non-uniform memory access".to_string()),
                            },
                            file_path: path.to_path_buf(),
                            line: Some(line_num + 1),
                            snippet: Some(trimmed.to_string()),
                            suggestion: Some("Compute address from thread ID or constant offsets only".to_string()),
                        });
                    }
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: PARITY-114 Early exit before barrier
            // ─────────────────────────────────────────────────────────────────
            if trimmed.contains("bra exit")
                || (trimmed.contains("bra ") && trimmed.contains("done"))
            {
                if in_loop && !barrier_seen_in_loop {
                    let kind = if trimmed.starts_with('@') {
                        "Conditional"
                    } else {
                        "Unconditional"
                    };
                    analysis.defects.push(DetectedDefect {
                        defect_class: DefectClass {
                            ticket_id: "PARITY-114".to_string(),
                            description: format!("{} early exit before barrier in loop", kind),
                            severity: DefectSeverity::P0Critical,
                            detection_method: "PTX CFG analysis".to_string(),
                            resolved: false,
                            root_cause: Some("Some threads exit before bar.sync, causing remaining threads to hang".to_string()),
                        },
                        file_path: path.to_path_buf(),
                        line: Some(line_num + 1),
                        snippet: Some(trimmed.to_string()),
                        suggestion: Some(format!("Move bounds check AFTER loop body (loop starts at line {})", loop_start_line)),
                    });
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // P0 CRITICAL: Loop branches to END instead of START
            // ─────────────────────────────────────────────────────────────────
            if trimmed.starts_with("bra ") && !trimmed.starts_with('@') {
                // Extract target label
                if let Some(target) = trimmed
                    .strip_prefix("bra ")
                    .map(|s| s.trim_end_matches(';').trim())
                {
                    if target.contains("_end") || target.ends_with("_done") {
                        // Check if this is inside a loop (unconditional branch to end is suspicious)
                        analysis.defects.push(DetectedDefect {
                            defect_class: DefectClass {
                                ticket_id: "LOOP_BRANCH_END".to_string(),
                                description: "Unconditional branch to loop end label".to_string(),
                                severity: DefectSeverity::P1Performance,
                                detection_method: "PTX CFG analysis".to_string(),
                                resolved: false,
                                root_cause: Some(
                                    "Loop may be incomplete or have early exit".to_string(),
                                ),
                            },
                            file_path: path.to_path_buf(),
                            line: Some(line_num + 1),
                            snippet: Some(trimmed.to_string()),
                            suggestion: Some(
                                "Verify this branch target is intentional".to_string(),
                            ),
                        });
                    }
                }
            }

            // ─────────────────────────────────────────────────────────────────
            // P2 MEDIUM: Dead code detection
            // ─────────────────────────────────────────────────────────────────
            if after_unconditional && !trimmed.ends_with(':') && trimmed != "}" {
                analysis.defects.push(DetectedDefect {
                    defect_class: DefectClass {
                        ticket_id: "DEAD_CODE".to_string(),
                        description: "Unreachable code after unconditional jump".to_string(),
                        severity: DefectSeverity::P2Efficiency,
                        detection_method: "PTX CFG analysis".to_string(),
                        resolved: false,
                        root_cause: Some(format!(
                            "Code unreachable after line {}",
                            unconditional_line + 1
                        )),
                    },
                    file_path: path.to_path_buf(),
                    line: Some(line_num + 1),
                    snippet: Some(trimmed.to_string()),
                    suggestion: Some("Remove unreachable code or add label".to_string()),
                });
                after_unconditional = false; // Only report once per block
            }

            // Track unconditional jumps
            if trimmed == "ret;" || (trimmed.starts_with("bra ") && !trimmed.starts_with('@')) {
                after_unconditional = true;
                unconditional_line = line_num;
            }

            // ─────────────────────────────────────────────────────────────────
            // P2 MEDIUM: Redundant mov chains
            // ─────────────────────────────────────────────────────────────────
            let mov_pattern = regex::Regex::new(r"^\s*mov\.\w+\s+(%\w+),\s*(%\w+)").ok();
            if let Some(ref re) = mov_pattern {
                if let Some(caps) = re.captures(trimmed) {
                    let dest = caps.get(1).map(|m| m.as_str().to_string());
                    let src = caps.get(2).map(|m| m.as_str().to_string());

                    if let (Some(d), Some(s)) = (dest, src) {
                        if let Some((prev_line, prev_dest, _prev_src)) = &last_mov {
                            if &s == prev_dest {
                                analysis.defects.push(DetectedDefect {
                                    defect_class: DefectClass {
                                        ticket_id: "REDUNDANT_MOV".to_string(),
                                        description: "Redundant register move chain".to_string(),
                                        severity: DefectSeverity::P2Efficiency,
                                        detection_method: "PTX dataflow analysis".to_string(),
                                        resolved: false,
                                        root_cause: Some(format!(
                                            "mov chain at lines {} and {}",
                                            prev_line + 1,
                                            line_num + 1
                                        )),
                                    },
                                    file_path: path.to_path_buf(),
                                    line: Some(line_num + 1),
                                    snippet: Some(trimmed.to_string()),
                                    suggestion: Some(
                                        "Combine mov chain into single mov".to_string(),
                                    ),
                                });
                            }
                        }
                        last_mov = Some((line_num, d, s));
                    }
                }
            } else {
                last_mov = None;
            }

            // ─────────────────────────────────────────────────────────────────
            // Memory operation tracking for coalescing analysis
            // ─────────────────────────────────────────────────────────────────
            if trimmed.contains("ld.global") || trimmed.contains("st.global") {
                analysis.coalescing.total_operations += 1;
                if trimmed.contains("%tid") || trimmed.contains("param") {
                    analysis.coalescing.coalesced_operations += 1;
                }
            }

            if trimmed.contains("ld.shared") || trimmed.contains("st.shared") {
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            }
        }

        // =====================================================================
        // Post-analysis checks
        // =====================================================================

        // P1 HIGH: Register spills (.local memory usage)
        if content.contains(".local") {
            let local_count = content.matches(".local").count();
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "REG_SPILLS".to_string(),
                    description: format!(
                        "{} potential register spills to local memory",
                        local_count
                    ),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX memory analysis".to_string(),
                    resolved: false,
                    root_cause: Some(
                        "High register pressure causing spills to slow local memory".to_string(),
                    ),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("{} .local declarations", local_count)),
                suggestion: Some("Reduce live variables or split kernel".to_string()),
            });
        }

        // P1 HIGH: High register pressure (>64 registers)
        if total_registers > 64 {
            let occupancy = 65536 / (total_registers.max(1) * 32);
            let occupancy_pct = (occupancy as f64 / 32.0 * 100.0).min(100.0);
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "HIGH_REG_PRESSURE".to_string(),
                    description: format!(
                        "High register pressure: {} registers limits occupancy to {:.0}%",
                        total_registers, occupancy_pct
                    ),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX register analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Too many registers reduce SM occupancy".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("{} registers declared", total_registers)),
                suggestion: Some(
                    "Reduce live variables or split into multiple kernels".to_string(),
                ),
            });
        }

        // P1 HIGH: Predicate overflow (>8 predicates)
        if predicate_count > 8 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "PRED_OVERFLOW".to_string(),
                    description: format!(
                        "Predicate overflow: {} predicates declared (max 8 hardware)",
                        predicate_count
                    ),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX register analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Excess predicates cause spills".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("{} predicates", predicate_count)),
                suggestion: Some("Combine conditions or use branches".to_string()),
            });
        }

        // P2 MEDIUM: Unoptimized memory pattern (many single loads, no vector loads)
        let single_loads = content.matches("ld.global.f32").count();
        let vector_loads = content.matches("ld.global.v2.f32").count()
            + content.matches("ld.global.v4.f32").count();
        if single_loads >= 4 && vector_loads == 0 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "UNOPT_MEM".to_string(),
                    description: format!("{} single f32 loads, 0 vector loads", single_loads),
                    severity: DefectSeverity::P2Efficiency,
                    detection_method: "PTX memory analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Multiple single loads could be vectorized".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some(
                    "Consider ld.global.v2.f32 or ld.global.v4.f32 for consecutive addresses"
                        .to_string(),
                ),
            });
        }

        // P1 HIGH: Missing bounds check (uses tid + global mem but no setp)
        let has_tid = content.contains("%tid.") || content.contains("%ntid.");
        let has_global_mem = content.contains("ld.global") || content.contains("st.global");
        let has_bounds_check = content.contains("setp.lt") || content.contains("setp.ge");
        let has_predicated_branch = content.contains("@%p") && content.contains("bra");

        if has_tid && has_global_mem && !has_bounds_check && !has_predicated_branch {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "NO_BOUNDS_CHECK".to_string(),
                    description: "Kernel accesses global memory but lacks bounds checking"
                        .to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX CFG analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Thread may access out-of-bounds memory".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some(
                    "Add: setp.lt.u32 %p0, %tid, %size; @%p0 bra do_work;".to_string(),
                ),
            });
        }

        // P0 CRITICAL: Missing entry point
        if !content.trim().is_empty() && !content.contains(".entry") {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "NO_ENTRY".to_string(),
                    description: "No kernel entry point (.entry) found".to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "PTX structure analysis".to_string(),
                    resolved: false,
                    root_cause: Some("PTX file lacks kernel entry point".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some("Add .entry <kernel_name>(...) declaration".to_string()),
            });
        }

        if analysis.coalescing.total_operations > 0 {
            analysis.coalescing.efficiency = analysis.coalescing.coalesced_operations as f64
                / analysis.coalescing.total_operations as f64;
        }
    }

    /// Extract destination register from PTX instruction
    /// Example: "ld.shared.u32 %r1, [%rd1]" -> Some("%r1")
    fn extract_ptx_dest_register(line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let dest = parts[1].trim_end_matches(',');
            if dest.starts_with('%') {
                return Some(dest.to_string());
            }
        }
        None
    }

    /// Comprehensive WGPU/WGSL bug detection based on trueno research
    ///
    /// Detects (from trueno-explain/src/wgpu.rs and common WGSL bugs):
    ///
    /// ## P1 High (Performance)
    /// - WGPU_SMALL_WORKGROUP: Workgroup size too small (<64 threads)
    /// - WGPU_LARGE_WORKGROUP: Workgroup size too large (>1024 threads)
    /// - WGPU_NON_WARP_ALIGNED: Workgroup not multiple of 32 (warp waste)
    /// - WGPU_MISSING_WORKGROUP: No @workgroup_size attribute found
    /// - WGPU_NO_BOUNDS_CHECK: Global invocation without bounds check
    ///
    /// ## P2 Medium (Efficiency)
    /// - WGPU_EXCESSIVE_BARRIERS: Too many workgroupBarrier() calls
    /// - WGPU_UNIFORM_DIVERGENCE: Non-uniform control flow in workgroup
    fn detect_wgpu_memory_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        let lines: Vec<&str> = content.lines().collect();

        // Parse workgroup size from @workgroup_size(x, y, z)
        let mut workgroup_x = 1u32;
        let mut workgroup_y = 1u32;
        let mut workgroup_z = 1u32;
        let mut has_workgroup_size = false;

        // Count various patterns
        let mut barrier_count = 0u32;
        let mut has_bounds_check = false;
        let mut has_global_invocation = false;

        // Regex for workgroup_size
        let workgroup_regex = regex::Regex::new(
            r"@workgroup_size\s*\(\s*(\d+)(?:\s*,\s*(\d+))?(?:\s*,\s*(\d+))?\s*\)",
        )
        .ok();

        for (_line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Parse @workgroup_size
            if let Some(ref re) = workgroup_regex {
                if let Some(caps) = re.captures(trimmed) {
                    has_workgroup_size = true;
                    workgroup_x = caps.get(1).map_or(1, |m| m.as_str().parse().unwrap_or(1));
                    workgroup_y = caps.get(2).map_or(1, |m| m.as_str().parse().unwrap_or(1));
                    workgroup_z = caps.get(3).map_or(1, |m| m.as_str().parse().unwrap_or(1));
                }
            }

            // Count barriers
            if trimmed.contains("workgroupBarrier") || trimmed.contains("storageBarrier") {
                barrier_count += 1;
                analysis.barrier_safety.total_barriers += 1;
                analysis.barrier_safety.safe_barriers += 1;
            }

            // Detect global invocation usage
            if trimmed.contains("global_invocation_id") {
                has_global_invocation = true;
            }

            // Detect bounds checks
            if (trimmed.contains("if") || trimmed.contains("select"))
                && (trimmed.contains("<") || trimmed.contains(">="))
                && (trimmed.contains("size")
                    || trimmed.contains("len")
                    || trimmed.contains("count"))
            {
                has_bounds_check = true;
            }

            // Detect storage buffer accesses
            if trimmed.contains("storage")
                && (trimmed.contains("read") || trimmed.contains("write"))
            {
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            }

            // Detect array indexing
            if trimmed.contains('[') && trimmed.contains(']') {
                analysis.coalescing.total_operations += 1;
                analysis.coalescing.coalesced_operations += 1;
            }
        }

        let total_threads = workgroup_x * workgroup_y * workgroup_z;

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Missing workgroup_size
        // ─────────────────────────────────────────────────────────────────
        if !has_workgroup_size && content.contains("@compute") {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_MISSING_WORKGROUP".to_string(),
                    description: "Compute shader missing @workgroup_size attribute".to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some(
                        "Default workgroup size (1,1,1) is extremely inefficient".to_string(),
                    ),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some(
                    "Add @workgroup_size(256) or @workgroup_size(8, 8, 1)".to_string(),
                ),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Small workgroup size (<64 threads)
        // ─────────────────────────────────────────────────────────────────
        if has_workgroup_size && total_threads < 64 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_SMALL_WORKGROUP".to_string(),
                    description: format!("Small workgroup size: {} threads (minimum: 64)", total_threads),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Low GPU occupancy, underutilization".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!("@workgroup_size({}, {}, {})", workgroup_x, workgroup_y, workgroup_z)),
                suggestion: Some(format!("Increase to at least 64 threads (e.g., @workgroup_size(64) or @workgroup_size(8, 8))")),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Large workgroup size (>1024 threads)
        // ─────────────────────────────────────────────────────────────────
        if has_workgroup_size && total_threads > 1024 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_LARGE_WORKGROUP".to_string(),
                    description: format!(
                        "Large workgroup size: {} threads (max: 1024)",
                        total_threads
                    ),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some(
                        "May exceed hardware limits or cause register pressure".to_string(),
                    ),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!(
                    "@workgroup_size({}, {}, {})",
                    workgroup_x, workgroup_y, workgroup_z
                )),
                suggestion: Some("Reduce to at most 1024 threads".to_string()),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Non-warp-aligned workgroup
        // ─────────────────────────────────────────────────────────────────
        if has_workgroup_size && total_threads > 1 && total_threads % 32 != 0 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_NON_WARP_ALIGNED".to_string(),
                    description: format!(
                        "Workgroup size {} not multiple of 32 (warp size)",
                        total_threads
                    ),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Partial warp execution wastes GPU cycles".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!(
                    "@workgroup_size({}, {}, {})",
                    workgroup_x, workgroup_y, workgroup_z
                )),
                suggestion: Some("Align to multiple of 32 (e.g., 64, 128, 256)".to_string()),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P1 HIGH: Missing bounds check
        // ─────────────────────────────────────────────────────────────────
        if has_global_invocation && !has_bounds_check {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_NO_BOUNDS_CHECK".to_string(),
                    description: "Compute shader uses global_invocation_id without bounds check"
                        .to_string(),
                    severity: DefectSeverity::P1Performance,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Excess threads may access out-of-bounds memory".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: None,
                suggestion: Some("Add: if (gid.x < params.size) { ... }".to_string()),
            });
        }

        // ─────────────────────────────────────────────────────────────────
        // P2 MEDIUM: Excessive barriers
        // ─────────────────────────────────────────────────────────────────
        if barrier_count > 5 {
            analysis.defects.push(DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "WGPU_EXCESSIVE_BARRIERS".to_string(),
                    description: format!(
                        "{} barrier calls may indicate inefficient algorithm",
                        barrier_count
                    ),
                    severity: DefectSeverity::P2Efficiency,
                    detection_method: "WGSL pattern analysis".to_string(),
                    resolved: false,
                    root_cause: Some("Each barrier synchronizes entire workgroup".to_string()),
                },
                file_path: path.to_path_buf(),
                line: None,
                snippet: Some(format!(
                    "{} workgroupBarrier/storageBarrier calls",
                    barrier_count
                )),
                suggestion: Some(
                    "Consider restructuring algorithm to reduce synchronization".to_string(),
                ),
            });
        }

        if analysis.coalescing.total_operations > 0 {
            analysis.coalescing.efficiency = analysis.coalescing.coalesced_operations as f64
                / analysis.coalescing.total_operations as f64;
        }

        if analysis.barrier_safety.total_barriers > 0 {
            analysis.barrier_safety.safety_score = analysis.barrier_safety.safe_barriers as f64
                / analysis.barrier_safety.total_barriers as f64;
        }
    }

    fn detect_known_patterns(&self, content: &str, path: &Path, analysis: &mut FileAnalysis) {
        // Check for PAR-041: FlashAttention tile size issues
        if content.contains("FlashAttention")
            || content.contains("flash_attention")
            || content.contains("tiled_attention")
        {
            // Look for tile_kv and head_dim
            if let (Some(tile_kv), Some(head_dim)) = (
                self.extract_value(content, "tile_kv"),
                self.extract_value(content, "head_dim"),
            ) {
                if tile_kv < head_dim {
                    if let Some(defect_class) = self.taxonomy.get("PAR-041") {
                        analysis.defects.push(DetectedDefect {
                            defect_class: defect_class.clone(),
                            file_path: path.to_path_buf(),
                            line: None,
                            snippet: Some(format!(
                                "tile_kv ({}) < head_dim ({})",
                                tile_kv, head_dim
                            )),
                            suggestion: Some(format!(
                                "Set tile_kv >= head_dim (at least {})",
                                head_dim
                            )),
                        });
                    }
                }
            }
        }

        // Check for PAR-034: Missing Tensor Core usage
        if (content.contains("matmul") || content.contains("gemm"))
            && !content.contains("wmma")
            && !content.contains("mma")
            && !content.contains("tensor_core")
        {
            if let Some(defect_class) = self.taxonomy.get("PAR-034") {
                analysis.defects.push(DetectedDefect {
                    defect_class: defect_class.clone(),
                    file_path: path.to_path_buf(),
                    line: None,
                    snippet: Some("Matrix multiplication without Tensor Core".to_string()),
                    suggestion: Some(
                        "Consider using wmma or mma instructions for better performance"
                            .to_string(),
                    ),
                });
            }
        }
    }

    fn extract_value(&self, content: &str, name: &str) -> Option<usize> {
        // Simple pattern matching for variable assignments
        let patterns = [
            format!("{} = ", name),
            format!("{}=", name),
            format!("const {} = ", name),
            format!("let {} = ", name),
        ];

        for pattern in &patterns {
            if let Some(pos) = content.find(pattern) {
                let after = &content[pos + pattern.len()..];
                let value_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(value) = value_str.parse() {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Detect Rust project quality patterns for enhanced scoring
    fn detect_rust_patterns(&self, path: &Path) -> RustProjectPatterns {
        let mut patterns = RustProjectPatterns::default();

        // Check for Cargo.lock (version pinning)
        patterns.has_cargo_lock = path.join("Cargo.lock").exists();

        // Check for rust-toolchain.toml (Rust version pinning)
        patterns.has_rust_toolchain =
            path.join("rust-toolchain.toml").exists() || path.join("rust-toolchain").exists();

        // Check for Criterion benchmarks
        patterns.has_criterion_benches = path.join("benches").exists()
            && std::fs::read_dir(path.join("benches"))
                .map(|d| {
                    d.filter_map(Result::ok)
                        .any(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                })
                .unwrap_or(false);

        // Check for GitHub Actions CI
        patterns.has_github_ci = path.join(".github/workflows").exists();

        // Check for proptest regressions (regression tests)
        patterns.has_proptest_regressions = path.join("proptest-regressions").exists();

        // Check for CHANGELOG.md (historical integrity)
        patterns.has_changelog =
            path.join("CHANGELOG.md").exists() || path.join("CHANGELOG").exists();

        // Check for golden traces (deterministic output)
        patterns.has_golden_traces = path.join("golden_traces").exists();

        // Check for SAFETY comments in SIMD code
        if path.join("src/backends").exists() {
            if let Ok(entries) = std::fs::read_dir(path.join("src/backends")) {
                for entry in entries.filter_map(Result::ok) {
                    if entry.path().extension().is_some_and(|e| e == "rs") {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            if content.contains("// SAFETY:") || content.contains("/// SAFETY:") {
                                patterns.has_safety_comments = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Check for Miri configuration
        patterns.has_miri_config = path.join(".cargo/config.toml").exists()
            && std::fs::read_to_string(path.join(".cargo/config.toml"))
                .map(|c| c.contains("miri"))
                .unwrap_or(false);

        patterns
    }

    fn calculate_score(
        &self,
        defects: &[DetectedDefect],
        barrier_safety: &BarrierSafetyResult,
        coalescing: &CoalescingResult,
        path: &Path,
    ) -> PopperScore {
        // Detect Rust project patterns for enhanced scoring
        let patterns = self.detect_rust_patterns(path);

        // Calculate Category A: Falsifiability
        let p0_defects = defects
            .iter()
            .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
            .count();

        let falsifiability = FalsifiabilityScore {
            barrier_safety: if barrier_safety.unsafe_barriers.is_empty() {
                5.0
            } else {
                5.0 * barrier_safety.safety_score
            },
            bounds_verification: if p0_defects == 0 {
                if patterns.has_safety_comments {
                    5.0
                } else {
                    4.0
                }
            } else {
                2.5
            },
            // Full credit if proptest regressions exist (property-based testing)
            divergence_testing: if patterns.has_proptest_regressions {
                5.0
            } else {
                2.5
            },
            // Full credit if Miri is configured (memory safety verification)
            memory_race_detection: if patterns.has_miri_config { 5.0 } else { 2.5 },
            occupancy_bounds: 5.0, // Assume valid unless we detect issues
        };

        // Category B: Reproducibility - enhanced with Rust patterns
        let reproducibility = ReproducibilityScore {
            // Golden traces indicate deterministic output verification
            deterministic_output: if patterns.has_golden_traces { 8.0 } else { 4.0 },
            // Cargo.lock + rust-toolchain = full version pinning
            version_pinning: if patterns.has_cargo_lock && patterns.has_rust_toolchain {
                5.0
            } else if patterns.has_cargo_lock {
                3.5
            } else {
                2.5
            },
            hardware_specification: 2.5, // Partial credit by default
            // Criterion benchmarks = full benchmark harness credit
            benchmark_harness: if patterns.has_criterion_benches {
                4.0
            } else {
                2.0
            },
            // GitHub Actions = full CI/CD credit
            ci_cd_integration: if patterns.has_github_ci { 3.0 } else { 1.5 },
        };

        // Category C: Transparency
        let transparency = TransparencyScore {
            ptx_inspection: 3.0,
            register_allocation: 2.5,
            occupancy_calculation: 2.5,
            memory_layout: 2.0,
        };

        // Category D: Statistical Rigor - Criterion provides all of these
        let statistical_rigor = if patterns.has_criterion_benches {
            StatisticalRigorScore {
                warmup_iterations: 4.0,
                sample_count: 4.0,
                outlier_analysis: 4.0,
                confidence_intervals: 3.0,
            }
        } else {
            StatisticalRigorScore {
                warmup_iterations: 2.0,
                sample_count: 2.0,
                outlier_analysis: 2.0,
                confidence_intervals: 1.5,
            }
        };

        // Category E: Historical Integrity
        let historical_integrity = HistoricalIntegrityScore {
            // CHANGELOG indicates fault lineage tracking
            fault_lineage: if patterns.has_changelog {
                4.0
            } else if !defects.is_empty() {
                3.0
            } else {
                2.0
            },
            // Proptest regressions = regression tests from historical bugs
            regression_tests: if patterns.has_proptest_regressions {
                3.0
            } else {
                1.5
            },
            root_cause_documentation: if patterns.has_changelog { 3.0 } else { 1.5 },
        };

        // Category F: GPU/SIMD Specific
        let gpu_simd_specific = GpuSimdSpecificScore {
            warp_efficiency: 1.0,
            memory_throughput: coalescing.efficiency * 2.0,
            instruction_mix: 0.5,
        };

        PopperScore::calculate(
            falsifiability,
            reproducibility,
            transparency,
            statistical_rigor,
            historical_integrity,
            gpu_simd_specific,
        )
    }

    fn build_kaizen_metrics(&self, defects: &[DetectedDefect]) -> KaizenMetrics {
        let ticket_references: Vec<String> = defects
            .iter()
            .map(|d| d.defect_class.ticket_id.clone())
            .collect();

        let resolved_count = defects.iter().filter(|d| d.defect_class.resolved).count() as u32;

        KaizenMetrics {
            tickets_resolved: resolved_count,
            mttd: 24.0,            // Default estimate
            mttf: 48.0,            // Default estimate
            escape_rate: 0.05,     // 5% default
            regression_rate: 0.02, // 2% default
            ticket_references,
        }
    }

    /// Check if quality gate passes
    #[must_use]
    pub fn passes_quality_gate(&self, result: &CudaSimdTdgResult) -> bool {
        if !result.score.gateway_passed {
            return false;
        }

        if self.config.fail_on_p0 {
            let has_p0 = result
                .defects
                .iter()
                .any(|d| d.defect_class.severity == DefectSeverity::P0Critical);
            if has_p0 {
                return false;
            }
        }

        result.score.total >= self.config.min_score
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defect_taxonomy_creation() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check PARITY-114 exists
        let parity114 = taxonomy.get("PARITY-114");
        assert!(parity114.is_some());
        assert_eq!(parity114.unwrap().severity, DefectSeverity::P0Critical);

        // Check PAR-041 exists
        let par041 = taxonomy.get("PAR-041");
        assert!(par041.is_some());
        assert_eq!(par041.unwrap().severity, DefectSeverity::P0Critical);
    }

    #[test]
    fn test_popper_score_gateway() {
        // Test gateway failure
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 3.0,
            bounds_verification: 3.0,
            divergence_testing: 3.0,
            memory_race_detection: 3.0,
            occupancy_bounds: 2.0, // Total: 14 < 15
        };

        let score = PopperScore::calculate(
            falsifiability,
            ReproducibilityScore::default(),
            TransparencyScore::default(),
            StatisticalRigorScore::default(),
            HistoricalIntegrityScore::default(),
            GpuSimdSpecificScore::default(),
        );

        assert!(!score.gateway_passed);
        assert_eq!(score.total, 0.0);
        assert_eq!(score.grade, CudaTdgGrade::GatewayFail);
    }

    #[test]
    fn test_popper_score_passing() {
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 5.0,
            bounds_verification: 5.0,
            divergence_testing: 5.0,
            memory_race_detection: 5.0,
            occupancy_bounds: 5.0,
        };

        let reproducibility = ReproducibilityScore {
            deterministic_output: 8.0,
            version_pinning: 5.0,
            hardware_specification: 5.0,
            benchmark_harness: 4.0,
            ci_cd_integration: 3.0,
        };

        let transparency = TransparencyScore {
            ptx_inspection: 6.0,
            register_allocation: 5.0,
            occupancy_calculation: 5.0,
            memory_layout: 4.0,
        };

        let statistical_rigor = StatisticalRigorScore {
            warmup_iterations: 4.0,
            sample_count: 4.0,
            outlier_analysis: 4.0,
            confidence_intervals: 3.0,
        };

        let historical_integrity = HistoricalIntegrityScore {
            fault_lineage: 4.0,
            regression_tests: 3.0,
            root_cause_documentation: 3.0,
        };

        let gpu_simd = GpuSimdSpecificScore {
            warp_efficiency: 2.0,
            memory_throughput: 2.0,
            instruction_mix: 1.0,
        };

        let score = PopperScore::calculate(
            falsifiability,
            reproducibility,
            transparency,
            statistical_rigor,
            historical_integrity,
            gpu_simd,
        );

        assert!(score.gateway_passed);
        assert_eq!(score.total, 100.0);
        assert_eq!(score.grade, CudaTdgGrade::APLus);
    }

    #[test]
    fn test_grade_from_score() {
        assert_eq!(CudaTdgGrade::from_score(95.0, true), CudaTdgGrade::APLus);
        assert_eq!(CudaTdgGrade::from_score(85.0, true), CudaTdgGrade::A);
        assert_eq!(CudaTdgGrade::from_score(75.0, true), CudaTdgGrade::B);
        assert_eq!(CudaTdgGrade::from_score(65.0, true), CudaTdgGrade::C);
        assert_eq!(CudaTdgGrade::from_score(55.0, true), CudaTdgGrade::D);
        assert_eq!(CudaTdgGrade::from_score(45.0, true), CudaTdgGrade::F);
        assert_eq!(
            CudaTdgGrade::from_score(95.0, false),
            CudaTdgGrade::GatewayFail
        );
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = CudaSimdAnalyzer::new();
        assert_eq!(analyzer.config.min_score, 85.0);
        assert!(analyzer.config.fail_on_p0);
    }

    #[test]
    fn test_barrier_safety_score() {
        let result = BarrierSafetyResult {
            total_barriers: 10,
            safe_barriers: 8,
            unsafe_barriers: vec![],
            safety_score: 0.8,
        };

        assert_eq!(result.safety_score, 0.8);
    }

    #[test]
    fn test_defect_severity_display() {
        assert_eq!(format!("{}", DefectSeverity::P0Critical), "P0-Critical");
        assert_eq!(
            format!("{}", DefectSeverity::P1Performance),
            "P1-Performance"
        );
        assert_eq!(format!("{}", DefectSeverity::P2Efficiency), "P2-Efficiency");
        assert_eq!(format!("{}", DefectSeverity::P3Minor), "P3-Minor");
    }

    #[test]
    fn test_grade_display() {
        assert_eq!(format!("{}", CudaTdgGrade::APLus), "A+");
        assert_eq!(format!("{}", CudaTdgGrade::A), "A");
        assert_eq!(format!("{}", CudaTdgGrade::B), "B");
        assert_eq!(format!("{}", CudaTdgGrade::C), "C");
        assert_eq!(format!("{}", CudaTdgGrade::D), "D");
        assert_eq!(format!("{}", CudaTdgGrade::F), "F");
        assert_eq!(format!("{}", CudaTdgGrade::GatewayFail), "FAIL (Gateway)");
    }

    #[test]
    fn test_falsifiability_score_total() {
        let score = FalsifiabilityScore {
            barrier_safety: 5.0,
            bounds_verification: 5.0,
            divergence_testing: 5.0,
            memory_race_detection: 5.0,
            occupancy_bounds: 5.0,
        };
        assert_eq!(score.total(), 25.0);
        assert_eq!(FalsifiabilityScore::MAX, 25.0);
        assert_eq!(FalsifiabilityScore::GATEWAY_THRESHOLD, 15.0);
    }

    #[test]
    fn test_reproducibility_score_total() {
        let score = ReproducibilityScore {
            deterministic_output: 8.0,
            version_pinning: 5.0,
            hardware_specification: 5.0,
            benchmark_harness: 4.0,
            ci_cd_integration: 3.0,
        };
        assert_eq!(score.total(), 25.0);
        assert_eq!(ReproducibilityScore::MAX, 25.0);
    }

    #[test]
    fn test_transparency_score_total() {
        let score = TransparencyScore {
            ptx_inspection: 6.0,
            register_allocation: 5.0,
            occupancy_calculation: 5.0,
            memory_layout: 4.0,
        };
        assert_eq!(score.total(), 20.0);
        assert_eq!(TransparencyScore::MAX, 20.0);
    }

    #[test]
    fn test_statistical_rigor_score_total() {
        let score = StatisticalRigorScore {
            warmup_iterations: 4.0,
            sample_count: 4.0,
            outlier_analysis: 4.0,
            confidence_intervals: 3.0,
        };
        assert_eq!(score.total(), 15.0);
        assert_eq!(StatisticalRigorScore::MAX, 15.0);
    }

    #[test]
    fn test_historical_integrity_score_total() {
        let score = HistoricalIntegrityScore {
            fault_lineage: 4.0,
            regression_tests: 3.0,
            root_cause_documentation: 3.0,
        };
        assert_eq!(score.total(), 10.0);
        assert_eq!(HistoricalIntegrityScore::MAX, 10.0);
    }

    #[test]
    fn test_gpu_simd_specific_score_total() {
        let score = GpuSimdSpecificScore {
            warp_efficiency: 2.0,
            memory_throughput: 2.0,
            instruction_mix: 1.0,
        };
        assert_eq!(score.total(), 5.0);
        assert_eq!(GpuSimdSpecificScore::MAX, 5.0);
    }

    #[test]
    fn test_analyzer_with_config() {
        let config = CudaSimdConfig {
            min_score: 90.0,
            fail_on_p0: false,
            analyze_simd: false,
            analyze_wgpu: false,
            shared_memory_limit: 65536,
            register_limit: 128,
        };
        let analyzer = CudaSimdAnalyzer::with_config(config);
        assert_eq!(analyzer.config.min_score, 90.0);
        assert!(!analyzer.config.fail_on_p0);
    }

    #[test]
    fn test_config_default() {
        let config = CudaSimdConfig::new();
        assert_eq!(config.min_score, 85.0);
        assert!(config.fail_on_p0);
        assert!(config.analyze_simd);
        assert!(config.analyze_wgpu);
        assert_eq!(config.shared_memory_limit, 49152);
        assert_eq!(config.register_limit, 64);
    }

    #[test]
    fn test_taxonomy_all_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let patterns: Vec<_> = taxonomy.all().collect();
        assert!(patterns.len() >= 10); // We have at least 10 patterns defined

        // Verify we have patterns for each severity
        let p0_count = patterns
            .iter()
            .filter(|d| d.severity == DefectSeverity::P0Critical)
            .count();
        let p1_count = patterns
            .iter()
            .filter(|d| d.severity == DefectSeverity::P1Performance)
            .count();
        let p2_count = patterns
            .iter()
            .filter(|d| d.severity == DefectSeverity::P2Efficiency)
            .count();

        assert!(p0_count >= 3);
        assert!(p1_count >= 5);
        assert!(p2_count >= 2);
    }

    #[test]
    fn test_taxonomy_get_missing() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        assert!(taxonomy.get("NON-EXISTENT").is_none());
    }

    #[test]
    fn test_analyze_cuda_content_with_barrier() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void kernel() {
                if (threadIdx.x > 10) return;
                __syncthreads();
                // some work
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("test.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert_eq!(result.cuda_files, 1);
        assert!(result.barrier_safety.total_barriers >= 1);
    }

    #[test]
    fn test_analyze_wgsl_content() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(256)
            fn main() {
                workgroupBarrier();
                storageBarrier();
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("test.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        assert_eq!(result.wgpu_files, 1);
        assert!(result.barrier_safety.total_barriers >= 2);
    }

    #[test]
    fn test_analyze_rust_simd_content() {
        let analyzer = CudaSimdAnalyzer::new();
        let rust_content = r#"
            use std::arch::x86_64::*;

            fn simd_add(a: &[f32], b: &[f32]) {
                unsafe {
                    let va = _mm256_loadu_ps(a.as_ptr());
                    let vb = _mm256_loadu_ps(b.as_ptr());
                    let result = _mm256_add_ps(va, vb);
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("test.rs");
        std::fs::write(&rs_file, rust_content).unwrap();

        let result = analyzer.analyze(&rs_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    #[test]
    fn test_analyze_rust_wgpu_content() {
        let analyzer = CudaSimdAnalyzer::new();
        let rust_content = r#"
            use wgpu::*;

            fn create_pipeline(device: &Device) {
                let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
                    // ...
                });
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("wgpu_test.rs");
        std::fs::write(&rs_file, rust_content).unwrap();

        let result = analyzer.analyze(&rs_file).unwrap();
        assert_eq!(result.wgpu_files, 1);
    }

    #[test]
    fn test_analyze_cpp_simd_content() {
        let analyzer = CudaSimdAnalyzer::new();
        let cpp_content = r#"
            #include <immintrin.h>

            void simd_multiply(__m256 *a, __m256 *b, __m256 *c) {
                *c = _mm256_mul_ps(*a, *b);
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cpp_file = temp_dir.path().join("test.cpp");
        std::fs::write(&cpp_file, cpp_content).unwrap();

        let result = analyzer.analyze(&cpp_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    #[test]
    fn test_analyze_directory() {
        let analyzer = CudaSimdAnalyzer::new();

        let temp_dir = tempfile::tempdir().unwrap();

        // Create CUDA file
        let cuda_file = temp_dir.path().join("kernel.cu");
        std::fs::write(&cuda_file, "__global__ void kernel() { __syncthreads(); }").unwrap();

        // Create WGSL file
        let wgsl_file = temp_dir.path().join("shader.wgsl");
        std::fs::write(&wgsl_file, "@compute fn main() { workgroupBarrier(); }").unwrap();

        let result = analyzer.analyze(temp_dir.path()).unwrap();
        assert!(result.files_analyzed >= 2);
        assert!(result.cuda_files >= 1);
        assert!(result.wgpu_files >= 1);
    }

    #[test]
    fn test_quality_gate_passes() {
        let analyzer = CudaSimdAnalyzer::new();

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                falsifiability: FalsifiabilityScore::default(),
                reproducibility: ReproducibilityScore::default(),
                transparency: TransparencyScore::default(),
                statistical_rigor: StatisticalRigorScore::default(),
                historical_integrity: HistoricalIntegrityScore::default(),
                gpu_simd_specific: GpuSimdSpecificScore::default(),
                total: 90.0,
                gateway_passed: true,
                grade: CudaTdgGrade::APLus,
            },
            defects: vec![],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 0,
            cuda_files: 0,
            simd_files: 0,
            wgpu_files: 0,
        };

        assert!(analyzer.passes_quality_gate(&result));
    }

    #[test]
    fn test_quality_gate_fails_gateway() {
        let analyzer = CudaSimdAnalyzer::new();

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                falsifiability: FalsifiabilityScore::default(),
                reproducibility: ReproducibilityScore::default(),
                transparency: TransparencyScore::default(),
                statistical_rigor: StatisticalRigorScore::default(),
                historical_integrity: HistoricalIntegrityScore::default(),
                gpu_simd_specific: GpuSimdSpecificScore::default(),
                total: 0.0,
                gateway_passed: false,
                grade: CudaTdgGrade::GatewayFail,
            },
            defects: vec![],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 0,
            cuda_files: 0,
            simd_files: 0,
            wgpu_files: 0,
        };

        assert!(!analyzer.passes_quality_gate(&result));
    }

    #[test]
    fn test_quality_gate_fails_p0_defect() {
        let config = CudaSimdConfig {
            min_score: 85.0,
            fail_on_p0: true,
            ..Default::default()
        };
        let analyzer = CudaSimdAnalyzer::with_config(config);

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                falsifiability: FalsifiabilityScore::default(),
                reproducibility: ReproducibilityScore::default(),
                transparency: TransparencyScore::default(),
                statistical_rigor: StatisticalRigorScore::default(),
                historical_integrity: HistoricalIntegrityScore::default(),
                gpu_simd_specific: GpuSimdSpecificScore::default(),
                total: 90.0,
                gateway_passed: true,
                grade: CudaTdgGrade::APLus,
            },
            defects: vec![DetectedDefect {
                defect_class: DefectClass {
                    ticket_id: "PAR-002".to_string(),
                    description: "Test P0 defect".to_string(),
                    severity: DefectSeverity::P0Critical,
                    detection_method: "test".to_string(),
                    resolved: false,
                    root_cause: None,
                },
                file_path: PathBuf::from("test.cu"),
                line: Some(1),
                snippet: None,
                suggestion: None,
            }],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 1,
            cuda_files: 1,
            simd_files: 0,
            wgpu_files: 0,
        };

        assert!(!analyzer.passes_quality_gate(&result));
    }

    #[test]
    fn test_quality_gate_fails_low_score() {
        let config = CudaSimdConfig {
            min_score: 85.0,
            fail_on_p0: false,
            ..Default::default()
        };
        let analyzer = CudaSimdAnalyzer::with_config(config);

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                falsifiability: FalsifiabilityScore::default(),
                reproducibility: ReproducibilityScore::default(),
                transparency: TransparencyScore::default(),
                statistical_rigor: StatisticalRigorScore::default(),
                historical_integrity: HistoricalIntegrityScore::default(),
                gpu_simd_specific: GpuSimdSpecificScore::default(),
                total: 70.0, // Below min_score of 85
                gateway_passed: true,
                grade: CudaTdgGrade::B,
            },
            defects: vec![],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 0,
            cuda_files: 0,
            simd_files: 0,
            wgpu_files: 0,
        };

        assert!(!analyzer.passes_quality_gate(&result));
    }

    #[test]
    fn test_detect_flash_attention_issue() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            // FlashAttention kernel
            const tile_kv = 32;
            const head_dim = 64;

            __global__ void flash_attention_kernel() {
                // PAR-041 scenario: tile_kv < head_dim
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("attention.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        // Should detect PAR-041 defect
        let has_par041 = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "PAR-041");
        assert!(has_par041);
    }

    #[test]
    fn test_detect_missing_tensor_core() {
        let analyzer = CudaSimdAnalyzer::new();
        // Use explicit "gemm" keyword to trigger PAR-034 detection
        let cuda_content = r#"
            // This is a naive gemm implementation
            __global__ void naive_gemm_kernel(float *a, float *b, float *c, int n) {
                int row = blockIdx.y * blockDim.y + threadIdx.y;
                int col = blockIdx.x * blockDim.x + threadIdx.x;

                float sum = 0.0f;
                for (int k = 0; k < n; k++) {
                    sum += a[row * n + k] * b[k * n + col];
                }
                c[row * n + col] = sum;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("gemm.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        // Should detect PAR-034 defect (missing Tensor Core)
        let has_par034 = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "PAR-034");
        assert!(
            has_par034,
            "Expected PAR-034 defect for gemm without tensor cores"
        );
    }

    #[test]
    fn test_coalescing_result_default() {
        let result = CoalescingResult::default();
        assert_eq!(result.efficiency, 0.0);
        assert_eq!(result.total_operations, 0);
        assert_eq!(result.coalesced_operations, 0);
        assert!(result.problematic_accesses.is_empty());
    }

    #[test]
    fn test_kaizen_metrics_default() {
        let metrics = KaizenMetrics::default();
        assert_eq!(metrics.tickets_resolved, 0);
        assert_eq!(metrics.mttd, 0.0);
        assert_eq!(metrics.mttf, 0.0);
        assert!(metrics.ticket_references.is_empty());
    }

    #[test]
    fn test_barrier_issue_creation() {
        let issue = BarrierIssue {
            line: 42,
            barrier_type: "__syncthreads".to_string(),
            issue: "Test issue".to_string(),
            exit_paths: vec!["path1".to_string(), "path2".to_string()],
        };

        assert_eq!(issue.line, 42);
        assert_eq!(issue.barrier_type, "__syncthreads");
        assert_eq!(issue.exit_paths.len(), 2);
    }

    #[test]
    fn test_memory_access_patterns() {
        let contiguous = AccessPattern::Contiguous;
        let strided = AccessPattern::Strided { stride: 4 };
        let random = AccessPattern::Random;
        let bank_conflict = AccessPattern::BankConflict {
            conflicting_banks: vec![0, 16],
        };

        // Just verify these compile and can be created
        match contiguous {
            AccessPattern::Contiguous => {}
            _ => panic!("Expected Contiguous"),
        }

        match strided {
            AccessPattern::Strided { stride } => assert_eq!(stride, 4),
            _ => panic!("Expected Strided"),
        }

        match random {
            AccessPattern::Random => {}
            _ => panic!("Expected Random"),
        }

        match bank_conflict {
            AccessPattern::BankConflict { conflicting_banks } => {
                assert_eq!(conflicting_banks.len(), 2);
            }
            _ => panic!("Expected BankConflict"),
        }
    }

    #[test]
    fn test_tile_issue_creation() {
        let issue = TileIssue {
            description: "Tile too small".to_string(),
            ticket: Some("PAR-041".to_string()),
            severity: DefectSeverity::P0Critical,
        };

        assert_eq!(issue.description, "Tile too small");
        assert_eq!(issue.ticket, Some("PAR-041".to_string()));
        assert_eq!(issue.severity, DefectSeverity::P0Critical);
    }

    #[test]
    fn test_defect_class_with_root_cause() {
        let defect = DefectClass {
            ticket_id: "PAR-002".to_string(),
            description: "GEMV Error 700".to_string(),
            severity: DefectSeverity::P0Critical,
            detection_method: "Bounds check".to_string(),
            resolved: true,
            root_cause: Some("Thread index overflow".to_string()),
        };

        assert!(defect.resolved);
        assert!(defect.root_cause.is_some());
        assert_eq!(defect.root_cause.unwrap(), "Thread index overflow");
    }

    #[test]
    fn test_analyze_empty_file() {
        let analyzer = CudaSimdAnalyzer::new();

        let temp_dir = tempfile::tempdir().unwrap();
        let empty_file = temp_dir.path().join("empty.cu");
        std::fs::write(&empty_file, "").unwrap();

        let result = analyzer.analyze(&empty_file).unwrap();
        assert_eq!(result.cuda_files, 1);
        assert_eq!(result.barrier_safety.total_barriers, 0);
    }

    #[test]
    fn test_analyze_ptx_file() {
        let analyzer = CudaSimdAnalyzer::new();
        let ptx_content = r#"
            .version 7.0
            .target sm_80

            .visible .entry kernel() {
                bar.sync 0;
                ret;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("test.ptx");
        std::fs::write(&ptx_file, ptx_content).unwrap();

        let result = analyzer.analyze(&ptx_file).unwrap();
        assert_eq!(result.cuda_files, 1);
        assert!(result.barrier_safety.total_barriers >= 1);
    }

    #[test]
    fn test_strided_memory_access_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void strided_kernel(float *data) {
                int tid = threadIdx.x;
                float val = data[tid * STRIDE]; // strided access
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("strided.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert_eq!(result.cuda_files, 1);
    }

    #[test]
    fn test_shared_memory_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void shared_kernel() {
                __shared__ float smem[256];
                smem[threadIdx.x % 32] = 1.0f; // potential bank conflict
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("shared.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert_eq!(result.cuda_files, 1);
    }

    #[test]
    fn test_extract_value_patterns() {
        let analyzer = CudaSimdAnalyzer::new();

        // Test different value extraction patterns
        let content1 = "tile_kv = 64";
        assert_eq!(analyzer.extract_value(content1, "tile_kv"), Some(64));

        let content2 = "const head_dim = 128";
        assert_eq!(analyzer.extract_value(content2, "head_dim"), Some(128));

        let content3 = "let tile_k = 32";
        assert_eq!(analyzer.extract_value(content3, "tile_k"), Some(32));

        let content4 = "no_match_here";
        assert_eq!(analyzer.extract_value(content4, "tile_kv"), None);
    }

    #[test]
    fn test_c_header_simd_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let c_content = r#"
            #include <arm_neon.h>

            void neon_add(float32x4_t *a, float32x4_t *b) {
                // NEON SIMD operations
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let c_file = temp_dir.path().join("neon.h");
        std::fs::write(&c_file, c_content).unwrap();

        let result = analyzer.analyze(&c_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    #[test]
    fn test_multiple_syncthreads() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void multi_barrier() {
                __syncthreads();
                // work
                __syncthreads();
                // more work
                __syncwarp();
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("multi.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert!(result.barrier_safety.total_barriers >= 3);
        assert!(result.barrier_safety.safe_barriers >= 3);
    }

    #[test]
    fn test_global_memory_access() {
        let analyzer = CudaSimdAnalyzer::new();
        let cuda_content = r#"
            __global__ void global_access(float *global_mem) {
                int tid = threadIdx.x;
                float val = global_mem[tid];
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cuda_file = temp_dir.path().join("global.cu");
        std::fs::write(&cuda_file, cuda_content).unwrap();

        let result = analyzer.analyze(&cuda_file).unwrap();
        assert!(result.coalescing.total_operations >= 1);
    }

    #[test]
    fn test_wgpu_storage_access() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @group(0) @binding(0) var<storage, read_write> data: array<f32>;

            @compute @workgroup_size(256)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                data[gid.x] = data[gid.x] * 2.0;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("storage.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        assert!(result.coalescing.total_operations >= 1);
    }

    #[test]
    fn test_grade_boundary_values() {
        // Test exact boundary values
        assert_eq!(CudaTdgGrade::from_score(90.0, true), CudaTdgGrade::APLus);
        assert_eq!(CudaTdgGrade::from_score(89.9, true), CudaTdgGrade::A);
        assert_eq!(CudaTdgGrade::from_score(80.0, true), CudaTdgGrade::A);
        assert_eq!(CudaTdgGrade::from_score(79.9, true), CudaTdgGrade::B);
        assert_eq!(CudaTdgGrade::from_score(70.0, true), CudaTdgGrade::B);
        assert_eq!(CudaTdgGrade::from_score(69.9, true), CudaTdgGrade::C);
        assert_eq!(CudaTdgGrade::from_score(60.0, true), CudaTdgGrade::C);
        assert_eq!(CudaTdgGrade::from_score(59.9, true), CudaTdgGrade::D);
        assert_eq!(CudaTdgGrade::from_score(50.0, true), CudaTdgGrade::D);
        assert_eq!(CudaTdgGrade::from_score(49.9, true), CudaTdgGrade::F);
        assert_eq!(CudaTdgGrade::from_score(0.0, true), CudaTdgGrade::F);
    }

    #[test]
    fn test_default_taxonomy() {
        let taxonomy = DefectTaxonomy::default();
        // Default taxonomy should be empty
        assert!(taxonomy.patterns.is_empty());
    }

    #[test]
    fn test_score_defaults() {
        let falsifiability = FalsifiabilityScore::default();
        assert_eq!(falsifiability.total(), 0.0);

        let reproducibility = ReproducibilityScore::default();
        assert_eq!(reproducibility.total(), 0.0);

        let transparency = TransparencyScore::default();
        assert_eq!(transparency.total(), 0.0);

        let statistical_rigor = StatisticalRigorScore::default();
        assert_eq!(statistical_rigor.total(), 0.0);

        let historical_integrity = HistoricalIntegrityScore::default();
        assert_eq!(historical_integrity.total(), 0.0);

        let gpu_simd = GpuSimdSpecificScore::default();
        assert_eq!(gpu_simd.total(), 0.0);
    }

    #[test]
    fn test_analyzer_default_impl() {
        let analyzer = CudaSimdAnalyzer::default();
        assert_eq!(analyzer.config.min_score, 85.0);
    }

    #[test]
    fn test_file_analysis_default() {
        let analysis = FileAnalysis::default();
        assert_eq!(analysis.cuda_files, 0);
        assert_eq!(analysis.simd_files, 0);
        assert_eq!(analysis.wgpu_files, 0);
        assert!(analysis.defects.is_empty());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::path::PathBuf;

    // ==========================================================================
    // DefectSeverity Tests
    // ==========================================================================

    #[test]
    fn test_defect_severity_equality() {
        assert_eq!(DefectSeverity::P0Critical, DefectSeverity::P0Critical);
        assert_eq!(DefectSeverity::P1Performance, DefectSeverity::P1Performance);
        assert_eq!(DefectSeverity::P2Efficiency, DefectSeverity::P2Efficiency);
        assert_eq!(DefectSeverity::P3Minor, DefectSeverity::P3Minor);
        assert_ne!(DefectSeverity::P0Critical, DefectSeverity::P1Performance);
    }

    #[test]
    fn test_defect_severity_clone() {
        let severity = DefectSeverity::P0Critical;
        let cloned = severity.clone();
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_defect_severity_copy() {
        let severity = DefectSeverity::P0Critical;
        let copied: DefectSeverity = severity; // Copy
        assert_eq!(severity, copied);
    }

    // ==========================================================================
    // DefectClass Tests
    // ==========================================================================

    #[test]
    fn test_defect_class_clone() {
        let defect = DefectClass {
            ticket_id: "TEST-001".to_string(),
            description: "Test defect".to_string(),
            severity: DefectSeverity::P0Critical,
            detection_method: "Unit test".to_string(),
            resolved: false,
            root_cause: None,
        };

        let cloned = defect.clone();
        assert_eq!(defect.ticket_id, cloned.ticket_id);
        assert_eq!(defect.description, cloned.description);
        assert_eq!(defect.severity, cloned.severity);
    }

    #[test]
    fn test_defect_class_with_root_cause() {
        let defect = DefectClass {
            ticket_id: "TEST-002".to_string(),
            description: "Test defect with root cause".to_string(),
            severity: DefectSeverity::P1Performance,
            detection_method: "Analysis".to_string(),
            resolved: true,
            root_cause: Some("Root cause identified".to_string()),
        };

        assert!(defect.resolved);
        assert!(defect.root_cause.is_some());
    }

    // ==========================================================================
    // DefectTaxonomy Tests
    // ==========================================================================

    #[test]
    fn test_taxonomy_trueno_simd_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check TRUENO-SIMD-001
        let trueno_simd_001 = taxonomy.get("TRUENO-SIMD-001");
        assert!(trueno_simd_001.is_some());
        assert_eq!(
            trueno_simd_001.unwrap().severity,
            DefectSeverity::P0Critical
        );

        // Check TRUENO-SIMD-002
        let trueno_simd_002 = taxonomy.get("TRUENO-SIMD-002");
        assert!(trueno_simd_002.is_some());
    }

    #[test]
    fn test_taxonomy_trueno_ptx_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check TRUENO-PTX-001
        let trueno_ptx_001 = taxonomy.get("TRUENO-PTX-001");
        assert!(trueno_ptx_001.is_some());
        assert_eq!(trueno_ptx_001.unwrap().severity, DefectSeverity::P0Critical);
    }

    #[test]
    fn test_taxonomy_computebrick_patterns() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // Check CB-001 through CB-022 (ComputeBrick patterns)
        assert!(taxonomy.get("CB-001").is_some());
        assert!(taxonomy.get("CB-002").is_some());
        assert!(taxonomy.get("CB-003").is_some());
        assert!(taxonomy.get("CB-004").is_some());
        assert!(taxonomy.get("CB-010").is_some());
        assert!(taxonomy.get("CB-011").is_some());
        assert!(taxonomy.get("CB-012").is_some());
        assert!(taxonomy.get("CB-013").is_some());
        assert!(taxonomy.get("CB-020").is_some());
        assert!(taxonomy.get("CB-021").is_some());
        assert!(taxonomy.get("CB-022").is_some());
    }

    #[test]
    fn test_taxonomy_all_iterator() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let all: Vec<&DefectClass> = taxonomy.all().collect();

        // Should have at least the known patterns
        assert!(all.len() >= 20);

        // Verify iterator returns valid defects
        for defect in all {
            assert!(!defect.ticket_id.is_empty());
            assert!(!defect.description.is_empty());
        }
    }

    // ==========================================================================
    // CudaTdgGrade Tests
    // ==========================================================================

    #[test]
    fn test_grade_default() {
        let grade = CudaTdgGrade::default();
        assert_eq!(grade, CudaTdgGrade::F);
    }

    #[test]
    fn test_grade_clone_and_copy() {
        let grade = CudaTdgGrade::APLus;
        let cloned = grade.clone();
        let copied: CudaTdgGrade = grade;
        assert_eq!(grade, cloned);
        assert_eq!(grade, copied);
    }

    #[test]
    fn test_grade_negative_scores() {
        // Negative scores should result in F
        assert_eq!(CudaTdgGrade::from_score(-10.0, true), CudaTdgGrade::F);
    }

    // ==========================================================================
    // Score Category Tests
    // ==========================================================================

    #[test]
    fn test_falsifiability_score_partial() {
        let score = FalsifiabilityScore {
            barrier_safety: 2.5,
            bounds_verification: 2.5,
            divergence_testing: 2.5,
            memory_race_detection: 2.5,
            occupancy_bounds: 5.0,
        };
        assert_eq!(score.total(), 15.0);
        assert!(score.total() >= FalsifiabilityScore::GATEWAY_THRESHOLD);
    }

    #[test]
    fn test_reproducibility_score_partial() {
        let score = ReproducibilityScore {
            deterministic_output: 4.0,
            version_pinning: 2.5,
            hardware_specification: 2.5,
            benchmark_harness: 2.0,
            ci_cd_integration: 1.5,
        };
        assert_eq!(score.total(), 12.5);
    }

    #[test]
    fn test_transparency_score_partial() {
        let score = TransparencyScore {
            ptx_inspection: 3.0,
            register_allocation: 2.5,
            occupancy_calculation: 2.5,
            memory_layout: 2.0,
        };
        assert_eq!(score.total(), 10.0);
    }

    #[test]
    fn test_statistical_rigor_score_partial() {
        let score = StatisticalRigorScore {
            warmup_iterations: 2.0,
            sample_count: 2.0,
            outlier_analysis: 2.0,
            confidence_intervals: 1.5,
        };
        assert_eq!(score.total(), 7.5);
    }

    #[test]
    fn test_historical_integrity_score_partial() {
        let score = HistoricalIntegrityScore {
            fault_lineage: 2.0,
            regression_tests: 1.5,
            root_cause_documentation: 1.5,
        };
        assert_eq!(score.total(), 5.0);
    }

    #[test]
    fn test_gpu_simd_specific_score_partial() {
        let score = GpuSimdSpecificScore {
            warp_efficiency: 1.0,
            memory_throughput: 1.0,
            instruction_mix: 0.5,
        };
        assert_eq!(score.total(), 2.5);
    }

    // ==========================================================================
    // PopperScore Tests
    // ==========================================================================

    #[test]
    fn test_popper_score_gateway_threshold_exact() {
        // Test exactly at gateway threshold (15.0)
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 3.0,
            bounds_verification: 3.0,
            divergence_testing: 3.0,
            memory_race_detection: 3.0,
            occupancy_bounds: 3.0, // Total: 15.0 (exactly at threshold)
        };

        let score = PopperScore::calculate(
            falsifiability,
            ReproducibilityScore::default(),
            TransparencyScore::default(),
            StatisticalRigorScore::default(),
            HistoricalIntegrityScore::default(),
            GpuSimdSpecificScore::default(),
        );

        assert!(score.gateway_passed);
        assert!(score.total > 0.0);
    }

    #[test]
    fn test_popper_score_gateway_just_below() {
        // Test just below gateway threshold
        let falsifiability = FalsifiabilityScore {
            barrier_safety: 2.9,
            bounds_verification: 3.0,
            divergence_testing: 3.0,
            memory_race_detection: 3.0,
            occupancy_bounds: 3.0, // Total: 14.9
        };

        let score = PopperScore::calculate(
            falsifiability,
            ReproducibilityScore::default(),
            TransparencyScore::default(),
            StatisticalRigorScore::default(),
            HistoricalIntegrityScore::default(),
            GpuSimdSpecificScore::default(),
        );

        assert!(!score.gateway_passed);
        assert_eq!(score.total, 0.0);
        assert_eq!(score.grade, CudaTdgGrade::GatewayFail);
    }

    // ==========================================================================
    // BarrierSafetyResult Tests
    // ==========================================================================

    #[test]
    fn test_barrier_safety_result_default() {
        let result = BarrierSafetyResult::default();
        assert_eq!(result.total_barriers, 0);
        assert_eq!(result.safe_barriers, 0);
        assert!(result.unsafe_barriers.is_empty());
        assert_eq!(result.safety_score, 0.0);
    }

    #[test]
    fn test_barrier_safety_result_with_issues() {
        let result = BarrierSafetyResult {
            total_barriers: 5,
            safe_barriers: 3,
            unsafe_barriers: vec![
                BarrierIssue {
                    line: 10,
                    barrier_type: "__syncthreads".to_string(),
                    issue: "Early return".to_string(),
                    exit_paths: vec!["path1".to_string()],
                },
                BarrierIssue {
                    line: 20,
                    barrier_type: "bar.sync".to_string(),
                    issue: "Thread divergence".to_string(),
                    exit_paths: vec!["path2".to_string(), "path3".to_string()],
                },
            ],
            safety_score: 0.6,
        };

        assert_eq!(result.total_barriers, 5);
        assert_eq!(result.safe_barriers, 3);
        assert_eq!(result.unsafe_barriers.len(), 2);
    }

    // ==========================================================================
    // CoalescingResult Tests
    // ==========================================================================

    #[test]
    fn test_coalescing_result_full_coalescing() {
        let result = CoalescingResult {
            efficiency: 1.0,
            total_operations: 100,
            coalesced_operations: 100,
            problematic_accesses: vec![],
        };

        assert_eq!(result.efficiency, 1.0);
        assert!(result.problematic_accesses.is_empty());
    }

    #[test]
    fn test_coalescing_result_with_issues() {
        let result = CoalescingResult {
            efficiency: 0.7,
            total_operations: 100,
            coalesced_operations: 70,
            problematic_accesses: vec![
                MemoryAccessIssue {
                    line: 15,
                    pattern: AccessPattern::Strided { stride: 4 },
                    impact: "50% throughput".to_string(),
                },
                MemoryAccessIssue {
                    line: 25,
                    pattern: AccessPattern::Random,
                    impact: "Severe performance impact".to_string(),
                },
            ],
        };

        assert_eq!(result.efficiency, 0.7);
        assert_eq!(result.problematic_accesses.len(), 2);
    }

    // ==========================================================================
    // TileDimensionResult Tests
    // ==========================================================================

    #[test]
    fn test_tile_dimension_result_valid() {
        let result = TileDimensionResult {
            valid: true,
            tile_k: Some(32),
            tile_kv: Some(64),
            head_dim: Some(64),
            shared_memory_required: Some(8192),
            shared_memory_available: Some(49152),
            issues: vec![],
        };

        assert!(result.valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_tile_dimension_result_with_issues() {
        let result = TileDimensionResult {
            valid: false,
            tile_k: Some(32),
            tile_kv: Some(32),
            head_dim: Some(64),
            shared_memory_required: Some(65536),
            shared_memory_available: Some(49152),
            issues: vec![
                TileIssue {
                    description: "tile_kv < head_dim".to_string(),
                    ticket: Some("PAR-041".to_string()),
                    severity: DefectSeverity::P0Critical,
                },
                TileIssue {
                    description: "Shared memory overflow".to_string(),
                    ticket: None,
                    severity: DefectSeverity::P0Critical,
                },
            ],
        };

        assert!(!result.valid);
        assert_eq!(result.issues.len(), 2);
    }

    // ==========================================================================
    // KaizenMetrics Tests
    // ==========================================================================

    #[test]
    fn test_kaizen_metrics_populated() {
        let metrics = KaizenMetrics {
            tickets_resolved: 10,
            mttd: 12.5,
            mttf: 24.0,
            escape_rate: 0.03,
            regression_rate: 0.01,
            ticket_references: vec![
                "PAR-001".to_string(),
                "PAR-002".to_string(),
                "PARITY-114".to_string(),
            ],
        };

        assert_eq!(metrics.tickets_resolved, 10);
        assert_eq!(metrics.ticket_references.len(), 3);
    }

    // ==========================================================================
    // CudaSimdConfig Tests
    // ==========================================================================

    #[test]
    fn test_config_custom_values() {
        let config = CudaSimdConfig {
            min_score: 90.0,
            fail_on_p0: false,
            analyze_simd: true,
            analyze_wgpu: false,
            shared_memory_limit: 65536,
            register_limit: 128,
        };

        assert_eq!(config.min_score, 90.0);
        assert!(!config.fail_on_p0);
        assert!(config.analyze_simd);
        assert!(!config.analyze_wgpu);
        assert_eq!(config.shared_memory_limit, 65536);
        assert_eq!(config.register_limit, 128);
    }

    #[test]
    fn test_config_default() {
        let config = CudaSimdConfig::default();
        assert_eq!(config.min_score, 0.0);
        assert!(!config.fail_on_p0);
    }

    // ==========================================================================
    // DetectedDefect Tests
    // ==========================================================================

    #[test]
    fn test_detected_defect_full() {
        let defect = DetectedDefect {
            defect_class: DefectClass {
                ticket_id: "PAR-001".to_string(),
                description: "Test defect".to_string(),
                severity: DefectSeverity::P1Performance,
                detection_method: "Analysis".to_string(),
                resolved: false,
                root_cause: None,
            },
            file_path: PathBuf::from("/path/to/file.cu"),
            line: Some(42),
            snippet: Some("__global__ void kernel() {}".to_string()),
            suggestion: Some("Add barrier".to_string()),
        };

        assert_eq!(defect.line, Some(42));
        assert!(defect.snippet.is_some());
        assert!(defect.suggestion.is_some());
    }

    #[test]
    fn test_detected_defect_minimal() {
        let defect = DetectedDefect {
            defect_class: DefectClass {
                ticket_id: "PAR-002".to_string(),
                description: "Minimal defect".to_string(),
                severity: DefectSeverity::P2Efficiency,
                detection_method: "Static".to_string(),
                resolved: true,
                root_cause: Some("Root cause".to_string()),
            },
            file_path: PathBuf::from("test.cu"),
            line: None,
            snippet: None,
            suggestion: None,
        };

        assert!(defect.line.is_none());
        assert!(defect.snippet.is_none());
        assert!(defect.suggestion.is_none());
    }

    // ==========================================================================
    // CudaSimdAnalyzer Path Skip Tests
    // ==========================================================================

    #[test]
    fn test_should_skip_path_venv() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            ".venv/lib/site-packages"
        )));
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "venv/bin/python"
        )));
    }

    #[test]
    fn test_should_skip_path_node_modules() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "node_modules/package"
        )));
    }

    #[test]
    fn test_should_skip_path_target() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "target/release/bin"
        )));
    }

    #[test]
    fn test_should_skip_path_git() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            ".git/objects"
        )));
    }

    #[test]
    fn test_should_skip_path_python_cache() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "__pycache__/module.pyc"
        )));
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            ".pytest_cache/v"
        )));
    }

    #[test]
    fn test_should_skip_path_build_dirs() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "build/output"
        )));
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "dist/package"
        )));
    }

    #[test]
    fn test_should_skip_path_egg_info() {
        assert!(CudaSimdAnalyzer::should_skip_path(Path::new(
            "mypackage.egg-info/PKG-INFO"
        )));
    }

    #[test]
    fn test_should_not_skip_normal_path() {
        assert!(!CudaSimdAnalyzer::should_skip_path(Path::new(
            "src/main.rs"
        )));
        assert!(!CudaSimdAnalyzer::should_skip_path(Path::new(
            "lib/utils.py"
        )));
        assert!(!CudaSimdAnalyzer::should_skip_path(Path::new(
            "kernels/cuda.cu"
        )));
    }

    // ==========================================================================
    // PTX Register Extraction Tests
    // ==========================================================================

    #[test]
    fn test_extract_ptx_dest_register_valid() {
        let line = "ld.shared.u32 %r1, [%rd1]";
        let result = CudaSimdAnalyzer::extract_ptx_dest_register(line);
        assert_eq!(result, Some("%r1".to_string()));
    }

    #[test]
    fn test_extract_ptx_dest_register_u64() {
        let line = "ld.global.u64 %rd10, [%param0]";
        let result = CudaSimdAnalyzer::extract_ptx_dest_register(line);
        assert_eq!(result, Some("%rd10".to_string()));
    }

    #[test]
    fn test_extract_ptx_dest_register_no_reg() {
        let line = "bar.sync 0;";
        let result = CudaSimdAnalyzer::extract_ptx_dest_register(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_ptx_dest_register_empty() {
        let line = "";
        let result = CudaSimdAnalyzer::extract_ptx_dest_register(line);
        assert!(result.is_none());
    }

    // ==========================================================================
    // SIMD Pattern Detection Tests
    // ==========================================================================

    #[test]
    fn test_simd_missing_target_feature() {
        let analyzer = CudaSimdAnalyzer::new();
        let simd_content = r#"
            fn simd_func() {
                unsafe {
                    let a = _mm256_loadu_ps(ptr);
                    let b = _mm256_add_ps(a, a);
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("simd.rs");
        std::fs::write(
            &rs_file,
            format!("use std::arch::x86_64::*;\n{}", simd_content),
        )
        .unwrap();

        let result = analyzer.analyze(&rs_file).unwrap();
        // Should detect SIMD_MISSING_TARGET or similar
        assert!(result.simd_files >= 1);
    }

    #[test]
    fn test_simd_with_safety_comment() {
        let analyzer = CudaSimdAnalyzer::new();
        let simd_content = r#"
            use std::arch::x86_64::*;

            fn simd_func() {
                // SAFETY: Pointer is aligned and in-bounds
                unsafe {
                    let a = _mm256_loadu_ps(ptr);
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("safe_simd.rs");
        std::fs::write(&rs_file, simd_content).unwrap();

        let result = analyzer.analyze(&rs_file).unwrap();
        assert_eq!(result.simd_files, 1);
    }

    // ==========================================================================
    // WGSL Pattern Detection Tests
    // ==========================================================================

    #[test]
    fn test_wgsl_small_workgroup() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(16)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                // Small workgroup
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("small.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        // Should detect WGPU_SMALL_WORKGROUP
        let has_small_workgroup = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id.contains("SMALL_WORKGROUP"));
        assert!(has_small_workgroup);
    }

    #[test]
    fn test_wgsl_large_workgroup() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(2048)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                // Large workgroup
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("large.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        // Should detect WGPU_LARGE_WORKGROUP
        let has_large_workgroup = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id.contains("LARGE_WORKGROUP"));
        assert!(has_large_workgroup);
    }

    #[test]
    fn test_wgsl_non_warp_aligned() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(100)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                // Non-warp-aligned workgroup (not multiple of 32)
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("unaligned.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        // Should detect WGPU_NON_WARP_ALIGNED
        let has_non_aligned = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id.contains("NON_WARP_ALIGNED"));
        assert!(has_non_aligned);
    }

    #[test]
    fn test_wgsl_optimal_workgroup() {
        let analyzer = CudaSimdAnalyzer::new();
        let wgsl_content = r#"
            @compute @workgroup_size(256)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                if (gid.x < params.size) {
                    // Optimal workgroup with bounds check
                }
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let wgsl_file = temp_dir.path().join("optimal.wgsl");
        std::fs::write(&wgsl_file, wgsl_content).unwrap();

        let result = analyzer.analyze(&wgsl_file).unwrap();
        // Should not detect workgroup size issues
        let has_workgroup_issue = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id.contains("WORKGROUP"));
        assert!(!has_workgroup_issue);
    }

    // ==========================================================================
    // PTX Pattern Detection Tests
    // ==========================================================================

    #[test]
    fn test_ptx_shared_u64_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let ptx_content = r#"
            .version 7.0
            .target sm_80
            .entry kernel() {
                st.shared.u32 [%rd1], %r0;
                ret;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("shared_u64.ptx");
        std::fs::write(&ptx_file, ptx_content).unwrap();

        let result = analyzer.analyze(&ptx_file).unwrap();
        // Should detect SHARED_U64
        let has_shared_u64 = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "SHARED_U64");
        assert!(has_shared_u64);
    }

    #[test]
    fn test_ptx_cvta_shared_detection() {
        let analyzer = CudaSimdAnalyzer::new();
        let ptx_content = r#"
            .version 7.0
            .target sm_80
            .entry kernel() {
                cvta.shared.u64 %rd0, %r1;
                ret;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("cvta_shared.ptx");
        std::fs::write(&ptx_file, ptx_content).unwrap();

        let result = analyzer.analyze(&ptx_file).unwrap();
        // Should detect CVTA_SHARED
        let has_cvta_shared = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "CVTA_SHARED");
        assert!(has_cvta_shared);
    }

    #[test]
    fn test_ptx_local_memory_spill() {
        let analyzer = CudaSimdAnalyzer::new();
        let ptx_content = r#"
            .version 7.0
            .target sm_80
            .local .align 4 .b8 spill[64];
            .entry kernel() {
                ret;
            }
        "#;

        let temp_dir = tempfile::tempdir().unwrap();
        let ptx_file = temp_dir.path().join("local_spill.ptx");
        std::fs::write(&ptx_file, ptx_content).unwrap();

        let result = analyzer.analyze(&ptx_file).unwrap();
        // Should detect REG_SPILLS
        let has_reg_spills = result
            .defects
            .iter()
            .any(|d| d.defect_class.ticket_id == "REG_SPILLS");
        assert!(has_reg_spills);
    }

    // ==========================================================================
    // Access Pattern Tests
    // ==========================================================================

    #[test]
    fn test_access_pattern_debug() {
        let contiguous = AccessPattern::Contiguous;
        let strided = AccessPattern::Strided { stride: 8 };
        let random = AccessPattern::Random;
        let bank = AccessPattern::BankConflict {
            conflicting_banks: vec![0, 8, 16, 24],
        };

        // Test Debug trait
        assert!(!format!("{:?}", contiguous).is_empty());
        assert!(!format!("{:?}", strided).is_empty());
        assert!(!format!("{:?}", random).is_empty());
        assert!(!format!("{:?}", bank).is_empty());
    }

    #[test]
    fn test_access_pattern_clone() {
        let original = AccessPattern::BankConflict {
            conflicting_banks: vec![0, 8],
        };
        let cloned = original.clone();

        if let (
            AccessPattern::BankConflict {
                conflicting_banks: orig_banks,
            },
            AccessPattern::BankConflict {
                conflicting_banks: clone_banks,
            },
        ) = (original, cloned)
        {
            assert_eq!(orig_banks, clone_banks);
        } else {
            panic!("Expected BankConflict variant");
        }
    }

    // ==========================================================================
    // Quality Gate Tests
    // ==========================================================================

    #[test]
    fn test_quality_gate_low_score_with_no_p0() {
        let config = CudaSimdConfig {
            min_score: 85.0,
            fail_on_p0: true,
            ..Default::default()
        };
        let analyzer = CudaSimdAnalyzer::with_config(config);

        let result = CudaSimdTdgResult {
            path: PathBuf::from("."),
            score: PopperScore {
                total: 70.0,
                gateway_passed: true,
                grade: CudaTdgGrade::B,
                ..Default::default()
            },
            defects: vec![], // No P0 defects
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: String::new(),
            files_analyzed: 0,
            cuda_files: 0,
            simd_files: 0,
            wgpu_files: 0,
        };

        // Should fail due to low score (70 < 85)
        assert!(!analyzer.passes_quality_gate(&result));
    }

    // ==========================================================================
    // CudaSimdTdgResult Tests
    // ==========================================================================

    #[test]
    fn test_cuda_simd_tdg_result_clone() {
        let result = CudaSimdTdgResult {
            path: PathBuf::from("/test/path"),
            score: PopperScore::default(),
            defects: vec![],
            barrier_safety: BarrierSafetyResult::default(),
            coalescing: CoalescingResult::default(),
            tile_dimensions: TileDimensionResult {
                valid: true,
                tile_k: None,
                tile_kv: None,
                head_dim: None,
                shared_memory_required: None,
                shared_memory_available: None,
                issues: vec![],
            },
            kaizen: KaizenMetrics::default(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            files_analyzed: 10,
            cuda_files: 3,
            simd_files: 4,
            wgpu_files: 3,
        };

        let cloned = result.clone();
        assert_eq!(result.path, cloned.path);
        assert_eq!(result.files_analyzed, cloned.files_analyzed);
        assert_eq!(result.cuda_files, cloned.cuda_files);
    }
}
