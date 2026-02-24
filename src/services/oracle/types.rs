#![cfg_attr(coverage_nightly, coverage(off))]
//! Core types for PMAT Oracle
//!
//! Unified Defect Schema (UDS) and supporting types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

// --- Defect categories (OIP CITL mappings) ---
include!("types_defect_category.rs");

// --- Core enums: Severity, SignalSource, evidence, location, fix types ---
include!("types_core_enums.rs");

// --- DefectReport: unified defect report with confidence calculation ---
include!("types_defect_report.rs");

// --- Convergence: targets, project metrics, status checking ---
include!("types_convergence.rs");

// --- Oracle configuration ---
include!("types_oracle_config.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // --- Tests for DefectCategory, Severity, SignalSource, evidence, location, fix types ---
    include!("types_tests_core.rs");

    // --- Tests for DefectReport, ConvergenceTargets, OracleConfig, ProjectMetrics, integration ---
    include!("types_tests_report_convergence.rs");
}
