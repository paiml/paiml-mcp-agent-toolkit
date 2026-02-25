//! COMPLY-008: 100-Point Popperian Falsification Test Suite
//!
//! Per Karl Popper's *The Logic of Scientific Discovery* (1934):
//! > "A theory which is not refutable by any conceivable event is non-scientific."
//!
//! This module contains adversarial test cases designed to FALSIFY our detection hypotheses.
//! Tests are written BEFORE implementation to ensure we're solving a real problem.
//!
//! ## Hypotheses Under Test
//!
//! **Hypothesis A (Detection)**: CB-050 correctly identifies all code-level stubs without false positives.
//!
//! **Hypothesis B (Regex Sufficiency)**: Regular expressions are sufficient to detect GPU barriers/branching
//! (CB-060) with >90% precision, without requiring a full AST parser.
//!
//! **Hypothesis C (Wild Stability)**: The checks are stable on unseen "Wild" code from external repos.
//!
//! ## Test Structure
//!
//! - Tests 001-030: CB-050 Stub Detection (attempts to falsify Hypothesis A)
//! - Tests 031-055: CB-060 GPU Quality (attempts to falsify Hypothesis B)
//! - Tests 056-070: SATD Manifestation Type
//! - Tests 071-085: Suppression Logic
//! - Tests 086-100: Integration & Wild Tests (attempts to falsify Hypothesis C)

// ============================================================================
// IMPORTS FROM IMPLEMENTATION
// CB-050 is now implemented; CB-060 remains in RED phase
// ============================================================================

// Import the real implementations from comply_cb_detect
use super::comply_cb_detect::{
    detect_cb050_code_stubs_in_str, detect_cb050_code_stubs_in_str_with_path,
};

// CB-060 functions are still stubs in comply_cb_detect.rs - import them too
use super::comply_cb_detect::{
    detect_ptx_barrier_divergence_in_str, detect_shared_memory_unbounded_in_str,
    detect_tiled_kernel_no_bounds_in_str, detect_wgsl_barrier_divergence_in_str,
};

// ============================================================================
// CB-050 STUB DETECTION FALSIFICATION TESTS (Tests 001-030)
// Hypothesis A: CB-050 correctly identifies all code-level stubs without FPs
// ============================================================================
include!("falsification_cb050.rs");

// ============================================================================
// CB-060 GPU QUALITY FALSIFICATION TESTS (Tests 031-055)
// Hypothesis B: Regex is sufficient for GPU pattern detection (>90% precision)
// ============================================================================
include!("falsification_cb060.rs");

// ============================================================================
// SATD MANIFESTATION TYPE TESTS (Tests 056-070)
// ============================================================================
include!("falsification_satd.rs");

// ============================================================================
// SUPPRESSION LOGIC TESTS (Tests 071-085)
// ============================================================================
include!("falsification_suppression.rs");

// ============================================================================
// INTEGRATION & WILD TESTS (Tests 086-100)
// Hypothesis C: Checks are stable on unseen "Wild" code
// ============================================================================
include!("falsification_integration.rs");
