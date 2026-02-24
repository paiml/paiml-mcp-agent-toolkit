#![cfg_attr(coverage_nightly, coverage(off))]
//! Fuzzing Integration for Mutation Testing - Phase 4.1
//!
//! EXTREME TDD: GREEN PHASE - Minimal implementation to pass RED tests

use super::{CoverageCorpus, CoverageInfo, CoverageTracker, Mutant, MutationEngine};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Type definitions: FuzzConfig, InputGeneratorType, FuzzResult, FuzzMutationReport
include!("fuzzing_types.rs");

// FuzzMutationStrategy struct and implementation
include!("fuzzing_strategy.rs");

// Free functions: execute_mutant_with_input, mutate_input
include!("fuzzing_helpers.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    include!("fuzzing_tests.rs");
    include!("fuzzing_tests_strategy.rs");
}
