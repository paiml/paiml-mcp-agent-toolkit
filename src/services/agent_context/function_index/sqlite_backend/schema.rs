#![cfg_attr(coverage_nightly, coverage(off))]

//! Database schema creation and connection setup.
//!
//! Handles opening SQLite connections with optimal pragmas and creating
//! all tables, indexes, and the FTS5 virtual table.

use rusqlite::{Connection, OpenFlags};
use std::path::Path;

/// Database schema version for migration tracking.
///
/// Bumped 2.0.0 → 3.0.0 in v3.30.0: the `functions.tdg_score` column changed
/// meaning (0-10 lower-is-better debt → 0-100 higher-is-better quality, the
/// scale `pmat tdg` reports) and `tdg_grade` widened from five letters to
/// `crate::tdg::Grade`'s eleven. The column layout is unchanged, so the version
/// alone would not stop a stale database from loading; the authoritative guard
/// is the `tdg_scale` metadata key checked by `stored_scale_is_current`.
pub(crate) const SCHEMA_VERSION: &str = "3.0.0";

/// Open or create a SQLite index database at the given path.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn open_db(db_path: &Path) -> Result<Connection, String> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("Failed to open index DB: {e}"))?;

    // Dynamic mmap: cover the full file to avoid read() syscall fallback.
    // For new DBs (size=0), use 256MB default. Cap at 2GB (SQLite limit on 32-bit).
    let file_size = std::fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);
    let mmap_size = if file_size > 0 {
        (file_size as i64 * 5 / 4).min(2_147_483_648) // 125% of file size, cap 2GB
    } else {
        268_435_456 // 256MB default for new DBs
    };

    conn.execute_batch(&format!(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA cache_size = -64000;
         PRAGMA mmap_size = {mmap_size};
         PRAGMA temp_store = MEMORY;",
    ))
    .map_err(|e| format!("Failed to set pragmas: {e}"))?;

    Ok(conn)
}

/// Create all tables and indexes if they don't exist.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn create_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS functions (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            function_name TEXT NOT NULL,
            signature TEXT NOT NULL,
            definition_type TEXT NOT NULL DEFAULT 'Function',
            doc_comment TEXT,
            source TEXT NOT NULL,
            start_line INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            language TEXT NOT NULL,
            checksum TEXT NOT NULL,
            tdg_score REAL NOT NULL DEFAULT 0.0,
            tdg_grade TEXT NOT NULL DEFAULT 'A',
            complexity INTEGER NOT NULL DEFAULT 1,
            cognitive_complexity INTEGER NOT NULL DEFAULT 1,
            big_o TEXT NOT NULL DEFAULT 'O(1)',
            satd_count INTEGER NOT NULL DEFAULT 0,
            loc INTEGER NOT NULL DEFAULT 0,
            commit_count INTEGER NOT NULL DEFAULT 0,
            churn_score REAL NOT NULL DEFAULT 0.0,
            clone_count INTEGER NOT NULL DEFAULT 0,
            pattern_diversity REAL NOT NULL DEFAULT 0.0,
            fault_annotations TEXT NOT NULL DEFAULT '[]',
            contract_level TEXT,
            contract_equation TEXT
        );

        CREATE TABLE IF NOT EXISTS call_graph (
            caller_id INTEGER NOT NULL REFERENCES functions(id),
            callee_id INTEGER NOT NULL REFERENCES functions(id),
            PRIMARY KEY (caller_id, callee_id)
        ) WITHOUT ROWID;

        CREATE TABLE IF NOT EXISTS graph_metrics (
            function_id INTEGER PRIMARY KEY REFERENCES functions(id),
            pagerank REAL NOT NULL DEFAULT 0.0,
            centrality REAL NOT NULL DEFAULT 0.0,
            in_degree INTEGER NOT NULL DEFAULT 0,
            out_degree INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_functions_file ON functions(file_path);
        CREATE INDEX IF NOT EXISTS idx_functions_name ON functions(function_name);
        CREATE INDEX IF NOT EXISTS idx_functions_lang ON functions(language);
        CREATE INDEX IF NOT EXISTS idx_functions_grade ON functions(tdg_grade);
        CREATE INDEX IF NOT EXISTS idx_call_graph_callee ON call_graph(callee_id);

        CREATE TABLE IF NOT EXISTS entropy_violations (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            pattern_type TEXT NOT NULL,
            pattern_hash TEXT NOT NULL,
            repetitions INTEGER NOT NULL,
            variation_score REAL NOT NULL,
            estimated_loc_reduction INTEGER NOT NULL,
            severity TEXT NOT NULL,
            example_code TEXT,
            UNIQUE(file_path, pattern_hash)
        );

        CREATE TABLE IF NOT EXISTS provability_scores (
            id INTEGER PRIMARY KEY,
            function_id INTEGER,
            file_path TEXT NOT NULL,
            function_name TEXT NOT NULL,
            provability_score REAL NOT NULL,
            verified_properties INTEGER DEFAULT 0,
            FOREIGN KEY (function_id) REFERENCES functions(id)
        );

        CREATE TABLE IF NOT EXISTS quality_violations (
            id INTEGER PRIMARY KEY,
            check_type TEXT NOT NULL,
            severity TEXT NOT NULL,
            file_path TEXT NOT NULL,
            line INTEGER,
            message TEXT NOT NULL,
            details_json TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_entropy_file ON entropy_violations(file_path);
        CREATE INDEX IF NOT EXISTS idx_entropy_severity ON entropy_violations(severity);
        CREATE INDEX IF NOT EXISTS idx_provability_score ON provability_scores(provability_score);
        CREATE INDEX IF NOT EXISTS idx_provability_file ON provability_scores(file_path);
        CREATE INDEX IF NOT EXISTS idx_qv_check_type ON quality_violations(check_type);
        CREATE INDEX IF NOT EXISTS idx_qv_file ON quality_violations(file_path);
        CREATE INDEX IF NOT EXISTS idx_qv_severity ON quality_violations(severity);",
    )
    .map_err(|e| format!("Failed to create schema: {e}"))?;

    // FTS5 virtual table for BM25 search (standalone, not content-synced)
    // porter tokenizer provides stemming (Porter, 1980)
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS functions_fts USING fts5(
            function_name,
            signature,
            doc_comment,
            file_path,
            identifiers,
            tokenize='porter unicode61 remove_diacritics 2'
        );",
    )
    .map_err(|e| format!("Failed to create FTS5 table: {e}"))?;

    // Document index schema (for `pmat query --docs`)
    crate::services::agent_context::document_index::create_documents_schema(conn)?;

    Ok(())
}

/// Whether the stored `tdg_score` values were written on the CURRENT TDG scale.
///
/// R30: `pmat query`/`pmat sql` used to persist a 0-10 lower-is-better debt
/// number while `pmat tdg` reported 0-100 higher-is-better. Both scales fit the
/// same `REAL` column, so a stale database loads without complaint and every
/// stored `0.12` — the BEST possible legacy score — reads as 0.12/100, an F.
///
/// A missing marker is NOT treated as current. Pre-v3.30.0 builds wrote no
/// marker at all, so "absent" is precisely the stale case; an unmeasured signal
/// must never pass as a clean one.
pub(crate) fn stored_scale_is_current(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT value FROM metadata WHERE key = 'tdg_scale'",
        [],
        |r| r.get::<_, String>(0),
    )
    .map(|v| v == crate::services::agent_context::TDG_SCALE)
    .unwrap_or(false)
}

/// Check if the database has a valid v2.0 schema (all required tables exist).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn has_valid_schema(conn: &Connection) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('functions', 'metadata', 'call_graph', 'graph_metrics')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    count == 4
}
