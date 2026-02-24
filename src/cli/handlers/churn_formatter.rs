#![cfg_attr(coverage_nightly, coverage(off))]
//! Toyota Way: Churn Analysis Formatting Handler
//! Complexity: Reduced from 17 to individual functions <=8
//! Purpose: Churn report formatting with clean separation of concerns

use anyhow::Result;
use std::fmt::Write;
use std::path::Path;

// Markdown formatting: format_churn_markdown, summary table, file details, author contributions
include!("churn_formatter_markdown.rs");

// Path detection: is_source_file, has_source_extension, is_test_path, is_test_filename
include!("churn_formatter_path_detection.rs");

#[cfg(test)]
include!("churn_formatter_tests.rs");

#[cfg(test)]
include!("churn_formatter_tests_path.rs");

#[cfg(test)]
include!("churn_formatter_property_tests.rs");
