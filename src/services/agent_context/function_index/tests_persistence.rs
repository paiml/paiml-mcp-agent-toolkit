// Persistence tests: save/load roundtrip, SQLite, incremental builds,
// workspace siblings, checksums, corpus_lower lazy loading

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
        r#"siblings = ["../project_b"]"#.to_string(),
    )
    .unwrap();

    let siblings = AgentContextIndex::discover_sibling_indexes(&project_a);
    assert_eq!(siblings.len(), 1);
    assert_eq!(siblings[0].1, "project_b");
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
