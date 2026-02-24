#![cfg_attr(coverage_nightly, coverage(off))]
// Deep context annotation with graph metrics
// Sprint 3: Integration with graph analysis
// Complexity: All functions <= 8

use super::*;
use std::collections::HashMap;

// --- Type definitions ---
include!("context_annotator_types.rs");

// --- Core implementation ---
include!("context_annotator_core.rs");

// --- Tests ---
include!("context_annotator_tests.rs");
