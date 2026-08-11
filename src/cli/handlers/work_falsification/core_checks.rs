#![cfg_attr(coverage_nightly, coverage(off))]
//! Core falsification checks: manifest, coverage, TDG, complexity, spec, roadmap, git.

use super::coverage_source;
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
        // An unresolvable baseline means the diff was never computed. Recording
        // that as `passed()` marked the claim measured:true and counted it as
        // corroborating evidence -- a claim that was never evaluated cannot
        // corroborate anything.
        return Ok(FalsificationResult::unmeasured(
            "baseline commit could not be resolved, so no diff was computed".to_string(),
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

    let Some(coverage) = coverage_source::load(project_path) else {
        return Ok(FalsificationResult::unmeasured(format!(
            "{} changed file(s), but no coverage artifact exists — {}",
            changed_files.len(),
            coverage_source::COVERAGE_HINT
        )));
    };

    // -U0 so hunk headers describe exactly the lines this work introduced.
    let diff = Command::new("git")
        .args(["diff", "-U0", baseline_commit, "HEAD", "--", "*.rs"])
        .current_dir(project_path)
        .output()
        .context("Failed to get git diff for differential coverage")?;
    let changed_lines = coverage_source::changed_lines(&String::from_utf8_lossy(&diff.stdout));

    Ok(evaluate_differential_coverage(&changed_lines, &coverage))
}

/// Judge whether every line this work added is executed by the test suite.
fn evaluate_differential_coverage(
    changed_lines: &std::collections::HashMap<String, Vec<usize>>,
    coverage: &coverage_source::LineCoverage,
) -> FalsificationResult {
    let mut uncovered: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for (file, lines) in changed_lines {
        // A file absent from the report is not instrumented (a test file, or
        // excluded from coverage); it has no lines to require hits for.
        let Some(file_cov) = coverage_source::lookup(coverage, file) else {
            continue;
        };
        for line in lines {
            // Lines with no coverage record are not instrumented — comments,
            // blank lines, `use` statements. Only recorded lines can be missed.
            let Some(hits) = file_cov.get(line) else {
                continue;
            };
            checked += 1;
            if *hits == 0 {
                uncovered.push(format!("{}:{}", file, line));
            }
        }
    }

    if checked == 0 {
        return FalsificationResult::unmeasured(format!(
            "no instrumented lines among the changed lines — {}",
            coverage_source::COVERAGE_HINT
        ));
    }

    if uncovered.is_empty() {
        return FalsificationResult::passed(format!("all {} changed line(s) covered", checked));
    }

    let shown: Vec<String> = uncovered.iter().take(5).cloned().collect();
    let rest = uncovered.len().saturating_sub(shown.len());
    let suffix = if rest > 0 {
        format!(", +{} more", rest)
    } else {
        String::new()
    };
    FalsificationResult::failed(
        format!(
            "{}/{} changed line(s) uncovered: {}{}",
            uncovered.len(),
            checked,
            shown.join(", "),
            suffix
        ),
        EvidenceType::NumericComparison {
            actual: uncovered.len() as f64,
            threshold: 0.0,
        },
    )
}

/// Test absolute coverage threshold
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn test_absolute_coverage(
    project_path: &Path,
    threshold: f64,
) -> Result<FalsificationResult> {
    print!("Checking coverage threshold... ");

    // ORDER MATTERS. The real coverage artifact is consulted FIRST; the
    // `.pmat-metrics/trends/test-coverage.json` history is only a fallback.
    //
    // Previously the trends file won, and it is an ordinary hand-writable JSON
    // file in the working tree: appending {"value": 100.0} to it satisfied a
    // 95% gate regardless of the lcov data sitting beside it. A gate whose
    // evidence can be edited by the thing under test is not evidence.
    let measured = coverage_source::load(project_path)
        .as_ref()
        .and_then(coverage_source::total_percent)
        .or_else(|| read_trend_coverage(project_path));

    Ok(match measured {
        Some(pct) => judge_coverage(pct, threshold),
        None => FalsificationResult::unmeasured(format!(
            "No coverage data (threshold {:.1}%) — {}",
            threshold,
            coverage_source::COVERAGE_HINT
        )),
    })
}

/// Latest recorded coverage percentage from `.pmat-metrics/trends`.
fn read_trend_coverage(project_path: &Path) -> Option<f64> {
    let path = project_path.join(".pmat-metrics/trends/test-coverage.json");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    json.as_array()?.last()?.get("value")?.as_f64()
}

/// Compare a measured coverage percentage against the claim's threshold.
fn judge_coverage(pct: f64, threshold: f64) -> FalsificationResult {
    if pct >= threshold {
        return FalsificationResult::passed(format!("{:.1}% >= {:.1}%", pct, threshold));
    }
    FalsificationResult::failed(
        format!("{:.1}% < {:.1}% threshold", pct, threshold),
        EvidenceType::NumericComparison {
            actual: pct,
            threshold,
        },
    )
}

/// Test TDG score regression
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) async fn test_tdg_regression(
    project_path: &Path,
    baseline_tdg: f64,
) -> Result<FalsificationResult> {
    print!("Checking TDG score... ");

    // Nothing writes .pmat-metrics/tdg-score.json, so that path alone made this
    // claim permanently vacuous. `.pmat/baseline.json` carries the same score
    // under `summary.avg_score` and IS written — by `pmat analyze tdg
    // --update-baseline`, which the pre-commit hook runs on every commit.
    let Some(current_tdg) = read_tdg_score(project_path) else {
        return Ok(FalsificationResult::unmeasured(format!(
            "No TDG data (baseline: {:.1}); run 'pmat analyze tdg --update-baseline'",
            baseline_tdg
        )));
    };

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
}

/// Current project TDG score, from whichever artifact carries one.
///
/// `.pmat-metrics/tdg-score.json` is the documented location but has no writer
/// anywhere; `.pmat/baseline.json` is written by `pmat analyze tdg
/// --update-baseline`, which the pre-commit hook runs on every commit, and
/// records the same figure as `summary.avg_score`.
fn read_tdg_score(project_path: &Path) -> Option<f64> {
    let read_json = |path: PathBuf| -> Option<serde_json::Value> {
        serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
    };

    read_json(project_path.join(".pmat-metrics/tdg-score.json"))
        .and_then(|j| j.get("score").and_then(serde_json::Value::as_f64))
        .or_else(|| {
            read_json(project_path.join(".pmat/baseline.json"))?
                .get("summary")?
                .get("avg_score")?
                .as_f64()
        })
}

/// Test complexity regression: no function should exceed threshold
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn test_complexity_regression(
    project_path: &Path,
    max_complexity: u32,
) -> Result<FalsificationResult> {
    print!("Analyzing function complexity... ");

    // Run pmat complexity check
    // Push the CONTRACT threshold down into the analyzer instead of
    // post-filtering its default output. Without these flags `violations[]`
    // only ever contains functions above pmat's own internal floors
    // (cyclomatic > 10, cognitive > 15), so every contract threshold from 0 to
    // 10 behaved identically to 10: a function at cyclomatic 8 against a
    // contract max of 5 was certified compliant, and at max 0 the gate still
    // reported "All functions <= 0 complexity / PASSED".
    //
    // Deliberately NOT passing --fail-on-violation: it makes the process exit
    // non-zero on a real violation, which lands in the `_ =>` arm below and
    // would report "could not be run" instead of the violation itself.
    //
    // Deliberately NOT reading files[]: it is truncated to top_files_limit
    // (default 10) and ordered by cyclomatic+cognitive sum, not by max, so on
    // this repo it exposes 409 of 39900 functions and hides the worst offender.
    // violations[] is not truncated.
    let max_str = max_complexity.to_string();
    let output = Command::new("pmat")
        .args([
            "analyze",
            "complexity",
            "--format",
            "json",
            "--path",
            &project_path.to_string_lossy(),
            "--max-cyclomatic",
            &max_str,
            "--max-cognitive",
            &max_str,
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
    // Absent `violations` means the shape changed. Fail closed: the file
    // already does this for unparseable JSON, and `unwrap_or_default()` into an
    // empty vec is exactly the silent pass this module keeps growing.
    let Some(violations) = json.get("violations").and_then(|v| v.as_array()) else {
        return FalsificationResult::failed(
            "'pmat analyze complexity' JSON has no violations[]; complexity was not checked"
                .to_string(),
            EvidenceType::BooleanCheck(false),
        );
    };

    // The analyzer emits ONE ROW PER RULE, so a single function over both
    // limits appears twice ("2 function(s) exceed complexity 20: huge, huge").
    // Dedupe by (file, function, line) and keep that function's worst value.
    let mut worst: std::collections::BTreeMap<(String, String, u64), u64> =
        std::collections::BTreeMap::new();
    for v in violations {
        let value = v.get("value").and_then(serde_json::Value::as_u64);
        // A row whose value is missing or not a number is drift, not
        // compliance. Previously `unwrap_or(0)` silently treated it as clean.
        let Some(value) = value else {
            return FalsificationResult::failed(
                "'pmat analyze complexity' violation row has no numeric value; \
                 complexity was not checked"
                    .to_string(),
                EvidenceType::BooleanCheck(false),
            );
        };
        let file = v
            .get("file")
            .and_then(|f| f.as_str())
            .unwrap_or_default()
            .to_string();
        let func = v
            .get("function")
            .and_then(|f| f.as_str())
            .unwrap_or("<unnamed>")
            .to_string();
        let line = v.get("line").and_then(serde_json::Value::as_u64).unwrap_or(0);
        worst
            .entry((file, func, line))
            .and_modify(|w| *w = (*w).max(value))
            .or_insert(value);
    }

    if worst.is_empty() {
        return FalsificationResult::passed(format!(
            "no function exceeds cyclomatic or cognitive complexity {max_complexity}"
        ));
    }

    let peak = worst.values().copied().max().unwrap_or(0);
    let mut names: Vec<String> = worst
        .iter()
        .map(|((_, func, _), v)| format!("{func} ({v})"))
        .collect();
    names.sort();
    let shown = names.len().min(10);
    let suffix = if names.len() > shown {
        format!(", +{} more", names.len() - shown)
    } else {
        String::new()
    };

    FalsificationResult::failed(
        format!(
            "{} function(s) exceed complexity {}: {}{}",
            worst.len(),
            max_complexity,
            names[..shown].join(", "),
            suffix
        ),
        EvidenceType::NumericComparison {
            // `actual` is the worst complexity observed and `threshold` is the
            // limit. Previously this reported actual=<row count> against a
            // hardcoded threshold=0.0, which is why the gate printed
            // "actual=10.0, threshold=0.0" for a limit documented as 20.
            actual: peak as f64,
            threshold: f64::from(max_complexity),
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

/// Parse spec score from pmat output (format: "Score: XX.X/100")
///
/// Returns f64. The previous implementation kept only ASCII digits, which
/// DELETED THE DECIMAL POINT: the scorer formats with `{:.1}`
/// (spec_handlers_scoring.rs), so "Score: 9.6/100" parsed as 96 — every score
/// inflated 10x. Combined with the exit-code bug below, that made the blocking
/// SpecQuality claim unfalsifiable for any threshold <= 950. Empirically, a
/// spec scoring 9.6/100 passed a 95/100 gate.
fn parse_spec_score(stdout: &str) -> Option<f64> {
    let score_line = stdout.lines().find(|l| l.contains("Score:"))?;
    let after = score_line.split("Score:").nth(1)?;
    let num: String = after
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    num.parse::<f64>().ok()
}

/// Evaluate parsed spec score against threshold
fn evaluate_spec_score(score: f64, min_score: u32) -> FalsificationResult {
    let min = f64::from(min_score);
    if score >= min {
        FalsificationResult::passed(format!("{score:.1}/100 >= {min_score}/100"))
    } else {
        FalsificationResult::failed(
            format!("{score:.1}/100 < {min_score}/100 threshold"),
            EvidenceType::NumericComparison {
                actual: score,
                threshold: min,
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
        // No spec to score. This is NOT evidence of quality, so it must not be
        // recorded as a corroborated pass -- `passed()` sets measured:true and
        // inflates the passed tally. `unmeasured()` reports the gap honestly.
        return Ok(FalsificationResult::unmeasured(format!(
            "No spec file at {}",
            spec_path.display()
        )));
    }

    // Run pmat spec score
    let output = Command::new("pmat")
        .args(["spec", "score", &spec_path.to_string_lossy()])
        .current_dir(project_path)
        .output();

    // EXIT CODE 1 IS THE SCORER'S "SPEC FAILED" SIGNAL, NOT "SCORER MISSING".
    // handle_spec_score exits 1 iff score < threshold, and it prints the score
    // on stdout either way. The previous `_ =>` arm mapped that exit status to
    // passed("Spec scorer not available"), so the one signal that means "this
    // spec is bad" was read as "nothing to see here". Proven by an A/B where
    // only the exit code varied: identical stdout, verdict flipped
    // FAILED -> PASSED. This claim is blocking, and it had never been able to
    // fire. Parse stdout on BOTH exit paths and let the score decide.
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            match parse_spec_score(&stdout) {
                Some(score) => Ok(evaluate_spec_score(score, min_score)),
                // Ran but produced no parseable score: a contract change or a
                // crash. Unmeasured, never passed -- an unreadable scorer is
                // not evidence that the spec is good.
                None => Ok(FalsificationResult::unmeasured(format!(
                    "spec scorer produced no parseable score for {}",
                    spec_path.display()
                ))),
            }
        }
        Err(e) => Ok(FalsificationResult::unmeasured(format!(
            "could not run spec scorer for {}: {e}",
            spec_path.display()
        ))),
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

#[cfg(test)]
mod falsifiability_regression_tests {
    use super::*;

    /// The scorer prints one decimal; keeping only ASCII digits deleted the
    /// point and multiplied every score by 10. A 9.6/100 spec then satisfied a
    /// 95/100 gate ("96/100 >= 95/100").
    #[test]
    fn spec_score_keeps_the_decimal_point() {
        assert_eq!(parse_spec_score("Score: 9.6/100"), Some(9.6));
        assert_eq!(parse_spec_score("Score: 100.0/100"), Some(100.0));
        assert_eq!(parse_spec_score("Score: 5.0/100\nStatus: FAIL"), Some(5.0));
        assert_eq!(parse_spec_score("no score here"), None);
    }

    /// The whole point of the claim: a bad spec must be falsifiable.
    #[test]
    fn a_bad_spec_is_falsified() {
        assert!(evaluate_spec_score(9.6, 95).falsified, "9.6 must fail 95");
        assert!(evaluate_spec_score(94.9, 95).falsified);
        assert!(!evaluate_spec_score(95.0, 95).falsified);
    }

    fn violation(rule: &str, func: &str, value: u64, line: u64) -> serde_json::Value {
        serde_json::json!({
            "rule": rule, "function": func, "value": value,
            "file": "src/lib.rs", "line": line
        })
    }

    /// One function over both limits emitted two rows and was reported twice
    /// ("2 function(s) exceed complexity 20: huge, huge").
    #[test]
    fn one_function_counts_once_across_rules() {
        let json = serde_json::json!({"violations": [
            violation("cyclomatic-complexity", "huge", 81, 10),
            violation("cognitive-complexity", "huge", 120, 10),
        ]});
        let r = evaluate_complexity_json(&json, 20);
        assert!(r.falsified);
        assert!(
            r.explanation.starts_with("1 function(s)"),
            "expected 1 function, got: {}",
            r.explanation
        );
    }

    /// Evidence used to report actual=<row count> against a hardcoded
    /// threshold=0.0, so a limit of 20 printed "actual=10.0, threshold=0.0".
    #[test]
    fn evidence_reports_worst_value_against_the_real_threshold() {
        let json = serde_json::json!({"violations": [
            violation("cyclomatic-complexity", "a", 25, 1),
            violation("cyclomatic-complexity", "b", 40, 2),
        ]});
        let r = evaluate_complexity_json(&json, 20);
        match r.evidence {
            Some(EvidenceType::NumericComparison { actual, threshold }) => {
                assert!((actual - 40.0).abs() < f64::EPSILON, "worst value");
                assert!((threshold - 20.0).abs() < f64::EPSILON, "real threshold");
            }
            other => panic!("expected NumericComparison, got {other:?}"),
        }
    }

    /// Missing or malformed analyzer output must fail closed. Previously
    /// `unwrap_or_default()` turned it into an empty violation list and passed.
    #[test]
    fn malformed_analyzer_output_fails_closed() {
        let no_key = serde_json::json!({"summary": {}});
        assert!(evaluate_complexity_json(&no_key, 20).falsified);

        let bad_value = serde_json::json!({"violations": [
            {"rule": "cyclomatic-complexity", "function": "x", "value": "not-a-number"}
        ]});
        assert!(evaluate_complexity_json(&bad_value, 20).falsified);
    }

    #[test]
    fn clean_project_still_passes() {
        let json = serde_json::json!({"violations": []});
        let r = evaluate_complexity_json(&json, 20);
        assert!(!r.falsified);
    }
}
