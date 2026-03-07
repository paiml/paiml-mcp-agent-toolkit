#![cfg_attr(coverage_nightly, coverage(off))]

use super::super::types::QueryResult;
use super::enrichment::enrich_with_coverage;
use super::parsing::build_coverage_map;
use super::profdata::{
    load_coverage_from_cache, run_cargo_llvm_cov_and_cache, write_negative_coverage_cache,
};
use super::types::CoverageCache;
use std::collections::HashMap;
use std::path::Path;

// ── Explicit Path / Env Loading ─────────────────────────────────────────────

/// Load coverage from an explicit file path.
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::type_complexity)]
fn try_load_coverage_from_explicit_path(
    path: &Path,
    project_root: &Path,
) -> Result<Option<HashMap<String, HashMap<usize, u64>>>, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read coverage file {}: {e}", path.display()))?;
    Ok(Some(build_coverage_map(&json, project_root)?))
}

/// Load coverage from `PMAT_COVERAGE_FILE` environment variable.
#[allow(clippy::type_complexity)]
fn try_load_coverage_from_env(
    project_root: &Path,
) -> Result<Option<HashMap<String, HashMap<usize, u64>>>, String> {
    let env_path = match std::env::var("PMAT_COVERAGE_FILE") {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    let path = std::path::PathBuf::from(&env_path);
    if !path.exists() {
        return Ok(None);
    }
    try_load_coverage_from_explicit_path(&path, project_root)
}

// ── Async Convenience (follows enrich_results_with_churn pattern) ───────────

/// Enrich results with LLVM coverage data.
///
/// Resolution order for coverage JSON:
/// 1. Explicit `coverage_file` parameter (from --coverage-file CLI arg)
/// 2. `PMAT_COVERAGE_FILE` environment variable
/// 3. `.pmat/coverage-cache.json` (if git HEAD matches)
/// 4. Run `cargo llvm-cov report --json` to generate fresh data
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn enrich_results_with_coverage(
    results: &mut [QueryResult],
    project_root: &Path,
    coverage_file: Option<&Path>,
) -> Result<(), String> {
    use std::process::Command;

    if results.is_empty() {
        return Ok(());
    }

    // 1. Explicit file, 2. Env var
    let file_coverage = if let Some(path) = coverage_file {
        try_load_coverage_from_explicit_path(path, project_root)?
    } else {
        try_load_coverage_from_env(project_root)?
    };
    if let Some(cov) = file_coverage {
        enrich_with_coverage(results, &cov);
        return Ok(());
    }

    // 3. Cache, 4. Run cargo llvm-cov
    let cache_path = project_root.join(".pmat/coverage-cache.json");
    let head_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_root)
        .output()
        .map_err(|e| format!("git rev-parse failed: {e}"))?;
    let head_hash = String::from_utf8_lossy(&head_hash.stdout)
        .trim()
        .to_string();

    // Check cache first
    if let Some(cached) = load_coverage_from_cache(&cache_path, &head_hash, project_root) {
        if cached.is_empty() {
            // Negative cache hit: previous attempt found no coverage data.
            // Invalidated when git hash or profdata mtime changes.
            return Err(
                "No coverage data available (cached from previous attempt).\n\n\
                To generate it, run:\n  \
                cargo llvm-cov test --lib --no-report\n\n\
                Then re-run with --coverage-gaps.\n\
                Or pass --coverage-file <path> to use existing coverage JSON."
                    .to_string(),
            );
        }
        enrich_with_coverage(results, &cached);
        return Ok(());
    }

    // Cache miss -- run cargo llvm-cov (writes positive cache on success)
    match run_cargo_llvm_cov_and_cache(project_root, &cache_path, &head_hash) {
        Ok(cov) => {
            enrich_with_coverage(results, &cov);
            Ok(())
        }
        Err(e) => {
            // Negative cache: avoid 30s subprocess retry on next invocation (#212).
            // Invalidated when git hash or profdata mtime changes.
            write_negative_coverage_cache(&cache_path, &head_hash, project_root);
            Err(e)
        }
    }
}

/// Load and merge coverage caches from sibling projects for workspace-level coverage gaps.
///
/// Each sibling's `.pmat/coverage-cache.json` is loaded independently.
/// File paths are prefixed with the project name (matching `load_with_prefix()`).
/// Siblings without a cache are silently skipped.
pub fn load_workspace_coverage(
    siblings: &[(std::path::PathBuf, String)],
) -> HashMap<String, HashMap<usize, u64>> {
    let mut merged: HashMap<String, HashMap<usize, u64>> = HashMap::new();

    for (idx_path, project_name) in siblings {
        // idx_path points to .pmat/context.idx; coverage cache is at .pmat/coverage-cache.json
        let pmat_dir = idx_path.parent().unwrap_or(Path::new("."));
        let cache_path = pmat_dir.join("coverage-cache.json");
        let cache_json = match std::fs::read_to_string(&cache_path) {
            Ok(j) => j,
            Err(_) => continue, // No cache for this sibling
        };
        let cache: CoverageCache = match serde_json::from_str(&cache_json) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Prefix file paths with project name
        for (file_path, line_hits) in cache.files {
            let prefixed = format!("{}/{}", project_name, file_path);
            merged.insert(prefixed, line_hits);
        }
    }

    merged
}
