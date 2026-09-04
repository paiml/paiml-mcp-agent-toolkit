// Persistence tests: save/load roundtrip, SQLite, incremental builds,
// workspace siblings, checksums, corpus_lower lazy loading

/// The SQLite schema version the code under test writes.
///
/// Read from the constant rather than hard-coded: these assertions used to
/// pin the literal "2.0.0", so a deliberate schema bump (v3.30.0, when
/// `tdg_score` changed scale) failed them for the wrong reason.
fn sqlite_schema_version() -> &'static str {
    super::sqlite_backend::SCHEMA_VERSION
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
    // load() prefers SQLite (SCHEMA_VERSION) over blob (v1.4.0) when both exist
    assert!(
        loaded.manifest.version == sqlite_schema_version()
            || loaded.manifest.version == "1.4.0",
        "expected {} or v1.4.0, got {}",
        sqlite_schema_version(),
        loaded.manifest.version,
    );
    assert_eq!(loaded.functions.len(), index.functions.len());
    // SQLite path skips corpus (FTS5 handles search); blob path has corpus
    if loaded.manifest.version == sqlite_schema_version() {
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
    assert_eq!(loaded.manifest.version, sqlite_schema_version());
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

/// R30: an index written under the OLD TDG scale must be rejected and rebuilt,
/// never reinterpreted.
///
/// The scales share a column type, so a pre-v3.30.0 database passes every
/// structural check while holding 0-10 lower-is-better debt numbers. Reading
/// those on today's 0-100 higher-is-better scale silently turns the BEST legacy
/// score (0.04) into an F.
#[test]
fn test_load_rejects_index_written_under_old_tdg_scale() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn omega() {}\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let index_path = project_path.join("idx");
    index.save(&index_path).unwrap();
    let db_path = index_path.with_extension("db");

    // Sanity: as saved, it loads.
    assert!(AgentContextIndex::load(&index_path).is_ok());

    // Rewrite the database exactly as a pre-v3.30.0 build left it: legacy
    // 0-10 debt scores, five-letter grades, and NO tdg_scale marker.
    {
        let conn = super::sqlite_backend::open_db(&db_path).unwrap();
        conn.execute("DELETE FROM metadata WHERE key = 'tdg_scale'", [])
            .unwrap();
        conn.execute(
            "UPDATE functions SET tdg_score = 0.12, tdg_grade = 'A'",
            [],
        )
        .unwrap();
    }

    // Blob path must reject it too, so remove nothing and assert the whole
    // load() fails rather than quietly returning 0.12-as-percent scores.
    let reloaded = AgentContextIndex::load(&index_path);
    assert!(
        reloaded.is_err(),
        "a stale-scale index must be rejected, got {} functions with scores {:?}",
        reloaded.as_ref().map(|i| i.functions.len()).unwrap_or(0),
        reloaded
            .as_ref()
            .map(|i| i
                .functions
                .iter()
                .map(|f| f.quality.tdg_score)
                .collect::<Vec<_>>())
            .unwrap_or_default()
    );

    // And it must have removed the stale database so the next build regenerates
    // it, rather than leaving a file that fails forever.
    assert!(
        !db_path.exists(),
        "stale-scale database should have been discarded"
    );
}

/// R30 (residual): rejecting a stale-scale index must DISCARD it, not merely
/// report it — otherwise recovery depends on which caller you are.
///
/// The SQLite branch used to delete only its own `.db` and leave `manifest.json`
/// behind. `pmat query` catches the error and rebuilds, so it self-healed; MCP's
/// `IndexManager` propagates the error, so `pmat_query_code` / `pmat_index_stats`
/// answered `-32603 … rebuild required` on every default call, forever. With the
/// whole stale index gone, `index_path.exists()` is false and every caller takes
/// its ordinary "no index yet" build path.
#[test]
fn test_stale_scale_rejection_discards_the_whole_index() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/lib.rs"), "fn zeta() {}\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let index_path = project_path.join("idx");
    index.save(&index_path).unwrap();
    let db_path = index_path.with_extension("db");

    // Downgrade both artifacts exactly as a pre-v3.30.0 build left them.
    {
        let conn = super::sqlite_backend::open_db(&db_path).unwrap();
        conn.execute("DELETE FROM metadata WHERE key = 'tdg_scale'", [])
            .unwrap();
        conn.execute("UPDATE functions SET tdg_score = 0.12, tdg_grade = 'A'", [])
            .unwrap();
    }
    let manifest_path = index_path.join("manifest.json");
    let raw = std::fs::read_to_string(&manifest_path).unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(&raw).unwrap();
    manifest.as_object_mut().unwrap().remove("tdg_scale");
    std::fs::write(&manifest_path, manifest.to_string()).unwrap();

    let first = AgentContextIndex::load(&index_path);
    assert!(first.is_err(), "a stale-scale index must be rejected");
    assert!(!db_path.exists(), "stale .db must be discarded");
    assert!(
        !index_path.exists(),
        "stale index directory must be discarded so the next load rebuilds; \
         leaving manifest.json behind is what made MCP fail forever"
    );

    // The second load must NOT repeat "rebuild required": there is nothing left
    // to reject, so callers that only check `exists()` now build from scratch.
    let err = match AgentContextIndex::load(&index_path) {
        Err(e) => e,
        Ok(idx) => panic!(
            "no index on disk, yet load returned {} functions",
            idx.functions.len()
        ),
    };
    assert!(
        !err.contains("rebuild required"),
        "stale-scale error must not survive the discard, got: {err}"
    );
}

/// R30 (residual): the blob branch discards too. A pre-v3.30.0 index whose
/// `.db` was already removed left `manifest.json` in place forever.
#[test]
fn test_stale_manifest_rejection_discards_index_dir() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let index_path = temp_dir.path().join("idx");
    std::fs::create_dir_all(&index_path).unwrap();
    std::fs::write(
        index_path.join("manifest.json"),
        r#"{"version":"1.4.0","built_at":"x","project_root":".","function_count":0,
            "file_count":0,"languages":[],"avg_tdg_score":0.12}"#,
    )
    .unwrap();

    let err = match AgentContextIndex::load(&index_path) {
        Err(e) => e,
        Ok(idx) => panic!(
            "unmarked manifest must be rejected, loaded {} functions",
            idx.functions.len()
        ),
    };
    assert!(err.contains("rebuild required"), "got: {err}");
    assert!(
        !index_path.exists(),
        "stale index directory must be discarded"
    );
}

/// R30: a blob manifest with no `tdg_scale` key is stale by definition.
#[test]
fn test_load_from_blob_rejects_unmarked_manifest() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let index_path = temp_dir.path().join("idx");
    std::fs::create_dir_all(&index_path).unwrap();
    // A pre-v3.30.0 manifest: every key the old format had, no scale marker.
    std::fs::write(
        index_path.join("manifest.json"),
        r#"{"version":"1.4.0","built_at":"x","project_root":".","function_count":0,
            "file_count":0,"languages":[],"avg_tdg_score":0.12}"#,
    )
    .unwrap();
    std::fs::write(index_path.join("functions.lz4"), b"unused").unwrap();

    let err = match AgentContextIndex::load(&index_path) {
        Err(e) => e,
        Ok(idx) => panic!(
            "unmarked manifest must be rejected, loaded {} functions",
            idx.functions.len()
        ),
    };
    assert!(
        err.contains("TDG scale"),
        "error should name the scale mismatch, got: {err}"
    );
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
fn test_incremental_build_propagates_db_path() {
    // Regression: build_incremental used to reset db_path to None, so
    // deferred-source backfill was skipped and every query after an
    // incremental update returned empty source (Bug B of the source wipe).
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src/lib.rs"),
        "fn alpha() { let a = 1; }\n",
    )
    .unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let index_path = project_path.join("idx");
    index.save(&index_path).unwrap();

    let loaded = AgentContextIndex::load(&index_path).unwrap();
    let db_path = loaded.db_path.clone().expect("SQLite load sets db_path");

    let updated = AgentContextIndex::build_incremental(project_path, &loaded).unwrap();
    assert_eq!(
        updated.db_path.as_deref(),
        Some(db_path.as_path()),
        "incremental index must keep the db_path it loaded from"
    );

    // Deferred source must remain retrievable through the incremental index
    let (file_path, start_line) = {
        let entry = &updated.functions[0];
        (entry.file_path.clone(), entry.start_line)
    };
    let src = updated.load_source_for(&file_path, start_line);
    assert!(
        src.contains("alpha"),
        "expected source via DB backfill, got {src:?}"
    );
}

#[test]
fn test_incremental_save_preserves_source_for_reused_entries() {
    // Regression (source wipe, Bug A): lightweight SQLite loads defer source
    // (entries carry ""). build_incremental clones those entries for
    // checksum-reused files, and saving used to rewrite the full DB with
    // source='' for every reused row — repeated incremental saves converged
    // the index to all-empty source, breaking --include-source,
    // pmat_query_code, and pmat_get_function.
    let temp_dir = tempfile::TempDir::new().unwrap();
    let project_path = temp_dir.path();

    std::fs::create_dir_all(project_path.join("src")).unwrap();
    std::fs::write(project_path.join("src/a.rs"), "fn alpha() { let a = 1; }\n").unwrap();
    std::fs::write(project_path.join("src/b.rs"), "fn beta() { let b = 2; }\n").unwrap();
    std::fs::write(project_path.join("src/c.rs"), "fn gamma() { let c = 3; }\n").unwrap();

    let index = AgentContextIndex::build(project_path).unwrap();
    let index_path = project_path.join("idx");
    index.save(&index_path).unwrap();

    // Lightweight SQLite load: source is deferred (empty) for every entry
    let loaded = AgentContextIndex::load(&index_path).unwrap();
    assert!(loaded.functions.iter().all(|f| f.source.is_empty()));

    // Change one of three files; a.rs and b.rs entries are checksum-reused
    std::fs::write(
        project_path.join("src/c.rs"),
        "fn gamma() { let c = 30; }\nfn delta() { let d = 4; }\n",
    )
    .unwrap();

    let mut updated = AgentContextIndex::build_incremental(project_path, &loaded).unwrap();
    assert_eq!(updated.functions.len(), 4);

    // The save path backfills deferred source before rewriting the DB
    updated.load_all_source();
    assert!(
        updated.functions.iter().all(|f| !f.source.is_empty()),
        "backfill must restore source for reused entries"
    );
    updated.save(&index_path).unwrap();

    // Reload raw rows: EVERY row must still have non-empty source
    let conn = super::sqlite_backend::open_db(&index_path.with_extension("db")).unwrap();
    let rows = super::sqlite_backend::load_functions(&conn).unwrap();
    assert_eq!(rows.len(), 4);
    for row in &rows {
        assert!(
            !row.source.is_empty(),
            "row '{}' persisted with empty source after incremental save",
            row.function_name
        );
    }
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
        r#"siblings = ["../project_b"]"#,
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

// ── CRUX-07: the index is a faithful view of the tree ──────────────────────

/// A manifest written before per-file stats existed holds bare checksum
/// strings. It must still load — and must authorise no skip.
#[test]
fn legacy_bare_checksum_manifest_loads_with_unknown_stats() {
    let record: FileRecord = serde_json::from_str("\"deadbeef\"").expect("a bare checksum string must deserialize");
    assert_eq!(record.checksum, "deadbeef");
    assert!(
        !record.has_stats(),
        "a legacy record carries no stat evidence"
    );
    assert!(
        !super::build::stats_agree(&record, (10, 5), 1_000),
        "unknown stats must never authorise skipping the read"
    );
}

/// The fast path may only skip when length AND ctime both say the file has not
/// been written since the build.
#[test]
fn stats_agree_only_when_length_and_ctime_both_predate_the_build() {
    let record = FileRecord {
        checksum: "c".to_string(),
        len: 32,
        ctime: 500,
    };
    assert!(super::build::stats_agree(&record, (32, 500), 1_000), "quiescent file");
    assert!(
        !super::build::stats_agree(&record, (31, 500), 1_000),
        "a different length is a rewrite, whatever mtime claims"
    );
    assert!(
        !super::build::stats_agree(&record, (32, 1_500), 1_000),
        "a ctime after the build is a rewrite, whatever mtime claims"
    );
    assert!(
        !super::build::stats_agree(&record, (32, 1_000), 1_000),
        "ctime == built_at is ambiguous, so it is not skippable"
    );
}

/// A rewrite that backdates mtime behind `built_at` must still be re-indexed:
/// it changes length and ctime, and either alone disqualifies the fast path.
#[test]
fn backdated_rewrite_is_reindexed_not_served_from_the_old_checksum() {
    let temp_dir = tempfile::TempDir::new().expect("a temp dir must be creatable");
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).expect("creating src/ must succeed");
    let file = project_path.join("src/lib.rs");
    std::fs::write(&file, "pub fn alpha_only() -> u32 { 1 }\n").expect("writing the fixture must succeed");

    let index = AgentContextIndex::build(project_path).expect("building the index must succeed");
    let built_at = index.manifest.built_at.clone();

    std::fs::write(&file, "pub fn beta_only() -> u32 { 2 }\n").expect("rewriting the fixture must succeed");
    // Backdate mtime to one second BEFORE the index was built.
    let secs = chrono::DateTime::parse_from_rfc3339(&built_at)
        .expect("built_at is RFC 3339")
        .timestamp() as u64;
    let backdated = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs - 1);
    let handle = std::fs::File::options().write(true).open(&file).expect("the fixture must be openable");
    handle
        .set_times(
            std::fs::FileTimes::new()
                .set_accessed(backdated)
                .set_modified(backdated),
        )
        .expect("backdating mtime must succeed");
    drop(handle);

    let updated = AgentContextIndex::build_incremental(project_path, &index).expect("the incremental build must succeed");
    let names: Vec<&str> = updated
        .functions
        .iter()
        .map(|f| f.function_name.as_str())
        .collect();
    assert!(
        names.contains(&"beta_only"),
        "the rewritten content must be indexed, got {names:?}"
    );
    assert!(
        !names.contains(&"alpha_only"),
        "the deleted function must not be served, got {names:?}"
    );
}

/// A manifest torn mid-write is not a manifest: the pair is rejected so the
/// caller rebuilds, rather than serving rows from the database beside it.
#[test]
fn a_torn_manifest_is_rejected_and_named() {
    let temp_dir = tempfile::TempDir::new().expect("a temp dir must be creatable");
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).expect("creating src/ must succeed");
    std::fs::write(project_path.join("src/lib.rs"), "pub fn delta() -> u32 { 4 }\n")
        .expect("writing the fixture must succeed");

    let index = AgentContextIndex::build(project_path).expect("building the index must succeed");
    let idx_path = project_path.join("context.idx");
    index.save(&idx_path).expect("saving to a writable dir must succeed");
    assert!(AgentContextIndex::load(&idx_path).is_ok(), "a clean pair loads");

    let manifest_file = idx_path.join("manifest.json");
    let whole = std::fs::read_to_string(&manifest_file).expect("the manifest must be readable");
    std::fs::write(&manifest_file, &whole[..whole.len() / 2]).expect("tearing the manifest must succeed");

    let err = match AgentContextIndex::load(&idx_path) {
        Ok(_) => unreachable!("a torn manifest must not load"),
        Err(e) => e.to_lowercase(),
    };
    assert!(
        err.contains("torn") || err.contains("corrupt"),
        "the error must name the problem, got: {err}"
    );
}

/// The manifest is replaced by rename, so a directory that cannot be written
/// fails the save instead of truncating the manifest that is already there.
#[test]
#[cfg(unix)]
fn save_into_a_read_only_index_dir_fails_instead_of_truncating() {
    use std::os::unix::fs::PermissionsExt;
    let temp_dir = tempfile::TempDir::new().expect("a temp dir must be creatable");
    let project_path = temp_dir.path();
    std::fs::create_dir_all(project_path.join("src")).expect("creating src/ must succeed");
    std::fs::write(project_path.join("src/lib.rs"), "pub fn eps() -> u32 { 5 }\n")
        .expect("writing the fixture must succeed");

    let index = AgentContextIndex::build(project_path).expect("building the index must succeed");
    let idx_path = project_path.join("context.idx");
    index.save(&idx_path).expect("saving to a writable dir must succeed");
    let before = std::fs::read_to_string(idx_path.join("manifest.json")).expect("the manifest must be readable");

    std::fs::set_permissions(&idx_path, std::fs::Permissions::from_mode(0o555))
        .expect("chmod must succeed");
    let result = index.save(&idx_path);
    std::fs::set_permissions(&idx_path, std::fs::Permissions::from_mode(0o755))
        .expect("chmod must succeed");

    assert!(result.is_err(), "a read-only index directory must fail the save");
    assert_eq!(
        std::fs::read_to_string(idx_path.join("manifest.json")).expect("the manifest must be readable"),
        before,
        "the manifest already on disk must survive a failed save intact"
    );
}
