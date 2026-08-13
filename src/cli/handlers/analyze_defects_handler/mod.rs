#![cfg_attr(coverage_nightly, coverage(off))]
//! CLI handler for `pmat analyze defects` command
//!
//! Scans projects for known defect patterns with text, JSON, and JUnit output formats
//! for CI/CD integration. Based on docs/issues/analyze-defects-command.md

pub mod handler;
pub mod output;
pub mod types;

// Re-export public API
pub use handler::handle_analyze_defects;
pub use types::{DefectReport, DefectSummary, OutputFormat, SeverityCount};

#[cfg(test)]
mod tests_unit;

#[cfg(test)]
mod tests_output;

#[cfg(test)]
mod tests_integration;

#[cfg(test)]
mod tests_nothing_measured;
