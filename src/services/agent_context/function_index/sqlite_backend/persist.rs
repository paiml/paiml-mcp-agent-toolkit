#![cfg_attr(coverage_nightly, coverage(off))]

//! Persistence of quality gate results, entropy violations, and provability scores.
//!
//! These functions operate on an existing context.db and are called
//! after quality analysis completes.

use rusqlite::{params, Connection, OpenFlags};
use std::path::Path;

/// Persist quality gate violations to the SQLite database.
///
/// Opens the existing context.db (must already exist from index build),
/// clears old violations, and inserts the new set. This makes all quality
/// gate results queryable via `pmat sql`.
#[allow(clippy::type_complexity)]
pub(crate) fn persist_quality_violations(
    db_path: &Path,
    violations: &[(
        String,
        String,
        String,
        Option<usize>,
        String,
        Option<String>,
    )],
) -> Result<(), String> {
    if !db_path.exists() {
        return Err(format!(
            "No index database at {}. Run `pmat query` first to build the index.",
            db_path.display()
        ));
    }

    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("Failed to open DB for violations: {e}"))?;

    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")
        .map_err(|e| format!("Failed to set pragmas: {e}"))?;

    // Ensure table exists (handles DBs created before this feature)
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS quality_violations (
            id INTEGER PRIMARY KEY,
            check_type TEXT NOT NULL,
            severity TEXT NOT NULL,
            file_path TEXT NOT NULL,
            line INTEGER,
            message TEXT NOT NULL,
            details_json TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_qv_check_type ON quality_violations(check_type);
        CREATE INDEX IF NOT EXISTS idx_qv_file ON quality_violations(file_path);
        CREATE INDEX IF NOT EXISTS idx_qv_severity ON quality_violations(severity);",
    )
    .map_err(|e| format!("Failed to ensure quality_violations table: {e}"))?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin violation transaction: {e}"))?;

    // Clear previous violations
    tx.execute("DELETE FROM quality_violations", [])
        .map_err(|e| format!("Failed to clear quality_violations: {e}"))?;

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT INTO quality_violations (check_type, severity, file_path, line, message, details_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|e| format!("Failed to prepare violation insert: {e}"))?;

        for (check_type, severity, file_path, line, message, details_json) in violations {
            let line_val: Option<i64> = line.map(|l| l as i64);
            stmt.execute(params![
                check_type,
                severity,
                file_path,
                line_val,
                message,
                details_json
            ])
            .map_err(|e| format!("Failed to insert violation: {e}"))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit violations: {e}"))?;

    Ok(())
}

/// Persist entropy violation details to the `entropy_violations` table (#231).
///
/// Each tuple: (file_path, pattern_type, pattern_hash, repetitions, variation_score,
///               estimated_loc_reduction, severity, example_code)
#[allow(clippy::type_complexity)]
pub(crate) fn persist_entropy_violations(
    db_path: &Path,
    violations: &[(
        String,
        String,
        String,
        usize,
        f64,
        usize,
        String,
        Option<String>,
    )],
) -> Result<(), String> {
    if !db_path.exists() || violations.is_empty() {
        return Ok(());
    }

    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("Failed to open DB for entropy violations: {e}"))?;

    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")
        .map_err(|e| format!("Failed to set pragmas: {e}"))?;

    // Ensure table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entropy_violations (
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
        );",
    )
    .map_err(|e| format!("Failed to ensure entropy_violations table: {e}"))?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin entropy violation transaction: {e}"))?;

    tx.execute("DELETE FROM entropy_violations", [])
        .map_err(|e| format!("Failed to clear entropy_violations: {e}"))?;

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT OR IGNORE INTO entropy_violations
                 (file_path, pattern_type, pattern_hash, repetitions, variation_score,
                  estimated_loc_reduction, severity, example_code)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .map_err(|e| format!("Failed to prepare entropy insert: {e}"))?;

        for (file_path, pattern_type, pattern_hash, reps, var_score, loc_red, severity, example) in
            violations
        {
            stmt.execute(params![
                file_path,
                pattern_type,
                pattern_hash,
                *reps as i64,
                var_score,
                *loc_red as i64,
                severity,
                example
            ])
            .map_err(|e| format!("Failed to insert entropy violation: {e}"))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit entropy violations: {e}"))?;

    Ok(())
}

/// Persist per-function provability scores to the `provability_scores` table (#231).
///
/// Each tuple: (file_path, function_name, provability_score, verified_properties_count)
pub(crate) fn persist_provability_scores(
    db_path: &Path,
    scores: &[(String, String, f64, usize)],
) -> Result<(), String> {
    if !db_path.exists() || scores.is_empty() {
        return Ok(());
    }

    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("Failed to open DB for provability scores: {e}"))?;

    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")
        .map_err(|e| format!("Failed to set pragmas: {e}"))?;

    // Ensure table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS provability_scores (
            id INTEGER PRIMARY KEY,
            function_id INTEGER,
            file_path TEXT NOT NULL,
            function_name TEXT NOT NULL,
            provability_score REAL NOT NULL,
            verified_properties INTEGER DEFAULT 0,
            FOREIGN KEY (function_id) REFERENCES functions(id)
        );",
    )
    .map_err(|e| format!("Failed to ensure provability_scores table: {e}"))?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin provability transaction: {e}"))?;

    tx.execute("DELETE FROM provability_scores", [])
        .map_err(|e| format!("Failed to clear provability_scores: {e}"))?;

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT INTO provability_scores
                 (file_path, function_name, provability_score, verified_properties)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .map_err(|e| format!("Failed to prepare provability insert: {e}"))?;

        for (file_path, func_name, score, props_count) in scores {
            stmt.execute(params![file_path, func_name, score, *props_count as i64])
                .map_err(|e| format!("Failed to insert provability score: {e}"))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit provability scores: {e}"))?;

    Ok(())
}
