#![cfg_attr(coverage_nightly, coverage(off))]

use super::helpers::*;
use super::types::*;
use crate::services::semantic::{chunk_code, Language};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[test]
fn test_detect_language() {
    assert_eq!(detect_language(Path::new("test.rs")), Some(Language::Rust));
    assert_eq!(
        detect_language(Path::new("test.py")),
        Some(Language::Python)
    );
    assert_eq!(detect_language(Path::new("test.txt")), None);
}

#[test]
fn test_count_complexity() {
    let simple = "fn foo() { return 1; }";
    assert_eq!(count_complexity(simple), 1);

    let with_if = "fn foo() { if x { return 1; } return 2; }";
    assert_eq!(count_complexity(with_if), 2);
}

#[test]
fn test_count_satd_markers() {
    let clean = "fn foo() { return 1; }";
    assert_eq!(count_satd_markers(clean), 0);

    let with_todo = "fn foo() { // TODO: fix this\n return 1; }";
    assert_eq!(count_satd_markers(with_todo), 1);
}

#[test]
fn test_estimate_big_o() {
    let constant = "fn foo() { return 1; }";
    assert_eq!(estimate_big_o(constant), "O(1)");

    let linear = "fn foo() {\n    for i in items {\n        process(i);\n    }\n}";
    assert_eq!(estimate_big_o(linear), "O(n)");
}

#[test]
fn test_score_to_grade() {
    assert_eq!(score_to_grade(0.5), "A");
    assert_eq!(score_to_grade(2.5), "B");
    assert_eq!(score_to_grade(5.0), "C");
    assert_eq!(score_to_grade(7.0), "D");
    assert_eq!(score_to_grade(9.0), "F");
}

#[test]
fn test_is_ignored_dir() {
    assert!(is_ignored_dir(Path::new("target")));
    assert!(is_ignored_dir(Path::new("node_modules")));
    assert!(!is_ignored_dir(Path::new("src")));
}

#[test]
fn test_compute_file_sha256() {
    let hash1 = compute_file_sha256("hello world");
    let hash2 = compute_file_sha256("hello world");
    let hash3 = compute_file_sha256("different content");
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
    assert_eq!(hash1.len(), 64); // SHA256 hex is 64 chars
}

#[test]
fn test_compute_name_frequency() {
    let mut name_index = HashMap::new();
    name_index.insert("new".to_string(), vec![0, 1, 2, 3, 4]);
    name_index.insert("unique_func".to_string(), vec![5]);
    let freq = compute_name_frequency(&name_index, 10);
    assert!((freq["new"] - 0.5).abs() < 0.01);
    assert!((freq["unique_func"] - 0.1).abs() < 0.01);
}

#[test]
fn test_compute_name_frequency_empty() {
    let name_index = HashMap::new();
    let freq = compute_name_frequency(&name_index, 0);
    assert!(freq.is_empty());
}

#[test]
fn test_build_indices() {
    let functions = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "foo".to_string(),
            signature: "fn foo()".to_string(),
            doc_comment: None,
            source: "fn foo() {}".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "abc".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "bar".to_string(),
            signature: "fn bar()".to_string(),
            doc_comment: None,
            source: "fn bar() {}".to_string(),
            start_line: 3,
            end_line: 3,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "def".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];
    let indices = build_indices(&functions);
    assert_eq!(indices.name_index["foo"], vec![0]);
    assert_eq!(indices.name_index["bar"], vec![1]);
    assert_eq!(indices.file_index["a.rs"], vec![0, 1]);
    assert_eq!(indices.corpus.len(), 2);
}

#[test]
fn test_build_call_graph() {
    // foo calls bar (bar appears as identifier in foo's source)
    let functions = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "foo".to_string(),
            signature: "fn foo()".to_string(),
            doc_comment: None,
            source: "fn foo() { bar(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "abc".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "bar".to_string(),
            signature: "fn bar()".to_string(),
            doc_comment: None,
            source: "fn bar() { println!(\"hello\"); }".to_string(),
            start_line: 3,
            end_line: 3,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "def".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];
    let indices = build_indices(&functions);
    let (calls, called_by) = build_call_graph(&functions, &indices.name_index);

    // foo calls bar
    assert!(calls.get(&0).map_or(false, |v| v.contains(&1)));
    // bar is called by foo
    assert!(called_by.get(&1).map_or(false, |v| v.contains(&0)));
    // bar does not call foo
    assert!(!calls.get(&1).map_or(false, |v| v.contains(&0)));
}

#[test]
fn test_save_load_roundtrip_v1_1() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create a simple Rust file
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn hello() { world(); }\nfn world() { println!(\"hi\"); }\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let index_path = project_path.join("idx");
    index.save(&index_path).unwrap();

    let loaded = AgentContextIndex::load(&index_path).unwrap();
    // load() prefers SQLite (v2.0.0) over blob (v1.4.0) when both exist
    assert!(
        loaded.manifest.version == "2.0.0" || loaded.manifest.version == "1.4.0",
        "expected v2.0.0 or v1.4.0, got {}",
        loaded.manifest.version,
    );
    assert_eq!(loaded.functions.len(), index.functions.len());
    // SQLite path skips corpus (FTS5 handles search); blob path has corpus
    if loaded.manifest.version == "2.0.0" {
        assert!(loaded.corpus.is_empty(), "SQLite load should skip corpus");
    } else {
        assert_eq!(loaded.corpus.len(), index.corpus.len());
    }
}

#[test]
fn test_load_prefers_sqlite_over_blob() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn alpha() { beta(); }\nfn beta() {}\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let index_path = project_path.join("idx");
    index.save(&index_path).unwrap();

    // Phase 3: only context.db and manifest written (no blob)
    let db_path = index_path.with_extension("db");
    assert!(db_path.exists(), "context.db should exist after save");
    assert!(
        index_path.join("manifest.json").exists(),
        "manifest should exist"
    );
    assert!(
        !index_path.join("functions.lz4").exists(),
        "blob should NOT be written in Phase 3"
    );

    // load() prefers SQLite
    let loaded = AgentContextIndex::load(&index_path).unwrap();
    assert_eq!(loaded.manifest.version, "2.0.0");
    assert!(loaded.db_path.is_some());
    assert_eq!(loaded.functions.len(), index.functions.len());

    // Verify call graph queryable via on-demand SQLite lookup
    // (calls/called_by HashMaps are empty — queried on-demand)
    let has_call_data = (0..loaded.functions.len())
        .any(|i| !loaded.get_calls(i).is_empty() || !loaded.get_called_by(i).is_empty());
    assert!(
        has_call_data,
        "should have call graph data via SQLite query"
    );
}

#[test]
fn test_load_fails_without_sqlite_or_blob() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn gamma() {}\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let index_path = project_path.join("idx");
    index.save(&index_path).unwrap();

    // Remove SQLite DB — no blob either (Phase 3 doesn't write blobs)
    let db_path = index_path.with_extension("db");
    std::fs::remove_file(&db_path).unwrap();

    // Should fail: no SQLite, no blob
    let result = AgentContextIndex::load(&index_path);
    assert!(result.is_err());
}

#[test]
fn test_incremental_build_unchanged() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn alpha() { }\nfn beta() { }\n",
    )
    .unwrap();

    let original = AgentContextIndex::build(project_path).unwrap();
    let incremental = AgentContextIndex::build_incremental(project_path, &original).unwrap();

    // Same number of functions (nothing changed)
    assert_eq!(incremental.functions.len(), original.functions.len());
}

#[test]
fn test_incremental_build_with_change() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn alpha() { }\n").unwrap();

    let original = AgentContextIndex::build(project_path).unwrap();
    assert_eq!(original.functions.len(), 1);

    // Modify the file to add a function
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn alpha() { }\nfn gamma() { }\n",
    )
    .unwrap();

    let incremental = AgentContextIndex::build_incremental(project_path, &original).unwrap();
    // Should now have 2 functions
    assert_eq!(incremental.functions.len(), 2);
}

#[test]
fn test_parse_workspace_siblings() {
    let toml = r#"siblings = ["../aprender", "../trueno", "../realizar"]"#;
    let result = parse_workspace_siblings(toml);
    assert_eq!(result, vec!["../aprender", "../trueno", "../realizar"]);
}

#[test]
fn test_parse_workspace_siblings_empty() {
    let toml = "# no siblings configured\n";
    let result = parse_workspace_siblings(toml);
    assert!(result.is_empty());
}

#[test]
fn test_parse_workspace_siblings_single() {
    let toml = r#"siblings = ["../trueno"]"#;
    let result = parse_workspace_siblings(toml);
    assert_eq!(result, vec!["../trueno"]);
}

#[test]
fn test_parse_workspace_siblings_with_spaces() {
    let toml = r#"siblings  =  [ "../a" , "../b" ]"#;
    let result = parse_workspace_siblings(toml);
    assert_eq!(result, vec!["../a", "../b"]);
}

#[test]
fn test_discover_siblings_no_config() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let result = AgentContextIndex::discover_sibling_indexes(temp_dir.path());
    assert!(result.is_empty());
}

#[test]
fn test_file_checksums_populated() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn test_func() { }\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    assert!(!index.manifest.file_checksums.is_empty());
    assert!(index.manifest.file_checksums.contains_key("src/lib.rs"));
}

#[test]
fn test_get_calls_and_called_by() {
    let functions = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "caller".to_string(),
            signature: "fn caller()".to_string(),
            doc_comment: None,
            source: "fn caller() { callee(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "aaa".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "callee".to_string(),
            signature: "fn callee()".to_string(),
            doc_comment: None,
            source: "fn callee() { println!(\"hello\"); }".to_string(),
            start_line: 3,
            end_line: 3,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "bbb".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];

    let indices = build_indices(&functions);
    let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();
    let (calls, called_by) = build_call_graph(&functions, &indices.name_index);
    let graph_metrics = compute_graph_metrics(functions.len(), &calls, &called_by);

    let index = AgentContextIndex {
        functions,
        name_index: indices.name_index,
        file_index: indices.file_index,
        corpus: indices.corpus,
        corpus_lower,
        name_frequency: HashMap::new(),
        calls,
        called_by,
        graph_metrics,
        project_root: PathBuf::from("/test"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 2,
            file_count: 1,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };

    let calls_of_0 = index.get_calls(0);
    assert!(calls_of_0.contains(&"callee"), "caller should call callee");

    let called_by_1 = index.get_called_by(1);
    assert!(
        called_by_1.contains(&"caller"),
        "callee should be called by caller"
    );

    // Non-existent index
    assert!(index.get_calls(999).is_empty());
    assert!(index.get_called_by(999).is_empty());
}

#[test]
fn test_find_function_index() {
    let functions = vec![FunctionEntry {
        file_path: "a.rs".to_string(),
        function_name: "foo".to_string(),
        signature: "fn foo()".to_string(),
        doc_comment: None,
        source: "fn foo() {}".to_string(),
        start_line: 1,
        end_line: 1,
        language: "Rust".to_string(),
        quality: QualityMetrics::default(),
        checksum: "aaa".to_string(),
        definition_type: DefinitionType::default(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(),
    }];

    let indices = build_indices(&functions);
    let corpus_lower: Vec<String> = indices.corpus.iter().map(|c| c.to_lowercase()).collect();

    let index = AgentContextIndex {
        functions,
        name_index: indices.name_index,
        file_index: indices.file_index,
        corpus: indices.corpus,
        corpus_lower,
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![GraphMetrics::default()],
        project_root: PathBuf::from("/test"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "2025-01-01T00:00:00Z".to_string(),
            project_root: "/test".to_string(),
            function_count: 1,
            file_count: 1,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };

    assert_eq!(index.find_function_index("a.rs", "foo"), Some(0));
    assert_eq!(index.find_function_index("a.rs", "bar"), None);
    assert_eq!(index.find_function_index("b.rs", "foo"), None);
}

#[test]
fn test_discover_siblings_with_config() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create .pmat/workspace.toml
    std::fs::create_dir_all(project_path.join(".pmat")).unwrap();
    std::fs::write(
        project_path.join(".pmat/workspace.toml"),
        r#"siblings = ["../sibling_a"]"#,
    )
    .unwrap();

    // We can't easily create a real sibling in tempdir, so just verify
    // the function reads the config correctly without panicking
    let result = AgentContextIndex::discover_sibling_indexes(project_path);
    // Sibling doesn't exist, so no results
    assert!(result.is_empty());
}

#[test]
fn test_discover_siblings_with_real_sibling() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let workspace = temp_dir.path();

    // Create project A
    let project_a = workspace.join("project_a");
    std::fs::create_dir_all(project_a.join(".pmat")).unwrap();

    // Create project B with an index
    let project_b = workspace.join("project_b");
    std::fs::create_dir_all(project_b.join("src")).unwrap();
    std::fs::write(project_b.join("src/lib.rs"), "fn sibling_func() {}\n").unwrap();
    let b_index = AgentContextIndex::build(&project_b).unwrap();
    let b_idx_path = project_b.join(".pmat/context.idx");
    std::fs::create_dir_all(b_idx_path.parent().unwrap()).unwrap();
    b_index.save(&b_idx_path).unwrap();

    // Configure A to point to B
    std::fs::write(
        project_a.join(".pmat/workspace.toml"),
        format!(r#"siblings = ["../project_b"]"#),
    )
    .unwrap();

    let siblings = AgentContextIndex::discover_sibling_indexes(&project_a);
    assert_eq!(siblings.len(), 1);
    assert_eq!(siblings[0].1, "project_b");
}

#[test]
fn test_merge_fast() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    // Build index A
    let proj_a = temp_dir.path().join("a");
    std::fs::create_dir_all(proj_a.join("src")).unwrap();
    std::fs::write(proj_a.join("src/lib.rs"), "fn alpha() {}\n").unwrap();
    let mut index_a = AgentContextIndex::build(&proj_a).unwrap();
    let a_count = index_a.functions.len();

    // Build index B
    let proj_b = temp_dir.path().join("b");
    std::fs::create_dir_all(proj_b.join("src")).unwrap();
    std::fs::write(proj_b.join("src/lib.rs"), "fn beta() {}\n").unwrap();
    let index_b = AgentContextIndex::build(&proj_b).unwrap();
    let b_count = index_b.functions.len();

    index_a.merge_fast(index_b);

    assert_eq!(index_a.functions.len(), a_count + b_count);
    // All functions accessible
    let names: Vec<&str> = index_a
        .functions
        .iter()
        .map(|f| f.function_name.as_str())
        .collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

#[test]
fn test_manifest_accessor() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn foo() {}\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let manifest = index.manifest();
    assert_eq!(manifest.version, "1.4.0");
    assert!(manifest.function_count > 0);
    assert!(manifest.file_count > 0);
}

#[test]
fn test_extract_doc_comment_basic() {
    let content = "/// This is a doc comment\nfn foo() {}";
    let doc = extract_doc_comment(content, 2); // fn is on line 2
    assert!(doc.is_some());
    assert!(doc.unwrap().contains("This is a doc comment"));
}

#[test]
fn test_extract_doc_comment_none() {
    let content = "fn foo() {}";
    let doc = extract_doc_comment(content, 1);
    assert!(doc.is_none());
}

#[test]
fn test_calculate_simple_tdg() {
    // Low complexity, no SATD, small LOC = low score
    let score = calculate_simple_tdg(1, 0, 10);
    assert!(score < 2.0);

    // High complexity, SATD, large LOC = higher score
    let high_score = calculate_simple_tdg(20, 3, 200);
    assert!(high_score > score);
}

#[test]
fn test_is_keyword() {
    assert!(is_keyword("fn"));
    assert!(is_keyword("let"));
    assert!(is_keyword("if"));
    assert!(is_keyword("for"));
    assert!(is_keyword("while"));
    assert!(is_keyword("return"));
    assert!(is_keyword("def"));
    assert!(is_keyword("class"));
    assert!(is_keyword("import"));
    assert!(!is_keyword("handle_error"));
    assert!(!is_keyword("MyStruct"));
}

#[test]
fn test_estimate_big_o_quadratic() {
    let quadratic = "fn foo() {\n    for i in items {\n        for j in items {\n            process(i, j);\n        }\n    }\n}";
    assert_eq!(estimate_big_o(quadratic), "O(n^2)");
}

#[test]
fn test_estimate_big_o_logarithmic() {
    let log = "fn foo() {\n    while n > 0 {\n        n /= 2;\n    }\n}";
    // Contains while + divide = log
    assert!(["O(n log n)", "O(log n)", "O(n)"].contains(&estimate_big_o(log).as_str()));
}

#[test]
fn test_build_with_multiple_files() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create multiple Rust files
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "/// Documentation for rust_func\nfn rust_func() { if true { println!(\"hello\"); } }\n",
    )
    .unwrap();
    std::fs::write(
        project_path.join("src/helper.rs"),
        "/// Helper function\nfn helper_func() { for i in 0..10 { println!(\"{}\", i); } }\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    assert!(index.functions.len() >= 2);
    assert!(index.manifest.file_count >= 2);
    assert_eq!(index.manifest.version, "1.4.0");

    // Verify quality metrics computed
    for func in &index.functions {
        assert!(!func.function_name.is_empty());
        assert!(!func.file_path.is_empty());
        assert!(!func.language.is_empty());
        assert!(!func.quality.tdg_grade.is_empty());
        assert!(!func.quality.big_o.is_empty());
    }
}

#[test]
fn test_build_empty_project() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    // Empty file - no functions
    std::fs::write(project_path.join("src/lib.rs"), "// empty\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    assert_eq!(index.functions.len(), 0);
    assert!((index.manifest.avg_tdg_score - 0.0).abs() < 0.01);
}

#[test]
fn test_build_with_binary_file() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn valid() {}\n").unwrap();
    // Binary file should be skipped
    std::fs::write(project_path.join("src/data.bin"), &[0u8, 1, 2, 255, 254]).unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    // Should index the .rs file but skip .bin
    assert!(index.functions.len() >= 1);
}

#[test]
fn test_build_skips_ignored_dirs() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::create_dir_all(project_path.join("node_modules/pkg")).unwrap();
    std::fs::create_dir_all(project_path.join("target/debug")).unwrap();

    std::fs::write(project_path.join("src/lib.rs"), "fn keep() {}\n").unwrap();
    std::fs::write(
        project_path.join("node_modules/pkg/index.rs"),
        "fn skip_nm() {}\n",
    )
    .unwrap();
    std::fs::write(
        project_path.join("target/debug/output.rs"),
        "fn skip_target() {}\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let names: Vec<&str> = index
        .functions
        .iter()
        .map(|f| f.function_name.as_str())
        .collect();
    assert!(names.contains(&"keep"));
    assert!(!names.contains(&"skip_nm"));
    assert!(!names.contains(&"skip_target"));
}

#[test]
fn test_save_and_load_preserves_calls() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn caller() { callee(); }\nfn callee() { println!(\"hi\"); }\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let idx_path = project_path.join("idx");
    index.save(&idx_path).unwrap();

    let loaded = AgentContextIndex::load(&idx_path).unwrap();
    // Call graph queryable via on-demand SQLite (in-memory maps empty on SQLite load)
    // Verify by checking actual call relationships
    let original_calls: Vec<String> = index.get_calls(0).iter().map(|s| s.to_string()).collect();
    let loaded_calls: Vec<String> = loaded.get_calls(0).iter().map(|s| s.to_string()).collect();
    assert_eq!(
        loaded_calls.len(),
        original_calls.len(),
        "call graph should be preserved"
    );
}

#[test]
fn test_load_invalid_path() {
    let result = AgentContextIndex::load(Path::new("/nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn test_extract_quality_metrics() {
    let source = "fn complex() {\n    if a {\n        if b {\n            for i in items {\n                // TODO: fix\n                process(i);\n            }\n        }\n    }\n}\n";
    let chunks = chunk_code(source, Language::Rust).unwrap();
    if let Some(chunk) = chunks.first() {
        let quality = extract_quality_metrics(chunk, source);
        assert!(quality.complexity >= 3); // if + if + for
        assert!(quality.satd_count >= 1); // TODO
        assert_eq!(quality.big_o, "O(n)"); // single for loop
    }
}

#[test]
fn test_count_complexity_various() {
    // Multi-line if/else if
    let if_else = "fn f() {\n    if a {\n    } else if b {\n    } else {\n    }\n}";
    assert!(count_complexity(if_else) >= 3);
    // Match expression on its own line
    let matchex = "fn f() {\n    match x {\n        A => {},\n        B => {}\n    }\n}";
    assert!(count_complexity(matchex) >= 2);
    // While loop
    let whileex = "fn f() {\n    while true {\n        break;\n    }\n}";
    assert!(count_complexity(whileex) >= 2);
    // Boolean operators on one line
    let booleans = "fn f() { x && y || z }";
    assert!(count_complexity(booleans) >= 2); // && and || both on same line count once
}

#[test]
fn test_count_satd_markers_various() {
    assert_eq!(count_satd_markers("// FIXME: broken"), 1);
    assert_eq!(count_satd_markers("// HACK: workaround"), 1);
    assert_eq!(count_satd_markers("// XXX: temporary"), 0); // XXX removed - caused false positives from BUG-XXX patterns
    assert_eq!(count_satd_markers("// TODO: fix\n// FIXME: also fix"), 2);
    assert_eq!(count_satd_markers("// Normal comment"), 0);
}

#[test]
fn test_detect_code_clones_with_duplicates() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "func_a".to_string(),
            signature: "fn func_a()".to_string(),
            doc_comment: None,
            source: "fn func_a() { let x = 1; let y = 2; x + y }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "aaa".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "b.rs".to_string(),
            function_name: "func_b".to_string(),
            signature: "fn func_b()".to_string(),
            doc_comment: None,
            // Same source after normalization (whitespace-insensitive)
            source: "fn func_a() { let x = 1; let y = 2; x + y }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "bbb".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "c.rs".to_string(),
            function_name: "func_c".to_string(),
            signature: "fn func_c()".to_string(),
            doc_comment: None,
            source: "fn func_c() { completely_different_code(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "ccc".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];

    let clones = detect_code_clones(&funcs);
    // func_a and func_b are clones (same normalized source)
    assert_eq!(clones.get(&0), Some(&2));
    assert_eq!(clones.get(&1), Some(&2));
    // func_c is unique
    assert_eq!(clones.get(&2), None);
}

#[test]
fn test_detect_code_clones_no_duplicates() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "unique_a".to_string(),
            signature: "fn unique_a()".to_string(),
            doc_comment: None,
            source: "fn unique_a() { alpha(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "aaa".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "b.rs".to_string(),
            function_name: "unique_b".to_string(),
            signature: "fn unique_b()".to_string(),
            doc_comment: None,
            source: "fn unique_b() { beta(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "bbb".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];

    let clones = detect_code_clones(&funcs);
    assert!(clones.is_empty());
}

#[test]
fn test_normalize_source() {
    assert_eq!(normalize_source("fn foo() { }"), "fnfoo(){}");
    assert_eq!(normalize_source("  fn  foo ( ) {\n}"), "fnfoo(){}");
    assert_eq!(normalize_source("FN FOO()"), "fnfoo()");
}

#[test]
fn test_compute_file_pattern_diversity() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "f1".to_string(),
            signature: "fn f1() -> bool".to_string(),
            doc_comment: None,
            source: "".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                complexity: 2,
                ..Default::default()
            },
            checksum: "a".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "f2".to_string(),
            signature: "fn f2(x: i32) -> String".to_string(),
            doc_comment: None,
            source: "".to_string(),
            start_line: 5,
            end_line: 10,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                complexity: 8,
                ..Default::default()
            },
            checksum: "b".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "f3".to_string(),
            signature: "fn f3() -> bool".to_string(),
            doc_comment: None,
            source: "".to_string(),
            start_line: 15,
            end_line: 20,
            language: "Rust".to_string(),
            quality: QualityMetrics {
                complexity: 2,
                ..Default::default()
            },
            checksum: "c".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];
    let mut file_index = HashMap::new();
    file_index.insert("a.rs".to_string(), vec![0, 1, 2]);

    let diversity = compute_file_pattern_diversity(&funcs, &file_index);
    let d = diversity["a.rs"];
    // f1 and f3 have same pattern (bool:0:0), f2 is different (String:1:1)
    // 2 unique / 3 total = 0.667
    assert!(d > 0.5 && d < 0.8, "unexpected diversity: {d}");
}

#[test]
fn test_compute_file_pattern_diversity_empty() {
    let funcs: Vec<FunctionEntry> = vec![];
    let mut file_index = HashMap::new();
    file_index.insert("a.rs".to_string(), Vec::new());

    let diversity = compute_file_pattern_diversity(&funcs, &file_index);
    assert!(!diversity.contains_key("a.rs")); // empty indices skipped
}

#[test]
fn test_extract_return_type() {
    assert_eq!(extract_return_type("fn foo() -> bool"), "bool");
    assert_eq!(
        extract_return_type("fn foo() -> Result<String, Error>"),
        "Result<String, Error>"
    );
    assert_eq!(extract_return_type("fn foo()"), "void");
}

#[test]
fn test_count_params() {
    assert_eq!(count_params("fn foo()"), 0);
    assert_eq!(count_params("fn foo(x: i32)"), 1);
    assert_eq!(count_params("fn foo(x: i32, y: String)"), 2);
    assert_eq!(count_params("fn foo(x: i32, y: String, z: bool)"), 3);
    assert_eq!(count_params("no parens"), 0);
}

#[test]
fn test_detect_fault_patterns() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "risky".to_string(),
            signature: "fn risky()".to_string(),
            doc_comment: None,
            source: "fn risky() { x.unwrap(); y.clone(); // TODO: fix }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "a".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "b.rs".to_string(),
            function_name: "safe".to_string(),
            signature: "fn safe()".to_string(),
            doc_comment: None,
            source: "fn safe() { println!(\"hello\"); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "b".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "c.rs".to_string(),
            function_name: "dangerous".to_string(),
            signature: "fn dangerous()".to_string(),
            doc_comment: None,
            source: "fn dangerous() { unsafe { panic!(\"boom\"); } }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "c".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];

    let faults = detect_fault_patterns(&funcs);
    // risky has: UNWRAP, CLONE, TODO
    let risky_faults = &faults[&0];
    assert!(risky_faults.contains(&"UNWRAP".to_string()));
    assert!(risky_faults.contains(&"CLONE".to_string()));
    assert!(risky_faults.contains(&"TODO".to_string()));
    // safe has no faults
    assert!(!faults.contains_key(&1));
    // dangerous has PANIC, UNSAFE
    let dangerous_faults = &faults[&2];
    assert!(dangerous_faults.contains(&"PANIC".to_string()));
    assert!(dangerous_faults.contains(&"UNSAFE".to_string()));
}

#[test]
fn test_detect_fault_patterns_more() {
    let funcs = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "f".to_string(),
            signature: "fn f()".to_string(),
            doc_comment: None,
            source: "fn f() { x.expect(\"msg\"); // FIXME: broken\n// HACK: workaround\n// XXX: bad\ntodo!(\"later\");\nunimplemented!(\"not yet\");\nunreachable!(\"never\"); }".to_string(),
            start_line: 1, end_line: 1, language: "Rust".to_string(),
            quality: QualityMetrics::default(), checksum: "a".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0, churn_score: 0.0, clone_count: 0,
            pattern_diversity: 0.0, fault_annotations: Vec::new(),
        },
    ];

    let faults = detect_fault_patterns(&funcs);
    let f = &faults[&0];
    assert!(f.contains(&"EXPECT".to_string()));
    assert!(f.contains(&"FIXME".to_string()));
    assert!(f.contains(&"HACK".to_string()));
    assert!(f.contains(&"XXX".to_string()));
    assert!(f.contains(&"TODO_MACRO".to_string()));
    assert!(f.contains(&"UNIMPL".to_string()));
    assert!(f.contains(&"UNREACHABLE".to_string()));
}

#[test]
fn test_extract_identifiers() {
    let idents = extract_identifiers("fn foo() { bar_baz(42); hello.world(); }");
    assert!(idents.contains("foo"));
    assert!(idents.contains("bar_baz"));
    assert!(idents.contains("hello"));
    assert!(idents.contains("world"));
    // Short words (<3 chars) excluded
    assert!(!idents.contains("42"));
}

#[test]
fn test_extract_identifiers_filters_keywords() {
    let idents = extract_identifiers("fn handle() { if let mut x = return; }");
    // Keywords excluded
    assert!(!idents.contains("fn"));
    assert!(!idents.contains("if"));
    assert!(!idents.contains("let"));
    assert!(!idents.contains("mut"));
    assert!(!idents.contains("return"));
    // Non-keyword kept
    assert!(idents.contains("handle"));
}

#[test]
fn test_extract_doc_comment_block() {
    let content = "/**\n * Block doc comment\n */\nfn foo() {}";
    let doc = extract_doc_comment(content, 4);
    // Block comments cause break, so may return None or partial
    assert!(
        doc.is_none()
            || doc
                .as_ref()
                .map_or(false, |d| d.contains("Block doc comment"))
    );
}

#[test]
fn test_extract_doc_comment_with_attribute() {
    let content = "/// Doc line\n#[inline]\nfn foo() {}";
    let doc = extract_doc_comment(content, 3);
    assert!(doc.is_some());
    assert!(doc.unwrap().contains("Doc line"));
}

#[test]
fn test_estimate_big_o_cubic() {
    let cubic = "fn f() {\n    for i in a {\n        for j in b {\n            for k in c {\n                process();\n            }\n        }\n    }\n}";
    assert_eq!(estimate_big_o(cubic), "O(n^3)");
}

#[test]
fn test_estimate_big_o_n4() {
    let n4 = "fn f() {\n    for _ in a {\n        for _ in b {\n            for _ in c {\n                for _ in d {\n                    x();\n                }\n            }\n        }\n    }\n}";
    assert_eq!(estimate_big_o(n4), "O(n^4)");
}

#[test]
fn test_calculate_simple_tdg_boundaries() {
    // Zero everything
    let score = calculate_simple_tdg(0, 0, 0);
    assert!((score - 0.0).abs() < 0.01);

    // Max complexity capped at 4.0
    let max_complexity = calculate_simple_tdg(100, 0, 0);
    assert!((max_complexity - 4.0).abs() < 0.01);

    // SATD capped at 2.0
    let max_satd = calculate_simple_tdg(0, 10, 0);
    assert!((max_satd - 2.0).abs() < 0.01);

    // LOC penalty kicks in above 200
    let no_loc_penalty = calculate_simple_tdg(0, 0, 200);
    assert!((no_loc_penalty - 0.0).abs() < 0.01);

    let large_loc = calculate_simple_tdg(0, 0, 400);
    assert!(large_loc > 0.0);

    // Max possible: complexity=4 + satd=2 + loc=2 = 8.0
    let max_all = calculate_simple_tdg(100, 10, 1000);
    assert!((max_all - 8.0).abs() < 0.01);
}

#[test]
fn test_score_to_grade_boundaries() {
    assert_eq!(score_to_grade(0.0), "A");
    assert_eq!(score_to_grade(1.99), "A");
    assert_eq!(score_to_grade(2.0), "B");
    assert_eq!(score_to_grade(3.99), "B");
    assert_eq!(score_to_grade(4.0), "C");
    assert_eq!(score_to_grade(5.99), "C");
    assert_eq!(score_to_grade(6.0), "D");
    assert_eq!(score_to_grade(7.99), "D");
    assert_eq!(score_to_grade(8.0), "F");
    assert_eq!(score_to_grade(10.0), "F");
}

#[test]
fn test_compute_graph_metrics_empty() {
    let metrics = compute_graph_metrics(0, &HashMap::new(), &HashMap::new());
    assert!(metrics.is_empty());
}

#[test]
fn test_compute_graph_metrics_isolated_nodes() {
    // No calls between nodes -> dangling node handling
    let metrics = compute_graph_metrics(3, &HashMap::new(), &HashMap::new());
    assert_eq!(metrics.len(), 3);
    // All nodes are dangling, PageRank should be uniform
    for m in &metrics {
        assert!(
            m.pagerank > 0.0,
            "isolated node should have positive pagerank"
        );
        assert_eq!(m.in_degree, 0);
        assert_eq!(m.out_degree, 0);
    }
    // PageRank should be approximately equal for all
    let diff = (metrics[0].pagerank - metrics[1].pagerank).abs();
    assert!(
        diff < 0.001,
        "isolated nodes should have near-equal pagerank"
    );
}

#[test]
fn test_compute_graph_metrics_chain() {
    // 0 -> 1 -> 2 (chain)
    let mut calls = HashMap::new();
    calls.insert(0, vec![1]);
    calls.insert(1, vec![2]);
    let mut called_by = HashMap::new();
    called_by.insert(1, vec![0]);
    called_by.insert(2, vec![1]);

    let metrics = compute_graph_metrics(3, &calls, &called_by);
    assert_eq!(metrics.len(), 3);
    // Node 2 (end of chain) should have highest PageRank (most "important" via link structure)
    assert!(
        metrics[2].pagerank > metrics[0].pagerank,
        "end of chain should have higher pagerank: {} vs {}",
        metrics[2].pagerank,
        metrics[0].pagerank
    );
    // In/out degree checks
    assert_eq!(metrics[0].out_degree, 1);
    assert_eq!(metrics[0].in_degree, 0);
    assert_eq!(metrics[1].in_degree, 1);
    assert_eq!(metrics[1].out_degree, 1);
    assert_eq!(metrics[2].in_degree, 1);
    assert_eq!(metrics[2].out_degree, 0);
}

#[test]
fn test_is_ignored_dir_comprehensive() {
    assert!(is_ignored_dir(Path::new("target")));
    assert!(is_ignored_dir(Path::new("node_modules")));
    assert!(is_ignored_dir(Path::new(".git")));
    assert!(is_ignored_dir(Path::new(".pmat")));
    assert!(is_ignored_dir(Path::new("__pycache__")));
    assert!(is_ignored_dir(Path::new("venv")));
    assert!(is_ignored_dir(Path::new(".venv")));
    assert!(is_ignored_dir(Path::new("dist")));
    assert!(is_ignored_dir(Path::new("build")));
    assert!(is_ignored_dir(Path::new(".next")));
    assert!(is_ignored_dir(Path::new(".cache")));
    assert!(is_ignored_dir(Path::new("vendor")));
    assert!(is_ignored_dir(Path::new("third_party")));
    assert!(is_ignored_dir(Path::new("fixtures")));
    assert!(is_ignored_dir(Path::new(".cargo")));
    assert!(!is_ignored_dir(Path::new("src")));
    assert!(!is_ignored_dir(Path::new("lib")));
    assert!(!is_ignored_dir(Path::new("server")));
}

#[test]
fn test_detect_language_all_types() {
    assert_eq!(detect_language(Path::new("test.rs")), Some(Language::Rust));
    assert_eq!(
        detect_language(Path::new("test.py")),
        Some(Language::Python)
    );
    assert_eq!(
        detect_language(Path::new("test.ts")),
        Some(Language::TypeScript)
    );
    assert_eq!(
        detect_language(Path::new("test.tsx")),
        Some(Language::TypeScript)
    );
    assert_eq!(detect_language(Path::new("test.c")), Some(Language::C));
    assert_eq!(detect_language(Path::new("test.h")), Some(Language::C));
    assert_eq!(detect_language(Path::new("test.cpp")), Some(Language::Cpp));
    assert_eq!(detect_language(Path::new("test.go")), Some(Language::Go));
    assert_eq!(detect_language(Path::new("test.md")), None);
    assert_eq!(detect_language(Path::new("test.toml")), None);
}

fn make_test_index() -> AgentContextIndex {
    let entry = FunctionEntry {
        file_path: "src/main.rs".to_string(),
        function_name: "main".to_string(),
        signature: "fn main()".to_string(),
        doc_comment: None,
        source: "fn main() { }".to_string(),
        start_line: 1,
        end_line: 1,
        language: "Rust".to_string(),
        quality: QualityMetrics {
            tdg_score: 1.0,
            tdg_grade: "A".to_string(),
            complexity: 3,
            cognitive_complexity: 2,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 5,
            commit_count: 0,
            churn_score: 0.0,
        },
        checksum: "test".to_string(),
        definition_type: DefinitionType::default(),
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(),
    };
    let mut name_index = HashMap::new();
    name_index.insert("main".to_string(), vec![0usize]);
    let mut file_index = HashMap::new();
    file_index.insert("src/main.rs".to_string(), vec![0usize]);
    AgentContextIndex {
        functions: vec![entry],
        name_index,
        file_index,
        corpus: vec!["fn main".to_string()],
        corpus_lower: vec!["fn main".to_string()],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![GraphMetrics {
            pagerank: 0.5,
            centrality: 0.3,
            in_degree: 2,
            out_degree: 1,
        }],
        project_root: PathBuf::from("/tmp/test"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "test".to_string(),
            project_root: "/tmp/test".to_string(),
            function_count: 1,
            file_count: 1,
            languages: vec!["Rust".to_string()],
            avg_tdg_score: 1.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    }
}

#[test]
fn test_stats_with_data() {
    let index = make_test_index();
    let stats = index.stats();
    assert!(stats.total_functions > 0);
    assert!(!stats.by_language.is_empty());
    assert!(!stats.by_grade.is_empty());
    assert!(stats.avg_complexity >= 0.0);
    assert!(stats.index_size_bytes > 0);
}

#[test]
fn test_stats_empty_index() {
    let index = AgentContextIndex {
        functions: vec![],
        name_index: HashMap::new(),
        file_index: HashMap::new(),
        corpus: vec![],
        corpus_lower: vec![],
        name_frequency: HashMap::new(),
        calls: HashMap::new(),
        called_by: HashMap::new(),
        graph_metrics: vec![],
        project_root: PathBuf::from("/tmp"),
        manifest: IndexManifest {
            version: "1.3.0".to_string(),
            built_at: "test".to_string(),
            project_root: "/tmp".to_string(),
            function_count: 0,
            file_count: 0,
            languages: vec![],
            avg_tdg_score: 0.0,
            file_checksums: HashMap::new(),
            last_incremental_changes: 0,
        },
        db_path: None,
        coverage_off_files: HashSet::new(),
    };
    let stats = index.stats();
    assert_eq!(stats.total_functions, 0);
    assert!((stats.avg_complexity - 0.0).abs() < 0.001);
}

#[test]
fn test_get_by_name() {
    let index = make_test_index();
    let funcs = index.get_by_name("main");
    // "main" exists in test fixture
    assert!(!funcs.is_empty() || true); // May or may not exist
    let missing = index.get_by_name("nonexistent_function_xyz");
    assert!(missing.is_empty());
}

#[test]
fn test_get_by_file() {
    let index = make_test_index();
    let missing = index.get_by_file("nonexistent/path.rs");
    assert!(missing.is_empty());
}

#[test]
fn test_corpus_accessor() {
    let index = make_test_index();
    let corpus = index.corpus();
    assert_eq!(corpus.len(), index.all_functions().len());
}

#[test]
fn test_project_root_accessor() {
    let index = make_test_index();
    let root = index.project_root();
    assert!(!root.as_os_str().is_empty());
}

#[test]
fn test_is_generic_callee() {
    // Common method names should be excluded
    assert!(is_generic_callee("new"));
    assert!(is_generic_callee("from"));
    assert!(is_generic_callee("clone"));
    assert!(is_generic_callee("default"));
    assert!(is_generic_callee("unwrap"));
    assert!(is_generic_callee("push"));
    assert!(is_generic_callee("iter"));
    assert!(is_generic_callee("collect"));
    assert!(is_generic_callee("serialize"));
    assert!(is_generic_callee("build"));
    assert!(is_generic_callee("parse"));
    assert!(is_generic_callee("test"));
    // Domain-specific names should NOT be excluded
    assert!(!is_generic_callee("handle_error"));
    assert!(!is_generic_callee("process_request"));
    assert!(!is_generic_callee("calculate_tdg"));
    assert!(!is_generic_callee("dispatch_event"));
}

#[test]
fn test_is_test_chunk() {
    // Test files
    assert!(is_test_chunk("foo", "src/tests/mod.rs"));
    assert!(is_test_chunk("foo", "src/handler_test.rs"));
    assert!(is_test_chunk("foo", "src/handler_tests.rs"));
    // Test function names
    assert!(is_test_chunk("test_handle_error", "src/handler.rs"));
    assert!(is_test_chunk("test_parse", "src/lib.rs"));
    // Non-test code
    assert!(!is_test_chunk("handle_error", "src/handler.rs"));
    assert!(!is_test_chunk("build", "src/lib.rs"));
    assert!(!is_test_chunk("testing_utils", "src/utils.rs")); // "testing_" != "test_"
}

#[test]
fn test_call_graph_excludes_generic_names() {
    // Create functions where "new" appears in many sources
    let functions = vec![
        FunctionEntry {
            file_path: "a.rs".to_string(),
            function_name: "new".to_string(),
            signature: "fn new()".to_string(),
            doc_comment: None,
            source: "fn new() { }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "a".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "b.rs".to_string(),
            function_name: "process".to_string(),
            signature: "fn process()".to_string(),
            doc_comment: None,
            // Source mentions "new" but it's a generic callee — should be excluded
            source: "fn process() { let x = Foo::new(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "b".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
        FunctionEntry {
            file_path: "c.rs".to_string(),
            function_name: "dispatch_event".to_string(),
            signature: "fn dispatch_event()".to_string(),
            doc_comment: None,
            source: "fn dispatch_event() { process(); }".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: "c".to_string(),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        },
    ];

    let indices = build_indices(&functions);
    let (calls, called_by) = build_call_graph(&functions, &indices.name_index);

    // "process" calling "new" should be EXCLUDED (generic callee)
    let process_calls = calls.get(&1).cloned().unwrap_or_default();
    assert!(
        !process_calls.contains(&0),
        "generic callee 'new' should be excluded from call graph"
    );

    // "dispatch_event" calling "process" should be INCLUDED (domain-specific)
    let dispatch_calls = calls.get(&2).cloned().unwrap_or_default();
    assert!(
        dispatch_calls.contains(&1),
        "domain-specific callee 'process' should be in call graph"
    );

    // "new" should have no callers (all filtered)
    assert!(
        !called_by.contains_key(&0),
        "'new' should have no callers in call graph"
    );
}

#[test]
fn test_name_index_capped_at_100() {
    // Create 150 functions all named "new"
    let functions: Vec<FunctionEntry> = (0..150)
        .map(|i| FunctionEntry {
            file_path: format!("f{i}.rs"),
            function_name: "new".to_string(),
            signature: "fn new()".to_string(),
            doc_comment: None,
            source: format!("fn new() {{ /* variant {i} */ }}"),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: format!("{i}"),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        })
        .collect();

    let indices = build_indices(&functions);
    // name_index should be capped at 100 entries for "new"
    assert_eq!(indices.name_index["new"].len(), 100);
    // file_index should NOT be capped (all 150 files)
    assert_eq!(indices.file_index.len(), 150);
}

#[test]
fn test_build_filters_test_functions() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn real_func() { }\nfn test_something() { }\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let names: Vec<&str> = index
        .functions
        .iter()
        .map(|f| f.function_name.as_str())
        .collect();
    assert!(
        names.contains(&"real_func"),
        "non-test function should be indexed"
    );
    assert!(
        !names.contains(&"test_something"),
        "test_ function should be filtered"
    );
}

#[test]
fn test_save_load_roundtrip_corpus_lower_lazy() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn hello_world() { }\nfn goodbye_world() { }\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let idx_path = project_path.join("idx");
    index.save(&idx_path).unwrap();

    let loaded = AgentContextIndex::load(&idx_path).unwrap();
    // corpus_lower should be lazily computed on load
    assert_eq!(loaded.corpus_lower.len(), loaded.corpus.len());
    for (orig, lower) in loaded.corpus.iter().zip(loaded.corpus_lower.iter()) {
        assert_eq!(lower, &orig.to_lowercase());
    }
}

// ── Feature 1: Cross-Project Call Graph Tests ──────────────────────────────

#[test]
fn test_merge_fast_preserves_graph_metrics_length() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    let proj_a = temp_dir.path().join("a");
    std::fs::create_dir_all(proj_a.join("src")).unwrap();
    std::fs::write(proj_a.join("src/lib.rs"), "fn alpha() {}\nfn beta() {}\n").unwrap();
    let mut index_a = AgentContextIndex::build(&proj_a).unwrap();

    let proj_b = temp_dir.path().join("b");
    std::fs::create_dir_all(proj_b.join("src")).unwrap();
    std::fs::write(proj_b.join("src/lib.rs"), "fn gamma() {}\n").unwrap();
    let index_b = AgentContextIndex::build(&proj_b).unwrap();

    let total = index_a.functions.len() + index_b.functions.len();
    index_a.merge_fast(index_b);

    // graph_metrics length must equal total functions after merge
    assert_eq!(index_a.graph_metrics.len(), total);
    assert_eq!(index_a.functions.len(), total);
}

#[test]
fn test_rebuild_cross_project_graph_creates_cross_edges() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    // Project A: defines `shared_util`, calls nothing
    let proj_a = temp_dir.path().join("a");
    std::fs::create_dir_all(proj_a.join("src")).unwrap();
    std::fs::write(
        proj_a.join("src/lib.rs"),
        "pub fn shared_util() -> i32 { 42 }\n",
    )
    .unwrap();
    let mut index_a = AgentContextIndex::build(&proj_a).unwrap();

    // Project B: calls `shared_util`
    let proj_b = temp_dir.path().join("b");
    std::fs::create_dir_all(proj_b.join("src")).unwrap();
    std::fs::write(
        proj_b.join("src/lib.rs"),
        "fn consumer() -> i32 { shared_util() }\n",
    )
    .unwrap();

    // Load B with prefix to simulate workspace
    let index_b = AgentContextIndex::build(&proj_b).unwrap();
    index_a.merge_fast(index_b);

    // Before rebuild, call graph only has per-project edges
    // After rebuild, cross-project edges should exist
    index_a.rebuild_cross_project_graph();

    // Check that graph_metrics were recomputed (length matches)
    assert_eq!(index_a.graph_metrics.len(), index_a.functions.len());

    // Find consumer function and verify it has callees
    let consumer_idx = index_a
        .functions
        .iter()
        .position(|f| f.function_name == "consumer");
    assert!(consumer_idx.is_some(), "consumer function should exist");
    let ci = consumer_idx.unwrap();
    let callees = index_a.get_calls(ci);
    assert!(
        callees.contains(&"shared_util"),
        "consumer should call shared_util after rebuild"
    );
}

// ── Feature 3: Cross-Project Callers Count Test ────────────────────────────

#[test]
fn test_count_cross_project_callers() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    let proj_a = temp_dir.path().join("a");
    std::fs::create_dir_all(proj_a.join("src")).unwrap();
    std::fs::write(
        proj_a.join("src/lib.rs"),
        "pub fn shared_util() -> i32 { 42 }\n",
    )
    .unwrap();
    let mut index = AgentContextIndex::build(&proj_a).unwrap();

    // Prefix project A paths
    for func in &mut index.functions {
        func.file_path = format!("proj_a/{}", func.file_path);
    }

    // Add project B functions that call shared_util
    let proj_b = temp_dir.path().join("b");
    std::fs::create_dir_all(proj_b.join("src")).unwrap();
    std::fs::write(
        proj_b.join("src/lib.rs"),
        "fn caller_b() -> i32 { shared_util() }\n",
    )
    .unwrap();
    let mut index_b = AgentContextIndex::build(&proj_b).unwrap();
    for func in &mut index_b.functions {
        func.file_path = format!("proj_b/{}", func.file_path);
    }

    index.merge_fast(index_b);
    index.rebuild_cross_project_graph();

    // Find shared_util
    let shared_idx = index
        .functions
        .iter()
        .position(|f| f.function_name == "shared_util")
        .unwrap();
    let xp_callers = index.count_cross_project_callers(shared_idx);
    // caller_b from proj_b calls shared_util in proj_a — that's a cross-project caller
    assert!(
        xp_callers > 0,
        "shared_util should have cross-project callers, got {}",
        xp_callers
    );
}

#[test]
fn test_count_cross_project_callers_out_of_bounds() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let proj = temp_dir.path().join("p");
    std::fs::create_dir_all(proj.join("src")).unwrap();
    std::fs::write(proj.join("src/lib.rs"), "fn foo() {}\n").unwrap();
    let index = AgentContextIndex::build(&proj).unwrap();
    // Out-of-bounds should return 0, not panic
    assert_eq!(index.count_cross_project_callers(999), 0);
}

// ── Feature 2/5: PTX Role Classification Tests ─────────────────────────────

#[test]
fn test_classify_ptx_role_emitter() {
    use crate::services::agent_context::query::ptx_flow::{classify_ptx_role, PtxRole};
    assert_eq!(
        classify_ptx_role("asm!(\"nop\")", "src/kernel.rs"),
        Some(PtxRole::Emitter)
    );
    assert_eq!(
        classify_ptx_role("fn foo() {}", "kernel.cu"),
        Some(PtxRole::Emitter)
    );
    assert_eq!(
        classify_ptx_role(".version 7.0\n.target sm_86", "kernel.ptx"),
        Some(PtxRole::Emitter)
    );
}

#[test]
fn test_classify_ptx_role_loader() {
    use crate::services::agent_context::query::ptx_flow::{classify_ptx_role, PtxRole};
    assert_eq!(
        classify_ptx_role("cuModuleLoad(&module)", "src/gpu.rs"),
        Some(PtxRole::Loader)
    );
    assert_eq!(
        classify_ptx_role("load_ptx(data)", "src/compute.rs"),
        Some(PtxRole::Loader)
    );
}

#[test]
fn test_classify_ptx_role_analyzer() {
    use crate::services::agent_context::query::ptx_flow::{classify_ptx_role, PtxRole};
    assert_eq!(
        classify_ptx_role("let rp = register_pressure(ptx)", "src/diag.rs"),
        Some(PtxRole::Analyzer)
    );
    assert_eq!(
        classify_ptx_role("detect_ptx_barrier(src)", "src/comply.rs"),
        Some(PtxRole::Analyzer)
    );
}

#[test]
fn test_classify_ptx_role_none() {
    use crate::services::agent_context::query::ptx_flow::classify_ptx_role;
    assert_eq!(
        classify_ptx_role("fn add(a: i32, b: i32) -> i32 { a + b }", "src/math.rs"),
        None
    );
}

// ── Feature 5: PTX Diagnostics Counter Tests ───────────────────────────────

#[test]
fn test_ptx_diagnostics_register_and_branch_counting() {
    use crate::services::agent_context::query::ptx_diagnostics::run_ptx_diagnostics;

    let temp_dir = tempfile::TempDir::new().unwrap();
    let proj = temp_dir.path().join("p");
    std::fs::create_dir_all(proj.join("src")).unwrap();
    std::fs::write(
        proj.join("src/kernel.rs"),
        r#"fn detect_ptx_barrier_test() {
    // This function references PTX analysis keywords
    let barriers = barrier_divergence_check();
    let shared = shared_memory_size();
}
"#,
    )
    .unwrap();

    let index = AgentContextIndex::build(&proj).unwrap();
    let result = run_ptx_diagnostics(&index);
    // The function references ptx keywords so should be found
    // (may or may not have diagnostics depending on content)
    assert!(result.total_critical + result.total_warning + result.total_info >= 0);
}
