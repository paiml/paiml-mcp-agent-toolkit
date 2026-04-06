#![cfg_attr(coverage_nightly, coverage(off))]
//! Helper functions for provability analysis to reduce complexity

use crate::services::lightweight_provability_analyzer::{FunctionId, ProofSummary};
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

/// Parse function specification string into `FunctionId`
pub fn parse_function_spec(spec: &str, project_path: &Path) -> Result<FunctionId> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    // Parse function specification in format: path/to/file.rs:function_name
    // or just function_name (search all files)
    if let Some((file_part, func_part)) = spec.split_once(':') {
        Ok(FunctionId {
            file_path: project_path.join(file_part).to_string_lossy().to_string(),
            function_name: func_part.to_string(),
            line_number: 0, // Will be populated by analyzer
        })
    } else {
        // Just function name - will search all files
        Ok(FunctionId {
            file_path: String::new(),
            function_name: spec.to_string(),
            line_number: 0,
        })
    }
}

/// Extract function name from a line
#[allow(dead_code)]
fn extract_function_name(line: &str) -> Option<String> {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let line = line.trim();
    let start = line.find("fn ")? + 3;
    let end = line.get(start..)?.find(['(', '<'])?;
    Some(
        line.get(start..start + end)
            .unwrap_or_default()
            .trim()
            .to_string(),
    )
}

/// Filter function summaries based on confidence
#[must_use]
pub fn filter_summaries(
    summaries: &[ProofSummary],
    high_confidence_only: bool,
) -> Vec<&ProofSummary> {
    summaries
        .iter()
        .filter(|s| !high_confidence_only || s.provability_score >= 0.8)
        .collect()
}

fn categorize_scores(summaries: &[ProofSummary]) -> (usize, usize, usize) {
    let high_provability = summaries
        .iter()
        .filter(|s| s.provability_score >= 0.8)
        .count();
    let medium_provability = summaries
        .iter()
        .filter(|s| s.provability_score >= 0.5 && s.provability_score < 0.8)
        .count();
    let low_provability = summaries
        .iter()
        .filter(|s| s.provability_score < 0.5)
        .count();

    (high_provability, medium_provability, low_provability)
}

fn calculate_average_score(summaries: &[ProofSummary]) -> f64 {
    if summaries.is_empty() {
        0.0
    } else {
        summaries.iter().map(|s| s.provability_score).sum::<f64>() / summaries.len() as f64
    }
}

fn extract_filename(file_path: &str) -> &str {
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
    std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path)
}

// --- Include split-out implementation files ---

include!("provability_helpers_discovery.rs");
include!("provability_helpers_json.rs");
include!("provability_helpers_summary.rs");
include!("provability_helpers_detailed.rs");
include!("provability_helpers_sarif.rs");
include!("provability_helpers_tests.rs");
