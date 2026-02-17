#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::QueryResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ── LLVM Coverage Export JSON Structs ───────────────────────────────────────

/// Top-level LLVM coverage export format (cargo llvm-cov export --format=json)
#[derive(Debug, Deserialize)]
struct LlvmCoverageExport {
    data: Vec<LlvmCoverageData>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    export_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LlvmCoverageData {
    files: Vec<LlvmFileCoverage>,
}

#[derive(Debug, Deserialize)]
struct LlvmFileCoverage {
    filename: String,
    segments: Vec<Vec<serde_json::Value>>,
    #[allow(dead_code)]
    summary: Option<LlvmSummary>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LlvmSummary {
    lines: Option<LlvmLineSummary>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct LlvmLineSummary {
    count: u32,
    covered: u32,
}

// ── Coverage Cache ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct CoverageCache {
    git_hash: String,
    /// mtime (seconds since epoch) of profdata source when cache was built
    #[serde(default)]
    coverage_mtime: Option<u64>,
    /// Path to the llvm-cov-target directory used when cache was built
    #[serde(default)]
    profdata_dir: Option<String>,
    files: HashMap<String, HashMap<usize, u64>>,
}

// ── Segment Walking ─────────────────────────────────────────────────────────

/// Walk LLVM segments to build line→exec_count map.
///
/// Segments encode coverage state transitions as:
/// `[line, col, count, has_count, is_region_entry, ...]`
///
/// We track the current execution count and assign it to each line
/// in the range between consecutive segments.
pub(super) fn segments_to_line_hits(segments: &[Vec<serde_json::Value>]) -> HashMap<usize, u64> {
    let mut line_hits: HashMap<usize, u64> = HashMap::new();

    if segments.is_empty() {
        return line_hits;
    }

    // Walk segments pairwise: each segment starts a region with its count
    for i in 0..segments.len() {
        let seg = &segments[i];
        if seg.len() < 4 {
            continue;
        }

        let line = seg[0].as_u64().unwrap_or(0) as usize;
        let count = seg[2].as_u64().unwrap_or(0);
        let has_count = seg[3]
            .as_bool()
            .or_else(|| seg[3].as_u64().map(|v| v != 0))
            .unwrap_or(false);

        if !has_count {
            continue;
        }

        // Determine the end line for this segment
        let end_line = if i + 1 < segments.len() {
            let next = &segments[i + 1];
            next[0].as_u64().unwrap_or(line as u64) as usize
        } else {
            line
        };

        // Fill lines from this segment to just before the next
        for l in line..=end_line {
            let entry = line_hits.entry(l).or_insert(0);
            *entry = (*entry).max(count);
        }
    }

    line_hits
}

// ── Coverage Map Builder ────────────────────────────────────────────────────

/// Parse full LLVM coverage export into per-file line hit maps.
///
/// File paths are normalized to be relative to the project root.
pub fn build_coverage_map(
    json: &str,
    project_root: &Path,
) -> Result<HashMap<String, HashMap<usize, u64>>, String> {
    let export: LlvmCoverageExport = serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse LLVM coverage JSON: {e}"))?;

    let mut coverage_map: HashMap<String, HashMap<usize, u64>> = HashMap::new();
    let canonical_root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let root_lossy = canonical_root.to_string_lossy();
    let root_str = root_lossy.trim_end_matches('/');

    // Also try the non-canonicalized root (handles symlinks, bind mounts)
    let raw_lossy = project_root.to_string_lossy();
    let raw_root_str = raw_lossy.trim_end_matches('/');

    for data in &export.data {
        for file in &data.files {
            let line_hits = segments_to_line_hits(&file.segments);
            if line_hits.is_empty() {
                continue;
            }

            // Normalize path to project-relative (try canonical root first, then raw)
            // Skip files outside the project (dependency sources, registry crates, etc.)
            let rel_path = if file.filename.starts_with(root_str) {
                file.filename
                    .get(root_str.len()..)
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string()
            } else if file.filename.starts_with(raw_root_str) {
                file.filename
                    .get(raw_root_str.len()..)
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string()
            } else {
                // File is outside project root — skip it (deps, registry, etc.)
                continue;
            };

            coverage_map.insert(rel_path, line_hits);
        }
    }

    Ok(coverage_map)
}

// ── Pure Enrichment Function ────────────────────────────────────────────────

/// Find coverage data for a file path, trying exact match then suffix matching.
///
/// Returns `None` if no coverage entry matches the given file path.
fn find_coverage_for_file<'a>(
    file_path: &str,
    file_coverage: &'a HashMap<String, HashMap<usize, u64>>,
) -> Option<&'a HashMap<usize, u64>> {
    // Try exact match first
    if let Some(hits) = file_coverage.get(file_path) {
        return Some(hits);
    }
    // Fallback: find coverage entry whose key ends with our file_path or vice versa
    file_coverage
        .iter()
        .find(|(k, _)| k.ends_with(file_path) || file_path.ends_with(k.as_str()))
        .map(|(_, v)| v)
}

/// Determine coverage status string from total and covered line counts.
///
/// Returns one of: `"no_data"`, `"uncovered"`, `"full"`, or `"partial"`.
fn determine_coverage_status(total: u32, covered: u32) -> &'static str {
    if total == 0 {
        "no_data"
    } else if covered == 0 {
        "uncovered"
    } else if covered == total {
        "full"
    } else {
        "partial"
    }
}

/// Annotate coverage gaps as fault patterns on a single result.
///
/// Missing coverage is a latent defect signal — uncovered code paths are
/// where bugs hide. By surfacing these as fault annotations, `--coverage`
/// composes naturally with `--faults` to show "fix the defect AND write
/// the test" opportunities in a single view.
fn annotate_coverage_faults(result: &mut QueryResult) {
    match result.coverage_status.as_str() {
        "uncovered" => {
            result.fault_annotations.push(format!(
                "NO_COVERAGE: 0/{} lines covered — untested code path",
                result.lines_total
            ));
        }
        "partial" if result.line_coverage_pct < 50.0 => {
            result.fault_annotations.push(format!(
                "LOW_COVERAGE: {:.0}% ({}/{} lines) — {} missed lines need tests",
                result.line_coverage_pct,
                result.lines_covered,
                result.lines_total,
                result.missed_lines
            ));
        }
        _ => {}
    }
    if result.impact_score > 5.0 {
        result.fault_annotations.push(format!(
            "COVERAGE_RISK: impact {:.1} — high-ROI test target (missed:{} x pagerank)",
            result.impact_score, result.missed_lines
        ));
    }
}

/// Enrich query results with coverage data (pure function).
///
/// For each result, intersects `start_line..=end_line` with the file's
/// line_hits map:
/// - Lines in line_hits with count > 0 → covered
/// - Lines in line_hits with count == 0 → uncovered
/// - Lines NOT in line_hits → non-instrumented (excluded from total)
///
/// Sets `coverage_status` to:
/// - `"no_data"` — file not in coverage map at all
/// - `"uncovered"` — file instrumented, 0 lines covered
/// - `"partial"` — some lines covered, some not
/// - `"full"` — all instrumented lines covered
pub fn enrich_with_coverage(
    results: &mut [QueryResult],
    file_coverage: &HashMap<String, HashMap<usize, u64>>,
) {
    for result in results.iter_mut() {
        let line_hits = match find_coverage_for_file(&result.file_path, file_coverage) {
            Some(hits) => hits,
            None => {
                result.coverage_status = "no_data".to_string();
                continue;
            }
        };

        let mut covered = 0u32;
        let mut total = 0u32;

        for line in result.start_line..=result.end_line {
            if let Some(&count) = line_hits.get(&line) {
                total += 1;
                if count > 0 {
                    covered += 1;
                }
            }
        }

        result.lines_covered = covered;
        result.lines_total = total;
        result.missed_lines = total.saturating_sub(covered);
        result.line_coverage_pct = if total > 0 {
            (covered as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        result.impact_score =
            compute_impact_score(result.missed_lines, result.pagerank, result.complexity);

        result.coverage_status = determine_coverage_status(total, covered).to_string();

        // Annotate coverage gaps as fault patterns (missing coverage = latent defect signal)
        annotate_coverage_faults(result);
    }
}

/// Compute coverage delta: enriches results with `coverage_diff` field showing
/// change from baseline. Positive = coverage improved, negative = regressed.
pub fn enrich_with_coverage_diff(
    results: &mut [QueryResult],
    baseline_coverage: &HashMap<String, HashMap<usize, u64>>,
) {
    for result in results.iter_mut() {
        let baseline_hits = match baseline_coverage.get(&result.file_path) {
            Some(hits) => hits,
            None => continue, // no baseline data for this file
        };

        let mut base_covered = 0u32;
        let mut base_total = 0u32;

        for line in result.start_line..=result.end_line {
            if let Some(&count) = baseline_hits.get(&line) {
                base_total += 1;
                if count > 0 {
                    base_covered += 1;
                }
            }
        }

        let base_pct = if base_total > 0 {
            base_covered as f32 / base_total as f32 * 100.0
        } else {
            0.0
        };

        // Store delta as current - baseline
        result.coverage_diff = result.line_coverage_pct - base_pct;
    }
}

/// Format a coverage summary line for display after enriched results.
///
/// Returns None if no results have coverage data.
pub fn format_coverage_summary(results: &[QueryResult]) -> Option<String> {
    let with_data: Vec<_> = results
        .iter()
        .filter(|r| r.coverage_status != "no_data" && !r.coverage_status.is_empty())
        .collect();

    if with_data.is_empty() {
        return None;
    }

    let total_covered: u32 = with_data.iter().map(|r| r.lines_covered).sum();
    let total_lines: u32 = with_data.iter().map(|r| r.lines_total).sum();
    let total_pct = if total_lines > 0 {
        total_covered as f64 / total_lines as f64 * 100.0
    } else {
        0.0
    };

    let uncovered_count = with_data
        .iter()
        .filter(|r| r.coverage_status == "uncovered")
        .count();
    let partial_count = with_data
        .iter()
        .filter(|r| r.coverage_status == "partial")
        .count();

    let top_impact = with_data.iter().max_by(|a, b| {
        a.impact_score
            .partial_cmp(&b.impact_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let pct_color = if total_pct >= 80.0 {
        "\x1b[32m"
    } else if total_pct >= 50.0 {
        "\x1b[33m"
    } else {
        "\x1b[1;31m"
    };
    let mut summary = format!(
        "Coverage: {}/{} lines ({}{:.1}%\x1b[0m) across {} functions",
        total_covered,
        total_lines,
        pct_color,
        total_pct,
        with_data.len()
    );

    if uncovered_count > 0 {
        summary.push_str(&format!(
            " | \x1b[1;31m{} uncovered\x1b[0m",
            uncovered_count
        ));
    }
    if partial_count > 0 {
        summary.push_str(&format!(" | \x1b[33m{} partial\x1b[0m", partial_count));
    }

    if let Some(top) = top_impact {
        if top.impact_score > 0.0 {
            summary.push_str(&format!(
                " | \x1b[1;33mTop impact: {} ({:.1})\x1b[0m",
                top.function_name, top.impact_score
            ));
        }
    }

    Some(summary)
}

// ── Impact Score ────────────────────────────────────────────────────────────

/// ROI formula: missed_lines * pagerank_scaled * (1 / complexity_factor)
///
/// Higher impact = more uncovered lines in important, low-complexity code
/// (i.e., easy wins for coverage improvement).
pub fn compute_impact_score(missed_lines: u32, pagerank: f32, complexity: u32) -> f32 {
    if missed_lines == 0 {
        return 0.0;
    }
    let pr_factor = (pagerank * 10000.0).max(0.1);
    let complexity_factor = (complexity as f32).max(1.0);
    missed_lines as f32 * pr_factor / complexity_factor
}

// ── Async Convenience (follows enrich_results_with_churn pattern) ───────────

/// Try to load coverage from the cache file.
///
/// Invalidation strategy (profdata-mtime-primary):
/// 1. If profdata dir found and mtime matches cached mtime → VALID (regardless of git hash)
/// 2. If profdata dir found but mtime differs → INVALID (profdata was regenerated)
/// 3. If no profdata dir found → fall back to git hash comparison
#[cfg_attr(coverage_nightly, coverage(off))]
fn load_coverage_from_cache(
    cache_path: &Path,
    head_hash: &str,
    project_root: &Path,
) -> Option<HashMap<String, HashMap<usize, u64>>> {
    let cache_json = std::fs::read_to_string(cache_path).ok()?;
    let cache: CoverageCache = serde_json::from_str(&cache_json).ok()?;

    // Primary: profdata mtime comparison (handles custom target dirs, symlinks)
    if let Some(cached_mtime) = cache.coverage_mtime {
        if let Some((current_mtime, _)) =
            get_profdata_mtime_and_dir(project_root, cache.profdata_dir.as_deref())
        {
            if current_mtime <= cached_mtime {
                return Some(cache.files); // profdata unchanged → cache valid
            }
            return None; // profdata was regenerated
        }
    }

    // Fallback: git hash (only when profdata mtime unavailable)
    if cache.git_hash != head_hash {
        return None;
    }

    Some(cache.files)
}

/// Get the mtime (seconds since epoch) of a specific directory.
#[cfg_attr(coverage_nightly, coverage(off))]
fn dir_mtime(dir: &Path) -> Option<u64> {
    std::fs::metadata(dir)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

/// Extract target-dir from a cargo config TOML file (simple line-based parsing).
#[cfg_attr(coverage_nightly, coverage(off))]
fn target_dir_from_cargo_config(
    config_path: &Path,
    project_root: &Path,
) -> Vec<std::path::PathBuf> {
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("target-dir") {
                return None;
            }
            let val = trimmed.split('=').nth(1)?;
            let dir = val.trim().trim_matches('"').trim_matches('\'');
            let target_path = if std::path::Path::new(dir).is_absolute() {
                std::path::PathBuf::from(dir)
            } else {
                project_root.join(dir)
            };
            Some(target_path.join("llvm-cov-target"))
        })
        .collect()
}

/// Scan /mnt/*/targets/{project_name}/llvm-cov-target for NVMe/RAID overrides.
///
/// Shell functions that wrap `cargo` may set CARGO_TARGET_DIR to NVMe paths,
/// but Command::new("cargo") bypasses shell functions. This heuristic finds
/// those directories directly.
#[cfg_attr(coverage_nightly, coverage(off))]
fn mnt_target_candidates(project_root: &Path) -> Vec<std::path::PathBuf> {
    let canonical = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let project_name = match canonical.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return vec![],
    };
    let entries = match std::fs::read_dir("/mnt") {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    entries
        .flatten()
        .map(|e| {
            e.path()
                .join("targets")
                .join(project_name)
                .join("llvm-cov-target")
        })
        .collect()
}

/// Fast profdata discovery: same as `get_profdata_mtime_and_dir()` but skips
/// `cargo metadata` subprocess (step 6). Returns immediately if no fast
/// candidate found. Used for pre-checks where hanging on a subprocess is
/// unacceptable (e.g., `--coverage-gaps` on repos without coverage data).
#[cfg_attr(coverage_nightly, coverage(off))]
fn get_profdata_mtime_fast(
    project_root: &Path,
    stored_path: Option<&str>,
) -> Option<(u64, String)> {
    // Fast path: check previously stored directory first
    if let Some(p) = stored_path {
        if let Some(mtime) = dir_mtime(std::path::Path::new(p)) {
            return Some((mtime, p.to_string()));
        }
    }

    let mut candidates: Vec<std::path::PathBuf> = Vec::with_capacity(8);

    // 1. CARGO_TARGET_DIR env var
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        candidates.push(std::path::PathBuf::from(&target_dir).join("llvm-cov-target"));
    }

    // 2. .cargo/config.toml (project-local, then global)
    candidates.extend(target_dir_from_cargo_config(
        &project_root.join(".cargo/config.toml"),
        project_root,
    ));
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| {
        std::env::var("HOME")
            .map(|h| format!("{h}/.cargo"))
            .unwrap_or_default()
    });
    if !cargo_home.is_empty() {
        let global_config = std::path::PathBuf::from(&cargo_home).join("config.toml");
        candidates.extend(target_dir_from_cargo_config(&global_config, project_root));
    }

    // 3. NVMe/RAID mount scan
    candidates.extend(mnt_target_candidates(project_root));

    // 4. Default target dir (follows symlinks via canonicalize)
    let default_target = project_root.join("target");
    if let Ok(canonical) = default_target.canonicalize() {
        candidates.push(canonical.join("llvm-cov-target"));
    }
    candidates.push(default_target.join("llvm-cov-target"));

    // Check fast candidates only — NO cargo metadata subprocess
    for dir in &candidates {
        if let Some(mtime) = dir_mtime(dir) {
            return Some((mtime, dir.to_string_lossy().to_string()));
        }
    }

    None
}

/// Try `cargo metadata` to discover target_directory (slow — subprocess spawn).
#[cfg_attr(coverage_nightly, coverage(off))]
fn cargo_metadata_target_dir(project_root: &Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    for toolchain_arg in &["+nightly", "+stable"] {
        let output = match std::process::Command::new("cargo")
            .args([toolchain_arg, "metadata", "--no-deps", "--format-version=1"])
            .current_dir(project_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(o) if o.status.success() => o,
            _ => continue,
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(dir) = extract_target_directory(&stdout) {
            result.push(std::path::PathBuf::from(dir).join("llvm-cov-target"));
        }
    }
    result
}

/// Extract "target_directory" value from cargo metadata JSON output.
fn extract_target_directory(json: &str) -> Option<&str> {
    let idx = json.find("\"target_directory\":\"")?;
    let rest = json.get(idx + 20..)?;
    let end = rest.find('"')?;
    rest.get(..end)
}

/// Resolve the cargo target directory, then find llvm-cov-target underneath.
///
/// Resolution order:
/// 1. `stored_path` from previous cache (fastest — skip all resolution)
/// 2. `CARGO_TARGET_DIR` env var
/// 3. `.cargo/config.toml` → `[build] target-dir` (project-local, then global)
/// 4. `/mnt/*/targets/{project_name}/` (NVMe/RAID overrides from shell functions)
/// 5. `project_root/target` (default, follows symlinks)
/// 6. `cargo metadata` target_directory (slow last resort)
///
/// Returns `(mtime, profdata_dir_path)` if found.
#[cfg_attr(coverage_nightly, coverage(off))]
fn get_profdata_mtime_and_dir(
    project_root: &Path,
    stored_path: Option<&str>,
) -> Option<(u64, String)> {
    // Fast path: check previously stored directory first
    if let Some(p) = stored_path {
        if let Some(mtime) = dir_mtime(std::path::Path::new(p)) {
            return Some((mtime, p.to_string()));
        }
    }

    let mut candidates: Vec<std::path::PathBuf> = Vec::with_capacity(8);

    // 1. CARGO_TARGET_DIR env var
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        candidates.push(std::path::PathBuf::from(&target_dir).join("llvm-cov-target"));
    }

    // 2. .cargo/config.toml (project-local, then global)
    candidates.extend(target_dir_from_cargo_config(
        &project_root.join(".cargo/config.toml"),
        project_root,
    ));
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| {
        std::env::var("HOME")
            .map(|h| format!("{h}/.cargo"))
            .unwrap_or_default()
    });
    if !cargo_home.is_empty() {
        let global_config = std::path::PathBuf::from(&cargo_home).join("config.toml");
        candidates.extend(target_dir_from_cargo_config(&global_config, project_root));
    }

    // 3. NVMe/RAID mount scan
    candidates.extend(mnt_target_candidates(project_root));

    // 4. Default target dir (follows symlinks via canonicalize)
    let default_target = project_root.join("target");
    if let Ok(canonical) = default_target.canonicalize() {
        candidates.push(canonical.join("llvm-cov-target"));
    }
    candidates.push(default_target.join("llvm-cov-target"));

    // Check fast candidates first, fall back to cargo metadata
    for dir in &candidates {
        if let Some(mtime) = dir_mtime(dir) {
            return Some((mtime, dir.to_string_lossy().to_string()));
        }
    }

    // 5. cargo metadata (heavyweight — spawns subprocess)
    for dir in cargo_metadata_target_dir(project_root) {
        if let Some(mtime) = dir_mtime(&dir) {
            return Some((mtime, dir.to_string_lossy().to_string()));
        }
    }

    None
}

/// Try loading coverage from an lcov.info file before running cargo llvm-cov.
///
/// Searches standard locations: `target/coverage/lcov.info`, `target/llvm-cov-target/lcov.info`.
#[cfg_attr(coverage_nightly, coverage(off))]
fn try_load_lcov_info(project_root: &Path) -> Option<HashMap<String, HashMap<usize, u64>>> {
    let candidates = [
        project_root.join("target/coverage/lcov.info"),
        project_root.join("target/llvm-cov-target/lcov.info"),
    ];
    for path in &candidates {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let map = parse_lcov_to_coverage_map(&content, project_root);
                if !map.is_empty() {
                    return Some(map);
                }
            }
        }
    }
    None
}

/// Try auto-discovering `coverage.json` files generated by
/// `cargo llvm-cov report --json --output-path target/coverage/coverage.json`.
///
/// This handles the case where users generate LLVM coverage JSON manually but
/// don't pass `--coverage-file`. Without this, pmat would skip the file and
/// run the subprocess, potentially getting stale or different data (#158).
#[cfg_attr(coverage_nightly, coverage(off))]
fn try_load_coverage_json(project_root: &Path) -> Option<HashMap<String, HashMap<usize, u64>>> {
    let candidates = [
        project_root.join("target/coverage/coverage.json"),
        project_root.join("target/llvm-cov-target/coverage.json"),
    ];
    for path in &candidates {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(map) = build_coverage_map(&content, project_root) {
                    if !map.is_empty() {
                        return Some(map);
                    }
                }
            }
        }
    }
    None
}

/// Parse lcov.info format into our coverage map.
///
/// Format: SF:<filename>, DA:<line>,<count>, end_of_record
fn parse_lcov_to_coverage_map(
    content: &str,
    project_root: &Path,
) -> HashMap<String, HashMap<usize, u64>> {
    let mut result: HashMap<String, HashMap<usize, u64>> = HashMap::new();
    let mut current_file: Option<String> = None;
    // Canonicalize project_root so strip_prefix works on absolute SF: paths
    let canonical_root = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let project_root_str = canonical_root.to_string_lossy();

    for line in content.lines() {
        if let Some(path) = line.strip_prefix("SF:") {
            // Normalize to relative path
            let rel = path
                .strip_prefix(project_root_str.as_ref())
                .or_else(|| path.strip_prefix('/'))
                .unwrap_or(path)
                .trim_start_matches('/');
            current_file = Some(rel.to_string());
        } else if let Some(da) = line.strip_prefix("DA:") {
            if let Some(ref file) = current_file {
                let parts: Vec<&str> = da.splitn(2, ',').collect();
                if parts.len() == 2 {
                    if let (Ok(line_no), Ok(count)) =
                        (parts[0].parse::<usize>(), parts[1].parse::<u64>())
                    {
                        result
                            .entry(file.clone())
                            .or_default()
                            .insert(line_no, count);
                    }
                }
            }
        } else if line == "end_of_record" {
            current_file = None;
        }
    }

    result
}

/// Run `cargo llvm-cov report --json` and parse the output into a coverage map.
///
/// On success, also writes the result to the cache file for future reuse.
/// Uses a 30-second timeout to prevent hanging on broken profdata.
#[cfg_attr(coverage_nightly, coverage(off))]
fn run_cargo_llvm_cov_and_cache(
    project_root: &Path,
    cache_path: &Path,
    head_hash: &str,
) -> Result<HashMap<String, HashMap<usize, u64>>, String> {
    // Try lcov.info fallback first (no subprocess needed)
    if let Some(cov) = try_load_lcov_info(project_root) {
        write_coverage_cache(cache_path, head_hash, project_root, &cov);
        return Ok(cov);
    }

    // Try coverage.json files (from `cargo llvm-cov report --json --output-path`) (#158)
    if let Some(cov) = try_load_coverage_json(project_root) {
        write_coverage_cache(cache_path, head_hash, project_root, &cov);
        return Ok(cov);
    }

    // Fast pre-check: verify profdata directory exists before spawning subprocess.
    // Uses get_profdata_mtime_fast() which skips `cargo metadata` subprocess —
    // prevents 30s hangs on repos without coverage data (#212).
    if get_profdata_mtime_fast(project_root, None).is_none() {
        return Err("No coverage data available.\n\n\
            To generate it, run:\n  \
            cargo llvm-cov test --lib --no-report\n\n\
            Then re-run with --coverage-gaps.\n\
            Or pass --coverage-file <path> to use existing coverage JSON."
            .to_string());
    }

    eprintln!("Generating coverage report...");
    let output = run_llvm_cov_subprocess(project_root)?;
    let json = String::from_utf8_lossy(&output.stdout);
    let file_coverage = build_coverage_map(&json, project_root)?;

    write_coverage_cache(cache_path, head_hash, project_root, &file_coverage);
    Ok(file_coverage)
}

/// Write coverage data to the cache file.
fn write_coverage_cache(
    cache_path: &Path,
    head_hash: &str,
    project_root: &Path,
    files: &HashMap<String, HashMap<usize, u64>>,
) {
    let (mtime, dir) = get_profdata_mtime_and_dir(project_root, None)
        .map(|(m, d)| (Some(m), Some(d)))
        .unwrap_or((None, None));
    let cache = CoverageCache {
        git_hash: head_hash.to_string(),
        coverage_mtime: mtime,
        profdata_dir: dir,
        files: files.clone(),
    };
    if let Ok(cache_json) = serde_json::to_string(&cache) {
        let _ = std::fs::create_dir_all(project_root.join(".pmat"));
        let _ = std::fs::write(cache_path, cache_json);
    }
}

/// Write a negative coverage cache — records that no coverage data is available.
///
/// Uses `get_profdata_mtime_fast()` (no subprocess) so this never blocks.
/// Invalidated when git hash changes or profdata mtime changes (user runs
/// `cargo llvm-cov test`). Avoids 30s subprocess timeout on every invocation (#212).
#[cfg_attr(coverage_nightly, coverage(off))]
fn write_negative_coverage_cache(cache_path: &Path, head_hash: &str, project_root: &Path) {
    let (mtime, dir) = get_profdata_mtime_fast(project_root, None)
        .map(|(m, d)| (Some(m), Some(d)))
        .unwrap_or((None, None));
    let cache = CoverageCache {
        git_hash: head_hash.to_string(),
        coverage_mtime: mtime,
        profdata_dir: dir,
        files: HashMap::new(),
    };
    if let Ok(cache_json) = serde_json::to_string(&cache) {
        let _ = std::fs::create_dir_all(project_root.join(".pmat"));
        let _ = std::fs::write(cache_path, cache_json);
    }
}

/// Spawn `cargo llvm-cov report --json` with timeout and pipe-safe I/O.
///
/// Tries `cargo +nightly` first (matching the toolchain used for instrumented builds),
/// falls back to default toolchain if nightly is unavailable.
fn run_llvm_cov_subprocess(project_root: &Path) -> Result<std::process::Output, String> {
    use std::process::{Command, Stdio};

    // Try nightly first (profdata is usually generated by nightly toolchain)
    let mut child = Command::new("cargo")
        .args(["+nightly", "llvm-cov", "report", "--json"])
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .or_else(|_| {
            // Fallback: default toolchain
            Command::new("cargo")
                .args(["llvm-cov", "report", "--json"])
                .current_dir(project_root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        })
        .map_err(|e| format!("cargo llvm-cov report --json failed to spawn: {e}"))?;

    let mut stdout_handle = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture stdout from cargo llvm-cov".to_string())?;
    let mut stderr_handle = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to capture stderr from cargo llvm-cov".to_string())?;

    let stdout_thread = std::thread::spawn(move || -> Vec<u8> {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stdout_handle.read_to_end(&mut buf);
        buf
    });
    let stderr_thread = std::thread::spawn(move || -> Vec<u8> {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stderr_handle.read_to_end(&mut buf);
        buf
    });

    wait_with_timeout(&mut child, std::time::Duration::from_secs(30))?;

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait on cargo llvm-cov: {e}"))?;
    let stdout = stdout_thread
        .join()
        .map_err(|_| "stdout reader thread panicked".to_string())?;
    let stderr = stderr_thread
        .join()
        .map_err(|_| "stderr reader thread panicked".to_string())?;
    let output = std::process::Output {
        status,
        stdout,
        stderr,
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "No coverage data available.\n\nTo generate it, run:\n  \
            cargo llvm-cov test --lib --no-report\n\nThen re-run with --coverage or --coverage-gaps.\n\
            Or pass --coverage-file <path> to use existing coverage JSON.\n\n\
            cargo llvm-cov report --json stderr: {}",
            stderr.lines().take(3).collect::<Vec<_>>().join("\n")
        ));
    }

    Ok(output)
}

/// Poll child process with timeout, killing if exceeded.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: std::time::Duration,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("No coverage data available.\n\n\
                    cargo llvm-cov report --json timed out after 30s.\n\
                    This usually means corrupted profdata. Try:\n  \
                    cargo llvm-cov clean\n  \
                    cargo llvm-cov test --lib --no-report\n\n\
                    Then re-run with --coverage or --coverage-gaps.\n\
                    Or pass --coverage-file <path> to use existing coverage JSON."
                    .to_string());
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
            Err(e) => return Err(format!("Failed to wait on cargo llvm-cov: {e}")),
        }
    }
}

/// Enrich results with LLVM coverage data.
///
/// Resolution order for coverage JSON:
/// 1. Explicit `coverage_file` parameter (from --coverage-file CLI arg)
/// 2. `PMAT_COVERAGE_FILE` environment variable
/// 3. `.pmat/coverage-cache.json` (if git HEAD matches)
/// 4. Run `cargo llvm-cov report --json` to generate fresh data
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

    // Cache miss — run cargo llvm-cov (writes positive cache on success)
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segments_to_line_hits_empty() {
        let result = segments_to_line_hits(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_segments_to_line_hits_basic() {
        // Segments: [line, col, count, has_count, is_region_entry]
        let segments = vec![
            vec![
                json_u64(10),
                json_u64(1),
                json_u64(5),
                json_bool(true),
                json_bool(true),
            ],
            vec![
                json_u64(15),
                json_u64(1),
                json_u64(0),
                json_bool(true),
                json_bool(false),
            ],
            vec![
                json_u64(20),
                json_u64(1),
                json_u64(3),
                json_bool(true),
                json_bool(true),
            ],
        ];
        let hits = segments_to_line_hits(&segments);

        // Lines 10-15 should have count 5
        assert_eq!(hits.get(&10), Some(&5));
        assert_eq!(hits.get(&12), Some(&5));
        assert_eq!(hits.get(&15), Some(&5)); // inclusive of range

        // Lines 15-20 also get count from seg[1] (0) but seg[0] covers 10..=15 with 5
        // seg[1] covers 15..=20 with 0
        // For line 15, max(5, 0) = 5
        // For line 17, count from seg[1] is 0
        assert_eq!(hits.get(&17), Some(&0));

        // Lines 20+ should have count 3
        assert_eq!(hits.get(&20), Some(&3));
    }

    #[test]
    fn test_segments_to_line_hits_no_count() {
        // Segment with has_count=false should be skipped
        let segments = vec![vec![
            json_u64(10),
            json_u64(1),
            json_u64(5),
            json_bool(false),
            json_bool(true),
        ]];
        let hits = segments_to_line_hits(&segments);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_enrich_with_coverage_basic() {
        let mut results = vec![make_result("src/main.rs", 10, 20, 0.5, 5)];

        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        // Lines 10-15 covered, 16-20 uncovered
        for l in 10..=15 {
            line_hits.insert(l, 1);
        }
        for l in 16..=20 {
            line_hits.insert(l, 0);
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);

        enrich_with_coverage(&mut results, &file_coverage);

        assert_eq!(results[0].lines_covered, 6); // 10,11,12,13,14,15
        assert_eq!(results[0].lines_total, 11); // 10..=20
        assert_eq!(results[0].missed_lines, 5); // 16,17,18,19,20
                                                // Coverage = 6/11 ≈ 54.5%
        assert!((results[0].line_coverage_pct - 54.545).abs() < 1.0);
        assert!(results[0].impact_score > 0.0);
        assert_eq!(results[0].coverage_status, "partial");
    }

    #[test]
    fn test_enrich_with_coverage_no_file_match() {
        let mut results = vec![make_result("src/other.rs", 1, 10, 0.0, 1)];

        let file_coverage = HashMap::new();
        enrich_with_coverage(&mut results, &file_coverage);

        assert_eq!(results[0].lines_covered, 0);
        assert_eq!(results[0].lines_total, 0);
        assert_eq!(results[0].line_coverage_pct, 0.0);
        assert_eq!(results[0].coverage_status, "no_data");
    }

    #[test]
    fn test_enrich_with_coverage_non_instrumented_lines() {
        let mut results = vec![make_result("src/main.rs", 1, 10, 0.1, 3)];

        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        // Only lines 3,5,7 are instrumented
        line_hits.insert(3, 1);
        line_hits.insert(5, 0);
        line_hits.insert(7, 1);
        file_coverage.insert("src/main.rs".to_string(), line_hits);

        enrich_with_coverage(&mut results, &file_coverage);

        assert_eq!(results[0].lines_total, 3); // only instrumented lines count
        assert_eq!(results[0].lines_covered, 2); // lines 3 and 7
        assert_eq!(results[0].missed_lines, 1); // line 5
    }

    #[test]
    fn test_compute_impact_score_zero_missed() {
        assert_eq!(compute_impact_score(0, 0.5, 10), 0.0);
    }

    #[test]
    fn test_compute_impact_score_basic() {
        // 10 missed lines, pagerank 0.001 (scaled to 10.0), complexity 5
        let score = compute_impact_score(10, 0.001, 5);
        // 10 * 10.0 / 5.0 = 20.0
        assert!((score - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_impact_score_zero_pagerank() {
        // 0 pagerank floors to 0.1
        let score = compute_impact_score(5, 0.0, 1);
        // 5 * 0.1 / 1.0 = 0.5
        assert!((score - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_compute_impact_score_zero_complexity() {
        // 0 complexity floors to 1.0
        let score = compute_impact_score(10, 0.001, 0);
        // 10 * 10.0 / 1.0 = 100.0
        assert!((score - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_uncovered_only_filter() {
        let mut results = vec![
            make_result_with_coverage("src/a.rs", 100.0, 10, 10, 0),
            make_result_with_coverage("src/b.rs", 50.0, 5, 10, 5),
            make_result_with_coverage("src/c.rs", 0.0, 0, 10, 10),
            make_result_with_coverage("src/d.rs", 0.0, 0, 0, 0), // non-instrumented
        ];

        // Simulate --uncovered-only filter
        results.retain(|r| r.lines_total > 0 && r.line_coverage_pct < 100.0);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].file_path, "src/b.rs");
        assert_eq!(results[1].file_path, "src/c.rs");
    }

    #[test]
    fn test_build_coverage_map_basic() {
        let json = r#"{
            "data": [{
                "files": [{
                    "filename": "/project/src/main.rs",
                    "segments": [
                        [10, 1, 5, true, true],
                        [15, 1, 0, true, false]
                    ],
                    "summary": {"lines": {"count": 20, "covered": 15}}
                }]
            }],
            "type": "llvm.coverage.json.export"
        }"#;

        let root = Path::new("/project");
        let map = build_coverage_map(json, root).unwrap();

        assert!(map.contains_key("src/main.rs"));
        let hits = &map["src/main.rs"];
        assert!(hits.contains_key(&10));
    }

    #[test]
    fn test_build_coverage_map_invalid_json() {
        let result = build_coverage_map("not json", Path::new("/project"));
        assert!(result.is_err());
    }

    // ── Test Helpers ────────────────────────────────────────────────────────

    fn json_u64(v: u64) -> serde_json::Value {
        serde_json::Value::Number(serde_json::Number::from(v))
    }

    fn json_bool(v: bool) -> serde_json::Value {
        serde_json::Value::Bool(v)
    }

    fn make_result(
        file_path: &str,
        start_line: usize,
        end_line: usize,
        pagerank: f32,
        complexity: u32,
    ) -> QueryResult {
        QueryResult {
            file_path: file_path.to_string(),
            function_name: "test_fn".to_string(),
            signature: "fn test_fn()".to_string(),
            definition_type: "function".to_string(),
            doc_comment: None,
            start_line,
            end_line,
            language: "rust".to_string(),
            tdg_score: 5.0,
            tdg_grade: "C".to_string(),
            complexity,
            big_o: "O(n)".to_string(),
            satd_count: 0,
            loc: (end_line - start_line + 1) as u32,
            relevance_score: 0.8,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: Default::default(),
            coverage_excluded: false,
            cross_project_callers: 0,
        }
    }

    fn make_result_with_coverage(
        file_path: &str,
        coverage_pct: f32,
        covered: u32,
        total: u32,
        missed: u32,
    ) -> QueryResult {
        let mut r = make_result(file_path, 1, 10, 0.0, 1);
        r.line_coverage_pct = coverage_pct;
        r.lines_covered = covered;
        r.lines_total = total;
        r.missed_lines = missed;
        r
    }

    #[test]
    fn test_coverage_status_full() {
        let mut results = vec![make_result("src/main.rs", 1, 5, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=5 {
            line_hits.insert(l, 1); // all covered
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);
        assert_eq!(results[0].coverage_status, "full");
        assert_eq!(results[0].line_coverage_pct, 100.0);
    }

    #[test]
    fn test_coverage_status_uncovered() {
        let mut results = vec![make_result("src/main.rs", 1, 5, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=5 {
            line_hits.insert(l, 0); // all instrumented but zero hits
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);
        assert_eq!(results[0].coverage_status, "uncovered");
        assert_eq!(results[0].lines_covered, 0);
        assert_eq!(results[0].lines_total, 5);
    }

    #[test]
    fn test_coverage_status_no_data_when_no_instrumented_lines() {
        // File is in coverage map but no lines in the function's range are instrumented
        let mut results = vec![make_result("src/main.rs", 100, 110, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        line_hits.insert(1, 1); // only line 1 instrumented, function is at 100-110
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);
        assert_eq!(results[0].coverage_status, "no_data");
        assert_eq!(results[0].lines_total, 0);
    }

    #[test]
    fn test_format_coverage_summary_basic() {
        let mut results = vec![
            make_result("src/a.rs", 1, 10, 0.01, 5),
            make_result("src/b.rs", 1, 10, 0.01, 5),
        ];
        results[0].coverage_status = "partial".to_string();
        results[0].lines_covered = 7;
        results[0].lines_total = 10;
        results[0].missed_lines = 3;
        results[0].impact_score = 5.0;
        results[0].function_name = "func_a".to_string();

        results[1].coverage_status = "uncovered".to_string();
        results[1].lines_covered = 0;
        results[1].lines_total = 10;
        results[1].missed_lines = 10;
        results[1].impact_score = 10.0;
        results[1].function_name = "func_b".to_string();

        let summary = format_coverage_summary(&results).unwrap();
        assert!(summary.contains("7/20 lines"));
        assert!(summary.contains("35.0%"));
        assert!(summary.contains("1 uncovered"));
        assert!(summary.contains("1 partial"));
        assert!(summary.contains("Top impact: func_b"));
    }

    #[test]
    fn test_format_coverage_summary_no_data() {
        let results = vec![make_result("src/a.rs", 1, 10, 0.0, 1)];
        // No coverage_status set (empty string) → should return None
        assert!(format_coverage_summary(&results).is_none());
    }

    #[test]
    fn test_coverage_fault_annotation_uncovered() {
        let mut results = vec![make_result("src/main.rs", 1, 10, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=10 {
            line_hits.insert(l, 0); // all instrumented, zero hits
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);

        // Should have NO_COVERAGE fault annotation
        assert!(
            results[0]
                .fault_annotations
                .iter()
                .any(|f| f.starts_with("NO_COVERAGE:")),
            "Expected NO_COVERAGE annotation, got: {:?}",
            results[0].fault_annotations
        );
        assert!(results[0].fault_annotations[0].contains("0/10 lines"));
    }

    #[test]
    fn test_coverage_fault_annotation_low_coverage() {
        let mut results = vec![make_result("src/main.rs", 1, 10, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        // 3 covered, 7 uncovered → 30% coverage
        for l in 1..=3 {
            line_hits.insert(l, 1);
        }
        for l in 4..=10 {
            line_hits.insert(l, 0);
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);

        // Should have LOW_COVERAGE fault annotation (30% < 50%)
        assert!(
            results[0]
                .fault_annotations
                .iter()
                .any(|f| f.starts_with("LOW_COVERAGE:")),
            "Expected LOW_COVERAGE annotation, got: {:?}",
            results[0].fault_annotations
        );
    }

    #[test]
    fn test_coverage_fault_annotation_high_impact() {
        // High pagerank = high impact score when lines are missed
        let mut results = vec![make_result("src/main.rs", 1, 20, 0.01, 2)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=5 {
            line_hits.insert(l, 1);
        }
        for l in 6..=20 {
            line_hits.insert(l, 0);
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);

        // Impact = 15 missed * (0.01*10000=100.0) / 2.0 = 750.0 → COVERAGE_RISK
        assert!(results[0].impact_score > 5.0);
        assert!(
            results[0]
                .fault_annotations
                .iter()
                .any(|f| f.starts_with("COVERAGE_RISK:")),
            "Expected COVERAGE_RISK annotation for impact={:.1}, got: {:?}",
            results[0].impact_score,
            results[0].fault_annotations
        );
    }

    #[test]
    fn test_coverage_no_fault_annotation_when_fully_covered() {
        let mut results = vec![make_result("src/main.rs", 1, 5, 0.1, 3)];
        let mut file_coverage = HashMap::new();
        let mut line_hits = HashMap::new();
        for l in 1..=5 {
            line_hits.insert(l, 1);
        }
        file_coverage.insert("src/main.rs".to_string(), line_hits);
        enrich_with_coverage(&mut results, &file_coverage);

        // Fully covered → no coverage fault annotations
        let coverage_faults: Vec<_> = results[0]
            .fault_annotations
            .iter()
            .filter(|f| {
                f.starts_with("NO_COVERAGE:")
                    || f.starts_with("LOW_COVERAGE:")
                    || f.starts_with("COVERAGE_RISK:")
            })
            .collect();
        assert!(
            coverage_faults.is_empty(),
            "Fully covered function should have no coverage faults, got: {:?}",
            coverage_faults
        );
    }

    #[test]
    fn test_build_coverage_map_skips_external_files() {
        // Files outside project root should be skipped (deps, registry, etc.)
        let json = r#"{
            "data": [{
                "files": [
                    {
                        "filename": "/project/src/main.rs",
                        "segments": [[10, 1, 5, true, true], [15, 1, 0, true, false]],
                        "summary": {"lines": {"count": 20, "covered": 15}}
                    },
                    {
                        "filename": "/home/user/.cargo/registry/src/crates.io-abc/dep-1.0.0/src/lib.rs",
                        "segments": [[1, 1, 3, true, true], [10, 1, 0, true, false]],
                        "summary": {"lines": {"count": 10, "covered": 5}}
                    }
                ]
            }],
            "type": "llvm.coverage.json.export"
        }"#;

        let root = Path::new("/project");
        let map = build_coverage_map(json, root).unwrap();

        // Only project file should be included
        assert_eq!(
            map.len(),
            1,
            "Should only contain project files, got: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        assert!(map.contains_key("src/main.rs"));
    }

    #[test]
    fn test_parse_lcov_basic() {
        let lcov = "\
SF:/project/src/lib.rs
DA:1,5
DA:2,3
DA:3,0
end_of_record
SF:/project/src/main.rs
DA:10,1
end_of_record
";
        let root = Path::new("/project");
        let map = parse_lcov_to_coverage_map(lcov, root);
        assert_eq!(map.len(), 2);
        let lib = map.get("src/lib.rs").unwrap();
        assert_eq!(lib.get(&1), Some(&5));
        assert_eq!(lib.get(&2), Some(&3));
        assert_eq!(lib.get(&3), Some(&0));
        let main = map.get("src/main.rs").unwrap();
        assert_eq!(main.get(&10), Some(&1));
    }

    #[test]
    fn test_parse_lcov_empty() {
        let map = parse_lcov_to_coverage_map("", Path::new("/project"));
        assert!(map.is_empty());
    }

    #[test]
    fn test_parse_lcov_relative_paths() {
        let lcov = "\
SF:src/foo.rs
DA:1,10
end_of_record
";
        let root = Path::new("/project");
        let map = parse_lcov_to_coverage_map(lcov, root);
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("src/foo.rs"));
    }

    #[test]
    fn test_try_load_lcov_info() {
        let temp = tempfile::TempDir::new().unwrap();
        let cov_dir = temp.path().join("target/coverage");
        std::fs::create_dir_all(&cov_dir).unwrap();
        std::fs::write(
            cov_dir.join("lcov.info"),
            format!(
                "SF:{}/src/lib.rs\nDA:1,5\nend_of_record\n",
                temp.path().display()
            ),
        )
        .unwrap();

        let result = try_load_lcov_info(temp.path());
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("src/lib.rs"));
    }

    #[test]
    fn test_try_load_lcov_info_missing() {
        let temp = tempfile::TempDir::new().unwrap();
        let result = try_load_lcov_info(temp.path());
        assert!(result.is_none());
    }
}
