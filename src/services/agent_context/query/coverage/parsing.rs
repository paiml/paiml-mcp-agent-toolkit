#![cfg_attr(coverage_nightly, coverage(off))]

use super::types::LlvmCoverageExport;
use std::collections::HashMap;
use std::path::Path;

// ── Segment Walking ─────────────────────────────────────────────────────────

/// Walk LLVM segments to build line->exec_count map.
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
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

// ── LCOV Parsing ────────────────────────────────────────────────────────────

/// Parse lcov.info format into our coverage map.
///
/// Format: SF:<filename>, DA:<line>,<count>, end_of_record
pub(super) fn parse_lcov_to_coverage_map(
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

// ── Coverage File Discovery ─────────────────────────────────────────────────

/// Try loading coverage from an lcov.info file before running cargo llvm-cov.
///
/// Searches standard locations: `target/coverage/lcov.info`, `target/llvm-cov-target/lcov.info`.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn try_load_lcov_info(
    project_root: &Path,
) -> Option<HashMap<String, HashMap<usize, u64>>> {
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
pub(super) fn try_load_coverage_json(
    project_root: &Path,
) -> Option<HashMap<String, HashMap<usize, u64>>> {
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
