#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use tempfile::TempDir;

fn create_test_repo() -> anyhow::Result<TempDir> {
    use std::process::Command;

    let temp = TempDir::new()?;

    // Initialize git repo
    let init_out = Command::new("git")
        .args(["init"])
        .current_dir(temp.path())
        .output()?;
    if !init_out.status.success() {
        anyhow::bail!(
            "git init failed: {}",
            String::from_utf8_lossy(&init_out.stderr)
        );
    }

    // Configure git user (required for commit)
    let _ = Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(temp.path())
        .output()?;
    let _ = Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp.path())
        .output()?;

    // Create a file and commit
    std::fs::write(temp.path().join("test.rs"), "fn main() {}")?;
    let add_out = Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()?;
    if !add_out.status.success() {
        anyhow::bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_out.stderr)
        );
    }

    let commit_out = Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(temp.path())
        .output()?;
    if !commit_out.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit_out.stderr)
        );
    }

    Ok(temp)
}

#[test]
fn test_cache_manager_init() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());

    manager.init().unwrap();

    assert!(temp.path().join(".pmat/hooks-cache").exists());
    assert!(temp.path().join(".pmat/hooks-cache/gates").exists());
    assert!(temp.path().join(".pmat/hooks-cache/files").exists());
    assert!(temp.path().join(".pmat/hooks-cache/metrics.json").exists());
}

#[test]
fn test_cache_miss_no_file() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    let result = manager.check().unwrap();

    match result {
        CacheCheckResult::Miss {
            reason: CacheMissReason::NoCacheFile,
        } => {}
        _ => panic!("Expected NoCacheFile miss"),
    }
}

#[test]
fn test_cache_hit_after_update() {
    let temp = create_test_repo().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Update cache
    let gates = std::collections::HashMap::new();
    manager.update(CacheResult::Pass, gates).unwrap();

    // Check should hit
    let result = manager.check().unwrap();

    match result {
        CacheCheckResult::Hit { result, .. } => {
            assert_eq!(result, CacheResult::Pass);
        }
        _ => panic!("Expected cache hit"),
    }
}

#[test]
fn test_cache_clear() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create some cache files
    std::fs::write(temp.path().join(".pmat/hooks-cache/tree-hash.json"), "{}").unwrap();

    manager.clear().unwrap();

    // Cache dir should be recreated empty
    assert!(temp.path().join(".pmat/hooks-cache").exists());
    assert!(!temp
        .path()
        .join(".pmat/hooks-cache/tree-hash.json")
        .exists());
}

#[test]
fn test_metrics_recording() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Record some runs
    manager.record_run(true, 5).unwrap(); // hit
    manager.record_run(true, 7).unwrap(); // hit
    manager.record_run(false, 1000).unwrap(); // miss

    let metrics = manager.get_metrics().unwrap();

    assert_eq!(metrics.total_runs, 3);
    assert_eq!(metrics.cache_hits, 2);
    assert_eq!(metrics.cache_misses, 1);
}

#[test]
fn test_hit_rate_calculation() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    manager.record_run(true, 5).unwrap();
    manager.record_run(true, 5).unwrap();
    manager.record_run(true, 5).unwrap();
    manager.record_run(false, 1000).unwrap();

    let hit_rate = manager.hit_rate().unwrap();

    assert!((hit_rate - 0.75).abs() < 0.01);
}

#[test]
fn test_cache_health_check() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Not enough data
    assert!(manager.is_healthy().unwrap());

    // Add 10 runs with good hit rate
    for _ in 0..8 {
        manager.record_run(true, 5).unwrap();
    }
    for _ in 0..2 {
        manager.record_run(false, 1000).unwrap();
    }

    // 80% hit rate - healthy
    assert!(manager.is_healthy().unwrap());
}

// =========================================================================
// Phase 2: Per-Gate Caching Tests
// =========================================================================

#[test]
fn test_gate_cache_miss_no_file() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    let result = manager.check_gate("complexity", &[]).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_gate_cache_hit_after_update() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create a test file
    let test_file = temp.path().join("test.rs");
    std::fs::write(&test_file, "fn main() {}").unwrap();

    let files = vec![test_file.clone()];

    // Update gate cache
    manager
        .update_gate("complexity", &files, CacheResult::Pass, 100, vec![])
        .unwrap();

    // Check should hit
    let result = manager.check_gate("complexity", &files).unwrap();
    assert!(result.is_some());
    let entry = result.unwrap();
    assert_eq!(entry.result, CacheResult::Pass);
    assert_eq!(entry.duration_ms, 100);
}

#[test]
fn test_gate_cache_miss_on_file_change() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create a test file
    let test_file = temp.path().join("test.rs");
    std::fs::write(&test_file, "fn main() {}").unwrap();

    let files = vec![test_file.clone()];

    // Update gate cache
    manager
        .update_gate("complexity", &files, CacheResult::Pass, 100, vec![])
        .unwrap();

    // Modify file
    std::fs::write(&test_file, "fn main() { println!(\"hello\"); }").unwrap();

    // Check should miss (file changed)
    let result = manager.check_gate("complexity", &files).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_check_gates_partial() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create test files
    let file1 = temp.path().join("file1.rs");
    let file2 = temp.path().join("file2.rs");
    std::fs::write(&file1, "fn one() {}").unwrap();
    std::fs::write(&file2, "fn two() {}").unwrap();

    // Cache only complexity gate
    manager
        .update_gate(
            "complexity",
            &[file1.clone()],
            CacheResult::Pass,
            50,
            vec![],
        )
        .unwrap();

    // Check gates - complexity should be cached, satd should not
    let gates = vec![
        GateDefinition::new("complexity", vec![file1]),
        GateDefinition::new("satd", vec![file2]),
    ];

    let result = manager.check_gates(&gates).unwrap();

    assert_eq!(result.cached.len(), 1);
    assert_eq!(result.cached[0].0, "complexity");
    assert_eq!(result.uncached.len(), 1);
    assert_eq!(result.uncached[0].name, "satd");
}

// =========================================================================
// Phase 3: Parallel Execution Tests
// =========================================================================

#[test]
fn test_parallel_gate_execution() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    let gates = vec![
        GateDefinition::new("gate1", vec![]),
        GateDefinition::new("gate2", vec![]),
        GateDefinition::new("gate3", vec![]),
    ];

    // Simple runner that always passes
    let runner = |_gate: &GateDefinition| -> anyhow::Result<GateRunResult> {
        Ok(GateRunResult {
            result: CacheResult::Pass,
            duration_ms: 10,
            warnings: vec![],
            from_cache: false,
        })
    };

    let results = manager.run_gates_parallel(gates, runner).unwrap();

    assert_eq!(results.overall, CacheResult::Pass);
    assert_eq!(results.results.len(), 3);
    assert!(results.errors.is_empty());
}

#[test]
fn test_parallel_gate_with_failure() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    let gates = vec![
        GateDefinition::new("pass_gate", vec![]),
        GateDefinition::new("fail_gate", vec![]),
    ];

    // Runner that fails for fail_gate
    let runner = |gate: &GateDefinition| -> anyhow::Result<GateRunResult> {
        Ok(GateRunResult {
            result: if gate.name == "fail_gate" {
                CacheResult::Fail
            } else {
                CacheResult::Pass
            },
            duration_ms: 10,
            warnings: vec![],
            from_cache: false,
        })
    };

    let results = manager.run_gates_parallel(gates, runner).unwrap();

    assert_eq!(results.overall, CacheResult::Fail);
}

#[test]
fn test_smart_gate_execution_with_cache() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create test files
    let file1 = temp.path().join("file1.rs");
    let file2 = temp.path().join("file2.rs");
    std::fs::write(&file1, "fn one() {}").unwrap();
    std::fs::write(&file2, "fn two() {}").unwrap();

    // Pre-cache gate1
    manager
        .update_gate("gate1", &[file1.clone()], CacheResult::Pass, 50, vec![])
        .unwrap();

    let gates = vec![
        GateDefinition::new("gate1", vec![file1]),
        GateDefinition::new("gate2", vec![file2]),
    ];

    let runner = |_gate: &GateDefinition| -> anyhow::Result<GateRunResult> {
        Ok(GateRunResult {
            result: CacheResult::Pass,
            duration_ms: 100,
            warnings: vec![],
            from_cache: false,
        })
    };

    let results = manager.run_gates_smart(gates, runner).unwrap();

    assert_eq!(results.overall, CacheResult::Pass);
    assert_eq!(results.gates_cached, 1); // gate1 from cache
    assert_eq!(results.gates_run, 1); // gate2 had to run
    assert_eq!(results.results.len(), 2);

    // Verify gate1 came from cache
    let gate1_result = results.results.iter().find(|(n, _)| n == "gate1");
    assert!(gate1_result.is_some());
    assert!(gate1_result.unwrap().1.from_cache);
}

// =========================================================================
// Additional Coverage Tests
// =========================================================================

#[test]
fn test_cache_miss_corrupted_json() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Write corrupted JSON
    let cache_path = temp.path().join(".pmat/hooks-cache/tree-hash.json");
    std::fs::write(&cache_path, "{ invalid json ]]").unwrap();

    let result = manager.check().unwrap();
    match result {
        CacheCheckResult::Miss {
            reason: CacheMissReason::CacheCorrupted(_),
        } => {}
        _ => panic!("Expected CacheCorrupted miss"),
    }
}

#[test]
fn test_cache_miss_tree_hash_changed() {
    let temp = create_test_repo().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Update cache
    manager
        .update(CacheResult::Pass, std::collections::HashMap::new())
        .unwrap();

    // Modify file and commit to change tree hash
    std::fs::write(
        temp.path().join("test.rs"),
        "fn main() { println!(\"changed\"); }",
    )
    .unwrap();
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(temp.path())
        .output()
        .unwrap();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "change"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Check should miss due to tree hash change
    let result = manager.check().unwrap();
    match result {
        CacheCheckResult::Miss {
            reason: CacheMissReason::TreeHashChanged { .. },
        } => {}
        _ => panic!("Expected TreeHashChanged miss, got {:?}", result),
    }
}

#[test]
fn test_cache_miss_stale() {
    let temp = create_test_repo().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create cache with old timestamp
    let cache = TreeHashCache {
        tree_hash: manager.get_tree_hash().unwrap(),
        result: CacheResult::Pass,
        gates: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now() - chrono::Duration::hours(48), // 48 hours old
        pmat_version: env!("CARGO_PKG_VERSION").to_string(),
        config_hash: manager.get_config_hash().unwrap(),
    };

    let cache_path = temp.path().join(".pmat/hooks-cache/tree-hash.json");
    std::fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();

    // Check should miss due to staleness
    let result = manager.check().unwrap();
    match result {
        CacheCheckResult::Miss {
            reason: CacheMissReason::CacheStale { age_hours },
        } => {
            assert!(age_hours >= 48);
        }
        _ => panic!("Expected CacheStale miss, got {:?}", result),
    }
}

#[test]
fn test_cache_miss_version_changed() {
    let temp = create_test_repo().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create cache with different version
    let cache = TreeHashCache {
        tree_hash: manager.get_tree_hash().unwrap(),
        result: CacheResult::Pass,
        gates: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
        pmat_version: "0.0.1-fake".to_string(), // Different version
        config_hash: manager.get_config_hash().unwrap(),
    };

    let cache_path = temp.path().join(".pmat/hooks-cache/tree-hash.json");
    std::fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();

    // Check should miss due to version change
    let result = manager.check().unwrap();
    match result {
        CacheCheckResult::Miss {
            reason: CacheMissReason::VersionChanged { old, new },
        } => {
            assert_eq!(old, "0.0.1-fake");
            assert_eq!(new, env!("CARGO_PKG_VERSION"));
        }
        _ => panic!("Expected VersionChanged miss, got {:?}", result),
    }
}

#[test]
fn test_cache_miss_config_changed() {
    let temp = create_test_repo().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create cache with different config hash
    let cache = TreeHashCache {
        tree_hash: manager.get_tree_hash().unwrap(),
        result: CacheResult::Pass,
        gates: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
        pmat_version: env!("CARGO_PKG_VERSION").to_string(),
        config_hash: "different_hash".to_string(), // Different config
    };

    let cache_path = temp.path().join(".pmat/hooks-cache/tree-hash.json");
    std::fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();

    // Check should miss due to config change
    let result = manager.check().unwrap();
    match result {
        CacheCheckResult::Miss {
            reason: CacheMissReason::ConfigHashChanged,
        } => {}
        _ => panic!("Expected ConfigHashChanged miss, got {:?}", result),
    }
}

#[test]
fn test_clear_gate() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create a test file
    let test_file = temp.path().join("test.rs");
    std::fs::write(&test_file, "fn main() {}").unwrap();

    // Cache a gate
    manager
        .update_gate(
            "complexity",
            &[test_file.clone()],
            CacheResult::Pass,
            100,
            vec![],
        )
        .unwrap();

    // Verify gate is cached
    let gate_path = temp.path().join(".pmat/hooks-cache/gates/complexity.json");
    assert!(gate_path.exists());

    // Clear specific gate
    manager.clear_gate("complexity").unwrap();

    // Verify gate is cleared
    assert!(!gate_path.exists());
}

#[test]
fn test_clear_gate_nonexistent() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Clear nonexistent gate should succeed
    manager.clear_gate("nonexistent").unwrap();
}

#[test]
fn test_cache_miss_reason_display() {
    // Test Display implementations
    let no_cache = CacheMissReason::NoCacheFile;
    assert_eq!(format!("{}", no_cache), "No cache file exists");

    let tree_changed = CacheMissReason::TreeHashChanged {
        old: "abc12345678".to_string(), // Need 8+ chars for truncation
        new: "def45678901".to_string(),
    };
    let tree_str = format!("{}", tree_changed);
    assert!(tree_str.contains("abc12345")); // First 8 chars
    assert!(tree_str.contains("def45678"));

    let config_changed = CacheMissReason::ConfigHashChanged;
    assert_eq!(format!("{}", config_changed), "Config file changed");

    let stale = CacheMissReason::CacheStale { age_hours: 48 };
    let stale_str = format!("{}", stale);
    assert!(stale_str.contains("48"));
    assert!(stale_str.contains("stale"));

    let version = CacheMissReason::VersionChanged {
        old: "1.0.0".to_string(),
        new: "2.0.0".to_string(),
    };
    let version_str = format!("{}", version);
    assert!(version_str.contains("1.0.0"));
    assert!(version_str.contains("2.0.0"));

    let corrupted = CacheMissReason::CacheCorrupted("bad json".to_string());
    assert!(format!("{}", corrupted).contains("bad json"));
}

#[test]
fn test_parallel_gate_with_error() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    let gates = vec![
        GateDefinition::new("good_gate", vec![]),
        GateDefinition::new("error_gate", vec![]),
    ];

    // Runner that errors for error_gate
    let runner = |gate: &GateDefinition| -> anyhow::Result<GateRunResult> {
        if gate.name == "error_gate" {
            anyhow::bail!("Simulated error")
        } else {
            Ok(GateRunResult {
                result: CacheResult::Pass,
                duration_ms: 10,
                warnings: vec![],
                from_cache: false,
            })
        }
    };

    let results = manager.run_gates_parallel(gates, runner).unwrap();

    // Should have errors recorded
    assert!(!results.errors.is_empty());
    assert!(results.errors.iter().any(|(name, _)| name == "error_gate"));
}

#[test]
fn test_parallel_gate_with_warnings() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    let gates = vec![GateDefinition::new("warn_gate", vec![])];

    // Runner that returns warnings
    let runner = |_gate: &GateDefinition| -> anyhow::Result<GateRunResult> {
        Ok(GateRunResult {
            result: CacheResult::Warn,
            duration_ms: 10,
            warnings: vec!["Test warning".to_string()],
            from_cache: false,
        })
    };

    let results = manager.run_gates_parallel(gates, runner).unwrap();

    assert_eq!(results.overall, CacheResult::Warn);
    assert_eq!(results.results.len(), 1);
    assert!(!results.results[0].1.warnings.is_empty());
}

#[test]
fn test_gate_definition_patterns() {
    let gate = GateDefinition {
        name: "complexity".to_string(),
        files: vec![std::path::PathBuf::from("test.rs")],
        patterns: vec!["*.rs".to_string(), "*.ts".to_string()],
    };

    assert_eq!(gate.name, "complexity");
    assert_eq!(gate.files.len(), 1);
    assert_eq!(gate.patterns.len(), 2);
}

#[test]
fn test_cache_result_warn() {
    let temp = create_test_repo().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Update cache with Warn result
    manager
        .update(CacheResult::Warn, std::collections::HashMap::new())
        .unwrap();

    // Check should hit with Warn
    let result = manager.check().unwrap();
    match result {
        CacheCheckResult::Hit { result, .. } => {
            assert_eq!(result, CacheResult::Warn);
        }
        _ => panic!("Expected cache hit"),
    }
}

#[test]
fn test_cache_result_fail() {
    let temp = create_test_repo().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Update cache with Fail result
    manager
        .update(CacheResult::Fail, std::collections::HashMap::new())
        .unwrap();

    // Check should hit with Fail
    let result = manager.check().unwrap();
    match result {
        CacheCheckResult::Hit { result, .. } => {
            assert_eq!(result, CacheResult::Fail);
        }
        _ => panic!("Expected cache hit"),
    }
}

#[test]
fn test_low_hit_rate_unhealthy() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Add 10 runs with low hit rate (20%)
    for _ in 0..2 {
        manager.record_run(true, 5).unwrap();
    }
    for _ in 0..8 {
        manager.record_run(false, 1000).unwrap();
    }

    // 20% hit rate - unhealthy (threshold is 60%)
    assert!(!manager.is_healthy().unwrap());
}

#[test]
fn test_empty_gates_parallel() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    let gates: Vec<GateDefinition> = vec![];

    let runner = |_gate: &GateDefinition| -> anyhow::Result<GateRunResult> {
        Ok(GateRunResult {
            result: CacheResult::Pass,
            duration_ms: 10,
            warnings: vec![],
            from_cache: false,
        })
    };

    let results = manager.run_gates_parallel(gates, runner).unwrap();

    // Empty gates should pass
    assert_eq!(results.overall, CacheResult::Pass);
    assert!(results.results.is_empty());
}

#[test]
fn test_smart_gates_all_cached() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    // Create test file
    let file1 = temp.path().join("file1.rs");
    std::fs::write(&file1, "fn one() {}").unwrap();

    // Pre-cache gate1
    manager
        .update_gate("gate1", &[file1.clone()], CacheResult::Pass, 50, vec![])
        .unwrap();

    let gates = vec![GateDefinition::new("gate1", vec![file1])];

    let runner = |_gate: &GateDefinition| -> anyhow::Result<GateRunResult> {
        panic!("Should not be called - gate is cached");
    };

    let results = manager.run_gates_smart(gates, runner).unwrap();

    assert_eq!(results.overall, CacheResult::Pass);
    assert_eq!(results.gates_cached, 1);
    assert_eq!(results.gates_run, 0);
}

#[test]
fn test_gate_with_warnings_stored() {
    let temp = TempDir::new().unwrap();
    let manager = HooksCacheManager::new(temp.path());
    manager.init().unwrap();

    let test_file = temp.path().join("test.rs");
    std::fs::write(&test_file, "fn main() {}").unwrap();

    let warnings = vec!["Warning 1".to_string(), "Warning 2".to_string()];
    manager
        .update_gate(
            "complexity",
            &[test_file.clone()],
            CacheResult::Warn,
            100,
            warnings.clone(),
        )
        .unwrap();

    let result = manager.check_gate("complexity", &[test_file]).unwrap();
    assert!(result.is_some());
    let entry = result.unwrap();
    assert_eq!(entry.result, CacheResult::Warn);
    assert_eq!(entry.warnings.len(), 2);
}
