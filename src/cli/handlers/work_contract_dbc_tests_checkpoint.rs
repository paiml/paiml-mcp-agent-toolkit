// Tests for Design by Contract types - Part 3: v5 Contracts + Checkpoints
// Spec: docs/specifications/dbc.md

// === WorkContract v5.0 ===

#[test]
fn test_work_contract_new_is_v4() {
    let contract = WorkContract::new("TEST-1".to_string(), "abc123".to_string());
    assert_eq!(contract.version, "4.0");
    assert!(!contract.is_dbc());
    assert!(contract.require.is_empty());
    assert!(contract.ensure.is_empty());
    assert!(contract.invariant.is_empty());
}

#[test]
fn test_work_contract_with_dbc_is_v5() {
    let dir = tempfile::tempdir().unwrap();
    // Create a minimal git repo to trigger Universal profile
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let contract =
        WorkContract::with_dbc("TEST-1".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();

    assert_eq!(contract.version, "5.0");
    assert!(contract.is_dbc());
    assert_eq!(
        contract.profile,
        Some(ContractProfile::Universal)
    );
    assert_eq!(contract.require.len(), 2);
    assert_eq!(contract.invariant.len(), 2);
    assert_eq!(contract.ensure.len(), 2);
    assert_eq!(contract.triad_claim_count(), 6);
}

#[test]
fn test_work_contract_with_dbc_rust_profile() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    let result =
        WorkContract::with_dbc("TEST-2".to_string(), "abc123".to_string(), dir.path(), &[], 1);

    match result {
        Ok(contract) => {
            assert_eq!(
                contract.profile,
                Some(ContractProfile::Rust)
            );
            assert_eq!(contract.triad_claim_count(), 14);
        }
        Err(e) => {
            // Rust profile may fail if toolchain not available or tempdir race
            let msg = e.to_string();
            assert!(
                msg.contains("missing tool") || msg.contains("Toolchain") || msg.contains("No such file"),
                "Unexpected error: {}", msg
            );
        }
    }
}

#[test]
fn test_work_contract_with_dbc_pmat_profile() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    std::fs::create_dir_all(dir.path().join(".pmat")).unwrap();
    std::fs::write(dir.path().join(".pmat").join("context.db"), "").unwrap();

    let result =
        WorkContract::with_dbc("TEST-3".to_string(), "abc123".to_string(), dir.path(), &[], 1);

    match result {
        Ok(contract) => {
            assert_eq!(
                contract.profile,
                Some(ContractProfile::Pmat)
            );
            assert_eq!(contract.triad_claim_count(), 25);
        }
        Err(e) => {
            // Pmat profile requires cargo-llvm-cov; skip if not installed
            let msg = e.to_string();
            assert!(
                msg.contains("missing tool") || msg.contains("cargo-llvm-cov"),
                "Unexpected error: {}", msg
            );
        }
    }
}

#[test]
fn test_work_contract_with_dbc_exclusions() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let without = vec!["ensure.git_sync".to_string()];
    let contract =
        WorkContract::with_dbc("TEST-4".to_string(), "abc123".to_string(), dir.path(), &without, 1)
            .unwrap();

    assert_eq!(contract.triad_claim_count(), 5); // 6 - 1 excluded
    assert_eq!(contract.excluded_claims.len(), 1);
    assert_eq!(contract.excluded_claims[0].id, "ensure.git_sync");

    // Contract quality should reflect exclusion
    let quality = contract.contract_quality.as_ref().unwrap();
    assert_eq!(quality.active_claims, 5);
    assert_eq!(quality.applicable_claims, 6);
    assert!(quality.score < 1.0);
}

#[test]
fn test_work_contract_with_dbc_config_override() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    // Override to Universal even though Cargo.toml exists
    let config_dir = dir.path().join(".pmat-work");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        "[dbc]\nprofile = \"universal\"\n",
    )
    .unwrap();

    let contract =
        WorkContract::with_dbc("TEST-5".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();

    assert_eq!(
        contract.profile,
        Some(ContractProfile::Universal)
    );
    assert_eq!(contract.triad_claim_count(), 6); // Universal, not Rust
}

// === v5.0 serialization round-trip ===

#[test]
fn test_work_contract_v5_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let contract =
        WorkContract::with_dbc("TEST-RT".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();

    // Save
    let saved_path = contract.save(dir.path()).unwrap();
    assert!(saved_path.exists());

    // Load
    let loaded = WorkContract::load(dir.path(), "TEST-RT").unwrap();
    assert_eq!(loaded.version, "5.0");
    assert!(loaded.is_dbc());
    assert_eq!(loaded.require.len(), 2);
    assert_eq!(loaded.ensure.len(), 2);
    assert_eq!(loaded.invariant.len(), 2);
    assert_eq!(loaded.profile, Some(ContractProfile::Universal));
}

// === v4.0 backward compat ===

#[test]
fn test_v4_contract_loads_with_empty_triad() {
    let dir = tempfile::tempdir().unwrap();

    // Create a v4.0 contract (no triad fields)
    let v4 = WorkContract::new("TEST-V4".to_string(), "abc123".to_string());
    v4.save(dir.path()).unwrap();

    // Load it back
    let loaded = WorkContract::load(dir.path(), "TEST-V4").unwrap();
    assert_eq!(loaded.version, "4.0");
    assert!(!loaded.is_dbc());
    assert!(loaded.require.is_empty());
    assert!(loaded.ensure.is_empty());
    assert!(loaded.invariant.is_empty());
    assert!(loaded.profile.is_none());
    assert_eq!(loaded.iteration, 1);
    assert_eq!(loaded.claims.len(), 22);
}

// === Profile detection ===

#[test]
fn test_profile_detect_universal() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    assert_eq!(ContractProfile::detect(dir.path()), ContractProfile::Universal);
}

#[test]
fn test_profile_detect_rust() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
    assert_eq!(ContractProfile::detect(dir.path()), ContractProfile::Rust);
}

#[test]
fn test_profile_detect_pmat() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
    std::fs::create_dir_all(dir.path().join(".pmat")).unwrap();
    std::fs::write(dir.path().join(".pmat").join("context.idx"), "").unwrap();
    assert_eq!(ContractProfile::detect(dir.path()), ContractProfile::Pmat);
}

#[test]
fn test_profile_detect_stack() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(
        dir.path().join(".dbc-stack.toml"),
        "[stack]\nname = \"test\"\n",
    )
    .unwrap();
    match ContractProfile::detect(dir.path()) {
        ContractProfile::Stack { manifest_path } => {
            assert!(manifest_path.ends_with(".dbc-stack.toml"));
        }
        other => panic!("Expected Stack, got {:?}", other),
    }
}

// === CheckpointRecord tests ===

#[test]
fn test_checkpoint_record_new_all_pass() {
    let results = vec![
        InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: true,
            explanation: "Lint clean".to_string(),
        },
        InvariantResult {
            clause_id: "invariant.complexity".to_string(),
            passed: true,
            explanation: "Max complexity: 18 (limit: 20)".to_string(),
        },
    ];

    let record = CheckpointRecord::new(
        "PMAT-500".to_string(),
        "abc123def".to_string(),
        1,
        results,
    );

    assert!(record.all_invariants_hold);
    assert_eq!(record.work_item_id, "PMAT-500");
    assert_eq!(record.git_sha, "abc123def");
    assert_eq!(record.iteration, 1);
    assert_eq!(record.invariant_results.len(), 2);
    assert!(!record.checkpoint_id.is_empty());
}

#[test]
fn test_checkpoint_record_new_with_failure() {
    let results = vec![
        InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: true,
            explanation: "Lint clean".to_string(),
        },
        InvariantResult {
            clause_id: "invariant.complexity".to_string(),
            passed: false,
            explanation: "Max complexity: 32 exceeds limit 20".to_string(),
        },
    ];

    let record = CheckpointRecord::new(
        "PMAT-500".to_string(),
        "def456".to_string(),
        1,
        results,
    );

    assert!(!record.all_invariants_hold);
    assert_eq!(record.invariant_results.len(), 2);
}

#[test]
fn test_checkpoint_record_empty_invariants() {
    let record = CheckpointRecord::new(
        "PMAT-500".to_string(),
        "abc123".to_string(),
        1,
        vec![],
    );

    assert!(record.all_invariants_hold); // vacuously true
    assert!(record.invariant_results.is_empty());
}

#[test]
fn test_checkpoint_record_save_and_load() {
    let dir = tempfile::tempdir().unwrap();

    let results = vec![
        InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: true,
            explanation: "Lint clean".to_string(),
        },
    ];

    let record = CheckpointRecord::new(
        "TEST-CK".to_string(),
        "abc123".to_string(),
        1,
        results,
    );

    // Save
    let path = record.save(dir.path()).unwrap();
    assert!(path.exists());
    assert!(path.to_string_lossy().contains("checkpoints"));

    // Load all
    let loaded = CheckpointRecord::load_all(dir.path(), "TEST-CK");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].work_item_id, "TEST-CK");
    assert_eq!(loaded[0].checkpoint_id, record.checkpoint_id);
    assert!(loaded[0].all_invariants_hold);
}

#[test]
fn test_checkpoint_record_load_multiple() {
    let dir = tempfile::tempdir().unwrap();

    // Save two checkpoints
    let r1 = CheckpointRecord::new(
        "TEST-MULTI".to_string(),
        "sha1".to_string(),
        1,
        vec![InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: true,
            explanation: "ok".to_string(),
        }],
    );
    r1.save(dir.path()).unwrap();

    let r2 = CheckpointRecord::new(
        "TEST-MULTI".to_string(),
        "sha2".to_string(),
        1,
        vec![InvariantResult {
            clause_id: "invariant.lint".to_string(),
            passed: false,
            explanation: "lint failed".to_string(),
        }],
    );
    r2.save(dir.path()).unwrap();

    let loaded = CheckpointRecord::load_all(dir.path(), "TEST-MULTI");
    assert_eq!(loaded.len(), 2);
    // Sorted by timestamp, first should pass, second should fail
    assert!(loaded[0].all_invariants_hold);
    assert!(!loaded[1].all_invariants_hold);
}

#[test]
fn test_checkpoint_record_load_nonexistent() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = CheckpointRecord::load_all(dir.path(), "NONEXISTENT");
    assert!(loaded.is_empty());
}

#[test]
fn test_checkpoint_serialization_round_trip() {
    let results = vec![
        InvariantResult {
            clause_id: "invariant.complexity".to_string(),
            passed: true,
            explanation: "Max complexity: 15".to_string(),
        },
    ];

    let record = CheckpointRecord::new(
        "PMAT-RT".to_string(),
        "deadbeef".to_string(),
        2,
        results,
    );

    let json = serde_json::to_string_pretty(&record).unwrap();
    let deserialized: CheckpointRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.checkpoint_id, record.checkpoint_id);
    assert_eq!(deserialized.work_item_id, "PMAT-RT");
    assert_eq!(deserialized.iteration, 2);
    assert_eq!(deserialized.invariant_results.len(), 1);
    assert!(deserialized.all_invariants_hold);
}

// === Subcontracting with iteration ===

#[test]
fn test_iteration_field_default() {
    let contract = WorkContract::new("TEST".to_string(), "abc".to_string());
    assert_eq!(contract.iteration, 1);
}

#[test]
fn test_iteration_persists_through_save_load() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let mut contract =
        WorkContract::with_dbc("TEST-IT".to_string(), "abc123".to_string(), dir.path(), &[], 1)
            .unwrap();
    // Simulate iteration increment
    contract.iteration = 2;
    contract.save(dir.path()).unwrap();

    let loaded = WorkContract::load(dir.path(), "TEST-IT").unwrap();
    assert_eq!(loaded.iteration, 2);
}
