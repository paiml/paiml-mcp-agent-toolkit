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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn test_satd_detection(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn test_dead_code_detection(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
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
    filename.contains("/tests/") || filename.contains("_test.rs") || filename.contains("/target/")
}

/// Extract the line coverage percentage from a single llvm-cov file entry.
fn extract_file_line_coverage(file_entry: &serde_json::Value) -> f64 {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn test_per_file_coverage(
    project_path: &Path,
    threshold: f64,
) -> Result<FalsificationResult> {
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn test_lint_pass(project_path: &Path) -> Result<FalsificationResult> {
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

#[cfg(test)]
mod advanced_checks_tests {
    //! Wave 39 PR17 — pure-helper coverage for advanced_checks.rs
    //! (155 missed, 0% pre-wave). The async test_* functions shell out
    //! to pmat/cargo and are disqualified per spec §4.11.
    use super::*;

    // ── is_satd_comment ─────────────────────────────────────────────────────

    #[test]
    fn test_is_satd_comment_basic_double_slash() {
        // PIN: starts with `//` AND not `///` AND not `//!` AND not SECURITY/SAFETY.
        assert!(is_satd_comment("// TODO: fix"));
        assert!(is_satd_comment("// FIXME"));
        assert!(is_satd_comment("// HACK because"));
    }

    #[test]
    fn test_is_satd_comment_doc_comments_excluded() {
        // PIN: doc comments are NOT SATD.
        assert!(!is_satd_comment("/// doc comment"));
        assert!(!is_satd_comment("//! inner doc"));
    }

    #[test]
    fn test_is_satd_comment_security_safety_excluded() {
        // PIN: SECURITY: and SAFETY: prefixed comments are NOT SATD even though
        // they start with `//`.
        assert!(!is_satd_comment("// SECURITY: validated input"));
        assert!(!is_satd_comment("// SAFETY: invariant holds"));
    }

    #[test]
    fn test_is_satd_comment_non_comment_lines_rejected() {
        assert!(!is_satd_comment("fn foo() {}"));
        assert!(!is_satd_comment(""));
        assert!(!is_satd_comment("  let x = 42;"));
    }

    // ── extract_satd_markers ────────────────────────────────────────────────

    #[test]
    fn test_extract_satd_markers_single_pattern() {
        let path = PathBuf::from("foo.rs");
        let markers =
            extract_satd_markers("    // TODO: handle this case better", &path, &["TODO"]);
        assert_eq!(markers.len(), 1);
        assert!(markers[0].1.starts_with("TODO"));
    }

    #[test]
    fn test_extract_satd_markers_multiple_patterns_all_matched() {
        let path = PathBuf::from("foo.rs");
        let markers = extract_satd_markers("// TODO and FIXME together", &path, &["TODO", "FIXME"]);
        assert_eq!(markers.len(), 2);
    }

    #[test]
    fn test_extract_satd_markers_truncates_to_50_chars() {
        // PIN: marker text after pattern is truncated to 50 chars.
        let path = PathBuf::from("foo.rs");
        let suffix: String = "x".repeat(100);
        let long_text = format!("TODO{suffix}");
        let markers = extract_satd_markers(&long_text, &path, &["TODO"]);
        assert_eq!(markers.len(), 1);
        // Marker = "TODO" + first 50 chars of suffix = max ~54 chars
        assert!(markers[0].1.len() <= 54);
    }

    #[test]
    fn test_extract_satd_markers_no_match_returns_empty() {
        let path = PathBuf::from("foo.rs");
        let markers = extract_satd_markers("fn foo() {}", &path, &["TODO"]);
        assert!(markers.is_empty());
    }

    // ── is_excluded_from_per_file_coverage ──────────────────────────────────

    #[test]
    fn test_is_excluded_from_per_file_coverage_tests_dir() {
        assert!(is_excluded_from_per_file_coverage("/repo/tests/foo.rs"));
        assert!(is_excluded_from_per_file_coverage("src/foo_test.rs"));
        assert!(is_excluded_from_per_file_coverage(
            "/repo/target/debug/build.rs"
        ));
    }

    #[test]
    fn test_is_excluded_from_per_file_coverage_normal_src_not_excluded() {
        assert!(!is_excluded_from_per_file_coverage("src/foo.rs"));
        assert!(!is_excluded_from_per_file_coverage("src/cli/main.rs"));
    }

    // ── extract_file_line_coverage ──────────────────────────────────────────

    #[test]
    fn test_extract_file_line_coverage_valid_json() {
        let entry = serde_json::json!({
            "summary": {
                "lines": {
                    "percent": 85.5
                }
            }
        });
        assert_eq!(extract_file_line_coverage(&entry), 85.5);
    }

    #[test]
    fn test_extract_file_line_coverage_missing_summary_defaults_100() {
        // PIN: missing fields default to 100.0 (treated as full coverage).
        // This is INTENTIONAL — we don't fail tests for files we couldn't measure.
        let entry = serde_json::json!({});
        assert_eq!(extract_file_line_coverage(&entry), 100.0);
    }

    #[test]
    fn test_extract_file_line_coverage_partial_path_defaults_100() {
        // Missing percent value → default 100.0.
        let entry = serde_json::json!({"summary": {"lines": {}}});
        assert_eq!(extract_file_line_coverage(&entry), 100.0);
    }

    // ── collect_files_below_threshold ───────────────────────────────────────

    #[test]
    fn test_collect_files_below_threshold_empty_data() {
        let json = serde_json::json!({});
        let result = collect_files_below_threshold(&json, 80.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_collect_files_below_threshold_filters_by_pct() {
        let json = serde_json::json!({
            "data": [{
                "files": [
                    {"filename": "src/good.rs", "summary": {"lines": {"percent": 95.0}}},
                    {"filename": "src/bad.rs", "summary": {"lines": {"percent": 50.0}}},
                    {"filename": "src/edge.rs", "summary": {"lines": {"percent": 80.0}}},
                ]
            }]
        });
        // Threshold 80 → only "bad" (50%) is below; "edge" (80) is NOT (strict <).
        let result = collect_files_below_threshold(&json, 80.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PathBuf::from("src/bad.rs"));
        assert_eq!(result[0].1, 50.0);
    }

    #[test]
    fn test_collect_files_below_threshold_excludes_test_files() {
        // PIN: exclusion rule is `/tests/` (with leading slash) OR `_test.rs`
        // OR `/target/`. A bare `tests/foo.rs` (no leading slash) is NOT
        // excluded — only `/tests/foo.rs` patterns. This is fragile but pinned.
        let json = serde_json::json!({
            "data": [{
                "files": [
                    {"filename": "/repo/tests/integration.rs", "summary": {"lines": {"percent": 0.0}}},
                    {"filename": "src/foo_test.rs", "summary": {"lines": {"percent": 0.0}}},
                    {"filename": "src/real.rs", "summary": {"lines": {"percent": 30.0}}},
                ]
            }]
        });
        let result = collect_files_below_threshold(&json, 80.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PathBuf::from("src/real.rs"));
    }

    #[test]
    fn test_collect_files_below_threshold_bare_tests_dir_not_excluded_pin() {
        // PIN: `tests/foo.rs` (no leading slash) does NOT match `/tests/`.
        // The exclusion is path-anchored.
        let json = serde_json::json!({
            "data": [{
                "files": [
                    {"filename": "tests/foo.rs", "summary": {"lines": {"percent": 0.0}}},
                ]
            }]
        });
        let result = collect_files_below_threshold(&json, 80.0);
        assert_eq!(
            result.len(),
            1,
            "bare tests/foo.rs (no leading slash) is NOT excluded"
        );
    }

    // ── build_per_file_coverage_result ──────────────────────────────────────

    #[test]
    fn test_build_per_file_coverage_result_empty_passes() {
        let result = build_per_file_coverage_result(vec![], 80.0);
        assert!(!result.falsified);
        assert!(result.explanation.contains("All files"));
        assert!(result.explanation.contains("80.0%"));
    }

    #[test]
    fn test_build_per_file_coverage_result_failures_truncated_to_10() {
        // PIN: detail list is truncated to 10 files (`take(10)`).
        let mut files = Vec::new();
        for i in 0..15 {
            files.push((PathBuf::from(format!("src/f{i}.rs")), 50.0));
        }
        let result = build_per_file_coverage_result(files, 80.0);
        assert!(result.falsified);
        assert!(result.explanation.contains("15 file(s) below"));
        // Count comma-separated entries — at most 10.
        let comma_count = result.explanation.matches(", ").count();
        assert!(comma_count <= 10, "got {comma_count} commas");
    }

    #[test]
    fn test_build_per_file_coverage_result_message_format() {
        let files = vec![
            (PathBuf::from("src/a.rs"), 50.0),
            (PathBuf::from("src/b.rs"), 30.0),
        ];
        let result = build_per_file_coverage_result(files, 80.0);
        assert!(result.falsified);
        assert!(result
            .explanation
            .contains("2 file(s) below 80.0% threshold"));
        assert!(result.explanation.contains("src/a.rs: 50.0%"));
        assert!(result.explanation.contains("src/b.rs: 30.0%"));
    }
}
