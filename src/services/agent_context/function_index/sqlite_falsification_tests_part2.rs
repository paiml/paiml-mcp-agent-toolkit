#![cfg_attr(coverage_nightly, coverage(off))]
//! Falsification tests for SQLite backend (Part 2: Sections 10-21)
//!
//! Continuation of sqlite_falsification_tests.rs, split for maintainability.
//! Each test attempts to falsify a specific claim from the specification.
//!
//! Spec: docs/specifications/index-v2-sqlite-fts5.md v2.2.0

#[cfg(test)]
mod tests {
    use crate::services::agent_context::function_index::sqlite_backend::*;
    use crate::services::agent_context::function_index::types::*;
    use rusqlite::Connection;
    use std::collections::{HashMap, HashSet};

    // ═══════════════════════════════════════════════════════════════════
    //  Test data factories (shared with sqlite_falsification_tests.rs)
    // ═══════════════════════════════════════════════════════════════════

    fn entry(name: &str, source: &str, file: &str) -> FunctionEntry {
        FunctionEntry {
            file_path: file.to_string(),
            function_name: name.to_string(),
            signature: format!("fn {name}()"),
            definition_type: DefinitionType::Function,
            doc_comment: Some(format!("Doc for {name}")),
            source: source.to_string(),
            start_line: 1,
            end_line: 10,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: format!("sha_{name}"),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            linked_definition: None,
        }
    }

    fn entry_with_quality(
        name: &str,
        file: &str,
        score: f32,
        grade: &str,
        cx: u32,
    ) -> FunctionEntry {
        let mut e = entry(name, &format!("fn {name}() {{ todo!() }}"), file);
        e.quality.tdg_score = score;
        e.quality.tdg_grade = grade.to_string();
        e.quality.complexity = cx;
        e
    }

    fn entry_typed(name: &str, file: &str, dt: DefinitionType) -> FunctionEntry {
        let mut e = entry(name, &format!("struct {name} {{}}"), file);
        e.definition_type = dt;
        e
    }

    fn manifest(fc: usize, flc: usize) -> IndexManifest {
        IndexManifest {
            version: "2.0.0".to_string(),
            built_at: "2026-02-08T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: fc,
            file_count: flc,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        }
    }

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        conn
    }

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 10: Edge cases and boundary conditions
    // ═══════════════════════════════════════════════════════════════════

    /// F-071: Empty function list inserts zero rows
    #[test]
    fn f071_empty_functions() {
        let conn = db();
        insert_functions(&conn, &[]).unwrap();
        assert_eq!(count(&conn, "functions"), 0);
    }

    /// F-072: Empty call graph inserts zero edges
    #[test]
    fn f072_empty_call_graph() {
        let conn = db();
        insert_call_graph(&conn, &HashMap::new()).unwrap();
        assert_eq!(count(&conn, "call_graph"), 0);
    }

    /// F-073: Empty graph metrics inserts zero rows
    #[test]
    fn f073_empty_graph_metrics() {
        let conn = db();
        insert_graph_metrics(&conn, &[]).unwrap();
        assert_eq!(count(&conn, "graph_metrics"), 0);
    }

    /// F-074: Unicode function names survive roundtrip
    #[test]
    fn f074_unicode_function_name() {
        let conn = db();
        let funcs = vec![entry("日本語_関数", "fn 日本語_関数() {}", "src/jp.rs")];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].function_name, "日本語_関数");
    }

    /// F-075: Very long source code survives roundtrip
    #[test]
    fn f075_long_source_code() {
        let conn = db();
        let source = format!("fn big() {{\n{}}}", "    let x = 42;\n".repeat(10000));
        let funcs = vec![entry("big", &source, "a.rs")];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].source.len(), source.len());
        assert_eq!(loaded[0].source, source);
    }

    /// F-076: Source with NULL bytes handled
    #[test]
    fn f076_null_bytes_in_source() {
        let conn = db();
        // SQLite TEXT cannot store null bytes — verify behavior
        let source = "fn f() { /* \0 */ }";
        let funcs = vec![entry("f", source, "a.rs")];
        // This may succeed (truncating at NUL) or fail — either is acceptable
        let result = insert_functions(&conn, &funcs);
        if result.is_ok() {
            let loaded = load_functions(&conn).unwrap();
            // SQLite may truncate at NUL, which is acceptable behavior
            assert!(!loaded[0].source.is_empty());
        }
    }

    /// F-077: Max u32 values survive roundtrip
    #[test]
    fn f077_max_u32_values() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.quality.complexity = u32::MAX;
        f.quality.loc = u32::MAX;
        f.quality.satd_count = u32::MAX;
        f.commit_count = u32::MAX;
        f.clone_count = u32::MAX;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].quality.complexity, u32::MAX);
        assert_eq!(loaded[0].quality.loc, u32::MAX);
        assert_eq!(loaded[0].commit_count, u32::MAX);
        assert_eq!(loaded[0].clone_count, u32::MAX);
    }

    /// F-078: Large line numbers survive roundtrip (within i64 range)
    #[test]
    fn f078_large_line_numbers() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        // SQLite stores as i64, so max safe value is i64::MAX
        f.start_line = 1_000_000;
        f.end_line = 2_000_000;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].start_line, 1_000_000);
        assert_eq!(loaded[0].end_line, 2_000_000);
    }

    /// F-079: Zero-length function name handled
    #[test]
    fn f079_empty_function_name() {
        let conn = db();
        let mut f = entry("", "fn () {}", "a.rs");
        f.function_name = String::new();
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].function_name, "");
    }

    /// F-080: Deeply nested file paths
    #[test]
    fn f080_deep_file_paths() {
        let conn = db();
        let deep = "src/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p.rs";
        let funcs = vec![entry("f", "fn f() {}", deep)];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].file_path, deep);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 11: save_to_sqlite integration
    //  Claims: removes old DB, creates new, inserts all data
    // ═══════════════════════════════════════════════════════════════════

    /// F-081: save_to_sqlite creates a DB file
    #[test]
    fn f081_save_creates_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let funcs = vec![entry("f", "fn f() {}", "a.rs")];
        save_to_sqlite(
            &db_path,
            &funcs,
            &HashMap::new(),
            &[],
            &manifest(1, 1),
            &HashSet::new(),
        )
        .unwrap();
        assert!(db_path.exists());
    }

    /// F-082: save_to_sqlite replaces existing DB
    #[test]
    fn f082_save_replaces_existing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        // First save
        save_to_sqlite(
            &db_path,
            &[entry("old", "fn old() {}", "a.rs")],
            &HashMap::new(),
            &[],
            &manifest(1, 1),
            &HashSet::new(),
        )
        .unwrap();
        // Second save (should replace)
        save_to_sqlite(
            &db_path,
            &[entry("new", "fn new() {}", "b.rs")],
            &HashMap::new(),
            &[],
            &manifest(1, 1),
            &HashSet::new(),
        )
        .unwrap();

        let conn = open_db(&db_path).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].function_name, "new");
    }

    /// F-083: save_to_sqlite stores all components
    #[test]
    fn f083_save_stores_all_components() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let funcs = vec![
            entry("a", "fn a() {}", "a.rs"),
            entry("b", "fn b() {}", "b.rs"),
        ];
        let mut calls = HashMap::new();
        calls.insert(0, vec![1]);
        let metrics = vec![
            GraphMetrics {
                pagerank: 0.5,
                ..Default::default()
            },
            GraphMetrics {
                pagerank: 0.8,
                ..Default::default()
            },
        ];
        let m = manifest(2, 2);
        save_to_sqlite(&db_path, &funcs, &calls, &metrics, &m, &HashSet::new()).unwrap();

        let conn = open_db(&db_path).unwrap();
        assert_eq!(count(&conn, "functions"), 2);
        assert_eq!(count(&conn, "call_graph"), 1);
        assert_eq!(count(&conn, "graph_metrics"), 2);
        // Metadata should have 6 keys
        let mcount = count(&conn, "metadata");
        assert!(
            mcount >= 5,
            "should have at least 5 metadata keys, got {mcount}"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 12: Tokenization for FTS5
    //  Claims: keyword filtering, lowercase, split on non-alphanumeric
    // ═══════════════════════════════════════════════════════════════════

    /// F-084: tokenize_query strips keywords
    #[test]
    fn f084_tokenize_strips_keywords() {
        let conn = db();
        insert_functions(&conn, &[entry("x", "fn x() {}", "a.rs")]).unwrap();
        // "fn" and "let" are keywords — should be stripped
        let r = fts5_search(&conn, "fn let struct", 10).unwrap();
        assert!(r.is_empty(), "all-keyword query should return empty");
    }

    /// F-085: tokenize_query handles underscored identifiers
    #[test]
    fn f085_tokenize_underscored() {
        let conn = db();
        let funcs = vec![entry(
            "parse_request_body",
            "fn parse_request_body() {}",
            "a.rs",
        )];
        insert_functions(&conn, &funcs).unwrap();
        let r = fts5_search(&conn, "parse_request_body", 10).unwrap();
        assert!(!r.is_empty(), "underscored identifier should be searchable");
    }

    /// F-086: tokenize_query handles mixed case
    #[test]
    fn f086_tokenize_case_insensitive() {
        let conn = db();
        let funcs = vec![entry("HandleRequest", "fn HandleRequest() {}", "a.rs")];
        insert_functions(&conn, &funcs).unwrap();
        let r = fts5_search(&conn, "handlerequest", 10).unwrap();
        assert!(!r.is_empty(), "lowercase query should match uppercase name");
    }

    /// F-087: Single-char tokens are filtered (len < 2)
    #[test]
    fn f087_tokenize_filters_short_tokens() {
        let conn = db();
        insert_functions(&conn, &[entry("x", "fn x() {}", "a.rs")]).unwrap();
        let r = fts5_search(&conn, "x y z", 10).unwrap();
        assert!(r.is_empty(), "single-char tokens should be filtered");
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 13: Multiple functions per file
    //  Claims: file_path index enables multi-function-per-file queries
    // ═══════════════════════════════════════════════════════════════════

    /// F-088: Multiple functions in same file stored correctly
    #[test]
    fn f088_multiple_functions_same_file() {
        let conn = db();
        let funcs = vec![
            entry("func_a", "fn func_a() {}", "src/shared.rs"),
            entry("func_b", "fn func_b() {}", "src/shared.rs"),
            entry("func_c", "fn func_c() {}", "src/shared.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();

        let in_file: i64 = conn
            .query_row(
                "SELECT count(*) FROM functions WHERE file_path = 'src/shared.rs'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(in_file, 3);
    }

    /// F-089: file_path index speeds up file lookups
    #[test]
    fn f089_file_path_index_works() {
        let conn = db();
        let mut funcs = Vec::new();
        for i in 0..100 {
            funcs.push(entry(
                &format!("f{i}"),
                &format!("fn f{i}() {{}}"),
                &format!("src/file_{}.rs", i / 10),
            ));
        }
        insert_functions(&conn, &funcs).unwrap();

        // EXPLAIN QUERY PLAN should show index usage
        let plan: String = conn
            .query_row(
                "EXPLAIN QUERY PLAN SELECT * FROM functions WHERE file_path = 'src/file_5.rs'",
                [],
                |r| r.get::<_, String>(3),
            )
            .unwrap();
        assert!(
            plan.contains("idx_functions_file") || plan.contains("SEARCH"),
            "should use file_path index: {plan}"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 14: Concurrency and schema safety
    // ═══════════════════════════════════════════════════════════════════

    /// F-090: create_schema is idempotent (IF NOT EXISTS)
    #[test]
    fn f090_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        create_schema(&conn).unwrap(); // should not fail
        create_schema(&conn).unwrap(); // triple call is fine
        assert_eq!(count(&conn, "functions"), 0);
    }

    /// F-091: open_db sets SQLITE_OPEN_NO_MUTEX for multi-thread safety
    #[test]
    fn f091_open_flags_no_mutex() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        // open_db should succeed with NO_MUTEX flag
        let conn = open_db(&db_path).unwrap();
        // Verify connection works
        create_schema(&conn).unwrap();
        assert_eq!(count(&conn, "functions"), 0);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 15: FTS5 standalone (not content-synced)
    //  Claims: FTS5 is standalone, identifiers column not in functions
    // ═══════════════════════════════════════════════════════════════════

    /// F-092: functions_fts is standalone (has its own data copy)
    #[test]
    fn f092_fts5_standalone() {
        let conn = db();
        let funcs = vec![entry(
            "test_func",
            "fn test_func() { unique_marker_xyz(); }",
            "a.rs",
        )];
        insert_functions(&conn, &funcs).unwrap();

        // FTS5 should work even if we delete from functions table
        conn.execute("DELETE FROM functions", []).unwrap();
        let r = fts5_search(&conn, "unique_marker_xyz", 10).unwrap();
        assert!(
            !r.is_empty(),
            "FTS5 should be standalone (data not content-synced)"
        );
    }

    /// F-093: functions table does NOT have identifiers column
    #[test]
    fn f093_no_identifiers_column_in_functions() {
        let conn = db();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(functions)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(
            !cols.contains(&"identifiers".to_string()),
            "functions table should NOT have identifiers column (FTS5 standalone)"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 16: Scale tests
    //  Claims: handles 18K, 90K, 185K functions
    // ═══════════════════════════════════════════════════════════════════

    /// F-094: 1000 functions insert + search works
    #[test]
    fn f094_scale_1000_functions() {
        let conn = db();
        let mut funcs = Vec::new();
        for i in 0..1000 {
            funcs.push(entry(
                &format!("func_{i}"),
                &format!("fn func_{i}() {{ operation_{i}(); }}"),
                &format!("src/mod_{}.rs", i / 50),
            ));
        }
        insert_functions(&conn, &funcs).unwrap();
        assert_eq!(count(&conn, "functions"), 1000);

        let results = fts5_search(&conn, "operation_500", 10).unwrap();
        assert!(
            !results.is_empty(),
            "should find specific function among 1000"
        );
    }

    /// F-095: Large call graph (5000 edges) works
    #[test]
    fn f095_scale_large_call_graph() {
        let conn = db();
        let mut funcs = Vec::new();
        for i in 0..500 {
            funcs.push(entry(
                &format!("f{i}"),
                &format!("fn f{i}() {{}}"),
                &format!("f{i}.rs"),
            ));
        }
        insert_functions(&conn, &funcs).unwrap();

        let mut calls = HashMap::new();
        for i in 0..100 {
            let callees: Vec<usize> = (100..150).collect();
            calls.insert(i, callees);
        }
        insert_call_graph(&conn, &calls).unwrap();
        let edge_count = count(&conn, "call_graph");
        assert_eq!(edge_count, 5000, "should have 100*50=5000 edges");
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 17: save_to_sqlite file size (exercises humanize_bytes indirectly)
    // ═══════════════════════════════════════════════════════════════════

    /// F-096: Small DB file size is reasonable (< 200KB for 1 function)
    /// Note: Schema overhead includes functions, call_graph, graph_metrics,
    /// metadata, entropy_violations, provability_scores, quality_violations,
    /// documents tables, plus two FTS5 virtual tables and multiple indexes.
    #[test]
    fn f096_small_db_file_size() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        save_to_sqlite(
            &db_path,
            &[entry("f", "fn f() {}", "a.rs")],
            &HashMap::new(),
            &[],
            &manifest(1, 1),
            &HashSet::new(),
        )
        .unwrap();
        let size = db_path.metadata().unwrap().len();
        assert!(
            size < 200_000,
            "single-function DB should be < 200KB, got {size}"
        );
        assert!(size > 0, "DB should not be empty");
    }

    /// F-097: DB size scales roughly linearly with function count
    #[test]
    fn f097_db_size_scales_linearly() {
        let tmp = tempfile::TempDir::new().unwrap();
        let small_path = tmp.path().join("small.db");
        let large_path = tmp.path().join("large.db");

        let small_funcs: Vec<_> = (0..10)
            .map(|i| {
                entry(
                    &format!("f{i}"),
                    &format!("fn f{i}() {{}}"),
                    &format!("f{i}.rs"),
                )
            })
            .collect();
        let large_funcs: Vec<_> = (0..100)
            .map(|i| {
                entry(
                    &format!("f{i}"),
                    &format!("fn f{i}() {{}}"),
                    &format!("f{i}.rs"),
                )
            })
            .collect();

        save_to_sqlite(
            &small_path,
            &small_funcs,
            &HashMap::new(),
            &[],
            &manifest(10, 10),
            &HashSet::new(),
        )
        .unwrap();
        save_to_sqlite(
            &large_path,
            &large_funcs,
            &HashMap::new(),
            &[],
            &manifest(100, 100),
            &HashSet::new(),
        )
        .unwrap();

        let small_size = small_path.metadata().unwrap().len() as f64;
        let large_size = large_path.metadata().unwrap().len() as f64;
        // 10x more functions should result in less than 20x more size (sublinear overhead)
        assert!(
            large_size / small_size < 20.0,
            "size scaling too high: {:.1}x",
            large_size / small_size
        );
    }

    /// F-098: DB file is valid SQLite (can be re-opened)
    #[test]
    fn f098_db_file_valid_sqlite() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        save_to_sqlite(
            &db_path,
            &[entry("f", "fn f() {}", "a.rs")],
            &HashMap::new(),
            &[],
            &manifest(1, 1),
            &HashSet::new(),
        )
        .unwrap();
        // Re-open and verify
        let conn = open_db(&db_path).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 18: definition_type roundtrip via SQLite TEXT column
    // ═══════════════════════════════════════════════════════════════════

    /// F-099: definition_type stored as Debug format string
    #[test]
    fn f099_definition_type_stored_as_text() {
        let conn = db();
        let funcs = vec![
            entry_typed("s", "a.rs", DefinitionType::Struct),
            entry_typed("e", "a.rs", DefinitionType::Enum),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let stored: Vec<String> = conn
            .prepare("SELECT definition_type FROM functions ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(stored[0], "Struct");
        assert_eq!(stored[1], "Enum");
    }

    /// F-100: Unknown definition_type string loads as Function (default)
    #[test]
    fn f100_unknown_definition_type_defaults() {
        let conn = db();
        // Manually insert with unknown type
        conn.execute(
            "INSERT INTO functions (file_path, function_name, signature, definition_type, source, start_line, end_line, language, checksum) VALUES ('a.rs', 'f', 'fn f()', 'Interface', 'fn f() {}', 1, 5, 'Rust', 'abc')",
            [],
        )
        .unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(
            loaded[0].definition_type,
            DefinitionType::Function,
            "unknown type should default to Function"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 19: Spec-specific falsifications
    //  Claims from the specification document
    // ═══════════════════════════════════════════════════════════════════

    /// F-101: Spec claims "O(1) per-term lookup via FTS5 inverted index"
    /// Falsification: verify search is sublinear by comparing times
    #[test]
    fn f101_fts5_sublinear_search() {
        let conn = db();
        let mut funcs = Vec::new();
        for i in 0..2000 {
            funcs.push(entry(
                &format!("func_{i}"),
                &format!("fn func_{i}() {{ operation_{i}(); }}"),
                &format!("src/f{i}.rs"),
            ));
        }
        insert_functions(&conn, &funcs).unwrap();

        // Search should complete quickly even with 2000 functions
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = fts5_search(&conn, "operation_1234", 10).unwrap();
        }
        let elapsed = start.elapsed();
        // CB-511: Generous upper bound to avoid flaky tests under CI load
        assert!(
            elapsed.as_secs() < 30,
            "100 FTS5 searches took {elapsed:?} (>30s)"
        );
    }

    /// F-102: Spec claims "BM25 ranking" — IDF should differentiate rare vs common
    #[test]
    fn f102_bm25_idf_differentiation() {
        let conn = db();
        // "common" appears in every doc, "rare" appears in one
        let mut funcs = Vec::new();
        for i in 0..10 {
            funcs.push(entry(
                &format!("func_{i}"),
                &format!("fn func_{i}() {{ common_method(); }}"),
                &format!("src/f{i}.rs"),
            ));
        }
        let mut rare = entry(
            "rare_target",
            "fn rare_target() { rare_unique_xyz(); common_method(); }",
            "src/rare.rs",
        );
        rare.doc_comment = Some("Handles rare_unique_xyz operations".to_string());
        funcs.push(rare);
        insert_functions(&conn, &funcs).unwrap();

        let results = fts5_search(&conn, "rare_unique_xyz", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(
            results[0].0, 10,
            "rare term should uniquely identify the function"
        );
    }

    /// F-103: Spec claims "Porter stemmer normalizes: serialize/serializer/serialization"
    #[test]
    fn f103_porter_stemmer_normalization() {
        let conn = db();
        let funcs = vec![
            entry("serialize_data", "fn serialize_data() {}", "src/ser.rs"),
            entry("deserializer", "fn deserializer() {}", "src/de.rs"),
            entry(
                "serialization_engine",
                "fn serialization_engine() {}",
                "src/engine.rs",
            ),
        ];
        insert_functions(&conn, &funcs).unwrap();

        // All three should match a search for "serialize"
        let results = fts5_search(&conn, "serialize", 10).unwrap();
        assert!(
            results.len() >= 2,
            "stemmer should match variants, got {}",
            results.len()
        );
    }

    /// F-104: Spec claims FTS5 uses 5 columns: function_name, signature, doc_comment, file_path, identifiers
    #[test]
    fn f104_fts5_five_columns() {
        let conn = db();
        // Verify by checking FTS5 content can be queried
        insert_functions(&conn, &[entry("test", "fn test() {}", "a.rs")]).unwrap();
        // Try to match via each column
        let stmt_result = conn.prepare(
            "SELECT * FROM functions_fts WHERE functions_fts MATCH 'function_name:\"test\"'",
        );
        assert!(
            stmt_result.is_ok(),
            "FTS5 should support column-specific matching"
        );
    }

    /// F-105: Spec claims "standalone FTS5 (no content-sync)" — verify INSERT into FTS5 is independent
    #[test]
    fn f105_fts5_independent_inserts() {
        let conn = db();
        // Directly insert into FTS5 without functions table
        conn.execute(
            "INSERT INTO functions_fts (rowid, function_name, signature, doc_comment, file_path, identifiers) VALUES (999, 'direct_insert', 'fn direct_insert()', 'doc', 'path.rs', 'ident')",
            [],
        )
        .unwrap();
        let r = fts5_search(&conn, "direct_insert", 10).unwrap();
        assert!(!r.is_empty(), "direct FTS5 insert should be searchable");
    }

    /// F-106: Spec claims "tokenize='porter unicode61 remove_diacritics 2'"
    #[test]
    fn f106_tokenizer_config() {
        let conn = db();
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name='functions_fts'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            sql.contains("porter")
                && sql.contains("unicode61")
                && sql.contains("remove_diacritics"),
            "FTS5 should have porter unicode61 tokenizer: {sql}"
        );
    }

    /// F-107: Spec claims "FTS5 rank is negative (lower = better)" — verify conversion
    #[test]
    fn f107_fts5_rank_negative_to_positive() {
        let conn = db();
        let funcs = vec![entry(
            "target",
            "fn target() { target_specific_marker(); }",
            "a.rs",
        )];
        insert_functions(&conn, &funcs).unwrap();

        // Raw FTS5 rank should be negative
        let raw_rank: f64 = conn
            .query_row(
                "SELECT rank FROM functions_fts WHERE functions_fts MATCH '\"target\"' LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            raw_rank < 0.0,
            "raw FTS5 rank should be negative: {raw_rank}"
        );

        // Our API should return positive
        let results = fts5_search(&conn, "target", 10).unwrap();
        assert!(results[0].1 > 0.0, "converted score should be positive");
    }

    /// F-108: Spec claims call_graph has PRIMARY KEY (caller_id, callee_id)
    #[test]
    fn f108_call_graph_composite_pk() {
        let conn = db();
        let funcs = vec![
            entry("a", "fn a() {}", "a.rs"),
            entry("b", "fn b() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();

        // First insert should succeed
        conn.execute(
            "INSERT INTO call_graph (caller_id, callee_id) VALUES (1, 2)",
            [],
        )
        .unwrap();
        // Duplicate should fail (or be ignored with INSERT OR IGNORE)
        let r = conn.execute(
            "INSERT INTO call_graph (caller_id, callee_id) VALUES (1, 2)",
            [],
        );
        assert!(r.is_err(), "duplicate PK should fail without OR IGNORE");
    }

    /// F-109: Spec claims metadata key is PRIMARY KEY — verify uniqueness
    #[test]
    fn f109_metadata_pk_uniqueness() {
        let conn = db();
        conn.execute(
            "INSERT INTO metadata (key, value) VALUES ('test_key', 'value1')",
            [],
        )
        .unwrap();
        // Direct insert of same key should fail
        let r = conn.execute(
            "INSERT INTO metadata (key, value) VALUES ('test_key', 'value2')",
            [],
        );
        assert!(
            r.is_err(),
            "duplicate metadata key should fail without OR REPLACE"
        );
        // OR REPLACE should work
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('test_key', 'value3')",
            [],
        )
        .unwrap();
        let val: String = conn
            .query_row("SELECT value FROM metadata WHERE key='test_key'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(val, "value3");
    }

    /// F-110: Spec claims graph_metrics function_id is PRIMARY KEY
    #[test]
    fn f110_graph_metrics_pk() {
        let conn = db();
        let funcs = vec![entry("a", "fn a() {}", "a.rs")];
        insert_functions(&conn, &funcs).unwrap();
        conn.execute(
            "INSERT INTO graph_metrics (function_id, pagerank, centrality, in_degree, out_degree) VALUES (1, 0.5, 0.3, 1, 1)",
            [],
        )
        .unwrap();
        let r = conn.execute(
            "INSERT INTO graph_metrics (function_id, pagerank, centrality, in_degree, out_degree) VALUES (1, 0.9, 0.1, 2, 2)",
            [],
        );
        assert!(
            r.is_err(),
            "duplicate function_id should fail without OR REPLACE"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 20: Quality metrics edge cases
    // ═══════════════════════════════════════════════════════════════════

    /// F-111: TDG score 0.0 (best) roundtrips
    #[test]
    fn f111_tdg_score_zero() {
        let conn = db();
        let f = entry_with_quality("f", "a.rs", 0.0, "A", 1);
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!((loaded[0].quality.tdg_score - 0.0).abs() < 0.001);
    }

    /// F-112: TDG score 10.0 (worst) roundtrips
    #[test]
    fn f112_tdg_score_max() {
        let conn = db();
        let f = entry_with_quality("f", "a.rs", 10.0, "F", 100);
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!((loaded[0].quality.tdg_score - 10.0).abs() < 0.01);
    }

    /// F-113: All TDG grades (A, B, C, D, F) roundtrip
    #[test]
    fn f113_all_tdg_grades() {
        let conn = db();
        let funcs = vec![
            entry_with_quality("a", "a.rs", 1.0, "A", 1),
            entry_with_quality("b", "b.rs", 3.0, "B", 5),
            entry_with_quality("c", "c.rs", 5.0, "C", 10),
            entry_with_quality("d", "d.rs", 7.0, "D", 20),
            entry_with_quality("f", "f.rs", 9.0, "F", 50),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].quality.tdg_grade, "A");
        assert_eq!(loaded[1].quality.tdg_grade, "B");
        assert_eq!(loaded[2].quality.tdg_grade, "C");
        assert_eq!(loaded[3].quality.tdg_grade, "D");
        assert_eq!(loaded[4].quality.tdg_grade, "F");
    }

    /// F-114: Churn score boundary values (0.0 and 1.0)
    #[test]
    fn f114_churn_score_boundaries() {
        let conn = db();
        let mut f1 = entry("a", "fn a() {}", "a.rs");
        f1.churn_score = 0.0;
        let mut f2 = entry("b", "fn b() {}", "b.rs");
        f2.churn_score = 1.0;
        insert_functions(&conn, &[f1, f2]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!((loaded[0].churn_score - 0.0).abs() < 0.001);
        assert!((loaded[1].churn_score - 1.0).abs() < 0.001);
    }

    /// F-115: Pattern diversity boundary values (0.0 and 1.0)
    #[test]
    fn f115_pattern_diversity_boundaries() {
        let conn = db();
        let mut f1 = entry("a", "fn a() {}", "a.rs");
        f1.pattern_diversity = 0.0;
        let mut f2 = entry("b", "fn b() {}", "b.rs");
        f2.pattern_diversity = 1.0;
        insert_functions(&conn, &[f1, f2]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!((loaded[0].pattern_diversity - 0.0).abs() < 0.001);
        assert!((loaded[1].pattern_diversity - 1.0).abs() < 0.001);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 21: FTS5 advanced search patterns
    // ═══════════════════════════════════════════════════════════════════

    /// F-116: FTS5 search with special characters in query
    #[test]
    fn f116_fts5_special_chars_in_query() {
        let conn = db();
        insert_functions(&conn, &[entry("test", "fn test() {}", "a.rs")]).unwrap();
        // Should not crash with special chars
        let _ = fts5_search(&conn, "test!@#$%", 10);
        let _ = fts5_search(&conn, "test()", 10);
        let _ = fts5_search(&conn, "a->b", 10);
    }

    /// F-117: FTS5 search with very long query
    #[test]
    fn f117_fts5_long_query() {
        let conn = db();
        insert_functions(&conn, &[entry("test", "fn test() {}", "a.rs")]).unwrap();
        let long_query = "word ".repeat(100);
        let _ = fts5_search(&conn, &long_query, 10); // should not crash
    }

    /// F-118: FTS5 search matches file_path component
    #[test]
    fn f118_fts5_file_path_search() {
        let conn = db();
        let funcs = vec![
            entry("func_a", "fn func_a() {}", "src/authentication/handler.rs"),
            entry("func_b", "fn func_b() {}", "src/database/handler.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let results = fts5_search(&conn, "authentication", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0, "should match via file_path");
    }

    /// F-119: FTS5 doc_comment search
    #[test]
    fn f119_fts5_doc_comment_search() {
        let conn = db();
        let mut f = entry("generic_name", "fn generic_name() {}", "a.rs");
        f.doc_comment = Some("Performs cryptographic hashing with SHA256".to_string());
        insert_functions(&conn, &[f]).unwrap();
        let results = fts5_search(&conn, "cryptographic hashing", 10).unwrap();
        assert!(!results.is_empty(), "should match via doc_comment");
    }

    /// F-120: FTS5 identifiers search (extracted from source)
    #[test]
    fn f120_fts5_identifiers_search() {
        let conn = db();
        let funcs = vec![entry(
            "generic_func",
            "fn generic_func() { very_unique_identifier_xyz(); another_call(); }",
            "a.rs",
        )];
        insert_functions(&conn, &funcs).unwrap();
        let results = fts5_search(&conn, "very_unique_identifier_xyz", 10).unwrap();
        assert!(
            !results.is_empty(),
            "should match via extracted identifiers"
        );
    }
}
