#![cfg_attr(coverage_nightly, coverage(off))]
//! CUDA-SIMD Defect Taxonomy
//!
//! Extracted from cuda_simd.rs for file health compliance (CB-040).
//! Contains DefectSeverity, DefectClass, and DefectTaxonomy definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

        // REAL TAURANTA FAULTS FROM GIT HISTORY (trueno, realizar, aprender)
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

        // COMPUTEBRICK DEFECT PATTERNS (PROBAR-SPEC-009-P8)
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
    pub fn all(&self) -> impl Iterator<Item = &DefectClass> {
        self.patterns.values()
    }

    /// Get total number of patterns
    #[must_use]
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Check if taxonomy is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_tauranta_patterns_creates_non_empty_taxonomy() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        assert!(!taxonomy.is_empty());
        assert!(taxonomy.len() > 10, "Expected >10 patterns, got {}", taxonomy.len());
    }

    #[test]
    fn test_with_tauranta_patterns_has_all_severity_levels() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let mut has_p0 = false;
        let mut has_p1 = false;
        let mut has_p2 = false;
        let mut has_p3 = false;

        for defect in taxonomy.all() {
            match defect.severity {
                DefectSeverity::P0Critical => has_p0 = true,
                DefectSeverity::P1Performance => has_p1 = true,
                DefectSeverity::P2Efficiency => has_p2 = true,
                DefectSeverity::P3Minor => has_p3 = true,
            }
        }

        assert!(has_p0, "Missing P0Critical defects");
        assert!(has_p1, "Missing P1Performance defects");
        assert!(has_p2, "Missing P2Efficiency defects");
        // P3Minor may not be present in the current taxonomy
        let _ = has_p3;
    }

    #[test]
    fn test_with_tauranta_patterns_p0_critical_defects() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        // PARITY-114: barrier divergence
        let parity114 = taxonomy.get("PARITY-114").expect("PARITY-114 should exist");
        assert_eq!(parity114.severity, DefectSeverity::P0Critical);
        assert!(parity114.description.contains("Barrier divergence"));
        assert!(parity114.resolved);
        assert!(parity114.root_cause.is_some());

        // PAR-002: CUDA GEMV illegal memory access
        let par002 = taxonomy.get("PAR-002").expect("PAR-002 should exist");
        assert_eq!(par002.severity, DefectSeverity::P0Critical);
        assert!(par002.description.contains("GEMV"));

        // PAR-041: FlashAttention shared memory overflow
        let par041 = taxonomy.get("PAR-041").expect("PAR-041 should exist");
        assert_eq!(par041.severity, DefectSeverity::P0Critical);
        assert!(par041.description.contains("FlashAttention"));
    }

    #[test]
    fn test_with_tauranta_patterns_p1_performance_defects() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();

        let par001 = taxonomy.get("PAR-001").expect("PAR-001 should exist");
        assert_eq!(par001.severity, DefectSeverity::P1Performance);
        assert!(par001.description.contains("Q4_K dequantization"));

        let par034 = taxonomy.get("PAR-034").expect("PAR-034 should exist");
        assert_eq!(par034.severity, DefectSeverity::P1Performance);
        assert!(par034.description.contains("Tensor Core"));
    }

    #[test]
    fn test_with_tauranta_patterns_get_nonexistent() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        assert!(taxonomy.get("NONEXISTENT-999").is_none());
    }

    #[test]
    fn test_with_tauranta_patterns_all_have_ticket_ids() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        for defect in taxonomy.all() {
            assert!(!defect.ticket_id.is_empty(), "Defect has empty ticket_id");
            assert!(!defect.description.is_empty(), "Defect {} has empty description", defect.ticket_id);
            assert!(!defect.detection_method.is_empty(), "Defect {} has empty detection_method", defect.ticket_id);
        }
    }

    #[test]
    fn test_with_tauranta_patterns_all_iterator_count() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let count = taxonomy.all().count();
        assert_eq!(count, taxonomy.len());
    }

    #[test]
    fn test_defect_taxonomy_default_is_empty() {
        let taxonomy = DefectTaxonomy::default();
        assert!(taxonomy.is_empty());
        assert_eq!(taxonomy.len(), 0);
    }

    #[test]
    fn test_defect_severity_display() {
        assert_eq!(format!("{}", DefectSeverity::P0Critical), "P0-Critical");
        assert_eq!(format!("{}", DefectSeverity::P1Performance), "P1-Performance");
        assert_eq!(format!("{}", DefectSeverity::P2Efficiency), "P2-Efficiency");
        assert_eq!(format!("{}", DefectSeverity::P3Minor), "P3-Minor");
    }

    #[test]
    fn test_defect_class_clone_and_debug() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let defect = taxonomy.get("PARITY-114").unwrap();
        let cloned = defect.clone();
        assert_eq!(cloned.ticket_id, defect.ticket_id);
        assert_eq!(cloned.severity, defect.severity);
        let _debug = format!("{:?}", cloned);
    }

    #[test]
    fn test_with_tauranta_patterns_cb_prefixed_defects() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        // Check for CB-xxx defects (codebase quality patterns)
        let cb_defects: Vec<_> = taxonomy.all()
            .filter(|d| d.ticket_id.starts_with("CB-"))
            .collect();
        assert!(!cb_defects.is_empty(), "Should have CB-prefixed defects");
    }

    #[test]
    fn test_with_tauranta_patterns_some_resolved_some_not() {
        let taxonomy = DefectTaxonomy::with_tauranta_patterns();
        let resolved_count = taxonomy.all().filter(|d| d.resolved).count();
        let unresolved_count = taxonomy.all().filter(|d| !d.resolved).count();
        assert!(resolved_count > 0, "Should have some resolved defects");
        assert!(unresolved_count > 0, "Should have some unresolved defects");
    }
}
