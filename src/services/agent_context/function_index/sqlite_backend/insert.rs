#![cfg_attr(coverage_nightly, coverage(off))]

//! Batch insert operations for functions, call graph, metrics, and metadata.
//!
//! All inserts use transactions for atomicity and `prepare_cached` for performance.

use super::helpers::extract_identifiers;
use super::schema::SCHEMA_VERSION;
use super::types::*;
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};

/// Insert all functions into the database within a transaction.
pub(crate) fn insert_functions(conn: &Connection, functions: &[FunctionEntry]) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    // Clear existing data
    tx.execute_batch("DELETE FROM functions; DELETE FROM functions_fts; DELETE FROM call_graph; DELETE FROM graph_metrics;")
        .map_err(|e| format!("Failed to clear tables: {e}"))?;

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT INTO functions (
                    file_path, function_name, signature, definition_type, doc_comment,
                    source, start_line, end_line, language, checksum,
                    tdg_score, tdg_grade, complexity, cognitive_complexity, big_o,
                    satd_count, loc, commit_count, churn_score, clone_count,
                    pattern_diversity, fault_annotations
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
            )
            .map_err(|e| format!("Failed to prepare insert: {e}"))?;

        let mut fts_stmt = tx
            .prepare_cached(
                "INSERT INTO functions_fts (rowid, function_name, signature, doc_comment, file_path, identifiers)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|e| format!("Failed to prepare FTS insert: {e}"))?;

        for (idx, func) in functions.iter().enumerate() {
            let def_type = format!("{:?}", func.definition_type);
            let faults_json = serde_json::to_string(&func.fault_annotations).unwrap_or_default();

            stmt.execute(params![
                func.file_path,
                func.function_name,
                func.signature,
                def_type,
                func.doc_comment,
                func.source,
                func.start_line,
                func.end_line,
                func.language,
                func.checksum,
                func.quality.tdg_score,
                func.quality.tdg_grade,
                func.quality.complexity,
                func.quality.cognitive_complexity,
                func.quality.big_o,
                func.quality.satd_count,
                func.quality.loc,
                func.commit_count,
                func.churn_score,
                func.clone_count,
                func.pattern_diversity,
                faults_json,
            ])
            .map_err(|e| format!("Failed to insert function {}: {e}", idx))?;

            let rowid = (idx + 1) as i64; // SQLite rowid is 1-based
            let identifiers = extract_identifiers(&func.source);
            fts_stmt
                .execute(params![
                    rowid,
                    func.function_name,
                    func.signature,
                    func.doc_comment.as_deref().unwrap_or(""),
                    func.file_path,
                    identifiers,
                ])
                .map_err(|e| format!("Failed to insert FTS for {}: {e}", idx))?;
        }
    } // stmt + fts_stmt dropped here, releasing borrow on tx

    tx.commit()
        .map_err(|e| format!("Failed to commit functions: {e}"))?;
    Ok(())
}

/// Insert call graph edges into the database.
pub(crate) fn insert_call_graph(
    conn: &Connection,
    calls: &HashMap<usize, Vec<usize>>,
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT OR IGNORE INTO call_graph (caller_id, callee_id) VALUES (?1, ?2)",
            )
            .map_err(|e| format!("Failed to prepare call_graph insert: {e}"))?;

        for (caller, callees) in calls {
            let caller_id = (*caller + 1) as i64;
            for callee in callees {
                let callee_id = (*callee + 1) as i64;
                stmt.execute(params![caller_id, callee_id])
                    .map_err(|e| format!("Failed to insert call edge: {e}"))?;
            }
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit call graph: {e}"))?;
    Ok(())
}

/// Insert graph metrics (PageRank, centrality) into the database.
pub(crate) fn insert_graph_metrics(
    conn: &Connection,
    metrics: &[GraphMetrics],
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT OR REPLACE INTO graph_metrics (function_id, pagerank, centrality, in_degree, out_degree)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(|e| format!("Failed to prepare metrics insert: {e}"))?;

        for (idx, m) in metrics.iter().enumerate() {
            let func_id = (idx + 1) as i64;
            stmt.execute(params![
                func_id,
                m.pagerank,
                m.centrality,
                m.in_degree,
                m.out_degree
            ])
            .map_err(|e| format!("Failed to insert metric {}: {e}", idx))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit metrics: {e}"))?;
    Ok(())
}

/// Insert metadata key-value pairs.
pub(crate) fn insert_metadata(conn: &Connection, manifest: &IndexManifest) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('version', ?1)",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("Failed to insert version: {e}"))?;

    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('built_at', ?1)",
        params![manifest.built_at],
    )
    .map_err(|e| format!("Failed to insert built_at: {e}"))?;

    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('project_root', ?1)",
        params![manifest.project_root],
    )
    .map_err(|e| format!("Failed to insert project_root: {e}"))?;

    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('function_count', ?1)",
        params![manifest.function_count.to_string()],
    )
    .map_err(|e| format!("Failed to insert function_count: {e}"))?;

    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('file_count', ?1)",
        params![manifest.file_count.to_string()],
    )
    .map_err(|e| format!("Failed to insert file_count: {e}"))?;

    let checksums_json = serde_json::to_string(&manifest.file_checksums).unwrap_or_default();
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('file_checksums', ?1)",
        params![checksums_json],
    )
    .map_err(|e| format!("Failed to insert checksums: {e}"))?;

    Ok(())
}

/// Store coverage_off_files set as JSON in metadata for O(1) query-time lookup.
pub(crate) fn insert_coverage_off_files(
    conn: &Connection,
    files: &HashSet<String>,
) -> Result<(), String> {
    let json = serde_json::to_string(files).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('coverage_off_files', ?1)",
        params![json],
    )
    .map_err(|e| format!("Failed to insert coverage_off_files: {e}"))?;
    Ok(())
}
