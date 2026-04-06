//! Kaizen scanning: opportunity detection from clippy, rustfmt, comply, defects, GitHub issues.

use super::{FindingSeverity, FindingSource, KaizenConfig, KaizenFinding};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Scan a single crate for all finding types, tagging each with crate_name.
/// Lint scans (clippy/fmt) run first sequentially (they need cargo lock),
/// then analysis scans (comply/defects/github/custom) run in parallel threads.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn scan_crate(
    path: &Path,
    crate_name: Option<&str>,
    config: &KaizenConfig,
) -> Result<Vec<KaizenFinding>> {
    let mut findings = Vec::new();

    // Phase 1: Lint scans (sequential — both use cargo)
    if !config.skip_clippy {
        findings.extend(scan_clippy(path)?);
    }
    if !config.skip_fmt {
        findings.extend(scan_rustfmt(path)?);
    }

    // Phase 2: Analysis scans (parallel — independent subprocesses)
    let path_buf = path.to_path_buf();
    let skip_comply = config.skip_comply;
    let skip_defects = config.skip_defects;
    let skip_github = config.skip_github;

    let p1 = path_buf.clone();
    let p2 = path_buf.clone();
    let p3 = path_buf.clone();
    let p4 = path_buf.clone();

    let comply_handle = std::thread::spawn(move || {
        if skip_comply {
            Ok(Vec::new())
        } else {
            scan_comply(&p1)
        }
    });
    let defects_handle = std::thread::spawn(move || {
        if skip_defects {
            Ok(Vec::new())
        } else {
            scan_defects(&p2)
        }
    });
    let github_handle = std::thread::spawn(move || {
        if skip_github {
            Ok(Vec::new())
        } else {
            scan_github_issues(&p3)
        }
    });
    let custom_handle = std::thread::spawn(move || Ok::<_, anyhow::Error>(scan_custom_scores(&p4)));

    // Collect parallel results
    if let Ok(Ok(r)) = comply_handle.join() {
        findings.extend(r);
    }
    if let Ok(Ok(r)) = defects_handle.join() {
        findings.extend(r);
    }
    if let Ok(Ok(r)) = github_handle.join() {
        findings.extend(r);
    }
    if let Ok(Ok(r)) = custom_handle.join() {
        findings.extend(r);
    }

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

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn extract_file_from_message(message: &serde_json::Value) -> Option<String> {
    message
        .get("spans")
        .and_then(|s| s.as_array())
        .and_then(|spans| spans.first())
        .and_then(|span| span.get("file_name"))
        .and_then(|f| f.as_str())
        .map(String::from)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or(s).to_string()
}

/// Extract a numeric score from command output (JSON {"score": N} or "SCORE: N")
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
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
