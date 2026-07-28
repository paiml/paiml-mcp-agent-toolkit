#![cfg_attr(coverage_nightly, coverage(off))]
//! Toyota Way: Extracted Provability Analysis Handler
//! Complexity: Reduced from 19 to individual functions ≤8
//! Purpose: Function formal provability analysis with confidence scoring
//!
//! Split into submodules for file health compliance (CB-040):
//! - provability_handler_core.rs: Config struct + handler functions
//! - provability_handler_tests.rs: Unit tests (config, prepare, format, write, resolve, run)
//! - provability_handler_tests_part2.rs: Integration tests, all formats, edge cases
//! - provability_handler_tests_part3.rs: Multiple files, detailed evidence, score distribution

use crate::cli::enums::ProvabilityOutputFormat;
use crate::services::lightweight_provability_analyzer::ProofSummary;
use anyhow::Result;
use std::path::PathBuf;

include!("provability_handler_core.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::services::lightweight_provability_analyzer::{
        FunctionId, ProofSummary, PropertyType, VerifiedProperty,
    };
    use std::path::PathBuf;
    use tempfile::TempDir;

    include!("provability_handler_tests.rs");
    include!("provability_handler_tests_part2.rs");
    include!("provability_handler_tests_part3.rs");
}
