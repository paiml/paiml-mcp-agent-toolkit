#![cfg_attr(coverage_nightly, coverage(off))]
//! 100+ point falsification test suite for index-v2-sqlite-fts5.md spec.
//!
//! Each test attempts to falsify a specific claim from the specification.
//! Claims are tagged with their spec section for traceability.
//!
//! Spec: docs/specifications/index-v2-sqlite-fts5.md v2.2.0

#[cfg(test)]
mod tests {
    use crate::services::agent_context::function_index::sqlite_backend::*;
    use crate::services::agent_context::function_index::types::*;
    use rusqlite::Connection;
    use std::collections::HashMap;

    // ═══════════════════════════════════════════════════════════════════
    //  Test data factories
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
            tdg_scale: crate::services::agent_context::TDG_SCALE.to_string(),
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
            run_counter: 0,
        }
    }

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        conn
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 1: Schema invariants (spec: "Schema" section)
    //  Claims: 5 tables exist, columns have correct types/defaults
    // ═══════════════════════════════════════════════════════════════════

    /// F-001: "functions" table exists with all 22 columns
    #[test]
    fn f001_functions_table_has_all_columns() {
        let conn = db();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(functions)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        let expected = [
            "id",
            "file_path",
            "function_name",
            "signature",
            "definition_type",
            "doc_comment",
            "source",
            "start_line",
            "end_line",
            "language",
            "checksum",
            "tdg_score",
            "tdg_grade",
            "complexity",
            "cognitive_complexity",
            "big_o",
            "satd_count",
            "loc",
            "commit_count",
            "churn_score",
            "clone_count",
            "pattern_diversity",
            "fault_annotations",
            "contract_level",
            "contract_equation",
        ];
        for col in &expected {
            assert!(cols.contains(&col.to_string()), "missing column: {col}");
        }
        assert_eq!(cols.len(), expected.len(), "unexpected extra columns");
    }

    /// F-002: "call_graph" table exists with caller_id and callee_id
    #[test]
    fn f002_call_graph_table_schema() {
        let conn = db();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(call_graph)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(cols, vec!["caller_id", "callee_id"]);
    }

    /// F-003: "graph_metrics" table has pagerank, centrality, in/out_degree
    #[test]
    fn f003_graph_metrics_table_schema() {
        let conn = db();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(graph_metrics)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(
            cols,
            vec![
                "function_id",
                "pagerank",
                "centrality",
                "in_degree",
                "out_degree"
            ]
        );
    }

    /// F-004: "metadata" table has key/value schema
    #[test]
    fn f004_metadata_table_schema() {
        let conn = db();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(metadata)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(cols, vec!["key", "value"]);
    }

    /// F-005: FTS5 virtual table "functions_fts" exists
    #[test]
    fn f005_fts5_table_exists() {
        let conn = db();
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='functions_fts'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    /// F-006: Spec says "call_graph WITHOUT ROWID" — verify
    #[test]
    fn f006_call_graph_without_rowid() {
        let conn = db();
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name='call_graph'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            sql.to_uppercase().contains("WITHOUT ROWID"),
            "call_graph should be WITHOUT ROWID, got: {sql}"
        );
    }

    /// F-007: Spec says "id INTEGER PRIMARY KEY" on functions (autoincrement)
    #[test]
    fn f007_functions_id_is_primary_key() {
        let conn = db();
        // pk column in PRAGMA table_info is 1 for primary key
        let pk: i64 = conn
            .query_row(
                "SELECT pk FROM pragma_table_info('functions') WHERE name='id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(pk, 1, "id should be primary key");
    }

    /// F-008: All 5 performance indexes exist
    #[test]
    fn f008_performance_indexes_exist() {
        let conn = db();
        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        let expected = [
            "idx_functions_file",
            "idx_functions_name",
            "idx_functions_lang",
            "idx_functions_grade",
            "idx_call_graph_callee",
        ];
        for idx in &expected {
            assert!(indexes.contains(&idx.to_string()), "missing index: {idx}");
        }
    }

    /// F-009: Spec says defaults: tdg_score=0.0, tdg_grade='A', complexity=1
    #[test]
    fn f009_column_defaults_correct() {
        let conn = db();
        // Insert minimal row to test defaults
        conn.execute(
            "INSERT INTO functions (file_path, function_name, signature, source, start_line, end_line, language, checksum) VALUES ('a.rs', 'f', 'fn f()', 'fn f() {}', 1, 5, 'Rust', 'abc')",
            [],
        )
        .unwrap();
        let (score, grade, cx, cog, big_o, satd, loc): (f64, String, i64, i64, String, i64, i64) =
            conn.query_row(
                "SELECT tdg_score, tdg_grade, complexity, cognitive_complexity, big_o, satd_count, loc FROM functions WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
            )
            .unwrap();
        assert_eq!(score, 0.0);
        assert_eq!(grade, "A");
        assert_eq!(cx, 1);
        assert_eq!(cog, 1);
        assert_eq!(big_o, "O(1)");
        assert_eq!(satd, 0);
        assert_eq!(loc, 0);
    }

    /// F-010: Spec says definition_type defaults to 'Function'
    #[test]
    fn f010_definition_type_default() {
        let conn = db();
        conn.execute(
            "INSERT INTO functions (file_path, function_name, signature, source, start_line, end_line, language, checksum) VALUES ('a.rs', 'f', 'fn f()', 'fn f() {}', 1, 5, 'Rust', 'abc')",
            [],
        )
        .unwrap();
        let dt: String = conn
            .query_row(
                "SELECT definition_type FROM functions WHERE id=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(dt, "Function");
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 2: FTS5 BM25 search (spec: "FTS5 BM25 Query" section)
    //  Claims: BM25 ranking, stemming, O(1) per-term lookup
    // ═══════════════════════════════════════════════════════════════════

    /// F-011: FTS5 search returns results sorted by relevance (BM25)
    #[test]
    fn f011_fts5_bm25_ranking() {
        let conn = db();
        let funcs = vec![
            entry(
                "parse_json",
                "fn parse_json() { serde_json::from_str(); }",
                "src/json.rs",
            ),
            entry(
                "unrelated",
                "fn unrelated() { println!(); }",
                "src/other.rs",
            ),
            entry(
                "parse_json_stream",
                "fn parse_json_stream() { json::parse(); json_value(); }",
                "src/stream.rs",
            ),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let results = fts5_search(&conn, "parse json", 10).unwrap();
        assert!(results.len() >= 2, "should find at least 2 matches");
        // Top result should be a parse/json function
        let top = &funcs[results[0].0].function_name;
        assert!(top.contains("parse") || top.contains("json"), "top={top}");
    }

    /// F-012: FTS5 uses Porter stemming — "serialize" matches "serialization"
    #[test]
    fn f012_fts5_porter_stemming() {
        let conn = db();
        let funcs = vec![entry(
            "do_serialization",
            "fn do_serialization() { serialize_data(); }",
            "src/ser.rs",
        )];
        insert_functions(&conn, &funcs).unwrap();
        let results = fts5_search(&conn, "serialize", 10).unwrap();
        assert!(
            !results.is_empty(),
            "stemming should match 'serialize' -> 'serialization'"
        );
    }

    /// F-013: FTS5 handles empty query gracefully
    #[test]
    fn f013_fts5_empty_query() {
        let conn = db();
        insert_functions(&conn, &[entry("a", "fn a() {}", "a.rs")]).unwrap();
        let r = fts5_search(&conn, "", 10).unwrap();
        assert!(r.is_empty());
    }

    /// F-014: FTS5 returns empty for non-existent terms
    #[test]
    fn f014_fts5_no_match() {
        let conn = db();
        insert_functions(&conn, &[entry("a", "fn a() {}", "a.rs")]).unwrap();
        let r = fts5_search(&conn, "zzzznonexistent", 10).unwrap();
        assert!(r.is_empty());
    }

    /// F-015: FTS5 scores are normalized to [0, 1]
    #[test]
    fn f015_fts5_scores_normalized() {
        let conn = db();
        let funcs = vec![
            entry(
                "handle_error",
                "fn handle_error() { log_error(); }",
                "src/err.rs",
            ),
            entry("log_error", "fn log_error() { stderr(); }", "src/log.rs"),
            entry("other", "fn other() { println!(); }", "src/o.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let results = fts5_search(&conn, "error", 10).unwrap();
        for (_, score) in &results {
            assert!(*score >= 0.0, "score should be >= 0: {score}");
            assert!(*score <= 1.0, "score should be <= 1: {score}");
        }
        // Max score should be exactly 1.0
        if !results.is_empty() {
            let max = results.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
            assert!((max - 1.0).abs() < 0.001, "max score should be ~1.0: {max}");
        }
    }

    /// F-016: FTS5 respects LIMIT parameter
    #[test]
    fn f016_fts5_limit() {
        let conn = db();
        let mut funcs = Vec::new();
        for i in 0..20 {
            funcs.push(entry(
                &format!("handler_{i}"),
                &format!("fn handler_{i}() {{ process(); }}"),
                &format!("src/h{i}.rs"),
            ));
        }
        insert_functions(&conn, &funcs).unwrap();
        let r5 = fts5_search(&conn, "handler", 5).unwrap();
        assert!(r5.len() <= 5, "limit 5 should return ≤5, got {}", r5.len());
        let r2 = fts5_search(&conn, "handler", 2).unwrap();
        assert!(r2.len() <= 2, "limit 2 should return ≤2, got {}", r2.len());
    }

    /// F-017: FTS5 ranks name matches higher than source-only matches
    #[test]
    fn f017_fts5_name_match_priority() {
        let conn = db();
        let funcs = vec![
            entry(
                "validate_input",
                "fn validate_input() { check(); }",
                "src/val.rs",
            ),
            entry(
                "unrelated",
                "fn unrelated() { validate_something(); }",
                "src/o.rs",
            ),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let results = fts5_search(&conn, "validate", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(
            funcs[results[0].0].function_name, "validate_input",
            "name match should rank first"
        );
    }

    /// F-018: FTS5 searches across all 5 indexed columns
    #[test]
    fn f018_fts5_searches_all_columns() {
        let conn = db();
        // Match via doc_comment
        let mut f1 = entry("alpha", "fn alpha() {}", "src/a.rs");
        f1.doc_comment = Some("Handles authentication flow".to_string());
        // Match via file_path
        let f2 = entry("beta", "fn beta() {}", "src/authentication/mod.rs");
        // Match via identifiers in source
        let f3 = entry(
            "gamma",
            "fn gamma() { authentication_check(); }",
            "src/g.rs",
        );
        insert_functions(&conn, &[f1, f2, f3]).unwrap();
        let results = fts5_search(&conn, "authentication", 10).unwrap();
        assert!(
            results.len() >= 2,
            "should match across columns, got {}",
            results.len()
        );
    }

    /// F-019: FTS5 tokenizer strips diacritics (remove_diacritics 2)
    #[test]
    fn f019_fts5_diacritics() {
        let conn = db();
        let funcs = vec![entry(
            "café_handler",
            "fn café_handler() { résumé(); }",
            "src/c.rs",
        )];
        insert_functions(&conn, &funcs).unwrap();
        let results = fts5_search(&conn, "cafe", 10).unwrap();
        // With remove_diacritics=2, "café" should match "cafe"
        assert!(
            !results.is_empty(),
            "diacritic stripping should match café→cafe"
        );
    }

    /// F-020: Query-only keywords are filtered by tokenize_query_for_fts5
    #[test]
    fn f020_keyword_filtering() {
        let conn = db();
        insert_functions(&conn, &[entry("abc", "fn abc() {}", "a.rs")]).unwrap();
        // Pure keyword query should produce empty FTS5 query
        let r = fts5_search(&conn, "fn let if", 10).unwrap();
        assert!(r.is_empty(), "keyword-only query should return empty");
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 3: Data integrity — roundtrip fidelity
    //  Claims: save → load preserves all fields exactly
    // ═══════════════════════════════════════════════════════════════════

    /// F-021: Function file_path survives roundtrip
    #[test]
    fn f021_roundtrip_file_path() {
        let conn = db();
        let funcs = vec![entry("f", "fn f() {}", "src/deep/nested/mod.rs")];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].file_path, "src/deep/nested/mod.rs");
    }

    /// F-022: Function name survives roundtrip
    #[test]
    fn f022_roundtrip_function_name() {
        let conn = db();
        let funcs = vec![entry(
            "my_complex_func_name",
            "fn my_complex_func_name() {}",
            "a.rs",
        )];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].function_name, "my_complex_func_name");
    }

    /// F-023: Signature survives roundtrip
    #[test]
    fn f023_roundtrip_signature() {
        let conn = db();
        let mut f = entry("f", "fn f(x: i32, y: &str) -> Result<(), Error>", "a.rs");
        f.signature = "fn f(x: i32, y: &str) -> Result<(), Error>".to_string();
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(
            loaded[0].signature,
            "fn f(x: i32, y: &str) -> Result<(), Error>"
        );
    }

    /// F-024: definition_type roundtrip — all variants
    #[test]
    fn f024_roundtrip_definition_type() {
        let conn = db();
        let funcs = vec![
            entry_typed("f", "a.rs", DefinitionType::Function),
            entry_typed("S", "a.rs", DefinitionType::Struct),
            entry_typed("E", "a.rs", DefinitionType::Enum),
            entry_typed("T", "a.rs", DefinitionType::Trait),
            entry_typed("A", "a.rs", DefinitionType::TypeAlias),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].definition_type, DefinitionType::Function);
        assert_eq!(loaded[1].definition_type, DefinitionType::Struct);
        assert_eq!(loaded[2].definition_type, DefinitionType::Enum);
        assert_eq!(loaded[3].definition_type, DefinitionType::Trait);
        assert_eq!(loaded[4].definition_type, DefinitionType::TypeAlias);
    }

    /// F-025: doc_comment None survives roundtrip
    #[test]
    fn f025_roundtrip_doc_comment_none() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.doc_comment = None;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!(
            loaded[0].doc_comment.is_none(),
            "None doc_comment should survive"
        );
    }

    /// F-026: doc_comment Some survives roundtrip
    #[test]
    fn f026_roundtrip_doc_comment_some() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.doc_comment = Some("A detailed doc comment with `code` and *markdown*".to_string());
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(
            loaded[0].doc_comment.as_deref(),
            Some("A detailed doc comment with `code` and *markdown*")
        );
    }

    /// F-027: Source code with special characters survives roundtrip
    #[test]
    fn f027_roundtrip_source_special_chars() {
        let conn = db();
        let source = r#"fn f() { let s = "hello \"world\""; let n = '\n'; }"#;
        let funcs = vec![entry("f", source, "a.rs")];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].source, source);
    }

    /// F-028: start_line and end_line survive roundtrip
    #[test]
    fn f028_roundtrip_line_numbers() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.start_line = 42;
        f.end_line = 99;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].start_line, 42);
        assert_eq!(loaded[0].end_line, 99);
    }

    /// F-029: Language survives roundtrip
    #[test]
    fn f029_roundtrip_language() {
        let conn = db();
        let mut f = entry("f", "function f() {}", "a.js");
        f.language = "JavaScript".to_string();
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].language, "JavaScript");
    }

    /// F-030: Quality metrics survive roundtrip (tdg_score, grade, complexity)
    #[test]
    fn f030_roundtrip_quality_metrics() {
        let conn = db();
        let f = entry_with_quality("f", "a.rs", 7.5, "D", 25);
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!((loaded[0].quality.tdg_score - 7.5).abs() < 0.01);
        assert_eq!(loaded[0].quality.tdg_grade, "D");
        assert_eq!(loaded[0].quality.complexity, 25);
    }

    /// F-031: cognitive_complexity survives roundtrip
    #[test]
    fn f031_roundtrip_cognitive_complexity() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.quality.cognitive_complexity = 18;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].quality.cognitive_complexity, 18);
    }

    /// F-032: big_o survives roundtrip
    #[test]
    fn f032_roundtrip_big_o() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.quality.big_o = "O(n log n)".to_string();
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].quality.big_o, "O(n log n)");
    }

    /// F-033: satd_count survives roundtrip
    #[test]
    fn f033_roundtrip_satd_count() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.quality.satd_count = 3;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].quality.satd_count, 3);
    }

    /// F-034: loc survives roundtrip
    #[test]
    fn f034_roundtrip_loc() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.quality.loc = 150;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].quality.loc, 150);
    }

    /// F-035: Checksum survives roundtrip
    #[test]
    fn f035_roundtrip_checksum() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.checksum = "sha256:abcdef0123456789".to_string();
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].checksum, "sha256:abcdef0123456789");
    }

    /// F-036: commit_count survives roundtrip
    #[test]
    fn f036_roundtrip_commit_count() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.commit_count = 42;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].commit_count, 42);
    }

    /// F-037: churn_score survives roundtrip
    #[test]
    fn f037_roundtrip_churn_score() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.churn_score = 0.85;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!((loaded[0].churn_score - 0.85).abs() < 0.01);
    }

    /// F-038: clone_count survives roundtrip
    #[test]
    fn f038_roundtrip_clone_count() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.clone_count = 7;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].clone_count, 7);
    }

    /// F-039: pattern_diversity survives roundtrip
    #[test]
    fn f039_roundtrip_pattern_diversity() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.pattern_diversity = 0.73;
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!((loaded[0].pattern_diversity - 0.73).abs() < 0.01);
    }

    /// F-040: fault_annotations survives roundtrip
    #[test]
    fn f040_roundtrip_fault_annotations() {
        let conn = db();
        let mut f = entry("f", "fn f() {}", "a.rs");
        f.fault_annotations = vec![
            "unwrap".to_string(),
            "panic".to_string(),
            "unsafe".to_string(),
        ];
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(
            loaded[0].fault_annotations,
            vec!["unwrap", "panic", "unsafe"]
        );
    }

    /// F-041: Empty fault_annotations survives roundtrip
    #[test]
    fn f041_roundtrip_empty_faults() {
        let conn = db();
        let f = entry("f", "fn f() {}", "a.rs");
        insert_functions(&conn, &[f]).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert!(loaded[0].fault_annotations.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 4: Call graph integrity
    //  Claims: caller→callee edges, bidirectional queries, dedup
    // ═══════════════════════════════════════════════════════════════════

    /// F-042: Call graph edges are stored correctly
    #[test]
    fn f042_call_graph_basic() {
        let conn = db();
        let funcs = vec![
            entry("caller", "fn caller() { callee(); }", "a.rs"),
            entry("callee", "fn callee() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let mut calls = HashMap::new();
        calls.insert(0, vec![1]);
        insert_call_graph(&conn, &calls).unwrap();

        let callees = query_callees(&conn, 0).unwrap();
        assert_eq!(callees, vec![1]);
        let callers = query_callers(&conn, 1).unwrap();
        assert_eq!(callers, vec![0]);
    }

    /// F-043: Call graph with multiple callees
    #[test]
    fn f043_call_graph_multiple_callees() {
        let conn = db();
        let funcs = vec![
            entry("main", "fn main() { a(); b(); c(); }", "a.rs"),
            entry("a", "fn a() {}", "a.rs"),
            entry("b", "fn b() {}", "a.rs"),
            entry("c", "fn c() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let mut calls = HashMap::new();
        calls.insert(0, vec![1, 2, 3]);
        insert_call_graph(&conn, &calls).unwrap();

        let callees = query_callees(&conn, 0).unwrap();
        assert_eq!(callees.len(), 3);
        assert!(callees.contains(&1));
        assert!(callees.contains(&2));
        assert!(callees.contains(&3));
    }

    /// F-044: Call graph with multiple callers
    #[test]
    fn f044_call_graph_multiple_callers() {
        let conn = db();
        let funcs = vec![
            entry("a", "fn a() { target(); }", "a.rs"),
            entry("b", "fn b() { target(); }", "a.rs"),
            entry("target", "fn target() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let mut calls = HashMap::new();
        calls.insert(0, vec![2]);
        calls.insert(1, vec![2]);
        insert_call_graph(&conn, &calls).unwrap();

        let callers = query_callers(&conn, 2).unwrap();
        assert_eq!(callers.len(), 2);
    }

    /// F-045: Call graph deduplication (PRIMARY KEY prevents duplicates)
    #[test]
    fn f045_call_graph_dedup() {
        let conn = db();
        let funcs = vec![
            entry("a", "fn a() {}", "a.rs"),
            entry("b", "fn b() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let mut calls = HashMap::new();
        calls.insert(0, vec![1, 1, 1]); // triple edge
        insert_call_graph(&conn, &calls).unwrap();

        let count: i64 = conn
            .query_row("SELECT count(*) FROM call_graph", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            count, 1,
            "duplicate edges should be ignored (INSERT OR IGNORE)"
        );
    }

    /// F-046: Call graph for non-existent function returns empty
    #[test]
    fn f046_call_graph_empty() {
        let conn = db();
        let funcs = vec![entry("a", "fn a() {}", "a.rs")];
        insert_functions(&conn, &funcs).unwrap();
        let callees = query_callees(&conn, 0).unwrap();
        assert!(callees.is_empty());
        let callers = query_callers(&conn, 0).unwrap();
        assert!(callers.is_empty());
    }

    /// F-047: load_call_graph returns bidirectional maps
    #[test]
    fn f047_load_call_graph_bidirectional() {
        let conn = db();
        let funcs = vec![
            entry("a", "fn a() { b(); }", "a.rs"),
            entry("b", "fn b() { c(); }", "a.rs"),
            entry("c", "fn c() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let mut calls = HashMap::new();
        calls.insert(0, vec![1]);
        calls.insert(1, vec![2]);
        insert_call_graph(&conn, &calls).unwrap();

        let (loaded_calls, loaded_called_by) = load_call_graph(&conn).unwrap();
        assert_eq!(loaded_calls[&0], vec![1]);
        assert_eq!(loaded_calls[&1], vec![2]);
        assert_eq!(loaded_called_by[&1], vec![0]);
        assert_eq!(loaded_called_by[&2], vec![1]);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 5: Graph metrics integrity
    //  Claims: PageRank, centrality, in/out_degree stored per function
    // ═══════════════════════════════════════════════════════════════════

    /// F-048: Graph metrics roundtrip preserves values
    #[test]
    fn f048_graph_metrics_roundtrip() {
        let conn = db();
        let funcs = vec![
            entry("a", "fn a() {}", "a.rs"),
            entry("b", "fn b() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let metrics = vec![
            GraphMetrics {
                pagerank: 0.123,
                centrality: 0.456,
                in_degree: 5,
                out_degree: 3,
            },
            GraphMetrics {
                pagerank: 0.789,
                centrality: 0.012,
                in_degree: 0,
                out_degree: 8,
            },
        ];
        insert_graph_metrics(&conn, &metrics).unwrap();
        let loaded = load_graph_metrics(&conn).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!((loaded[0].pagerank - 0.123).abs() < 0.001);
        assert!((loaded[0].centrality - 0.456).abs() < 0.001);
        assert_eq!(loaded[0].in_degree, 5);
        assert_eq!(loaded[0].out_degree, 3);
        assert!((loaded[1].pagerank - 0.789).abs() < 0.001);
        assert_eq!(loaded[1].out_degree, 8);
    }

    /// F-049: Graph metrics default to zero
    #[test]
    fn f049_graph_metrics_defaults() {
        let conn = db();
        let funcs = vec![entry("a", "fn a() {}", "a.rs")];
        insert_functions(&conn, &funcs).unwrap();
        let metrics = vec![GraphMetrics::default()];
        insert_graph_metrics(&conn, &metrics).unwrap();
        let loaded = load_graph_metrics(&conn).unwrap();
        assert_eq!(loaded[0], GraphMetrics::default());
    }

    /// F-050: Graph metrics INSERT OR REPLACE updates existing
    #[test]
    fn f050_graph_metrics_upsert() {
        let conn = db();
        let funcs = vec![entry("a", "fn a() {}", "a.rs")];
        insert_functions(&conn, &funcs).unwrap();
        let m1 = vec![GraphMetrics {
            pagerank: 0.1,
            centrality: 0.2,
            in_degree: 1,
            out_degree: 1,
        }];
        insert_graph_metrics(&conn, &m1).unwrap();
        let m2 = vec![GraphMetrics {
            pagerank: 0.9,
            centrality: 0.8,
            in_degree: 5,
            out_degree: 5,
        }];
        insert_graph_metrics(&conn, &m2).unwrap();
        let loaded = load_graph_metrics(&conn).unwrap();
        assert!(
            (loaded[0].pagerank - 0.9).abs() < 0.001,
            "should be updated to 0.9"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 6: Metadata persistence
    //  Claims: version, built_at, project_root, counts, checksums stored
    // ═══════════════════════════════════════════════════════════════════

    /// F-051: Metadata stores schema version as "2.0.0"
    #[test]
    fn f051_metadata_version() {
        let conn = db();
        let m = manifest(10, 5);
        insert_metadata(&conn, &m).unwrap();
        let loaded = load_metadata(&conn).unwrap();
        assert_eq!(
            loaded.version,
            crate::services::agent_context::function_index::sqlite_backend::SCHEMA_VERSION
        );
    }

    /// F-052: Metadata stores built_at timestamp
    #[test]
    fn f052_metadata_built_at() {
        let conn = db();
        let m = manifest(10, 5);
        insert_metadata(&conn, &m).unwrap();
        let loaded = load_metadata(&conn).unwrap();
        assert_eq!(loaded.built_at, "2026-02-08T00:00:00Z");
    }

    /// F-053: Metadata stores project_root
    #[test]
    fn f053_metadata_project_root() {
        let conn = db();
        let m = manifest(10, 5);
        insert_metadata(&conn, &m).unwrap();
        let loaded = load_metadata(&conn).unwrap();
        assert_eq!(loaded.project_root, "/test");
    }

    /// F-054: Metadata stores function_count
    #[test]
    fn f054_metadata_function_count() {
        let conn = db();
        let m = manifest(42, 10);
        insert_metadata(&conn, &m).unwrap();
        let loaded = load_metadata(&conn).unwrap();
        assert_eq!(loaded.function_count, 42);
    }

    /// F-055: Metadata stores file_count
    #[test]
    fn f055_metadata_file_count() {
        let conn = db();
        let m = manifest(42, 10);
        insert_metadata(&conn, &m).unwrap();
        let loaded = load_metadata(&conn).unwrap();
        assert_eq!(loaded.file_count, 10);
    }

    /// F-056: Metadata stores file_checksums
    #[test]
    fn f056_metadata_checksums() {
        let conn = db();
        let mut m = manifest(10, 2);
        m.file_checksums.insert(
            "src/a.rs".to_string(),
            FileRecord::from_checksum("sha256:aaa".to_string()),
        );
        m.file_checksums.insert(
            "src/b.rs".to_string(),
            FileRecord::from_checksum("sha256:bbb".to_string()),
        );
        insert_metadata(&conn, &m).unwrap();
        let loaded = load_metadata(&conn).unwrap();
        assert_eq!(loaded.file_checksums.len(), 2);
        assert_eq!(loaded.file_checksums["src/a.rs"].checksum, "sha256:aaa");
        assert_eq!(loaded.file_checksums["src/b.rs"].checksum, "sha256:bbb");
    }

    /// F-057: Metadata INSERT OR REPLACE updates existing keys
    #[test]
    fn f057_metadata_upsert() {
        let conn = db();
        let m1 = manifest(10, 5);
        insert_metadata(&conn, &m1).unwrap();
        let m2 = manifest(99, 33);
        insert_metadata(&conn, &m2).unwrap();
        let loaded = load_metadata(&conn).unwrap();
        assert_eq!(loaded.function_count, 99);
        assert_eq!(loaded.file_count, 33);
    }

    /// F-058: Metadata missing key self-heals with defaults (#162)
    #[test]
    fn f058_metadata_missing_key_self_heals() {
        let conn = db();
        let result = load_metadata(&conn);
        // Self-healing: returns Ok with defaults instead of error
        assert!(
            result.is_ok(),
            "missing metadata should self-heal with defaults"
        );
        let manifest = result.unwrap();
        assert_eq!(manifest.version, "2.0");
        assert_eq!(manifest.function_count, 0);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 7: ID mapping (0-based ↔ 1-based)
    //  Claims: internal 0-based indices map to 1-based SQLite rowids
    // ═══════════════════════════════════════════════════════════════════

    /// F-059: Functions are loaded in ID order (ORDER BY id)
    #[test]
    fn f059_functions_ordered_by_id() {
        let conn = db();
        let funcs = vec![
            entry("first", "fn first() {}", "a.rs"),
            entry("second", "fn second() {}", "b.rs"),
            entry("third", "fn third() {}", "c.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].function_name, "first");
        assert_eq!(loaded[1].function_name, "second");
        assert_eq!(loaded[2].function_name, "third");
    }

    /// F-060: FTS5 rowid maps to 0-based index correctly
    #[test]
    fn f060_fts5_rowid_to_index() {
        let conn = db();
        let funcs = vec![
            entry("alpha", "fn alpha() { unique_alpha_only(); }", "a.rs"),
            entry("beta", "fn beta() { unique_beta_only(); }", "b.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let results = fts5_search(&conn, "unique_alpha_only", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0, "rowid 1 should map to index 0");

        let results2 = fts5_search(&conn, "unique_beta_only", 10).unwrap();
        assert!(!results2.is_empty());
        assert_eq!(results2[0].0, 1, "rowid 2 should map to index 1");
    }

    /// F-061: Call graph uses 0-based indices in API
    #[test]
    fn f061_call_graph_0_based_api() {
        let conn = db();
        let funcs = vec![
            entry("a", "fn a() {}", "a.rs"),
            entry("b", "fn b() {}", "a.rs"),
            entry("c", "fn c() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let mut calls = HashMap::new();
        calls.insert(0, vec![2]); // a→c, 0-based
        insert_call_graph(&conn, &calls).unwrap();

        // Verify query uses 0-based
        let callees = query_callees(&conn, 0).unwrap();
        assert_eq!(callees, vec![2]);
        // Verify internal storage is 1-based
        let internal: i64 = conn
            .query_row("SELECT callee_id FROM call_graph LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(internal, 3, "internal callee_id should be 1-based (3)");
    }

    /// F-062: Graph metrics alignment with functions by index
    #[test]
    fn f062_graph_metrics_alignment() {
        let conn = db();
        let funcs = vec![
            entry("a", "fn a() {}", "a.rs"),
            entry("b", "fn b() {}", "a.rs"),
            entry("c", "fn c() {}", "a.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let metrics = vec![
            GraphMetrics {
                pagerank: 0.1,
                ..Default::default()
            },
            GraphMetrics {
                pagerank: 0.2,
                ..Default::default()
            },
            GraphMetrics {
                pagerank: 0.3,
                ..Default::default()
            },
        ];
        insert_graph_metrics(&conn, &metrics).unwrap();
        let loaded_m = load_graph_metrics(&conn).unwrap();
        let loaded_f = load_functions(&conn).unwrap();
        assert_eq!(loaded_m.len(), loaded_f.len());
        assert!((loaded_m[0].pagerank - 0.1).abs() < 0.001);
        assert!((loaded_m[2].pagerank - 0.3).abs() < 0.001);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 8: Transaction safety and atomicity
    //  Claims: inserts use transactions, clears before insert
    // ═══════════════════════════════════════════════════════════════════

    /// F-063: insert_functions clears old data first
    #[test]
    fn f063_insert_clears_old_data() {
        let conn = db();
        insert_functions(&conn, &[entry("old", "fn old() {}", "a.rs")]).unwrap();
        assert_eq!(count(&conn, "functions"), 1);
        insert_functions(&conn, &[entry("new", "fn new() {}", "b.rs")]).unwrap();
        assert_eq!(
            count(&conn, "functions"),
            1,
            "should clear old before insert"
        );
        let loaded = load_functions(&conn).unwrap();
        assert_eq!(loaded[0].function_name, "new");
    }

    /// F-064: insert_functions also clears FTS5 data
    #[test]
    fn f064_insert_clears_fts5() {
        let conn = db();
        insert_functions(
            &conn,
            &[entry("old", "fn old() { unique_old_marker(); }", "a.rs")],
        )
        .unwrap();
        let r1 = fts5_search(&conn, "unique_old_marker", 10).unwrap();
        assert!(!r1.is_empty());

        insert_functions(
            &conn,
            &[entry("new", "fn new() { unique_new_marker(); }", "b.rs")],
        )
        .unwrap();
        let r2 = fts5_search(&conn, "unique_old_marker", 10).unwrap();
        assert!(r2.is_empty(), "old FTS5 data should be cleared");
        let r3 = fts5_search(&conn, "unique_new_marker", 10).unwrap();
        assert!(!r3.is_empty());
    }

    /// F-065: insert_functions clears call_graph and graph_metrics
    /// Note: SQLite FK enforcement must be OFF (default) for delete order to work
    #[test]
    fn f065_insert_clears_related_tables() {
        let conn = db();
        // Ensure FK enforcement is off (SQLite default) so DELETE order doesn't matter
        conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        let funcs = vec![
            entry("a", "fn a() {}", "a.rs"),
            entry("b", "fn b() {}", "b.rs"),
        ];
        insert_functions(&conn, &funcs).unwrap();
        let mut calls = HashMap::new();
        calls.insert(0, vec![1]);
        insert_call_graph(&conn, &calls).unwrap();
        insert_graph_metrics(&conn, &[GraphMetrics::default(), GraphMetrics::default()]).unwrap();

        // Re-insert functions should clear call_graph and graph_metrics
        insert_functions(&conn, &[entry("c", "fn c() {}", "c.rs")]).unwrap();
        assert_eq!(count(&conn, "call_graph"), 0);
        assert_eq!(count(&conn, "graph_metrics"), 0);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Section 9: Performance pragmas (spec: "open_db()" section)
    //  Claims: WAL mode, cache_size, mmap_size, temp_store
    // ═══════════════════════════════════════════════════════════════════

    /// F-066: WAL journal mode is set (requires file-based DB, not in-memory)
    #[test]
    fn f066_wal_journal_mode() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal", "should use WAL mode");
    }

    /// F-067: synchronous = NORMAL (requires open_db for pragma setup)
    #[test]
    fn f067_synchronous_normal() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        let sync: i64 = conn
            .query_row("PRAGMA synchronous", [], |r| r.get(0))
            .unwrap();
        // NORMAL = 1
        assert_eq!(sync, 1, "synchronous should be NORMAL (1)");
    }

    /// F-068: cache_size is set to -64000 (64MB)
    #[test]
    fn f068_cache_size() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        let cs: i64 = conn
            .query_row("PRAGMA cache_size", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cs, -64000, "cache_size should be -64000");
    }

    /// F-069: mmap_size is set to 256MB
    #[test]
    fn f069_mmap_size() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        let mmap: i64 = conn
            .query_row("PRAGMA mmap_size", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mmap, 268435456, "mmap_size should be 256MB");
    }

    /// F-070: temp_store = MEMORY
    #[test]
    fn f070_temp_store_memory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        let ts: i64 = conn
            .query_row("PRAGMA temp_store", [], |r| r.get(0))
            .unwrap();
        // MEMORY = 2
        assert_eq!(ts, 2, "temp_store should be MEMORY (2)");
    }

    // ═══════════════════════════════════════════════════════════════════
    //  Utility
    // ═══════════════════════════════════════════════════════════════════

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }
}
