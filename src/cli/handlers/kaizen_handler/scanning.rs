//! Kaizen scanning: opportunity detection from clippy, rustfmt, comply, defects, GitHub issues.

use super::{FindingSeverity, FindingSource, KaizenConfig, KaizenFinding};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Scan a single crate for all finding types, tagging each with crate_name
pub(crate) fn scan_crate(
    path: &Path,
    crate_name: Option<&str>,
    config: &KaizenConfig,
) -> Result<Vec<KaizenFinding>> {
    let mut findings = Vec::new();

    if !config.skip_clippy {
        findings.extend(scan_clippy(path)?);
    }
    if !config.skip_fmt {
        findings.extend(scan_rustfmt(path)?);
    }
    if !config.skip_comply {
        findings.extend(scan_comply(path)?);
    }
    if !config.skip_defects {
        findings.extend(scan_defects(path)?);
    }
    if !config.skip_github {
        findings.extend(scan_github_issues(path)?);
    }
    findings.extend(scan_custom_scores(path));

    // Tag with crate name
    if let Some(name) = crate_name {
        for f in &mut findings {
            f.crate_name = Some(name.to_string());
        }
    }

    Ok(findings)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub(crate) fn extract_file_from_message(message: &serde_json::Value) -> Option<String> {
    message
        .get("spans")
        .and_then(|s| s.as_array())
        .and_then(|spans| spans.first())
        .and_then(|span| span.get("file_name"))
        .and_then(|f| f.as_str())
        .map(String::from)
}

pub(crate) fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or(s).to_string()
}

/// Extract a numeric score from command output (JSON {"score": N} or "SCORE: N")
pub(crate) fn extract_score_from_command_output(output: &str) -> Option<f64> {
    for line in output.lines() {
        let line = line.trim();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(score) = json.get("score").and_then(|s| s.as_f64()) {
                return Some(score);
            }
        }
    }
    for line in output.lines() {
        if let Some(rest) = line.trim().strip_prefix("SCORE:") {
            if let Ok(score) = rest.trim().parse::<f64>() {
                return Some(score);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Submodule includes
// ---------------------------------------------------------------------------

include!("scanning_lint.rs");
include!("scanning_analysis.rs");
include!("scanning_tests.rs");
