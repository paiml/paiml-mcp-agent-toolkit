#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI Documentation Checker
//!
//! TICKET: PMAT-7001 Phase 2 (GREEN)
//!
//! This module validates that CLI commands have complete, accurate help text.
//! It checks that all flags are documented and descriptions are non-generic.
//!
//! ## Module Structure
//! - `cli_checker_validation.rs` — Core validation logic (validate, execute, sections)
//! - `cli_checker_parsing.rs` — Flag extraction and parsing helpers
//! - `cli_checker_tests.rs` — Unit tests

use crate::docs_enforcement::generic_detector::is_generic_description;
use anyhow::Result;
use std::collections::HashSet;
use std::process::Command;

/// CLI documentation validation result
#[derive(Debug, Clone)]
pub struct CliDocumentationReport {
    pub command: String,
    pub has_help: bool,
    pub has_usage_section: bool,
    pub has_options_section: bool,
    pub has_examples_section: bool,
    pub documented_flags: Vec<String>,
    pub generic_descriptions: Vec<String>,
    pub missing_descriptions: Vec<String>,
    pub issues: Vec<String>,
}

impl CliDocumentationReport {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_valid(&self) -> bool {
        self.has_help
            && self.has_usage_section
            && self.has_options_section
            && self.generic_descriptions.is_empty()
            && self.missing_descriptions.is_empty()
    }
}

// Core validation logic: validate_cli_documentation, execute_help_command,
// validate_sections, find_generic_flag_descriptions, extract_description_from_flag_line
include!("cli_checker_validation.rs");

// Flag extraction and parsing: extract_flags_from_help, extract_options_section_lines,
// is_options_section_start, is_section_boundary, parse_flags_from_line,
// extract_flag_name, find_undocumented_flags
include!("cli_checker_parsing.rs");

// Unit tests
include!("cli_checker_tests.rs");
