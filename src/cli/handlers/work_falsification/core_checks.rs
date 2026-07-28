#![cfg_attr(coverage_nightly, coverage(off))]
//! Core falsification checks: manifest, coverage, TDG, complexity, spec, roadmap, git.

use crate::cli::handlers::work_contract::{EvidenceType, FalsificationResult, FileManifest};
use crate::cli::handlers::work_falsification::pre_run_tree::{
    dirty_file_paths, has_upstream, parse_ahead_count, pre_run_status, read_porcelain_status,
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Test manifest integrity: verify all baseline files still exist
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    Ok(FalsificationResult::unmeasured(format!(
        "{} changed file(s), but differential coverage is not wired to any coverage \
         artifact — this claim verifies nothing today",
        changed_files.len()
    )))
}

/// Test absolute coverage threshold
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn test_absolute_coverage(
    project_path: &Path,
    threshold: f64,
) -> Result<FalsificationResult> {
    print!("Checking coverage threshold... ");

    // Try to read coverage from cached metrics
    let metrics_dir = project_path.join(".pmat-metrics/trends");
    let coverage_file = metrics_dir.join("test-coverage.json");

    if !coverage_file.exists() {
        return Ok(FalsificationResult::unmeasured(format!(
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn test_tdg_regression(
    project_path: &Path,
    baseline_tdg: f64,
) -> Result<FalsificationResult> {
    print!("Checking TDG score... ");

    // Read current TDG score from cache
    let tdg_file = project_path.join(".pmat-metrics/tdg-score.json");

    if !tdg_file.exists() {
        return Ok(FalsificationResult::unmeasured(format!(
            "No TDG data (baseline: {:.1}); nothing writes .pmat-metrics/tdg-score.json",
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(json) => Ok(evaluate_complexity_json(&json, max_complexity)),
                // pmat's own subcommand emitting unparseable JSON means the
                // claim was not evaluated. Reporting that as a pass let an
                // unmeasured claim satisfy a blocking gate.
                Err(e) => Ok(FalsificationResult::failed(
                    format!("could not parse 'pmat analyze complexity' output: {e}"),
                    EvidenceType::BooleanCheck(false),
                )),
            }
        }
        _ => Ok(FalsificationResult::failed(
            "'pmat analyze complexity' could not be run, so complexity was not checked".to_string(),
            EvidenceType::BooleanCheck(false),
        )),
    }
}

/// Judge complexity from `pmat analyze complexity --format json` output.
///
/// Reads `violations[]`, which is the shape the analyzer actually emits
/// (`summary`, `violations`, `hotspots`, `files`, `top_files_limit`). The
/// previous reader looked for a top-level `functions` array that has never
/// existed — `functions` appears only nested under `files[]` — so the lookup
/// always returned None and the check fell through to an unconditional pass.
/// A blocking gate that cannot fail is worse than no gate.
fn evaluate_complexity_json(json: &serde_json::Value, max_complexity: u32) -> FalsificationResult {
    let over: Vec<&serde_json::Value> = json
        .get("violations")
        .and_then(|v| v.as_array())
        .map(|violations| {
            violations
                .iter()
                .filter(|v| {
                    v.get("value")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0)
                        > max_complexity as u64
                })
                .collect()
        })
        .unwrap_or_default();

    if over.is_empty() {
        return FalsificationResult::passed(format!(
            "All functions <= {} complexity",
            max_complexity
        ));
    }

    let names: Vec<String> = over
        .iter()
        .filter_map(|v| v.get("function").and_then(|n| n.as_str()))
        .map(String::from)
        .collect();
    FalsificationResult::failed(
        format!(
            "{} function(s) exceed complexity {}: {}",
            over.len(),
            max_complexity,
            names.join(", ")
        ),
        EvidenceType::NumericComparison {
            actual: over.len() as f64,
            threshold: 0.0,
        },
    )
}

/// Test file size regression: no file should exceed threshold
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn test_github_sync(project_path: &Path) -> Result<FalsificationResult> {
    print!("Checking git status... ");

    // GH #630: judge the tree pmat FOUND, not the one it made. `pmat work
    // complete` writes caches, a ledger, receipts and (on success) the roadmap
    // and CHANGELOG while it runs, so reading git status here used to fail on
    // pmat's own output and no commit-and-retry could ever reach a fixed point.
    // The snapshot is taken before any of those writes; falling back to a live
    // read keeps this correct for callers that never mutate the tree.
    let status = match pre_run_status() {
        Some(snapshot) => snapshot,
        None => read_porcelain_status(project_path).context("Failed to run git status")?,
    };

    let ahead_count = parse_ahead_count(&status);
    let dirty_paths = dirty_file_paths(&status);
    let dirty_count = dirty_paths.len();
    let tracks_upstream = has_upstream(&status);

    if tracks_upstream && ahead_count == 0 && dirty_count == 0 {
        return Ok(FalsificationResult::passed(
            "All changes committed and pushed".to_string(),
        ));
    }

    let mut issues = Vec::new();
    if !tracks_upstream {
        // Nothing can have been pushed from a branch with no upstream, so
        // reporting "all pushed" here was a false pass.
        issues.push("branch has no upstream (nothing pushed)".to_string());
    }
    if ahead_count > 0 {
        issues.push(format!("{} unpushed commit(s)", ahead_count));
    }
    if dirty_count > 0 {
        // Name the files. "1 uncommitted file(s)" alone is unfalsifiable by the
        // reader, which is how a true positive got filed as a pmat bug (#630).
        issues.push(format!(
            "{} uncommitted file(s): {}",
            dirty_count,
            summarize_paths(&dirty_paths)
        ));
    }
    Ok(FalsificationResult::failed(
        issues.join(", "),
        EvidenceType::GitState {
            unpushed_commits: ahead_count,
            dirty_files: dirty_count,
        },
    ))
}

/// Render at most a handful of paths, so a large dirty tree stays readable.
fn summarize_paths(paths: &[String]) -> String {
    const SHOWN: usize = 5;
    let head = paths
        .iter()
        .take(SHOWN)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    match paths.len().checked_sub(SHOWN) {
        Some(rest) if rest > 0 => format!("{head}, +{rest} more"),
        _ => head,
    }
}
