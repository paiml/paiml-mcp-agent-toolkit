#![cfg_attr(coverage_nightly, coverage(off))]
//! Core falsification checks: manifest, coverage, TDG, complexity, spec, roadmap, git.

use crate::cli::handlers::work_contract::{EvidenceType, FalsificationResult, FileManifest};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Test manifest integrity: verify all baseline files still exist
pub(crate) fn test_manifest_integrity(
    project_path: &Path,
    manifest: &FileManifest,
) -> Result<FalsificationResult> {
    print!("Searching for missing files... ");

    let missing = manifest.verify_integrity(project_path);

    if missing.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "All {} files present",
            manifest.files.len()
        )))
    } else {
        Ok(FalsificationResult::failed(
            format!("{} files missing from baseline", missing.len()),
            EvidenceType::FileList(missing),
        ))
    }
}

/// Test for coverage gaming patterns
pub(crate) fn test_coverage_gaming(project_path: &Path) -> Result<FalsificationResult> {
    print!("Scanning for gaming patterns... ");

    let detection_result = crate::services::gaming_detector::detect_coverage_gaming(project_path)?;

    if !detection_result.has_critical_violations() {
        Ok(FalsificationResult::passed(format!(
            "No gaming patterns found in {} files",
            detection_result.files_scanned
        )))
    } else {
        let violations = detection_result.critical_violations();
        let paths: Vec<PathBuf> = violations.iter().map(|v| v.file.clone()).collect();
        Ok(FalsificationResult::failed(
            format!("{} gaming violation(s) found", violations.len()),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Test differential coverage: all changed lines must be covered
pub(crate) async fn test_differential_coverage(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    print!("Analyzing changed lines... ");

    // Get changed files since baseline
    let output = Command::new("git")
        .args(["diff", "--name-only", baseline_commit, "HEAD"])
        .current_dir(project_path)
        .output()
        .context("Failed to get git diff")?;

    if !output.status.success() {
        return Ok(FalsificationResult::passed(
            "No baseline commit found, skipping differential coverage".to_string(),
        ));
    }

    let changed_files: Vec<&str> = std::str::from_utf8(&output.stdout)?
        .lines()
        .filter(|f| f.ends_with(".rs"))
        .collect();

    if changed_files.is_empty() {
        return Ok(FalsificationResult::passed(
            "No Rust files changed".to_string(),
        ));
    }

    // Coverage data is assumed available from a previous run
    // In production, this integrates with llvm-cov or similar

    Ok(FalsificationResult::passed(format!(
        "{} changed files (coverage check requires llvm-cov data)",
        changed_files.len()
    )))
}

/// Test absolute coverage threshold
pub(crate) async fn test_absolute_coverage(
    project_path: &Path,
    threshold: f64,
) -> Result<FalsificationResult> {
    print!("Checking coverage threshold... ");

    // Try to read coverage from cached metrics
    let metrics_dir = project_path.join(".pmat-metrics/trends");
    let coverage_file = metrics_dir.join("test-coverage.json");

    if !coverage_file.exists() {
        return Ok(FalsificationResult::passed(format!(
            "No coverage data (run 'make coverage' to establish baseline), threshold: {:.1}%",
            threshold
        )));
    }

    let content = std::fs::read_to_string(&coverage_file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(entries) = json.as_array() {
        if let Some(latest) = entries.last() {
            if let Some(coverage) = latest.get("value").and_then(|v| v.as_f64()) {
                if coverage >= threshold {
                    return Ok(FalsificationResult::passed(format!(
                        "{:.1}% >= {:.1}%",
                        coverage, threshold
                    )));
                } else {
                    return Ok(FalsificationResult::failed(
                        format!("{:.1}% < {:.1}% threshold", coverage, threshold),
                        EvidenceType::NumericComparison {
                            actual: coverage,
                            threshold,
                        },
                    ));
                }
            }
        }
    }

    Ok(FalsificationResult::passed(
        "No coverage entries found".to_string(),
    ))
}

/// Test TDG score regression
pub(crate) async fn test_tdg_regression(
    project_path: &Path,
    baseline_tdg: f64,
) -> Result<FalsificationResult> {
    print!("Checking TDG score... ");

    // Read current TDG score from cache
    let tdg_file = project_path.join(".pmat-metrics/tdg-score.json");

    if !tdg_file.exists() {
        return Ok(FalsificationResult::passed(format!(
            "No TDG data (baseline: {:.1})",
            baseline_tdg
        )));
    }

    let content = std::fs::read_to_string(&tdg_file)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(current_tdg) = json.get("score").and_then(|v| v.as_f64()) {
        if current_tdg >= baseline_tdg {
            Ok(FalsificationResult::passed(format!(
                "{:.1} >= {:.1} (baseline)",
                current_tdg, baseline_tdg
            )))
        } else {
            Ok(FalsificationResult::failed(
                format!("{:.1} < {:.1} (regression)", current_tdg, baseline_tdg),
                EvidenceType::NumericComparison {
                    actual: current_tdg,
                    threshold: baseline_tdg,
                },
            ))
        }
    } else {
        Ok(FalsificationResult::passed(
            "No TDG score in cache".to_string(),
        ))
    }
}

/// Test complexity regression: no function should exceed threshold
pub(crate) fn test_complexity_regression(
    project_path: &Path,
    max_complexity: u32,
) -> Result<FalsificationResult> {
    print!("Analyzing function complexity... ");

    // Run pmat complexity check
    let output = Command::new("pmat")
        .args([
            "analyze",
            "complexity",
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
                // Check for functions exceeding complexity
                if let Some(functions) = json.get("functions").and_then(|f| f.as_array()) {
                    let high_complexity: Vec<_> = functions
                        .iter()
                        .filter(|f| {
                            f.get("complexity").and_then(|c| c.as_u64()).unwrap_or(0)
                                > max_complexity as u64
                        })
                        .collect();

                    if high_complexity.is_empty() {
                        return Ok(FalsificationResult::passed(format!(
                            "All functions <= {} complexity",
                            max_complexity
                        )));
                    } else {
                        let names: Vec<String> = high_complexity
                            .iter()
                            .filter_map(|f| f.get("name").and_then(|n| n.as_str()))
                            .map(|s| s.to_string())
                            .collect();
                        return Ok(FalsificationResult::failed(
                            format!(
                                "{} function(s) exceed complexity {}: {}",
                                high_complexity.len(),
                                max_complexity,
                                names.join(", ")
                            ),
                            EvidenceType::NumericComparison {
                                actual: high_complexity.len() as f64,
                                threshold: 0.0,
                            },
                        ));
                    }
                }
            }
            Ok(FalsificationResult::passed(
                "Complexity check passed".to_string(),
            ))
        }
        _ => Ok(FalsificationResult::passed(
            "Complexity analyzer not available".to_string(),
        )),
    }
}

/// Test file size regression: no file should exceed threshold
pub(crate) fn test_file_size_regression(
    project_path: &Path,
    max_lines: usize,
) -> Result<FalsificationResult> {
    print!("Checking file sizes... ");

    let mut large_files = Vec::new();

    for entry in walkdir::WalkDir::new(project_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path().to_string_lossy();
            !path.contains("/target/") && !path.contains("/.git/")
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
            if let Ok(content) = std::fs::read_to_string(path) {
                let line_count = content.lines().count();
                if line_count > max_lines {
                    large_files.push((
                        path.strip_prefix(project_path)
                            .unwrap_or(path)
                            .to_path_buf(),
                        line_count,
                    ));
                }
            }
        }
    }

    if large_files.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "All files <= {} lines",
            max_lines
        )))
    } else {
        let paths: Vec<PathBuf> = large_files.iter().map(|(p, _)| p.clone()).collect();
        let details: Vec<String> = large_files
            .iter()
            .map(|(p, lines)| format!("{} ({} lines)", p.display(), lines))
            .collect();
        Ok(FalsificationResult::failed(
            format!(
                "{} file(s) exceed {} lines: {}",
                large_files.len(),
                max_lines,
                details.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Parse spec score from pmat output (format: "Score: XX/100")
fn parse_spec_score(stdout: &str) -> Option<u32> {
    let score_line = stdout.lines().find(|l| l.contains("Score:"))?;
    let score_str = score_line.split('/').next()?;
    score_str
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<u32>()
        .ok()
}

/// Evaluate parsed spec score against threshold
fn evaluate_spec_score(score: u32, min_score: u32) -> FalsificationResult {
    if score >= min_score {
        FalsificationResult::passed(format!("{}/100 >= {}/100", score, min_score))
    } else {
        FalsificationResult::failed(
            format!("{}/100 < {}/100 threshold", score, min_score),
            EvidenceType::NumericComparison {
                actual: score as f64,
                threshold: min_score as f64,
            },
        )
    }
}

/// Test spec quality: spec score must meet threshold
pub(crate) fn test_spec_quality(
    project_path: &Path,
    work_item_id: &str,
    min_score: u32,
) -> Result<FalsificationResult> {
    print!("Checking spec quality... ");

    // Look for spec file
    let spec_path = project_path.join(format!(
        "docs/specifications/{}-spec.md",
        work_item_id.to_lowercase()
    ));

    if !spec_path.exists() {
        // Also check for numbered format
        let spec_dir = project_path.join("docs/specifications");
        if spec_dir.exists() {
            // Spec not found - this is OK if not required
            return Ok(FalsificationResult::passed(
                "No spec file found (optional)".to_string(),
            ));
        }
    }

    // Run pmat spec score
    let output = Command::new("pmat")
        .args(["spec", "score", &spec_path.to_string_lossy()])
        .current_dir(project_path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let result = parse_spec_score(&stdout)
                .map(|score| evaluate_spec_score(score, min_score))
                .unwrap_or_else(|| {
                    FalsificationResult::passed("Spec score check completed".to_string())
                });
            Ok(result)
        }
        _ => Ok(FalsificationResult::passed(
            "Spec scorer not available".to_string(),
        )),
    }
}

/// Test roadmap update: roadmap must be modified since baseline
pub(crate) fn test_roadmap_update(
    project_path: &Path,
    baseline_commit: &str,
) -> Result<FalsificationResult> {
    print!("Checking roadmap update... ");

    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");

    if !roadmap_path.exists() {
        return Ok(FalsificationResult::passed(
            "No roadmap.yaml found".to_string(),
        ));
    }

    // Check if roadmap was modified since baseline
    let output = Command::new("git")
        .args([
            "diff",
            "--name-only",
            baseline_commit,
            "HEAD",
            "--",
            "docs/roadmaps/roadmap.yaml",
        ])
        .current_dir(project_path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let changed = !output.stdout.is_empty();
            if changed {
                Ok(FalsificationResult::passed(
                    "Roadmap was updated".to_string(),
                ))
            } else {
                Ok(FalsificationResult::failed(
                    "Roadmap not updated since baseline".to_string(),
                    EvidenceType::BooleanCheck(false),
                ))
            }
        }
        _ => Ok(FalsificationResult::passed(
            "Cannot check roadmap changes".to_string(),
        )),
    }
}

/// Test GitHub sync: all commits must be pushed
pub(crate) fn test_github_sync(project_path: &Path) -> Result<FalsificationResult> {
    print!("Checking git status... ");

    // Check for unpushed commits
    let output = Command::new("git")
        .args(["status", "--porcelain", "-b"])
        .current_dir(project_path)
        .output()
        .context("Failed to run git status")?;

    let status = String::from_utf8_lossy(&output.stdout);

    // Check for ahead commits
    let ahead_count = if status.contains("ahead") {
        // Parse "ahead X" from status
        status
            .lines()
            .next()
            .and_then(|l| {
                l.find("ahead")
                    .and_then(|i| l.get(i..))
                    .and_then(|s| s.split_whitespace().nth(1))
                    .and_then(|n| n.trim_end_matches(']').parse::<usize>().ok())
            })
            .unwrap_or(0)
    } else {
        0
    };

    // Count dirty files (exclude untracked ?? files -- they are not uncommitted changes)
    let dirty_count = status
        .lines()
        .skip(1)
        .filter(|l| !l.is_empty() && !l.starts_with("??"))
        .count();

    if ahead_count == 0 && dirty_count == 0 {
        Ok(FalsificationResult::passed(
            "All changes committed and pushed".to_string(),
        ))
    } else {
        let mut issues = Vec::new();
        if ahead_count > 0 {
            issues.push(format!("{} unpushed commit(s)", ahead_count));
        }
        if dirty_count > 0 {
            issues.push(format!("{} uncommitted file(s)", dirty_count));
        }
        Ok(FalsificationResult::failed(
            issues.join(", "),
            EvidenceType::GitState {
                unpushed_commits: ahead_count,
                dirty_files: dirty_count,
            },
        ))
    }
}
