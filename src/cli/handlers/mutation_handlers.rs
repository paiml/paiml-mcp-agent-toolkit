#![cfg_attr(coverage_nightly, coverage(off))]
//! Mutation testing handlers
//!
//! Handles the `pmat analyze mutate` command for mutation testing with ML prediction.

use crate::cli::OutputFormat;
use crate::services::mutation::{
    MutantExecutor, MutationConfig, MutationEngine, MutationScore, RustAdapter,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

/// Configuration for mutation testing
#[derive(Debug, Clone)]
pub struct MutationTestConfig {
    pub operators: Option<Vec<String>>,
    pub ml_predict: bool,
    pub distributed: bool,
    pub workers: usize,
    pub progress: bool,
    pub min_score: Option<f64>,
    pub ci_learning: bool,
    pub ci_provider: Option<String>,
    pub auto_train_threshold: usize,
}

impl MutationTestConfig {
    /// Create config from individual parameters
    #[allow(clippy::too_many_arguments)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(
        operators: Option<Vec<String>>,
        ml_predict: bool,
        distributed: bool,
        workers: usize,
        progress: bool,
        min_score: Option<f64>,
        ci_learning: bool,
        ci_provider: Option<String>,
        auto_train_threshold: usize,
    ) -> Self {
        Self {
            operators,
            ml_predict,
            distributed,
            workers,
            progress,
            min_score,
            ci_learning,
            ci_provider,
            auto_train_threshold,
        }
    }
}

// --- Handler core: handle_mutate + helper functions ---
include!("mutation_handlers_core.rs");

// --- Tests ---
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("mutation_handlers_tests.rs");
}
