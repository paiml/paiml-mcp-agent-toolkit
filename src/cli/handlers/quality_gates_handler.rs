#![cfg_attr(coverage_nightly, coverage(off))]
//! Quality gates command handler for TICKET-PMAT-5023 and TICKET-PMAT-5024
//!
//! Executes quality gates using the gate executor from TICKET-PMAT-5020.
//!
//! ## Module Structure
//! - `quality_gates_handler_execution.rs`: Gate execution and config management
//! - `quality_gates_handler_output.rs`: Output formatting (JSON, markdown, summary)
//! - `quality_gates_handler_tests.rs`: Unit tests

use crate::cli::commands::{ConfigFormat, QualityGatesCommand};
use crate::quality::gates::{execute_all_gates, format_report, GateConfig, QualityReport};
use crate::quality::{generate_config_toml, generate_default_config, validate_config};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// TOML configuration structure
#[derive(Debug, serde::Deserialize)]
struct GateConfigToml {
    gates: GateConfigInner,
}

/// Inner gate configuration
#[derive(Debug, serde::Deserialize)]
struct GateConfigInner {
    run_clippy: bool,
    clippy_strict: bool,
    run_tests: bool,
    test_timeout: u64,
    check_coverage: bool,
    min_coverage: f64,
    check_complexity: bool,
    max_complexity: u32,
}

impl From<GateConfigToml> for GateConfig {
    fn from(toml: GateConfigToml) -> Self {
        let g = toml.gates;
        GateConfig {
            run_clippy: g.run_clippy,
            clippy_strict: g.clippy_strict,
            run_tests: g.run_tests,
            test_timeout: g.test_timeout,
            check_coverage: g.check_coverage,
            min_coverage: g.min_coverage,
            check_complexity: g.check_complexity,
            max_complexity: g.max_complexity,
        }
    }
}

// Gate execution and config management functions
include!("quality_gates_handler_execution.rs");

// Output formatting functions (JSON, markdown, summary)
include!("quality_gates_handler_output.rs");

// Unit tests
include!("quality_gates_handler_tests.rs");
