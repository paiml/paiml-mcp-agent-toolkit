// CB-050/CB-060 Detection Logic
// Implements stub and GPU quality checks per specification COMPLY-008
//
// Per Popper's falsification methodology:
// - Each pattern is a hypothesis that can be falsified
// - False positives falsify our precision hypothesis
// - False negatives falsify our detection hypothesis

// Regex::new() on compile-time constant patterns cannot fail; use .expect("valid regex")

use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// A violation detected by CB-050 or CB-060 checks
#[derive(Debug, Clone, PartialEq)]
pub struct CbViolation {
    pub line: u32,
    pub pattern_id: &'static str,
    pub description: String,
    pub code_snippet: Option<String>,
}

// COMPLY-005: SATD manifestation types and classification
include!("comply_cb_detect_satd.rs");

// COMPLY-007: Suppression configuration with O(1) lookups
include!("comply_cb_detect_suppression.rs");

// CB-050: Stub detection patterns and functions
include!("comply_cb_detect_cb050.rs");

// CB-060: GPU quality detection (PTX, WGSL, shared memory, tiled kernels)
include!("comply_cb_detect_cb060.rs");

// Unit tests
include!("comply_cb_detect_tests.rs");
