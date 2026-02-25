//! cargo-mutants Backend Handler (Sprint 70 - Phase 3 GREEN/REFACTOR)
//!
//! Executes cargo-mutants and parses results into PMAT format.
//! This module bridges cargo-mutants JSON output with PMAT's mutation testing types.
//!
//! ## Module Organization
//!
//! Split into submodules via include!():
//! - `cargo_mutants_backend_runner.rs` — execute() function (command building + execution)
//! - `cargo_mutants_backend_display.rs` — display_statistics() function (output formatting)
//! - `cargo_mutants_backend_tests.rs` — all tests

use crate::services::mutation::cargo_mutants_wrapper::CargoMutantsWrapper;
use crate::services::mutation::json_parser::{CargoMutantsReport, MutantOutcome};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Configuration for cargo-mutants execution
#[derive(Debug, Clone)]
pub struct CargoMutantsConfig {
    pub path: PathBuf,
    pub output: Option<PathBuf>,
    pub timeout: u64,
    pub jobs: Option<usize>,
    pub features: Option<Vec<String>>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub no_shuffle: bool,
}

// Execute cargo-mutants and return path to output directory
include!("cargo_mutants_backend_runner.rs");

// Display mutation testing statistics
include!("cargo_mutants_backend_display.rs");

// Tests
include!("cargo_mutants_backend_tests.rs");
