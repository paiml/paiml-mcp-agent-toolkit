#![cfg_attr(coverage_nightly, coverage(off))]
//! Toyota Way: Quality Gate Formatting Handler
//! Complexity: Reduced from 20 to individual functions ≤8
//! Purpose: Quality gate report formatting with clean separation of concerns
//!
//! Split into include files for maintainability:
//! - quality_gate_format_single_file.rs: Single-file formatting functions
//! - quality_gate_format_project.rs: Project-wide formatting and JUnit output
//! - quality_gate_check_runner.rs: Check execution and orchestration
//! - quality_gate_formatter_tests.rs: All test modules

use crate::cli::analysis_utilities::{QualityGateResults, QualityViolation};
use crate::cli::{QualityCheckType, QualityGateOutputFormat};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

// --- Single-file formatting: format_single_file_output, format_single_file_summary,
//     add_summary_section, add_violations_section, add_violation_entry, get_severity_icon
include!("quality_gate_format_single_file.rs");

// --- Project-wide formatting: format_qg_as_junit, write_junit_test_case,
//     format_project_output, format_project_summary, group_violations_by_type,
//     format_violations_markdown
include!("quality_gate_format_project.rs");

// --- Check execution: print_checks_to_run, QualityCheckConfig, run_project_checks,
//     run_all_checks, IndividualChecksConfig, run_individual_checks, print_check_timing
include!("quality_gate_check_runner.rs");

// --- Tests
include!("quality_gate_formatter_tests.rs");
