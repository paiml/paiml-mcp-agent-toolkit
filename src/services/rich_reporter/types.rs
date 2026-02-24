#![cfg_attr(coverage_nightly, coverage(off))]
//! Core types for PMAT-REPORT-V1 Universal Rich Reporting
//!
//! Implements the Unified Finding and Report structures per specification.
//! Toyota Way: Mieruka (Visual Management) - all findings include visual indicators

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Severity, AndonStatus, TrendDirection enums with impls
include!("types_enums.rs");

// SourceLocation, FixSuggestion, Finding, FindingCluster,
// CodeCommunity, AnomalyPoint, MetricTrend structs
include!("types_data_structs.rs");

// OutputFormat, ColorMode, ReportConfig
include!("types_config.rs");

// RichReport struct and impl
include!("types_rich_report.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    include!("types_tests.rs");
}
