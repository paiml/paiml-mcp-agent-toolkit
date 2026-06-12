//! Index loading, workspace indexing, incremental updates.

use crate::services::agent_context::AgentContextIndex;
use std::path::PathBuf;

/// Load the function index with workspace support.
///
/// Performance optimization: skip sibling discovery when no cross-project
/// flags are used. Local-only index loads in ~150ms vs ~90s for workspace.
pub(super) fn load_query_index(
    project_path: &PathBuf,
    rebuild_index: bool,
    include_project: &[PathBuf],
    quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    let index_path = project_path.join(".pmat/context.idx");
    let workspace_idx = project_path.join(".pmat/workspace.idx");

    // Fast path: if no --include-project, check if workspace cache is fresh
    // and load it, otherwise load local-only (skip expensive sibling discovery)
    if include_project.is_empty() && !rebuild_index {
        let siblings = AgentContextIndex::discover_sibling_indexes(project_path);
        if let Some(cached) =
            try_load_fresh_workspace_cache(&workspace_idx, &siblings, &index_path, quiet)
        {
            return Ok(cached);
        }
        // No fresh cache — load local only for speed, merge lazily
        return load_or_build_index(project_path, &index_path, false, quiet);
    }

    let mut siblings = AgentContextIndex::discover_sibling_indexes(project_path);
    extend_siblings_with_includes(&mut siblings, include_project, quiet);

    if !rebuild_index {
        if let Some(cached) =
            try_load_fresh_workspace_cache(&workspace_idx, &siblings, &index_path, quiet)
        {
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

/// Load the cached workspace index if it exists and is fresh (O(1) mtime check)
fn try_load_fresh_workspace_cache(
    workspace_idx: &std::path::Path,
    siblings: &[(PathBuf, String)],
    index_path: &std::path::Path,
    quiet: bool,
) -> Option<AgentContextIndex> {
    if siblings.is_empty() || !is_workspace_cache_fresh(workspace_idx, siblings, index_path) {
        return None;
    }
    if !quiet {
        eprintln!("Loading cached workspace index...");
    }
    AgentContextIndex::load(workspace_idx).ok()
}

/// Add explicitly-included projects to the sibling list (deduplicated)
fn extend_siblings_with_includes(
    siblings: &mut Vec<(PathBuf, String)>,
    include_project: &[PathBuf],
    quiet: bool,
) {
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
///
/// No db_path guard here: `load_source_for()` already falls back from
/// in-memory to SQLite to filesystem, and an early return on a missing
/// db_path silently broke `--include-source` after incremental updates
/// (which used to drop the db_path).
pub(super) fn backfill_results_source(
    results: &mut [crate::services::agent_context::QueryResult],
    index: &AgentContextIndex,
) {
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
    if existing.manifest().file_checksums.is_empty() {
        return existing;
    }
    if !quiet {
        eprintln!("Checking for incremental updates...");
    }
    match AgentContextIndex::build_incremental(project_path, &existing) {
        Ok(mut updated) => {
            maybe_save_incremental(&mut updated, index_path, quiet);
            updated
        }
        Err(_) => existing,
    }
}

/// Save the index only when changes exceed 50 files or 5% of index size.
/// Avoids rewriting 660MB SQLite for a handful of changes (#212).
///
/// Before saving, backfills deferred (empty) source for checksum-reused
/// entries: they were cloned from a lightweight SQLite load, and rewriting
/// the DB without their source would persist `source=''` for every reused
/// row, converging the index to all-empty source (the source-wipe bug).
/// The backfill is guarded by the save threshold so pure-read queries keep
/// their lazy-load performance.
fn maybe_save_incremental(index: &mut AgentContextIndex, index_path: &PathBuf, quiet: bool) {
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
        // Must run before save(): it reads the old DB this save overwrites.
        index.load_all_source();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_context::function_index::sqlite_backend;

    fn write_file(root: &std::path::Path, rel: &str, content: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn build_and_save(project: &std::path::Path) -> PathBuf {
        let index_path = project.join(".pmat/context.idx");
        std::fs::create_dir_all(index_path.parent().unwrap()).unwrap();
        let built = AgentContextIndex::build(project).unwrap();
        built.save(&index_path).unwrap();
        index_path
    }

    /// Regression (source wipe): an incremental update that crosses the save
    /// threshold must not rewrite the database with empty source for
    /// checksum-reused entries. Before the fix, every reused row persisted
    /// with source='' and repeated saves converged the DB to all-empty.
    #[test]
    fn test_incremental_save_keeps_source_in_db() {
        let temp = tempfile::TempDir::new().unwrap();
        let project = temp.path().to_path_buf();
        write_file(&project, "src/a.rs", "fn alpha() { let a = 1; }\n");
        write_file(&project, "src/b.rs", "fn beta() { let b = 2; }\n");
        write_file(&project, "src/c.rs", "fn gamma() { let c = 3; }\n");
        let index_path = build_and_save(&project);

        let loaded = AgentContextIndex::load(&index_path).unwrap();

        // Change 1/3 files: 1 reparse out of 4 functions (25% > 5% threshold)
        write_file(
            &project,
            "src/c.rs",
            "fn gamma() { let c = 30; }\nfn delta() { let d = 4; }\n",
        );

        let updated = try_incremental_update(&project, &index_path, loaded, true);
        assert!(
            updated.db_path().is_some(),
            "incremental index must keep the db_path it loaded from"
        );

        let conn = sqlite_backend::open_db(&index_path.with_extension("db")).unwrap();
        let rows = sqlite_backend::load_functions(&conn).unwrap();
        assert_eq!(rows.len(), 4);
        for row in &rows {
            assert!(
                !row.source.is_empty(),
                "incremental save wiped source for '{}'",
                row.function_name
            );
        }
    }

    /// Regression (Bug B): queries served right after an incremental update
    /// must return non-empty source. The incremental build used to reset
    /// db_path to None, so backfill_results_source early-returned and
    /// --include-source / pmat_get_function returned source:"" even with a
    /// healthy database on disk.
    #[test]
    fn test_backfill_results_source_after_incremental_update() {
        let temp = tempfile::TempDir::new().unwrap();
        let project = temp.path().to_path_buf();
        write_file(&project, "src/a.rs", "fn alpha() { let a = 1; }\n");
        let index_path = build_and_save(&project);

        let loaded = AgentContextIndex::load(&index_path).unwrap();

        // No file changes: pure reuse pass, below save threshold, so the
        // in-memory entries keep deferred (empty) source.
        let updated = try_incremental_update(&project, &index_path, loaded, true);

        let result = updated
            .get_function("src/a.rs", "alpha")
            .expect("alpha should be indexed");
        let mut results = [result];
        assert_eq!(
            results[0].source.as_deref(),
            Some(""),
            "precondition: source is deferred after incremental update"
        );

        backfill_results_source(&mut results, &updated);
        let source = results[0].source.as_deref().unwrap_or("");
        assert!(
            source.contains("alpha"),
            "expected backfilled source, got {source:?}"
        );
    }
}
