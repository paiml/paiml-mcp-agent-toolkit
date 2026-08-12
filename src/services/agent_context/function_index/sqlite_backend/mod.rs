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

// Re-import parent helpers and types so submodules can use `super::helpers` / `super::types`
use super::helpers;
use super::types;

mod insert;
mod load;
mod persist;
mod query;
mod save;
mod schema;

// --- Public re-exports (preserving original visibility) ---

// From schema (used by build.rs and external callers)
pub(crate) use schema::{has_valid_schema, open_db, stored_scale_is_current};
// Read by tests that assert what version the code under test writes, instead of
// hard-coding the literal (which turns a deliberate bump into a red test).
#[cfg(test)]
pub(crate) use schema::SCHEMA_VERSION;

// From save (used by build.rs)
pub(crate) use save::save_to_sqlite;

// From query (used by build.rs and query engine)
pub(crate) use query::{fts5_search, query_callees, query_callers};

// From load (used by build.rs and query engine)
pub(crate) use load::{
    load_call_graph, load_functions_lightweight, load_graph_metrics, load_metadata,
    load_source_by_location, load_source_map,
};

// From persist (used by quality gate handlers)
pub(crate) use persist::{
    persist_entropy_violations, persist_provability_scores, persist_quality_violations,
};

// Test-only re-exports: used by sibling test modules (sqlite_falsification_tests)
// and internal tests via `use sqlite_backend::*`
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use insert::{
    insert_call_graph, insert_coverage_off_files, insert_functions, insert_graph_metrics,
    insert_metadata,
};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use load::load_functions;
#[cfg(test)]
#[allow(unused_imports)]
pub(super) use schema::create_schema;

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::types::*;
    use super::*;
    use rusqlite::Connection;
    use std::collections::{HashMap, HashSet};

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
            linked_definition: None,
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
            make_test_entry(
                "handle_request",
                "fn handle_request() { validate(); }",
                "src/server.rs",
            ),
            make_test_entry("validate", "fn validate() { check_auth(); }", "src/auth.rs"),
            make_test_entry(
                "render_page",
                "fn render_page() { template(); }",
                "src/view.rs",
            ),
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
            make_test_entry(
                "handle_request",
                "fn handle_request() { validate(); process_response(); }",
                "src/server.rs",
            ),
            make_test_entry(
                "validate_input",
                "fn validate_input() { check_bounds(); }",
                "src/validation.rs",
            ),
            make_test_entry(
                "render_page",
                "fn render_page() { template_engine(); css_loader(); }",
                "src/view.rs",
            ),
        ];

        insert_functions(&conn, &functions).unwrap();

        let results = fts5_search(&conn, "validate", 10).unwrap();
        assert!(
            !results.is_empty(),
            "should find functions matching 'validate'"
        );
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
            tdg_scale: crate::services::agent_context::TDG_SCALE.to_string(),
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
        // insert_metadata writes the CURRENT SCHEMA_VERSION, not whatever the
        // caller's manifest happened to carry.
        assert_eq!(version, SCHEMA_VERSION);

        let scale: String = conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'tdg_scale'",
                [],
                |r| r.get(0),
            )
            .expect("R30: every saved index must record the TDG scale it used");
        assert_eq!(scale, crate::services::agent_context::TDG_SCALE);
    }

    #[test]
    fn test_save_to_sqlite_roundtrip() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let functions = vec![
            make_test_entry(
                "handle_error",
                "fn handle_error() { log_error(); notify(); }",
                "src/error.rs",
            ),
            make_test_entry(
                "log_error",
                "fn log_error() { write_log(); }",
                "src/logging.rs",
            ),
        ];

        let mut calls = HashMap::new();
        calls.insert(0, vec![1]);

        let metrics = vec![
            GraphMetrics {
                pagerank: 0.6,
                centrality: 0.4,
                in_degree: 0,
                out_degree: 1,
            },
            GraphMetrics {
                pagerank: 0.8,
                centrality: 0.5,
                in_degree: 1,
                out_degree: 0,
            },
        ];

        let manifest = IndexManifest {
            version: "2.0.0".to_string(),
            built_at: "2026-02-07T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 2,
            file_count: 2,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 0.0,
            tdg_scale: crate::services::agent_context::TDG_SCALE.to_string(),
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        };

        save_to_sqlite(
            &db_path,
            &functions,
            &calls,
            &metrics,
            &manifest,
            &HashSet::new(),
        )
        .unwrap();

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
        use super::query::tokenize_query_for_fts5;
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
        use super::save::humanize_bytes;
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
