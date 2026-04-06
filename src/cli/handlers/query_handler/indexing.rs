//! Index loading, workspace indexing, incremental updates.

use crate::services::agent_context::AgentContextIndex;
use std::path::PathBuf;

/// Load the function index with workspace support
pub(super) fn load_query_index(
    project_path: &PathBuf,
    rebuild_index: bool,
    include_project: &[PathBuf],
    quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let index_path = project_path.join(".pmat/context.idx");
    let workspace_idx = project_path.join(".pmat/workspace.idx");
    let mut siblings = AgentContextIndex::discover_sibling_indexes(project_path);

    for project in include_project {
        let idx_path = project.join(".pmat/context.idx");
        if idx_path.exists() {
            let name = project
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| project.display().to_string());
            if !siblings.iter().any(|(_, n)| n == &name) {
                siblings.push((idx_path, name));
            }
        } else if !quiet {
            eprintln!(
                "Warning: No index at {:?}, run 'pmat query --rebuild-index' in that project first",
                idx_path
            );
        }
    }

    if !siblings.is_empty()
        && !rebuild_index
        && is_workspace_cache_fresh(&workspace_idx, &siblings, &index_path)
    {
        if !quiet {
            eprintln!("Loading cached workspace index...");
        }
        if let Ok(cached) = AgentContextIndex::load(&workspace_idx) {
            return Ok(cached);
        }
    }

    load_and_merge_index(
        project_path,
        &index_path,
        &workspace_idx,
        &siblings,
        rebuild_index,
        quiet,
    )
}

/// Pre-load source and call graph into the index based on the query mode.
pub(super) fn prepare_index_for_mode(
    index: &mut AgentContextIndex,
    is_regex_or_literal: bool,
    is_ptx: bool,
    rank_by: &Option<String>,
) {
    if is_regex_or_literal || is_ptx {
        index.load_all_source();
    }
    let needs_call_graph = is_ptx
        || rank_by.as_deref() == Some("cross-project")
        || rank_by.as_deref() == Some("crossproject")
        || rank_by.as_deref() == Some("xproject");
    if needs_call_graph {
        index.ensure_call_graph();
    }
}

/// Print index stats to stderr (unless in quiet/JSON mode).
pub(super) fn emit_index_stats(index: &AgentContextIndex, quiet: bool) {
    if !quiet {
        let manifest = index.manifest();
        eprintln!(
            "Index: {} functions in {} files (avg TDG: {:.1})",
            manifest.function_count, manifest.file_count, manifest.avg_tdg_score
        );
    }
}

/// Collect sibling project indexes for workspace coverage merging.
pub(super) fn collect_siblings(
    project_path: &std::path::Path,
    include_project: &[PathBuf],
) -> Vec<(PathBuf, String)> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let mut siblings = AgentContextIndex::discover_sibling_indexes(project_path);
    for project in include_project {
        let idx_path = project.join(".pmat/context.idx");
        let name = project
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| project.display().to_string());
        if !siblings.iter().any(|(_, n)| n == &name) {
            siblings.push((idx_path, name));
        }
    }
    siblings
}

/// Backfill source code for query results from SQLite.
///
/// In deferred-source mode, `QueryResult.source` is `Some("")` (empty) for
/// semantic queries. This fetches source on-demand for the top N results
/// that need it for display (--include-source, --code, context lines).
pub(super) fn backfill_results_source(
    results: &mut [crate::services::agent_context::QueryResult],
    index: &AgentContextIndex,
) {
    if index.db_path().is_none() {
        return; // Blob-loaded index already has source
    }
    for r in results.iter_mut() {
        // Skip results that already have non-empty source
        if r.source.as_ref().is_some_and(|s| !s.is_empty()) {
            continue;
        }
        // Only backfill if source was requested (Some(""))
        if r.source.is_none() {
            continue;
        }
        let src = index.load_source_for(&r.file_path, r.start_line);
        if !src.is_empty() {
            r.source = Some(src);
        }
    }
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Load local index, do incremental update if needed, and merge siblings.
///
/// Save threshold: only rewrites the full SQLite index when changes exceed
/// 50 files or 5% of the index. This avoids rewriting 660MB for a handful
/// of changes during development (#212).
fn try_incremental_update(
    project_path: &PathBuf,
    index_path: &PathBuf,
    existing: AgentContextIndex,
    quiet: bool,
) -> AgentContextIndex {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if existing.manifest().file_checksums.is_empty() {
        return existing;
    }
    if !quiet {
        eprintln!("Checking for incremental updates...");
    }
    match AgentContextIndex::build_incremental(project_path, &existing) {
        Ok(updated) => {
            maybe_save_incremental(&updated, index_path, quiet);
            updated
        }
        Err(_) => existing,
    }
}

/// Save the index only when changes exceed 50 files or 5% of index size.
/// Avoids rewriting 660MB SQLite for a handful of changes (#212).
fn maybe_save_incremental(index: &AgentContextIndex, index_path: &PathBuf, quiet: bool) {
    let changes = index.manifest().last_incremental_changes;
    if changes == 0 {
        return;
    }
    let total = index.functions.len();
    let pct = if total > 0 {
        changes as f64 / total as f64
    } else {
        0.0
    };
    if changes > 50 || pct > 0.05 {
        if !quiet {
            eprintln!("Saving index ({} changes)...", changes);
        }
        let _ = index.save(index_path);
    } else if !quiet {
        eprintln!("Skipping save ({} minor changes)", changes);
    }
}

fn load_or_build_index(
    project_path: &PathBuf,
    index_path: &PathBuf,
    rebuild_index: bool,
    quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    // Check for either SQLite (.db) or blob (.idx/) index
    let db_path = index_path.with_extension("db");

    // Fail-fast: detect partial/corrupt index (manifest exists but data missing)
    let manifest_exists = index_path.join("manifest.json").exists();
    let blob_exists = index_path.join("functions.lz4").exists();
    if !db_path.exists() && manifest_exists && !blob_exists {
        eprintln!("Detected partial index (manifest without data), rebuilding...");
        let _ = std::fs::remove_dir_all(index_path);
        return build_and_save_index(project_path, index_path);
    }

    if (!index_path.exists() && !db_path.exists()) || rebuild_index {
        if !quiet {
            eprintln!("Building index for {:?}...", project_path);
            eprintln!("  This may take 1-3 minutes for large repos (progress below).");
        }
        return build_and_save_index(project_path, index_path);
    }
    if !quiet {
        eprintln!("Loading index from {:?}...", index_path);
    }
    match AgentContextIndex::load(index_path) {
        Ok(existing) => Ok(try_incremental_update(
            project_path,
            index_path,
            existing,
            quiet,
        )),
        Err(e) => {
            eprintln!("Failed to load index ({}), rebuilding...", e);
            eprintln!("  This may take 1-3 minutes for large repos.");
            eprintln!("  Hint: run 'pmat index' explicitly if this is slow.");
            build_and_save_index(project_path, index_path)
        }
    }
}

fn load_and_merge_index(
    project_path: &PathBuf,
    index_path: &PathBuf,
    workspace_idx: &std::path::Path,
    siblings: &[(PathBuf, String)],
    rebuild_index: bool,
    quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    debug_assert!(
        workspace_idx.exists(),
        "workspace_idx must exist: {}",
        workspace_idx.display()
    );
    let mut index = load_or_build_index(project_path, index_path, rebuild_index, quiet)?;
    if !siblings.is_empty() {
        merge_and_cache_workspace(&mut index, siblings, workspace_idx, quiet);
    }
    Ok(index)
}

/// Check if the cached workspace index is newer than all sibling indexes and local index.
fn is_workspace_cache_fresh(
    workspace_idx: &std::path::Path,
    siblings: &[(PathBuf, String)],
    local_idx: &std::path::Path,
) -> bool {
    debug_assert!(
        workspace_idx.exists(),
        "workspace_idx must exist: {}",
        workspace_idx.display()
    );
    debug_assert!(
        local_idx.exists(),
        "local_idx must exist: {}",
        local_idx.display()
    );
    // Prefer workspace.db mtime, fall back to workspace.idx/manifest.json
    let cache_mtime = newest_index_mtime(workspace_idx);
    let cache_mtime = match cache_mtime {
        Some(t) => t,
        None => return false, // No cache
    };

    // Check local index is not newer than cache
    if let Some(local_mtime) = newest_index_mtime(local_idx) {
        if local_mtime > cache_mtime {
            return false; // Local index updated since cache
        }
    }

    // Cache is fresh if it's newer than every sibling's index
    siblings.iter().all(|(idx_path, _)| {
        match newest_index_mtime(idx_path) {
            Some(sibling_mtime) => cache_mtime > sibling_mtime,
            None => true, // Sibling gone, cache still valid for others
        }
    })
}

/// Get the newest mtime for an index (checks context.db and context.idx/manifest.json).
fn newest_index_mtime(idx_path: &std::path::Path) -> Option<std::time::SystemTime> {
    debug_assert!(
        idx_path.exists(),
        "idx_path must exist: {}",
        idx_path.display()
    );
    let db_path = idx_path.with_extension("db");
    let manifest_path = idx_path.join("manifest.json");

    let db_mtime = std::fs::metadata(&db_path).and_then(|m| m.modified()).ok();
    let manifest_mtime = std::fs::metadata(&manifest_path)
        .and_then(|m| m.modified())
        .ok();

    match (db_mtime, manifest_mtime) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Merge siblings into index and save the combined result as workspace cache.
fn merge_and_cache_workspace(
    index: &mut AgentContextIndex,
    siblings: &[(PathBuf, String)],
    workspace_idx: &std::path::Path,
    quiet: bool,
) {
    debug_assert!(
        workspace_idx.exists(),
        "workspace_idx must exist: {}",
        workspace_idx.display()
    );
    if !quiet {
        eprintln!("Merging {} sibling project(s):", siblings.len());
    }
    index.merge_siblings(siblings);

    // Cache the merged index for next time
    match index.save(workspace_idx) {
        Ok(()) => {
            if !quiet {
                eprintln!("Workspace index cached.");
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("Failed to cache workspace index: {}", e);
            }
        }
    }
}

/// Build index and save to disk.
///
/// Save failures are non-fatal: the in-memory index is returned so the query
/// can still proceed. This prevents "database is locked" errors from killing
/// the entire query (#161).
fn build_and_save_index(
    project_path: &PathBuf,
    index_path: &PathBuf,
) -> anyhow::Result<AgentContextIndex> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let start = std::time::Instant::now();
    let index = AgentContextIndex::build(project_path)
        .map_err(|e| anyhow::anyhow!("Failed to build index: {}", e))?;
    eprintln!(
        "  Index built: {} functions in {:.1}s",
        index.all_functions().len(),
        start.elapsed().as_secs_f32()
    );

    // Create .pmat directory if needed
    if let Some(parent) = index_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Save index -- non-fatal on failure (index is still usable in memory)
    match index.save(index_path) {
        Ok(()) => eprintln!("Index saved to {:?}", index_path),
        Err(e) => eprintln!(
            "Warning: Failed to save index ({}), using in-memory index",
            e
        ),
    }

    Ok(index)
}
