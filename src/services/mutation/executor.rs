#![cfg_attr(coverage_nightly, coverage(off))]
//! Mutant test execution
//!
//! Executes tests on mutated code to empirically measure mutation score.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::time::timeout;

use super::types::{Mutant, MutantStatus, MutationResult};

/// Default timeout for test execution (10 minutes)
const DEFAULT_TIMEOUT_SECS: u64 = 600;

/// Executes tests on mutants
#[derive(Clone)]
pub struct MutantExecutor {
    /// Timeout for test execution
    timeout: Duration,

    /// Working directory for test execution
    work_dir: PathBuf,
}

impl MutantExecutor {
    /// Create new executor with default settings
    pub fn new(work_dir: PathBuf) -> Self {
        debug_assert!(
            work_dir.exists(),
            "work_dir must exist: {}",
            work_dir.display()
        );
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            work_dir,
        }
    }

    /// Create executor with custom timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout = Duration::from_secs(timeout_secs);
        self
    }
}

// Single and sequential execution
include!("executor_single.rs");

// Parallel execution
include!("executor_parallel.rs");

// Resumable execution with signal handling
include!("executor_resumable.rs");

// Helper methods: backup, test execution, atomic write, parsing
include!("executor_helpers.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("executor_tests.rs");
}
