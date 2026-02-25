#![cfg_attr(coverage_nightly, coverage(off))]
//! Equivalent Mutant Detector - Phase 4.2
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass RED tests

use super::Mutant;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

include!("equivalent_detector_types.rs");
include!("equivalent_detector_core.rs");
include!("equivalent_detector_patterns.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mutation::types::{MutationOperator, SourceLocation};
    use tempfile::TempDir;

    include!("equivalent_detector_verification.rs");
    include!("equivalent_detector_integration.rs");
}
