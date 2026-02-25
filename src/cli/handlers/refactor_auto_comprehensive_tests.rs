//! Comprehensive coverage tests for refactor_auto_handlers
//! Extracted for file health compliance (CB-040)

use super::super::*;
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
}
