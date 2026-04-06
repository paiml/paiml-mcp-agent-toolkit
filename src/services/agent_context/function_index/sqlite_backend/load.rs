#![cfg_attr(coverage_nightly, coverage(off))]

//! Load operations for functions, call graph, metrics, and metadata.
//!
//! Supports both full and lightweight (no source) loading patterns,
//! plus on-demand source backfill.

use super::types::*;
use rusqlite::{params, Connection};
use std::collections::HashMap;

fn parse_definition_type(s: &str) -> DefinitionType {
    match s {
        "Struct" => DefinitionType::Struct,
        "Enum" => DefinitionType::Enum,
        "Trait" => DefinitionType::Trait,
        "TypeAlias" => DefinitionType::TypeAlias,
        _ => DefinitionType::Function,
    }
}

#[allow(clippy::cast_possible_truncation)]
fn read_quality_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<QualityMetrics> {
    Ok(QualityMetrics {
        tdg_score: row.get::<_, f64>(10)? as f32,
        tdg_grade: row.get(11)?,
        complexity: row.get::<_, i64>(12)? as u32,
        cognitive_complexity: row.get::<_, i64>(13)? as u32,
        big_o: row.get(14)?,
        satd_count: row.get::<_, i64>(15)? as u32,
        loc: row.get::<_, i64>(16)? as u32,
        commit_count: row.get::<_, i64>(17)? as u32,
        churn_score: row.get::<_, f64>(18)? as f32,
    })
}

/// Load all functions from the SQLite database.
#[allow(dead_code)]
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn load_functions(conn: &Connection) -> Result<Vec<FunctionEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT file_path, function_name, signature, definition_type, doc_comment,
                    source, start_line, end_line, language, checksum,
                    tdg_score, tdg_grade, complexity, cognitive_complexity, big_o,
                    satd_count, loc, commit_count, churn_score, clone_count,
                    pattern_diversity, fault_annotations
             FROM functions ORDER BY id",
        )
        .map_err(|e| format!("Failed to prepare load: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let def_type_str: String = row.get(3)?;
            let faults_json: String = row.get(21)?;
            let fault_annotations: Vec<String> =
                serde_json::from_str(&faults_json).unwrap_or_default();

            Ok(FunctionEntry {
                file_path: row.get(0)?,
                function_name: row.get(1)?,
                signature: row.get(2)?,
                definition_type: parse_definition_type(&def_type_str),
                doc_comment: row.get(4)?,
                source: row.get(5)?,
                start_line: row.get::<_, i64>(6)? as usize,
                end_line: row.get::<_, i64>(7)? as usize,
                language: row.get(8)?,
                quality: read_quality_from_row(row)?,
                checksum: row.get(9)?,
                commit_count: row.get::<_, i64>(17)? as u32,
                churn_score: row.get::<_, f64>(18)? as f32,
                clone_count: row.get::<_, i64>(19)? as u32,
                pattern_diversity: row.get::<_, f64>(20)? as f32,
                fault_annotations,
                linked_definition: None,
            })
        })
        .map_err(|e| format!("Failed to query functions: {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect functions: {e}"))
}

/// Load all functions from SQLite without the `source` column (lightweight).
///
/// Saves ~200ms by skipping deserialization of 70K source strings (~35MB).
/// Source is loaded on-demand via `load_source_by_location()` or `load_source_into()`.
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn load_functions_lightweight(conn: &Connection) -> Result<Vec<FunctionEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT file_path, function_name, signature, definition_type, doc_comment,
                    start_line, end_line, language, checksum,
                    tdg_score, tdg_grade, complexity, cognitive_complexity, big_o,
                    satd_count, loc, commit_count, churn_score, clone_count,
                    pattern_diversity, fault_annotations
             FROM functions ORDER BY id",
        )
        .map_err(|e| format!("Failed to prepare lightweight load: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let def_type_str: String = row.get(3)?;
            let faults_json: String = row.get(20)?;
            let fault_annotations: Vec<String> =
                serde_json::from_str(&faults_json).unwrap_or_default();

            Ok(FunctionEntry {
                file_path: row.get(0)?,
                function_name: row.get(1)?,
                signature: row.get(2)?,
                definition_type: parse_definition_type(&def_type_str),
                doc_comment: row.get(4)?,
                source: String::new(), // Deferred — loaded on-demand
                start_line: row.get::<_, i64>(5)? as usize,
                end_line: row.get::<_, i64>(6)? as usize,
                language: row.get(7)?,
                quality: QualityMetrics {
                    tdg_score: row.get::<_, f64>(9)? as f32,
                    tdg_grade: row.get(10)?,
                    complexity: row.get::<_, i64>(11)? as u32,
                    cognitive_complexity: row.get::<_, i64>(12)? as u32,
                    big_o: row.get(13)?,
                    satd_count: row.get::<_, i64>(14)? as u32,
                    loc: row.get::<_, i64>(15)? as u32,
                    commit_count: row.get::<_, i64>(16)? as u32,
                    churn_score: row.get::<_, f64>(17)? as f32,
                },
                checksum: row.get(8)?,
                commit_count: row.get::<_, i64>(16)? as u32,
                churn_score: row.get::<_, f64>(17)? as f32,
                clone_count: row.get::<_, i64>(18)? as u32,
                pattern_diversity: row.get::<_, f64>(19)? as f32,
                fault_annotations,
                linked_definition: None,
            })
        })
        .map_err(|e| format!("Failed to query functions (lightweight): {e}"))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect functions (lightweight): {e}"))
}

/// Bulk-load source code into an existing functions slice.
///
/// Reads `(id, source)` from SQLite and updates `functions[id-1].source`.
/// Used when regex/literal mode needs full source for pattern matching.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn load_source_into(
    conn: &Connection,
    functions: &mut [FunctionEntry],
) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id, source FROM functions ORDER BY id")
        .map_err(|e| format!("Failed to prepare source load: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let source: String = row.get(1)?;
            Ok((id, source))
        })
        .map_err(|e| format!("Failed to query source: {e}"))?;

    for row in rows {
        let (id, source) = row.map_err(|e| format!("Bad source row: {e}"))?;
        let idx = (id - 1) as usize;
        if idx < functions.len() {
            functions[idx].source = source;
        }
    }

    Ok(())
}

/// Load source code for a single function by file path and start line.
///
/// Uses the `idx_functions_file` index for O(log n) lookup.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "non_empty_index")]
pub(crate) fn load_source_by_location(
    conn: &Connection,
    file_path: &str,
    start_line: usize,
) -> Result<String, String> {
    conn.query_row(
        "SELECT source FROM functions WHERE file_path = ?1 AND start_line = ?2 LIMIT 1",
        params![file_path, start_line as i64],
        |row| row.get(0),
    )
    .map_err(|e| format!("Failed to load source for {file_path}:{start_line}: {e}"))
}

/// Load all call graph edges from the SQLite database.
///
/// Kept for backward compat and tests. Normal load path uses on-demand
/// `query_callees()`/`query_callers()` instead.
#[allow(dead_code, clippy::type_complexity)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn load_call_graph(
    conn: &Connection,
) -> Result<(HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>), String> {
    let mut stmt = conn
        .prepare("SELECT caller_id, callee_id FROM call_graph")
        .map_err(|e| format!("Failed to prepare call_graph load: {e}"))?;

    let mut calls: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut called_by: HashMap<usize, Vec<usize>> = HashMap::new();

    let rows = stmt
        .query_map([], |row| {
            let caller: i64 = row.get(0)?;
            let callee: i64 = row.get(1)?;
            Ok(((caller - 1) as usize, (callee - 1) as usize))
        })
        .map_err(|e| format!("Failed to query call_graph: {e}"))?;

    for row in rows {
        let (caller, callee) = row.map_err(|e| format!("Bad call_graph row: {e}"))?;
        calls.entry(caller).or_default().push(callee);
        called_by.entry(callee).or_default().push(caller);
    }

    Ok((calls, called_by))
}

/// Load graph metrics from the SQLite database.
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn load_graph_metrics(conn: &Connection) -> Result<Vec<GraphMetrics>, String> {
    let count: i64 = conn
        .query_row("SELECT count(*) FROM graph_metrics", [], |r| r.get(0))
        .map_err(|e| format!("Failed to count metrics: {e}"))?;

    let mut metrics = vec![GraphMetrics::default(); count as usize];

    let mut stmt = conn
        .prepare(
            "SELECT function_id, pagerank, centrality, in_degree, out_degree FROM graph_metrics",
        )
        .map_err(|e| format!("Failed to prepare metrics load: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let pagerank: f64 = row.get(1)?;
            let centrality: f64 = row.get(2)?;
            let in_degree: i64 = row.get(3)?;
            let out_degree: i64 = row.get(4)?;
            Ok((id, pagerank, centrality, in_degree, out_degree))
        })
        .map_err(|e| format!("Failed to query metrics: {e}"))?;

    for row in rows {
        let (id, pr, cent, ind, outd) = row.map_err(|e| format!("Bad metric row: {e}"))?;
        let idx = (id - 1) as usize;
        if idx < metrics.len() {
            metrics[idx] = GraphMetrics {
                pagerank: pr as f32,
                centrality: cent as f32,
                in_degree: ind as u32,
                out_degree: outd as u32,
            };
        }
    }

    Ok(metrics)
}

/// Load index metadata from the SQLite database.
///
/// Reads all metadata key-value pairs in a single query instead of 6 individual queries.
/// Self-heals on missing keys by using sensible defaults (#162).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn load_metadata(conn: &Connection) -> Result<IndexManifest, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM metadata")
        .map_err(|e| format!("Failed to prepare metadata load: {e}"))?;

    let rows: HashMap<String, String> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to query metadata: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    // Self-heal: use defaults for missing metadata instead of failing (#162)
    let version = rows
        .get("version")
        .cloned()
        .unwrap_or_else(|| "2.0".to_string());
    let built_at = rows
        .get("built_at")
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());
    let project_root = rows
        .get("project_root")
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    let function_count: usize = rows
        .get("function_count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let file_count: usize = rows
        .get("file_count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let checksums_json = rows
        .get("file_checksums")
        .cloned()
        .unwrap_or_else(|| "{}".to_string());
    let file_checksums: HashMap<String, String> =
        serde_json::from_str(&checksums_json).unwrap_or_default();

    // If critical metadata was missing, try to self-heal by inserting defaults
    if !rows.contains_key("version") {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('version', '2.0')",
            [],
        );
    }

    Ok(IndexManifest {
        version,
        built_at,
        project_root,
        function_count,
        file_count,
        languages: Vec::new(), // Populated from functions
        avg_tdg_score: 0.0,
        file_checksums,
        last_incremental_changes: 0,
    })
}
