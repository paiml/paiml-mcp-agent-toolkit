#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper functions for defect prediction analysis
//!
//! This module provides file discovery, probability analysis, and multiple
//! output format functions (JSON, summary, markdown, SARIF) for defect
//! prediction results.

mod analysis;
mod format_json;
mod format_markdown;
mod format_sarif;

pub use analysis::{analyze_defect_probability, discover_files_for_defect_analysis};
pub use format_json::format_defect_json;
pub use format_markdown::{format_defect_markdown, format_defect_summary};
pub use format_sarif::format_defect_sarif;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_property;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_format_json;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_format_summary;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_format_markdown;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_format_sarif;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_helpers;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_integration;

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_async;
