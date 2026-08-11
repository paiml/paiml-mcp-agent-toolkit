#![cfg_attr(coverage_nightly, coverage(off))]
//! Automated Clippy Fix Engine
//!
//! A+ Code Standard: ALL functions d10 complexity
//! Single Responsibility: Each function does ONE thing
//! TDD: Tests written first, implementation follows

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Clippy diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClippyDiagnostic {
    pub code: String,
    pub level: DiagnosticLevel,
    pub message: String,
    pub file: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub suggestion: Option<String>,
}

impl ClippyDiagnostic {
    /// Parse from JSON output (complexity: 3)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_json(json: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        Self::parse_json_value(&value)
    }

    /// Extract diagnostic from JSON value (complexity: 5)
    ///
    /// Cargo's `--message-format=json` stream interleaves compiler-artifact,
    /// build-finished and "generated N warnings" summary lines with the real
    /// diagnostics. This used to `unwrap_or` every field, so each of those
    /// lines materialised as a fix with file:"" line:0 code:"unknown" and
    /// inflated `total_fixes`. A line that carries neither a lint code nor a
    /// source span is not a diagnostic we can fix, so reject it outright.
    fn parse_json_value(value: &serde_json::Value) -> Result<Self> {
        if let Some(reason) = value.get("reason").and_then(serde_json::Value::as_str) {
            if reason != "compiler-message" {
                return Err(anyhow::anyhow!("not a compiler-message line: {reason}"));
            }
        }

        let message = &value["message"];
        let code = message["code"]["code"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("diagnostic has no lint code"))?;
        let level = message["level"].as_str().unwrap_or("warning");
        let spans = &message["spans"][0];
        if !spans["file_name"].is_string() {
            return Err(anyhow::anyhow!("diagnostic has no primary source span"));
        }

        Ok(Self {
            code: code.to_string(),
            level: DiagnosticLevel::from_str(level),
            message: message["message"].as_str().unwrap_or("").to_string(),
            file: PathBuf::from(spans["file_name"].as_str().unwrap_or("")),
            line_start: spans["line_start"].as_u64().unwrap_or(0) as usize,
            line_end: spans["line_end"].as_u64().unwrap_or(0) as usize,
            column_start: spans["column_start"].as_u64().unwrap_or(0) as usize,
            column_end: spans["column_end"].as_u64().unwrap_or(0) as usize,
            suggestion: None,
        })
    }
}

/// Diagnostic severity level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

impl DiagnosticLevel {
    /// Parse from string (complexity: 2)
    fn from_str(s: &str) -> Self {
        match s {
            "error" => Self::Error,
            "warning" => Self::Warning,
            "note" => Self::Note,
            _ => Self::Help,
        }
    }
}

/// Confidence level for automated fixes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    High,   // Safe to auto-apply
    Medium, // Probably safe
    Low,    // Requires review
}

/// Result of applying a fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    pub success: bool,
    pub diagnostic: ClippyDiagnostic,
    pub modified_source: String,
    pub confidence: ConfidenceLevel,
    pub duration: Duration,
    pub error: Option<String>,
}

/// Fix report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixReport {
    pub total_diagnostics: usize,
    pub successful_fixes: usize,
    pub failed_fixes: usize,
    pub skipped_low_confidence: usize,
    pub success_rate: f64,
    pub total_duration: Duration,
    pub fixed_files: Vec<PathBuf>,
}

/// Main clippy fix engine
pub struct ClippyFixEngine {
    cache: HashMap<String, FixResult>,
    confidence_rules: HashMap<String, ConfidenceLevel>,
}

impl Default for ClippyFixEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Engine implementation: confidence rules, fix application, validation, reporting
include!("clippy_fix_engine.rs");

// Tests: property tests and coverage tests
include!("clippy_fix_tests.rs");
