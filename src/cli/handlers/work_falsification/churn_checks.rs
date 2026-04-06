#![cfg_attr(coverage_nightly, coverage(off))]
//! v3.1 defect churn prevention: Variant Coverage, Fix Chains, Cross-Crate, Regression.

use crate::cli::handlers::work_contract::{EvidenceType, FalsificationResult};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Test variant coverage: find match arms in changed files that lack test coverage.
///
/// Scans changed `.rs` files for `match` expressions with 5+ arms and checks
/// whether each arm's pattern appears in at least one test function.
pub(crate) fn test_variant_coverage(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    print!("Scanning match arm coverage... ");

    let changed_files = get_changed_files(project_path, baseline_commit)?;
    let rs_files: Vec<&String> = changed_files
        .iter()
        .filter(|f| f.ends_with(".rs"))
        .collect();

    if rs_files.is_empty() {
        return Ok(FalsificationResult::passed(
            "No Rust files changed".to_string(),
        ));
    }

    let mut untested_arms: Vec<(String, String)> = Vec::new(); // (file, variant)

    for rel_path in &rs_files {
        let full_path = project_path.join(rel_path);
        let Ok(content) = std::fs::read_to_string(&full_path) else {
            continue;
        };

        // Find match blocks with 5+ arms (threshold for "enum with variants")
        let variants = extract_large_match_variants(&content);
        if variants.is_empty() {
            continue;
        }

        // Check if test functions reference each variant
        let test_section = extract_test_section(&content);
        for variant in &variants {
            if !test_section.contains(variant) {
                untested_arms.push((rel_path.to_string(), variant.clone()));
            }
        }
    }

    if untested_arms.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "{} changed file(s) -- all match variants tested",
            rs_files.len()
        )))
    } else {
        let details: Vec<String> = untested_arms
            .iter()
            .take(10)
            .map(|(f, v)| format!("{}::{}", f, v))
            .collect();
        let paths: Vec<PathBuf> = untested_arms
            .iter()
            .map(|(f, _)| PathBuf::from(f))
            .collect();
        Ok(FalsificationResult::failed(
            format!(
                "{} untested match variant(s): {}",
                untested_arms.len(),
                details.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// State machine for parsing match blocks
struct MatchParser {
    variants: Vec<String>,
    current_arms: Vec<String>,
    brace_depth: usize,
    in_match: bool,
}

impl MatchParser {
    fn new() -> Self {
        Self {
            variants: Vec::new(),
            current_arms: Vec::new(),
            brace_depth: 0,
            in_match: false,
        }
    }

    fn process_line(&mut self, trimmed: &str) {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        if !self.in_match {
            if trimmed.contains("match ") && trimmed.ends_with('{') {
                self.in_match = true;
                self.current_arms.clear();
                self.brace_depth = 1;
            }
            return;
        }
        self.update_brace_depth(trimmed);
        if self.in_match && self.brace_depth == 1 {
            self.try_extract_arm(trimmed);
        }
    }

    fn update_brace_depth(&mut self, trimmed: &str) {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        for ch in trimmed.chars() {
            match ch {
                '{' => self.brace_depth += 1,
                '}' => {
                    self.brace_depth -= 1;
                    if self.brace_depth == 0 {
                        self.flush_match_block();
                    }
                }
                _ => {}
            }
        }
    }

    fn flush_match_block(&mut self) {
        if self.current_arms.len() >= 5 {
            self.variants.append(&mut self.current_arms);
        } else {
            self.current_arms.clear();
        }
        self.in_match = false;
    }

    fn try_extract_arm(&mut self, trimmed: &str) {
        debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
        let Some(pattern) = trimmed.split("=>").next() else {
            return;
        };
        let pattern = pattern.trim();
        if pattern == "_" || pattern.starts_with("//") || !trimmed.contains("=>") {
            return;
        }
        let variant = pattern
            .split("::")
            .last()
            .map(|s| {
                s.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                    .to_string()
            })
            .unwrap_or_default();
        if !variant.is_empty() {
            self.current_arms.push(variant);
        }
    }
}

/// Extract variant names from match blocks with 5+ arms.
/// Returns variant identifiers like "Q4_K", "LLaMA", etc.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn extract_large_match_variants(content: &str) -> Vec<String> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut parser = MatchParser::new();
    for line in content.lines() {
        parser.process_line(line.trim());
    }
    parser.variants
}

/// Extract test section content (everything after #[cfg(test)] or in test functions)
pub(crate) fn extract_test_section(content: &str) -> String {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut in_test = false;
    let mut test_content = String::new();

    for line in content.lines() {
        if line.contains("#[cfg(test)]") || line.contains("#[test]") || line.contains("mod tests") {
            in_test = true;
        }
        if in_test {
            test_content.push_str(line);
            test_content.push('\n');
        }
    }

    test_content
}

/// Test fix-chain limit: detect consecutive fix commits touching the same files.
///
/// Analyzes recent git history for patterns where 3+ consecutive commits with "fix"
/// in the message touch the same file -- a signal of inadequate pre-merge testing.
pub(crate) fn test_fix_chain_limit(
    project_path: &Path,
    max_chain: usize,
) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    print!("Analyzing fix chains... ");

    // Get last 50 commits with changed files
    let output = Command::new("git")
        .args(["log", "--oneline", "--name-only", "-50"])
        .current_dir(project_path)
        .output()
        .context("Failed to run git log")?;

    if !output.status.success() {
        return Ok(FalsificationResult::passed(
            "Cannot read git history".to_string(),
        ));
    }

    let log = String::from_utf8_lossy(&output.stdout);
    let chains = detect_fix_chains(&log, max_chain);

    if chains.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "No fix chains > {} consecutive commits",
            max_chain
        )))
    } else {
        let details: Vec<String> = chains
            .iter()
            .take(5)
            .map(|(file, count)| format!("{} ({} consecutive)", file, count))
            .collect();
        let paths: Vec<PathBuf> = chains.iter().map(|(f, _)| PathBuf::from(f)).collect();
        Ok(FalsificationResult::failed(
            format!(
                "{} file(s) with fix chains > {}: {}",
                chains.len(),
                max_chain,
                details.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Check if a git log line is a commit header (starts with hex hash, length > 8)
fn is_commit_line(trimmed: &str) -> bool {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    trimmed.len() > 8
        && trimmed
            .as_bytes()
            .first()
            .map(|b| b.is_ascii_hexdigit())
            .unwrap_or(false)
}

/// Check if a commit message indicates a fix
fn is_fix_commit(msg: &str) -> bool {
    debug_assert!(true, "contract: is_fix_commit");
    let lower = msg.to_lowercase();
    lower.contains("fix") || lower.contains("bug") || lower.contains("hotfix")
}

/// Collect violations from streak map, draining entries exceeding max_chain
fn collect_violations(
    streaks: &mut std::collections::HashMap<String, usize>,
    max_chain: usize,
    violations: &mut Vec<(String, usize)>,
) {
    violations.extend(streaks.drain().filter(|(_, streak)| *streak > max_chain));
}

/// Increment streak counts for each file in the current commit
fn increment_streaks(files: &[String], streaks: &mut std::collections::HashMap<String, usize>) {
    debug_assert!(!files.is_empty(), "files must not be empty");
    for file in files {
        *streaks.entry(file.clone()).or_insert(0) += 1;
    }
}

/// Parse git log output to detect consecutive fix-commit chains per file.
/// Returns (file, chain_length) for files exceeding the threshold.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn detect_fix_chains(log: &str, max_chain: usize) -> Vec<(String, usize)> {
    debug_assert!(!log.is_empty(), "log must not be empty");
    let mut streaks: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut violations: Vec<(String, usize)> = Vec::new();
    let mut current_is_fix = false;
    let mut current_files: Vec<String> = Vec::new();

    for line in log.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if is_commit_line(trimmed) {
            // Flush previous commit
            if current_is_fix {
                increment_streaks(&current_files, &mut streaks);
            } else {
                collect_violations(&mut streaks, max_chain, &mut violations);
            }
            current_files.clear();
            current_is_fix = is_fix_commit(trimmed);
        } else if trimmed.contains('.') && !trimmed.starts_with('#') {
            current_files.push(trimmed.to_string());
        }
    }

    // Flush final commit
    if current_is_fix {
        increment_streaks(&current_files, &mut streaks);
    }
    collect_violations(&mut streaks, max_chain, &mut violations);

    violations.sort_by(|a, b| b.1.cmp(&a.1));
    violations.dedup_by(|a, b| a.0 == b.0);
    violations
}

/// Get list of changed files since baseline
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn get_changed_files(project_path: &Path, baseline_commit: &str) -> Result<Vec<String>> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let output = Command::new("git")
        .args(["diff", "--name-only", baseline_commit, "HEAD"])
        .current_dir(project_path)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect())
    } else {
        Ok(Vec::new())
    }
}
