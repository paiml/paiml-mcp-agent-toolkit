#![cfg_attr(coverage_nightly, coverage(off))]
//! Cache reading utilities for O(1) falsification checks.

use super::types::{CachedMetric, CACHE_BLOCK_HOURS, CACHE_WARN_HOURS};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Read a cached metric from .pmat-metrics/
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn read_cached_metric(project_path: &Path, filename: &str) -> Option<CachedMetric> {
    let cache_path = project_path.join(".pmat-metrics").join(filename);
    if !cache_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&cache_path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Check file modification time for staleness
    let metadata = std::fs::metadata(&cache_path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = std::time::SystemTime::now().duration_since(modified).ok()?;
    let age_minutes = age.as_secs() as i64 / 60;
    let age_hours = age_minutes / 60;

    Some(CachedMetric {
        value,
        age_minutes,
        is_stale_warn: age_hours >= CACHE_WARN_HOURS,
        is_stale_block: age_hours >= CACHE_BLOCK_HOURS,
    })
}

/// Fallback: try reading deny cache from .pmat-work/<item>/ or .pmat/ directories.
/// Converts raw text output to the expected JSON format with `passed` field.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn read_deny_cache_fallback(project_path: &Path) -> Option<CachedMetric> {
    // Try .pmat-work/**/deny-cache.txt then .pmat/deny-cache.txt
    let candidates = find_cache_file(project_path, "deny-cache.txt");
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            // `FAILED` is how cargo-deny reports a failing check on stdout
            // (`advisories FAILED`); without it, a stdout-only capture of a
            // failing run scored as passing.
            let passed = !content.contains("error")
                && !content.contains("DENIED")
                && !content.contains("FAILED");
            let age_minutes = file_age_minutes(&path);
            return Some(CachedMetric {
                value: serde_json::json!({ "passed": passed }),
                age_minutes,
                is_stale_warn: age_minutes >= CACHE_WARN_HOURS * 60,
                is_stale_block: age_minutes >= CACHE_BLOCK_HOURS * 60,
            });
        }
    }
    None
}

/// Fallback: try reading lint cache from .pmat-work/<item>/ or .pmat/ directories.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn read_lint_cache_fallback(project_path: &Path) -> Option<CachedMetric> {
    let candidates = find_cache_file(project_path, "lint-cache.txt");
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let passed = !content.contains("error");
            let error_count = content.matches("error").count() as u64;
            let age_minutes = file_age_minutes(&path);
            return Some(CachedMetric {
                value: serde_json::json!({ "passed": passed, "error_count": error_count }),
                age_minutes,
                is_stale_warn: age_minutes >= CACHE_WARN_HOURS * 60,
                is_stale_block: age_minutes >= CACHE_BLOCK_HOURS * 60,
            });
        }
    }
    None
}

/// Find cache file candidates in .pmat-work/*/ and .pmat/ directories.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn find_cache_file(project_path: &Path, filename: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Check .pmat-work/*/<filename> (most specific, sorted by mtime desc)
    let work_dir = project_path.join(".pmat-work");
    if work_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&work_dir) {
            let mut work_candidates: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path().join(filename))
                .filter(|p| p.exists())
                .collect();
            // Sort by mtime descending (most recent first)
            work_candidates.sort_by(|a, b| {
                let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
                let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
                b_time.cmp(&a_time)
            });
            candidates.extend(work_candidates);
        }
    }

    // Check .pmat/<filename>
    let pmat_path = project_path.join(".pmat").join(filename);
    if pmat_path.exists() {
        candidates.push(pmat_path);
    }

    candidates
}

/// Get file age in minutes.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn file_age_minutes(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
        .map(|d| d.as_secs() as i64 / 60)
        .unwrap_or(0)
}

/// Capture baseline metrics for a new work contract
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn capture_baseline(project_path: &Path) -> Result<(f64, f64, Option<f64>)> {
    println!("   \u{1f4ca} Capturing baseline metrics...");

    // Capture TDG score
    let tdg_score = capture_metric_from_cache(project_path, "tdg-score.json", "score")
        .await
        .unwrap_or(0.0);

    // Capture coverage
    let coverage = capture_coverage_from_cache(project_path)
        .await
        .unwrap_or(0.0);

    // Capture Rust project score (if applicable)
    let rust_score = if project_path.join("Cargo.toml").exists() {
        Some(
            capture_metric_from_cache(project_path, "rust-project-score.json", "total_earned")
                .await
                .unwrap_or(0.0),
        )
    } else {
        None
    };

    println!("      TDG: {:.1}, Coverage: {:.1}%", tdg_score, coverage);
    if let Some(rs) = rust_score {
        println!("      Rust Score: {:.1}/134", rs);
    }

    Ok((tdg_score, coverage, rust_score))
}

/// Capture a metric from the cache
async fn capture_metric_from_cache(
    project_path: &Path,
    filename: &str,
    field: &str,
) -> Result<f64> {
    let metrics_dir = project_path.join(".pmat-metrics");
    let file_path = metrics_dir.join(filename);

    if file_path.exists() {
        let content = std::fs::read_to_string(&file_path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(value) = json.get(field).and_then(|v| v.as_f64()) {
            return Ok(value);
        }
    }

    Ok(0.0)
}

/// Capture coverage from trends cache
async fn capture_coverage_from_cache(project_path: &Path) -> Result<f64> {
    let coverage_file = project_path.join(".pmat-metrics/trends/test-coverage.json");

    if coverage_file.exists() {
        let content = std::fs::read_to_string(&coverage_file)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(entries) = json.as_array() {
            if let Some(latest) = entries.last() {
                if let Some(coverage) = latest.get("value").and_then(|v| v.as_f64()) {
                    return Ok(coverage);
                }
            }
        }
    }

    Ok(0.0)
}
