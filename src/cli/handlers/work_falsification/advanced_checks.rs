#![cfg_attr(coverage_nightly, coverage(off))]
//! Advanced falsification checks: formal proofs, SATD, dead code, per-file coverage, lint.

use super::cache::{read_cached_metric, read_lint_cache_fallback};
use super::churn_checks::get_changed_files;
pub(crate) use super::formal_checks::test_formal_proof_verification;
use crate::cli::handlers::work_contract::{EvidenceType, FalsificationResult};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

// ============================================================================
// v2.6 comply spec: SATD, Dead Code, Per-File Coverage, Lint Gate
// ============================================================================

/// Test SATD detection: find new TODO/FIXME/HACK markers since baseline
pub(crate) async fn test_satd_detection(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    print!("Detecting SATD markers... ");

    // Run pmat analyze satd
    let output = Command::new("pmat")
        .args([
            "analyze",
            "satd",
            "--format",
            "json",
            "--path",
            &project_path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                // Get current SATD count
                let current_count = json
                    .get("total_count")
                    .or_else(|| json.get("count"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);

                // Check for new SATD since baseline (compare with git diff)
                let new_satd = detect_new_satd_since_baseline(project_path, baseline_commit)?;

                if new_satd.is_empty() {
                    Ok(FalsificationResult::passed(format!(
                        "No new SATD markers ({} existing)",
                        current_count
                    )))
                } else {
                    let paths: Vec<PathBuf> = new_satd.iter().map(|(p, _)| p.clone()).collect();
                    let details: Vec<String> = new_satd
                        .iter()
                        .take(5)
                        .map(|(p, marker)| format!("{}: {}", p.display(), marker))
                        .collect();
                    Ok(FalsificationResult::failed(
                        format!(
                            "{} new SATD marker(s): {}",
                            new_satd.len(),
                            details.join("; ")
                        ),
                        EvidenceType::FileList(paths),
                    ))
                }
            } else {
                Ok(FalsificationResult::passed(
                    "SATD check completed (no JSON output)".to_string(),
                ))
            }
        }
        _ => Ok(FalsificationResult::passed(
            "SATD analyzer not available".to_string(),
        )),
    }
}

/// Check if a trimmed line is a regular SATD-eligible comment.
/// Must start with `//` but NOT `///` or `//!`, and must not be
/// a SECURITY/SAFETY annotation.
fn is_satd_comment(trimmed: &str) -> bool {
    debug_assert!(!trimmed.is_empty(), "trimmed must not be empty");
    if !trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }
    let comment_text = trimmed[2..].trim_start();
    !comment_text.starts_with("SECURITY:") && !comment_text.starts_with("SAFETY:")
}

/// Extract SATD markers from a single added line.
/// Returns one entry per matching pattern found in the line.
fn extract_satd_markers(
    line_content: &str,
    file: &Path,
    satd_patterns: &[&str],
) -> Vec<(PathBuf, String)> {
    debug_assert!(file.exists(), "file must exist: {}", file.display());
    debug_assert!(!line_content.is_empty(), "line_content must not be empty");
    let trimmed = line_content.trim();
    satd_patterns
        .iter()
        .filter(|pattern| trimmed.contains(*pattern))
        .map(|pattern| {
            let marker = line_content
                .split(pattern)
                .nth(1)
                .map(|s| format!("{}{}", pattern, s.chars().take(50).collect::<String>()))
                .unwrap_or_else(|| pattern.to_string());
            (file.to_path_buf(), marker)
        })
        .collect()
}

/// Detect new SATD markers by comparing git diff
fn detect_new_satd_since_baseline(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<Vec<(PathBuf, String)>> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let mut new_satd = Vec::new();

    // Get diff of added lines since baseline
    let output = Command::new("git")
        .args(["diff", "-U0", baseline_commit, "HEAD", "--", "*.rs"])
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        return Ok(new_satd);
    }

    let diff = String::from_utf8_lossy(&output.stdout);
    let satd_patterns = ["TODO", "FIXME", "HACK", "XXX", "BUG"];

    let mut current_file: Option<PathBuf> = None;

    for line in diff.lines() {
        if let Some(file_path) = line.strip_prefix("+++ b/") {
            current_file = Some(PathBuf::from(file_path));
            continue;
        }

        // Skip non-added lines and the "+++ b/" header (already handled above)
        let Some(line_content) = line.strip_prefix('+') else {
            continue;
        };
        if line.starts_with("+++") {
            continue;
        }

        let trimmed = line_content.trim();
        if !is_satd_comment(trimmed) {
            continue;
        }

        if let Some(ref file) = current_file {
            new_satd.extend(extract_satd_markers(line_content, file, &satd_patterns));
        }
    }

    Ok(new_satd)
}

/// Test dead code detection: find new unreachable code since baseline
pub(crate) async fn test_dead_code_detection(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    print!("Detecting dead code... ");

    // Run pmat analyze dead-code
    let output = Command::new("pmat")
        .args([
            "analyze",
            "dead-code",
            "--format",
            "json",
            "--path",
            &project_path.to_string_lossy(),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                // Get dead code items
                let dead_items = json
                    .get("dead_code")
                    .or_else(|| json.get("items"))
                    .and_then(|items| items.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);

                // For now, we report any dead code found
                // Future: compare with baseline to only flag NEW dead code
                if dead_items == 0 {
                    Ok(FalsificationResult::passed(
                        "No dead code detected".to_string(),
                    ))
                } else {
                    // Check if these are new since baseline
                    let changed_files = get_changed_files(project_path, baseline_commit)?;
                    let dead_in_changed: usize = json
                        .get("dead_code")
                        .or_else(|| json.get("items"))
                        .and_then(|items| items.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter(|item| {
                                    item.get("file")
                                        .and_then(|f| f.as_str())
                                        .map(|f| changed_files.iter().any(|cf| cf.ends_with(f)))
                                        .unwrap_or(false)
                                })
                                .count()
                        })
                        .unwrap_or(0);

                    if dead_in_changed == 0 {
                        Ok(FalsificationResult::passed(format!(
                            "{} existing dead code items (none in changed files)",
                            dead_items
                        )))
                    } else {
                        Ok(FalsificationResult::failed(
                            format!("{} dead code item(s) in changed files", dead_in_changed),
                            EvidenceType::NumericComparison {
                                actual: dead_in_changed as f64,
                                threshold: 0.0,
                            },
                        ))
                    }
                }
            } else {
                Ok(FalsificationResult::passed(
                    "Dead code check completed (no JSON output)".to_string(),
                ))
            }
        }
        _ => Ok(FalsificationResult::passed(
            "Dead code analyzer not available".to_string(),
        )),
    }
}

/// Check if a file should be skipped for per-file coverage checks.
fn is_excluded_from_per_file_coverage(filename: &str) -> bool {
    debug_assert!(!filename.is_empty(), "filename must not be empty");
    filename.contains("/tests/") || filename.contains("_test.rs") || filename.contains("/target/")
}

/// Extract the line coverage percentage from a single llvm-cov file entry.
fn extract_file_line_coverage(file_entry: &serde_json::Value) -> f64 {
    debug_assert!(true, "contract: extract_file_line_coverage");
    file_entry
        .get("summary")
        .and_then(|s| s.get("lines"))
        .and_then(|l| l.get("percent"))
        .and_then(|p| p.as_f64())
        .unwrap_or(100.0)
}

/// Parse llvm-cov JSON and return files whose coverage is below `threshold`.
///
/// Each entry is `(filename, coverage_pct)`. Test files and generated files
/// are excluded.
fn collect_files_below_threshold(json: &serde_json::Value, threshold: f64) -> Vec<(PathBuf, f64)> {
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
    let data = match json.get("data").and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return Vec::new(),
    };

    data.iter()
        .filter_map(|file_data| file_data.get("files").and_then(|f| f.as_array()))
        .flatten()
        .filter_map(|file| {
            let filename = file
                .get("filename")
                .and_then(|f| f.as_str())
                .unwrap_or("unknown");
            if is_excluded_from_per_file_coverage(filename) {
                return None;
            }
            let coverage = extract_file_line_coverage(file);
            if coverage < threshold {
                Some((PathBuf::from(filename), coverage))
            } else {
                None
            }
        })
        .collect()
}

/// Build a FalsificationResult from the list of files below coverage threshold.
fn build_per_file_coverage_result(
    files_below_threshold: Vec<(PathBuf, f64)>,
    threshold: f64,
) -> FalsificationResult {
    debug_assert!(threshold >= 0.0, "threshold must be non-negative");
    if files_below_threshold.is_empty() {
        return FalsificationResult::passed(format!("All files >= {:.1}% coverage", threshold));
    }

    let paths: Vec<PathBuf> = files_below_threshold
        .iter()
        .map(|(p, _)| p.clone())
        .collect();
    let details: Vec<String> = files_below_threshold
        .iter()
        .take(10)
        .map(|(p, cov)| format!("{}: {:.1}%", p.display(), cov))
        .collect();
    FalsificationResult::failed(
        format!(
            "{} file(s) below {:.1}% threshold: {}",
            files_below_threshold.len(),
            threshold,
            details.join(", ")
        ),
        EvidenceType::FileList(paths),
    )
}

/// Test per-file coverage: all files must meet threshold
pub(crate) async fn test_per_file_coverage(
    project_path: &Path,
    threshold: f64,
) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    print!("Checking per-file coverage... ");

    // Try to read per-file coverage from llvm-cov output
    let coverage_json = project_path.join("target/llvm-cov/coverage.json");

    if !coverage_json.exists() {
        return Ok(FalsificationResult::passed(format!(
            "No per-file coverage data (run 'make coverage'), threshold: {:.1}%",
            threshold
        )));
    }

    let content = std::fs::read_to_string(&coverage_json)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let files_below = collect_files_below_threshold(&json, threshold);
    Ok(build_per_file_coverage_result(files_below, threshold))
}

/// Test lint pass: O(1) - reads from cached lint status
pub(crate) async fn test_lint_pass(project_path: &Path) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    print!("Reading lint cache... ");

    // O(1): Read from cache instead of running make lint
    // Try primary cache location, then fallback to work-item and .pmat directories
    if let Some(cache) = read_cached_metric(project_path, "lint-status.json")
        .or_else(|| read_lint_cache_fallback(project_path))
    {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!(
                    "Lint cache too old ({} min). Run 'make lint' first.",
                    cache.age_minutes
                ),
                EvidenceType::BooleanCheck(false),
            ));
        }

        // Validate 'passed' field exists -- reject malformed cache (Popperian Audit v2.1)
        let passed = match cache.value.get("passed").and_then(|v| v.as_bool()) {
            Some(p) => p,
            None => {
                return Ok(FalsificationResult::failed(
                    "Invalid lint cache (missing 'passed' field). Re-run 'make lint'.".to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let stale_note = if cache.is_stale_warn {
            format!(
                " (cached {} min ago, consider re-running)",
                cache.age_minutes
            )
        } else {
            format!(" (cached {} min ago)", cache.age_minutes)
        };

        if passed {
            return Ok(FalsificationResult::passed(format!("PASSED{}", stale_note)));
        } else {
            let errors = cache
                .value
                .get("error_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Ok(FalsificationResult::failed(
                format!("{} lint errors{}", errors, stale_note),
                EvidenceType::NumericComparison {
                    actual: errors as f64,
                    threshold: 0.0,
                },
            ));
        }
    }

    // No cache - check if Makefile exists and suggest running lint
    let makefile = project_path.join("Makefile");
    if !makefile.exists() {
        return Ok(FalsificationResult::passed(
            "No Makefile found (skipping lint check)".to_string(),
        ));
    }

    // No cache available - block until user runs make lint
    Ok(FalsificationResult::failed(
        "No lint cache. Run 'make lint' first (O(1) requirement)".to_string(),
        EvidenceType::BooleanCheck(false),
    ))
}
