#![cfg_attr(coverage_nightly, coverage(off))]

//! SQLite + FTS5 backend for the function index.
//!
//! Replaces the monolithic LZ4+bincode blob with a SQLite database that provides:
//! - BM25 ranking via FTS5 (Robertson & Zaragoza, 2009)
//! - Inverted index for O(1) per-term lookup (Zobel & Moffat, 2006)
//! - Incremental updates via checksum-based upsert
//! - Partial loading (query what you need, not the whole index)
//!
//! Spec: docs/specifications/index-v2-sqlite-fts5.md

use super::helpers::{extract_identifiers, is_keyword};
use super::types::*;
use rusqlite::{params, Connection, OpenFlags};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Database schema version for migration tracking
const SCHEMA_VERSION: &str = "2.0.0";

/// Open or create a SQLite index database at the given path.
pub(crate) fn open_db(db_path: &Path) -> Result<Connection, String> {
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("Failed to open index DB: {e}"))?;

    // Performance + concurrency pragmas
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA cache_size = -64000;
         PRAGMA mmap_size = 268435456;
         PRAGMA temp_store = MEMORY;",
    )
    .map_err(|e| format!("Failed to set pragmas: {e}"))?;

    Ok(conn)
}

/// Create all tables and indexes if they don't exist.
pub(super) fn create_schema(conn: &Connection) -> Result<(), String> {
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
            fault_annotations TEXT NOT NULL DEFAULT '[]'
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
        CREATE INDEX IF NOT EXISTS idx_call_graph_callee ON call_graph(callee_id);",
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

    Ok(())
}

/// Insert all functions into the database within a transaction.
pub(super) fn insert_functions(
    conn: &Connection,
    functions: &[FunctionEntry],
) -> Result<(), String> {
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
pub(super) fn insert_call_graph(
    conn: &Connection,
    calls: &HashMap<usize, Vec<usize>>,
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    {
        let mut stmt = tx
            .prepare_cached("INSERT OR IGNORE INTO call_graph (caller_id, callee_id) VALUES (?1, ?2)")
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
pub(super) fn insert_graph_metrics(
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
            stmt.execute(params![func_id, m.pagerank, m.centrality, m.in_degree, m.out_degree])
                .map_err(|e| format!("Failed to insert metric {}: {e}", idx))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit metrics: {e}"))?;
    Ok(())
}

/// Insert metadata key-value pairs.
pub(super) fn insert_metadata(
    conn: &Connection,
    manifest: &IndexManifest,
) -> Result<(), String> {
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
fn insert_coverage_off_files(conn: &Connection, files: &HashSet<String>) -> Result<(), String> {
    let json = serde_json::to_string(files).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('coverage_off_files', ?1)",
        params![json],
    )
    .map_err(|e| format!("Failed to insert coverage_off_files: {e}"))?;
    Ok(())
}

/// Save the full index to a SQLite database.
///
/// Uses atomic write: builds a temporary DB, then renames into place.
/// This prevents concurrent readers from seeing a partial/empty file.
pub(crate) fn save_to_sqlite(
    db_path: &Path,
    functions: &[FunctionEntry],
    calls: &HashMap<usize, Vec<usize>>,
    graph_metrics: &[GraphMetrics],
    manifest: &IndexManifest,
    coverage_off_files: &HashSet<String>,
) -> Result<(), String> {
    let tmp_path = db_path.with_extension("db.tmp");

    // Remove stale temp file from a previous interrupted save
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

/// Execute a BM25 search query using FTS5.
///
/// Returns (function_id (0-based), bm25_score) pairs sorted by relevance.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn fts5_search(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<(usize, f32)>, String> {
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
fn tokenize_query_for_fts5(query: &str) -> String {
    query
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| s.len() >= 2 && !is_keyword(s))
        .map(|s| format!("\"{}\"", s.to_lowercase()))
        .collect::<Vec<_>>()
        .join(" ")
}

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
pub(crate) fn load_source_into(conn: &Connection, functions: &mut [FunctionEntry]) -> Result<(), String> {
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
pub(crate) fn load_graph_metrics(conn: &Connection) -> Result<Vec<GraphMetrics>, String> {
    let count: i64 = conn
        .query_row("SELECT count(*) FROM graph_metrics", [], |r| r.get(0))
        .map_err(|e| format!("Failed to count metrics: {e}"))?;

    let mut metrics = vec![GraphMetrics::default(); count as usize];

    let mut stmt = conn
        .prepare("SELECT function_id, pagerank, centrality, in_degree, out_degree FROM graph_metrics")
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

    let version = rows.get("version").ok_or("Missing metadata key 'version'")?.clone();
    let built_at = rows.get("built_at").ok_or("Missing metadata key 'built_at'")?.clone();
    let project_root = rows.get("project_root").ok_or("Missing metadata key 'project_root'")?.clone();
    let function_count: usize = rows.get("function_count")
        .ok_or("Missing metadata key 'function_count'")?
        .parse()
        .map_err(|e| format!("Bad function_count: {e}"))?;
    let file_count: usize = rows.get("file_count")
        .ok_or("Missing metadata key 'file_count'")?
        .parse()
        .map_err(|e| format!("Bad file_count: {e}"))?;
    let checksums_json = rows.get("file_checksums").cloned().unwrap_or_else(|| "{}".to_string());
    let file_checksums: HashMap<String, String> =
        serde_json::from_str(&checksums_json).unwrap_or_default();

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

/// Check if the database has a valid v2.0 schema (all required tables exist).
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

fn humanize_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_entry(name: &str, source: &str, file: &str) -> FunctionEntry {
        FunctionEntry {
            file_path: file.to_string(),
            function_name: name.to_string(),
            signature: format!("fn {name}()"),
            definition_type: DefinitionType::Function,
            doc_comment: Some(format!("Documentation for {name}")),
            source: source.to_string(),
            start_line: 1,
            end_line: 10,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: format!("checksum_{name}"),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        }
    }

    #[test]
    fn test_create_schema() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        // Verify tables exist
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='functions'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_and_count_functions() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();

        let functions = vec![
            make_test_entry("handle_request", "fn handle_request() { validate(); }", "src/server.rs"),
            make_test_entry("validate", "fn validate() { check_auth(); }", "src/auth.rs"),
            make_test_entry("render_page", "fn render_page() { template(); }", "src/view.rs"),
        ];

        insert_functions(&conn, &functions).unwrap();

        let count: i64 = conn
            .query_row("SELECT count(*) FROM functions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_fts5_search_basic() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();

        let functions = vec![
            make_test_entry("handle_request", "fn handle_request() { validate(); process_response(); }", "src/server.rs"),
            make_test_entry("validate_input", "fn validate_input() { check_bounds(); }", "src/validation.rs"),
            make_test_entry("render_page", "fn render_page() { template_engine(); css_loader(); }", "src/view.rs"),
        ];

        insert_functions(&conn, &functions).unwrap();

        let results = fts5_search(&conn, "validate", 10).unwrap();
        assert!(!results.is_empty(), "should find functions matching 'validate'");
        // validate_input should rank higher (name match)
        let top_name = &functions[results[0].0].function_name;
        assert!(
            top_name.contains("validate"),
            "top result should match 'validate', got '{top_name}'"
        );
    }

    #[test]
    fn test_fts5_search_empty_query() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        let results = fts5_search(&conn, "", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_fts5_search_no_results() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();

        let functions = vec![make_test_entry("alpha", "fn alpha() {}", "a.rs")];
        insert_functions(&conn, &functions).unwrap();

        let results = fts5_search(&conn, "zzz_nonexistent_xyz", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_insert_call_graph() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();

        let functions = vec![
            make_test_entry("caller", "fn caller() { callee(); }", "a.rs"),
            make_test_entry("callee", "fn callee() {}", "a.rs"),
        ];
        insert_functions(&conn, &functions).unwrap();

        let mut calls = HashMap::new();
        calls.insert(0, vec![1]);
        insert_call_graph(&conn, &calls).unwrap();

        let count: i64 = conn
            .query_row("SELECT count(*) FROM call_graph", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_graph_metrics() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();

        let functions = vec![make_test_entry("func", "fn func() {}", "a.rs")];
        insert_functions(&conn, &functions).unwrap();

        let metrics = vec![GraphMetrics {
            pagerank: 0.5,
            centrality: 0.3,
            in_degree: 2,
            out_degree: 1,
        }];
        insert_graph_metrics(&conn, &metrics).unwrap();

        let pr: f64 = conn
            .query_row(
                "SELECT pagerank FROM graph_metrics WHERE function_id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!((pr - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_insert_metadata() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();

        let manifest = IndexManifest {
            version: "2.0.0".to_string(),
            built_at: "2026-02-07T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 42,
            file_count: 10,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 1.5,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        };
        insert_metadata(&conn, &manifest).unwrap();

        let version: String = conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'version'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(version, "2.0.0");
    }

    #[test]
    fn test_save_to_sqlite_roundtrip() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let functions = vec![
            make_test_entry("handle_error", "fn handle_error() { log_error(); notify(); }", "src/error.rs"),
            make_test_entry("log_error", "fn log_error() { write_log(); }", "src/logging.rs"),
        ];

        let mut calls = HashMap::new();
        calls.insert(0, vec![1]);

        let metrics = vec![
            GraphMetrics { pagerank: 0.6, centrality: 0.4, in_degree: 0, out_degree: 1 },
            GraphMetrics { pagerank: 0.8, centrality: 0.5, in_degree: 1, out_degree: 0 },
        ];

        let manifest = IndexManifest {
            version: "2.0.0".to_string(),
            built_at: "2026-02-07T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 2,
            file_count: 2,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        };

        save_to_sqlite(&db_path, &functions, &calls, &metrics, &manifest, &HashSet::new()).unwrap();

        // Verify file exists and has reasonable size
        assert!(db_path.exists());
        let size = db_path.metadata().unwrap().len();
        assert!(size > 0 && size < 1_000_000, "DB should be small: {size}");

        // Verify search works
        let conn = open_db(&db_path).unwrap();
        let results = fts5_search(&conn, "error handling", 10).unwrap();
        assert!(!results.is_empty(), "should find 'error handling' results");
    }

    #[test]
    fn test_tokenize_query_for_fts5() {
        assert_eq!(
            tokenize_query_for_fts5("error handling"),
            "\"error\" \"handling\""
        );
        assert_eq!(tokenize_query_for_fts5("fn let if"), ""); // all keywords
        assert_eq!(
            tokenize_query_for_fts5("parse_request validation"),
            "\"parse_request\" \"validation\""
        );
    }

    #[test]
    fn test_humanize_bytes() {
        assert_eq!(humanize_bytes(500), "500 B");
        assert_eq!(humanize_bytes(2048), "2.0 KB");
        assert_eq!(humanize_bytes(5_242_880), "5.0 MB");
    }

    #[test]
    fn test_has_valid_schema_with_tables() {
        let conn = Connection::open_in_memory().expect("open in-memory");
        create_schema(&conn).expect("create schema");
        assert!(has_valid_schema(&conn));
    }

    #[test]
    fn test_has_valid_schema_empty_db() {
        let conn = Connection::open_in_memory().expect("open in-memory");
        assert!(!has_valid_schema(&conn));
    }

    #[test]
    fn test_has_valid_schema_partial_tables() {
        let conn = Connection::open_in_memory().expect("open in-memory");
        conn.execute_batch("CREATE TABLE functions (id INTEGER PRIMARY KEY)")
            .expect("create partial");
        assert!(!has_valid_schema(&conn));
    }
}
