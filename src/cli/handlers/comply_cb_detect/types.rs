#![cfg_attr(coverage_nightly, coverage(off))]
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub severity: Severity,
}

/// Status of a compliance check
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

/// Severity level for compliance issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// ComputeBrick pattern detection result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

pub struct CbPatternViolation {
    pub pattern_id: String,
    pub file: String,
    pub line: usize,
    pub description: String,
    pub severity: Severity,
}

/// BrickProfiler anomaly from JSON output
#[derive(Debug, Clone)]
pub struct ProfilerAnomaly {
    pub brick_name: String,
    pub anomaly_type: String,
    pub value: f64,
    pub threshold: f64,
}

/// Compute line ranges that are inside test code (#[cfg(test)] mod tests { ... })
/// Returns a HashSet of line indices that should be skipped for production code analysis.
/// Mark all lines in a brace-delimited block starting at `start`.
pub(super) fn mark_braced_block(
    lines: &[&str],
    start: usize,
    result: &mut std::collections::HashSet<usize>,
) {
    let mut depth = 0;
    for k in start..lines.len() {
        depth += lines[k].matches('{').count();
        depth = depth.saturating_sub(lines[k].matches('}').count());
        result.insert(k);
        if depth == 0 && k > start {
            break;
        }
    }
}

/// Find the next line containing `needle` within `start..start+window`, return its index.
pub(super) fn find_line_within(
    lines: &[&str],
    start: usize,
    window: usize,
    needle: &str,
) -> Option<usize> {
    let end = std::cmp::min(start + window, lines.len());
    (start..end).find(|&j| lines[j].contains(needle))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
/// Compute test code lines.
pub fn compute_test_code_lines(lines: &[&str]) -> std::collections::HashSet<usize> {
    let mut test_lines = std::collections::HashSet::new();

    for i in 0..lines.len() {
        let line = lines[i].trim();

        // Detect #[cfg(test)] followed by mod (within next 3 lines)
        if line.starts_with("#[cfg(test)]") {
            if let Some(j) = find_line_within(lines, i, 4, "mod ") {
                mark_braced_block(lines, j, &mut test_lines);
                test_lines.insert(i);
            }
        }

        // Detect standalone `mod tests {` without #[cfg(test)]
        if (line.starts_with("mod tests") || line.starts_with("pub mod tests"))
            && line.contains('{')
        {
            mark_braced_block(lines, i, &mut test_lines);
        }

        // Detect #[test] and #[tokio::test] functions
        if line.starts_with("#[test]")
            || line.starts_with("#[tokio::test]")
            || line.starts_with("#[tokio::test]")
            || line.starts_with("#[async_std::test]")
        {
            test_lines.insert(i);
            if let Some(j) = find_line_within(lines, i + 1, 4, "fn ") {
                mark_braced_block(lines, j, &mut test_lines);
            }
        }
    }

    test_lines
}

/// Scan Rust files for CB-020 (unsafe without SAFETY comment)
/// NOTE: Skips test code (#[cfg(test)], mod tests, #[test]) - test code can use .unwrap() freely
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn detect_cb020_unsafe_without_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .iter()
        .flat_map(|entry| scan_file_for_unsafe_violations(entry))
        .collect()
}

fn scan_file_for_unsafe_violations(path: &Path) -> Vec<CbPatternViolation> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let test_lines = compute_test_code_lines(&lines);
    let file = path.display().to_string();

    lines
        .iter()
        .enumerate()
        .filter(|(i, _)| !test_lines.contains(i))
        .filter(|(_, line)| {
            let t = line.trim();
            t.starts_with("unsafe {") || t.starts_with("unsafe{")
        })
        .filter(|(i, _)| !has_preceding_safety_comment(&lines, *i))
        .map(|(i, _)| CbPatternViolation {
            pattern_id: "CB-020".to_string(),
            file: file.clone(),
            line: i + 1,
            description: "unsafe block without SAFETY comment".to_string(),
            severity: Severity::Warning,
        })
        .collect()
}

fn has_preceding_safety_comment(lines: &[&str], line_num: usize) -> bool {
    lines
        .iter()
        .take(line_num)
        .rev()
        .take(10)
        .any(|l| l.contains("// SAFETY:") || l.contains("// SAFETY :") || l.contains("/ SAFETY:"))
}

/// Helper to walk directory for .rs files
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn walkdir_rs_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir_rs_files(&path)?);
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            files.push(path);
        }
    }
    Ok(files)
}

/// Check if a file is entirely test code based on naming conventions.
/// Matches: *_tests.rs, *_test.rs, tests.rs, tests_*.rs, *_tests_*.rs (include!() fragments)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn is_test_file(path: &Path) -> bool {
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    name.ends_with("_tests")
        || name.ends_with("_test")
        || name == "tests"
        || name.starts_with("tests_")
        || name.contains("_tests_")   // include!() test fragments like executor_tests_basic
        || name.starts_with("test_")
        || path.components().any(|c| c.as_os_str() == "tests")
}
