#![cfg_attr(coverage_nightly, coverage(off))]

//! FTS5 search and on-demand call graph queries.
//!
//! Provides BM25-ranked full-text search via FTS5 and per-function
//! call graph lookups for lazy loading.

use super::helpers::is_keyword;
use rusqlite::{params, Connection};

/// Execute a BM25 search query using FTS5.
///
/// Returns (function_id (0-based), bm25_score) pairs sorted by relevance.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn fts5_search(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<(usize, f32)>, String> {
    debug_assert!(!query.is_empty(), "query must not be empty");
    // FTS5 match syntax: quote terms for phrase, OR for alternatives
    let fts_query = tokenize_query_for_fts5(query);
    if fts_query.is_empty() {
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare_cached(
            "SELECT rowid, rank
             FROM functions_fts
             WHERE functions_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )
        .map_err(|e| format!("FTS5 query failed: {e}"))?;

    let results: Vec<(usize, f32)> = stmt
        .query_map(params![fts_query, limit as i64], |row| {
            let rowid: i64 = row.get(0)?;
            let rank: f64 = row.get(1)?;
            // FTS5 rank is negative (lower = better), convert to positive score
            Ok((
                (rowid - 1) as usize, // Convert 1-based to 0-based
                (-rank) as f32,
            ))
        })
        .map_err(|e| format!("FTS5 query_map failed: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    // Normalize scores to 0-1
    let max_score = results.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
    if max_score > 0.0 {
        Ok(results
            .into_iter()
            .map(|(idx, s)| (idx, s / max_score))
            .collect())
    } else {
        Ok(results)
    }
}

/// Convert a natural language query to FTS5 match syntax.
///
/// Splits into tokens, filters keywords/stop words, joins with implicit AND.
pub(crate) fn tokenize_query_for_fts5(query: &str) -> String {
    debug_assert!(!query.is_empty(), "query must not be empty");
    query
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| s.len() >= 2 && !is_keyword(s))
        .map(|s| format!("\"{}\"", s.to_lowercase()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Query call graph for a single function from SQLite (on-demand).
///
/// Returns 0-based callee indices for `get_calls()`.
pub(crate) fn query_callees(conn: &Connection, func_idx: usize) -> Result<Vec<usize>, String> {
    let caller_id = (func_idx + 1) as i64;
    let mut stmt = conn
        .prepare_cached("SELECT callee_id FROM call_graph WHERE caller_id = ?1")
        .map_err(|e| format!("Failed to prepare callees query: {e}"))?;
    let rows = stmt
        .query_map(params![caller_id], |row| {
            let id: i64 = row.get(0)?;
            Ok((id - 1) as usize)
        })
        .map_err(|e| format!("Failed to query callees: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Bad callee row: {e}"))
}

/// Query call graph for a single function from SQLite (on-demand).
///
/// Returns 0-based caller indices for `get_called_by()`.
pub(crate) fn query_callers(conn: &Connection, func_idx: usize) -> Result<Vec<usize>, String> {
    let callee_id = (func_idx + 1) as i64;
    let mut stmt = conn
        .prepare_cached("SELECT caller_id FROM call_graph WHERE callee_id = ?1")
        .map_err(|e| format!("Failed to prepare callers query: {e}"))?;
    let rows = stmt
        .query_map(params![callee_id], |row| {
            let id: i64 = row.get(0)?;
            Ok((id - 1) as usize)
        })
        .map_err(|e| format!("Failed to query callers: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Bad caller row: {e}"))
}
