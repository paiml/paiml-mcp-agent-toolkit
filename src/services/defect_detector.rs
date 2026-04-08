#![cfg_attr(coverage_nightly, coverage(off))]
//! Shared Known Defects Detection Module
//!
//! Provides defect detection capabilities for:
//! - rust-project-score (KnownDefectsScorer)
//! - TDG analyzer (auto-fail on critical defects)
//! - analyze defects command (project-wide scanning)
//!
//! Based on specification: docs/specifications/components/language-support.md

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Defect severity levels (based on production impact)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical, // Auto-fail in TDG, exit code 1 in analyze
    High,
    Medium,
    Low,
}

impl Severity {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// As str.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
}

/// A detected defect instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectInstance {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub code_snippet: String,
}

/// A known defect pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectPattern {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub fix_recommendation: String,
    pub bad_example: String,
    pub good_example: String,
    pub evidence_description: String,
    pub evidence_url: Option<String>,
    pub instances: Vec<DefectInstance>,
}

/// Defect detector for Rust code
pub struct RustDefectDetector {
    unwrap_regex: Regex,
}

/// Defect detector for Lua code
pub struct LuaDefectDetector {
    global_assign_re: Regex,
    nil_chain_re: Regex,
    unchecked_pcall_re: Regex,
    dangerous_api_re: Regex,
}

// --- Implementation split into include files ---
include!("defect_detector_rust.rs");
include!("defect_detector_lua.rs");
include!("defect_detector_tests.rs");
