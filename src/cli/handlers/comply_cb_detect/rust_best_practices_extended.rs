#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-500 Series: Rust Best Practices Detection (Extended)
//!
//! CB-522 through CB-530 detectors, split from rust_best_practices.rs
//! for file health compliance (CB-040).

use super::types::*;
use std::fs;
use std::path::Path;

// CB-522 through CB-527: Pattern-based best practice detectors
// (untested path normalization, external config heuristics, incomplete enum matches,
// hardcoded field names, single-path resolution, incomplete classification chains)
include!("rust_best_practices_extended_checks.rs");

// CB-528 and CB-530: Numeric safety detectors
// (division-by-length without empty guards, log calls without clamp/max guards)
include!("rust_best_practices_extended_numeric.rs");
