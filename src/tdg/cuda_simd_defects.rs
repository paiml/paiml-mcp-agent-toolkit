#![cfg_attr(coverage_nightly, coverage(off))]
//! CUDA-SIMD Defect Taxonomy
//!
//! Extracted from cuda_simd.rs for file health compliance (CB-040).
//! Contains DefectSeverity, DefectClass, and DefectTaxonomy definitions.
//!
//! Split into submodules via include!() for file health compliance (PMAT-503):
//! - cuda_simd_defects_patterns.rs: Tauranta defect pattern definitions
//! - cuda_simd_defects_tests.rs: Unit tests

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
    /// Get defect pattern by ticket ID
    #[must_use]
    pub fn get(&self, ticket_id: &str) -> Option<&DefectClass> {
        debug_assert!(!ticket_id.is_empty(), "ticket_id must not be empty");
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

// Tauranta defect pattern definitions (with_tauranta_patterns impl)
include!("cuda_simd_defects_patterns.rs");

// Unit tests
include!("cuda_simd_defects_tests.rs");
