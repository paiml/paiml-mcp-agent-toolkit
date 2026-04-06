#![cfg_attr(coverage_nightly, coverage(off))]

//! Atomic save of the full index to a SQLite database.
//!
//! Builds a temporary DB then renames into place to prevent concurrent
//! readers from seeing a partial/empty file.

use super::insert::{
    insert_call_graph, insert_coverage_off_files, insert_functions, insert_graph_metrics,
    insert_metadata,
};
use super::schema::{create_schema, open_db};
use super::types::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Save the full index to a SQLite database.
///
/// Uses atomic write: builds a temporary DB, then renames into place.
/// This prevents concurrent readers from seeing a partial/empty file.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn save_to_sqlite(
    db_path: &Path,
    functions: &[FunctionEntry],
    calls: &HashMap<usize, Vec<usize>>,
    graph_metrics: &[GraphMetrics],
    manifest: &IndexManifest,
    coverage_off_files: &HashSet<String>,
) -> Result<(), String> {
    debug_assert!(!functions.is_empty(), "functions must not be empty");
    let tmp_path = db_path.with_extension("db.tmp");

    // Remove stale scratch file from a previous interrupted save
    let _ = std::fs::remove_file(&tmp_path);

    let conn = open_db(&tmp_path)?;
    create_schema(&conn)?;
    insert_functions(&conn, functions)?;
    insert_call_graph(&conn, calls)?;
    insert_graph_metrics(&conn, graph_metrics)?;
    insert_metadata(&conn, manifest)?;
    insert_coverage_off_files(&conn, coverage_off_files)?;

    // Close connection before rename
    drop(conn);

    // Atomic rename into place (same filesystem, so this is atomic on POSIX)
    std::fs::rename(&tmp_path, db_path)
        .map_err(|e| format!("Failed to rename temp DB into place: {e}"))?;

    // Clean up stale WAL/SHM files from the old DB (rename doesn't move them)
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));

    eprintln!(
        "  SQLite index saved: {} functions, {} call edges, {}",
        functions.len(),
        calls.values().map(|v| v.len()).sum::<usize>(),
        humanize_bytes(db_path.metadata().map(|m| m.len()).unwrap_or(0)),
    );

    Ok(())
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn humanize_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
