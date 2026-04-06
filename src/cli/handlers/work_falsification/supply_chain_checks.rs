#![cfg_attr(coverage_nightly, coverage(off))]
//! Supply chain, examples, book validation, cross-crate, and regression gate checks.

use super::cache::{read_cached_metric, read_deny_cache_fallback};
use crate::cli::handlers::work_contract::{EvidenceType, FalsificationResult};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Test supply chain integrity: O(1) - reads from cached cargo deny status
pub(crate) async fn test_supply_chain_integrity(
    project_path: &Path,
) -> Result<FalsificationResult> {
    print!("Reading deny cache... ");

    // O(1): Read from cache instead of running cargo deny
    // Try primary cache location, then fallback to work-item and .pmat directories
    if let Some(cache) = read_cached_metric(project_path, "deny-status.json")
        .or_else(|| read_deny_cache_fallback(project_path))
    {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!(
                    "Deny cache too old ({} min). Run 'cargo deny check' first.",
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
                    "Invalid deny cache (missing 'passed' field). Re-run 'cargo deny check'."
                        .to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let stale_note = format!(" (cached {} min ago)", cache.age_minutes);

        if passed {
            return Ok(FalsificationResult::passed(format!(
                "No vulnerabilities{}",
                stale_note
            )));
        } else {
            let count = cache
                .value
                .get("vulnerability_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Ok(FalsificationResult::failed(
                format!("{} vulnerabilities{}", count, stale_note),
                EvidenceType::NumericComparison {
                    actual: count as f64,
                    threshold: 0.0,
                },
            ));
        }
    }

    // No cache - FAIL (Popperian Audit v1.2 fix: empty cache bypass)
    Ok(FalsificationResult::failed(
        "No deny cache. Run 'cargo deny check' first (O(1) requirement)".to_string(),
        EvidenceType::BooleanCheck(false),
    ))
}

/// Test examples compile: O(1) - reads from cached examples status
pub(crate) async fn test_examples_compile(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading examples cache... ");

    // O(1): Read from cache instead of running cargo build
    if let Some(cache) = read_cached_metric(project_path, "examples-status.json") {
        if cache.is_stale_block {
            return Ok(FalsificationResult::failed(
                format!(
                    "Examples cache too old ({} min). Run 'cargo build --examples' first.",
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
                    "Invalid examples cache (missing 'passed' field). Re-run 'cargo build --examples'.".to_string(),
                    EvidenceType::BooleanCheck(false),
                ));
            }
        };
        let count = cache
            .value
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let stale_note = format!(" (cached {} min ago)", cache.age_minutes);

        if passed {
            return Ok(FalsificationResult::passed(format!(
                "{} examples OK{}",
                count, stale_note
            )));
        } else {
            let failed = cache
                .value
                .get("failed")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            return Ok(FalsificationResult::failed(
                format!("Examples failed{}: {}", stale_note, failed),
                EvidenceType::CounterExample { details: failed },
            ));
        }
    }

    // Check if examples directory exists
    let examples_dir = project_path.join("examples");
    if !examples_dir.exists() {
        return Ok(FalsificationResult::passed(
            "No examples directory found (skipping)".to_string(),
        ));
    }

    // No cache available - pass with warning (examples are optional)
    Ok(FalsificationResult::passed(
        "No examples cache (run 'cargo build --examples' to populate)".to_string(),
    ))
}

/// Run `make validate-book` and return the result.
///
/// Returns `Some(result)` if the command executed (pass or fail),
/// or `None` if `make` was not available.
fn try_make_validate_book(project_path: &Path) -> Option<FalsificationResult> {
    let output = Command::new("make")
        .args(["validate-book"])
        .current_dir(project_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(FalsificationResult::passed(
            "pmat-book validation passed".to_string(),
        ))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Some(FalsificationResult::failed(
            "pmat-book validation failed".to_string(),
            EvidenceType::CounterExample {
                details: stderr.chars().take(500).collect(),
            },
        ))
    }
}

/// Run pmat-book chapter test script as a fallback.
///
/// Returns `Some(result)` if the script exists and executed,
/// or `None` if not available.
fn try_book_chapter_tests(book_path: &Path) -> Option<FalsificationResult> {
    let test_script = book_path.join("tests/ch13/test_language_examples.sh");
    if !test_script.exists() {
        return None;
    }

    let output = Command::new("bash")
        .arg(&test_script)
        .current_dir(book_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(FalsificationResult::passed(
            "pmat-book chapter tests passed".to_string(),
        ))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Some(FalsificationResult::failed(
            "pmat-book chapter tests failed".to_string(),
            EvidenceType::CounterExample {
                details: stderr.chars().take(500).collect(),
            },
        ))
    }
}

/// Test pmat-book validation: book tests must pass
pub(crate) async fn test_book_validation(project_path: &Path) -> Result<FalsificationResult> {
    print!("Validating pmat-book... ");

    // Check if this is the pmat repository by looking for pmat-book sibling
    let book_path = match project_path.parent().map(|p| p.join("pmat-book")) {
        Some(p) if p.exists() => p,
        _ => {
            return Ok(FalsificationResult::passed(
                "pmat-book not found (skipping validation)".to_string(),
            ));
        }
    };

    // Try make validate-book first, then fall back to chapter test script
    if let Some(result) = try_make_validate_book(project_path) {
        return Ok(result);
    }

    if let Some(result) = try_book_chapter_tests(&book_path) {
        return Ok(result);
    }

    // No validation method available
    Ok(FalsificationResult::passed(
        "pmat-book not found (skipping validation)".to_string(),
    ))
}

/// Test cross-crate parity: verify sibling project tests still pass after changes.
///
/// Reads `.pmat-work/cross-crate.json` config for sibling project paths and test commands.
/// Only runs if config exists (opt-in per project).
pub(crate) async fn test_cross_crate_parity(project_path: &Path) -> Result<FalsificationResult> {
    print!("Checking cross-crate config... ");

    // Look for cross-crate config
    let config_path = project_path.join(".pmat-work/cross-crate.json");

    if !config_path.exists() {
        return Ok(FalsificationResult::passed(
            "No cross-crate config (create .pmat-work/cross-crate.json to enable)".to_string(),
        ));
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    let Some(projects) = config.get("projects").and_then(|p| p.as_array()) else {
        return Ok(FalsificationResult::passed(
            "No projects in cross-crate config".to_string(),
        ));
    };

    let mut failures = Vec::new();
    let mut passed_count = 0;

    for project in projects {
        let Some(path) = project.get("path").and_then(|p| p.as_str()) else {
            continue;
        };
        let test_cmd = project
            .get("test_command")
            .and_then(|c| c.as_str())
            .unwrap_or("cargo test --lib");

        let full_path = project_path.join(path);
        if !full_path.exists() {
            continue;
        }

        // Split test command into program + args
        let parts: Vec<&str> = test_cmd.split_whitespace().collect();
        let (program, args) = parts.split_first().unwrap_or((&"cargo", &[]));

        let output = Command::new(program)
            .args(args)
            .current_dir(&full_path)
            .output();

        match output {
            Ok(out) if out.status.success() => passed_count += 1,
            Ok(_) => failures.push(path.to_string()),
            Err(_) => failures.push(format!("{} (command failed)", path)),
        }
    }

    if failures.is_empty() {
        Ok(FalsificationResult::passed(format!(
            "{} sibling project(s) pass",
            passed_count
        )))
    } else {
        let paths: Vec<PathBuf> = failures.iter().map(PathBuf::from).collect();
        Ok(FalsificationResult::failed(
            format!(
                "{} sibling project(s) failed: {}",
                failures.len(),
                failures.join(", ")
            ),
            EvidenceType::FileList(paths),
        ))
    }
}

/// Test regression gate: verify no performance regressions from cached benchmarks.
///
/// Reads `.pmat-metrics/benchmark-status.json` for cached benchmark results.
pub(crate) async fn test_regression_gate(project_path: &Path) -> Result<FalsificationResult> {
    print!("Reading benchmark cache... ");

    // O(1): Read from cache
    if let Some(cache) = read_cached_metric(project_path, "benchmark-status.json") {
        if cache.is_stale_block {
            return Ok(FalsificationResult::passed(format!(
                "Benchmark cache old ({} min), skipping",
                cache.age_minutes
            )));
        }

        let passed = cache
            .value
            .get("passed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let stale_note = format!(" (cached {} min ago)", cache.age_minutes);

        if passed {
            let benchmarks = cache
                .value
                .get("count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Ok(FalsificationResult::passed(format!(
                "{} benchmark(s) OK{}",
                benchmarks, stale_note
            )));
        } else {
            let regressions = cache
                .value
                .get("regressions")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            return Ok(FalsificationResult::failed(
                format!("Performance regression{}: {}", stale_note, regressions),
                EvidenceType::CounterExample {
                    details: regressions,
                },
            ));
        }
    }

    // No cache -- skip gracefully
    Ok(FalsificationResult::passed(
        "No benchmark cache (run benchmarks to populate .pmat-metrics/benchmark-status.json)"
            .to_string(),
    ))
}

/// Test meta-falsification: verify the falsifier itself is not broken
pub(crate) fn test_meta_falsification(project_path: &Path) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    print!("Injecting dummy failure... ");

    let detector_working = crate::services::gaming_detector::run_meta_falsification(project_path)?;

    if detector_working {
        Ok(FalsificationResult::passed(
            "Detected dummy gaming pattern correctly".to_string(),
        ))
    } else {
        Ok(FalsificationResult::failed(
            "Falsifier FAILED to detect known gaming pattern (SYSTEM BROKEN)".to_string(),
            EvidenceType::CounterExample {
                details: "Dummy #[cfg(not(coverage))] was ignored by detector".into(),
            },
        ))
    }
}
