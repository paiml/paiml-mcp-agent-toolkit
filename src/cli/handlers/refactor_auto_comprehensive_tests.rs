//! Comprehensive coverage tests for refactor_auto_handlers.
//!
//! This is the body of `mod comprehensive_coverage_tests`, declared with
//! `#[path]` from refactor_auto_handlers_tests.rs. Two mechanical extractions
//! (CB-040 then PMAT-503) each dropped a line and left the module wrapper split
//! across files: the closing brace of the original inline `mod` stayed here with
//! no opener, and the closing brace of the last test in the setup_context chunk
//! was dropped entirely. Both are restored (#1023); the concatenation of the five
//! include!s below is byte-identical to c8dd80a8e:refactor_auto_handlers_tests.rs
//! lines 1282-2928. Keep each include! file independently balanced.

use super::super::*;
// Moved out of refactor_auto_handlers into refactor_auto_types by CB-040
// (1008e33ec), which did not import them back; `use super::super::*` has not
// reached them since.
use crate::cli::handlers::refactor_auto_types::{
    analyze_markdown_issues, create_markdown_refactor_request, has_broken_relative_links,
    print_markdown_summary,
};
use std::path::PathBuf;
use tempfile::TempDir;

// Setup and context creation tests
include!("refactor_auto_comprehensive_tests_setup_context.rs");

// Project quality analysis and refactoring request generation tests
include!("refactor_auto_comprehensive_tests_analysis_generation.rs");

// Request creation tests (lint, SATD, coverage, quality improvement)
include!("refactor_auto_comprehensive_tests_request_creation.rs");

// Apply refactoring functions, helper functions, markdown, and output tests
include!("refactor_auto_comprehensive_tests_apply_helpers_output.rs");

// Special modes, validation, type construction, and serialization tests
include!("refactor_auto_comprehensive_tests_modes_validation_types.rs");
