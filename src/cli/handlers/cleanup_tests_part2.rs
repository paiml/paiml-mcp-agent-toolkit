// Tests for cleanup_resources_handler (part 2: data, edge cases, output)
// Included by cleanup_tests.rs

// ============================================================================
// CleanupResult with data tests
// ============================================================================

#[test]
fn test_cleanup_result_with_candidates() {
    let mut result = CleanupResult::default();
    result.candidates.push(CleanupCandidate {
        path: PathBuf::from("/test"),
        size_bytes: 1024,
        category: "rust".to_string(),
        description: "test".to_string(),
        age_days: 0,
    });
    result.total_size_bytes = 1024;
    result.items_found = 1;

    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.total_size_bytes, 1024);
    assert_eq!(result.items_found, 1);
}

#[test]
fn test_cleanup_result_with_errors() {
    let mut result = CleanupResult::default();
    result.errors.push("Error 1".to_string());
    result.errors.push("Error 2".to_string());

    assert_eq!(result.errors.len(), 2);
    assert!(result.errors.contains(&"Error 1".to_string()));
}

#[test]
fn test_cleanup_result_accumulation() {
    let mut result = CleanupResult::default();

    for i in 0..5 {
        result.candidates.push(CleanupCandidate {
            path: PathBuf::from(format!("/test/{}", i)),
            size_bytes: 100 * (i as u64 + 1),
            category: "rust".to_string(),
            description: format!("test {}", i),
            age_days: i,
        });
        result.total_size_bytes += 100 * (i as u64 + 1);
        result.items_found += 1;
    }

    assert_eq!(result.items_found, 5);
    assert_eq!(result.total_size_bytes, 100 + 200 + 300 + 400 + 500);
}

// ============================================================================
// is_excluded edge cases
// ============================================================================

#[test]
fn test_is_excluded_complex_glob() {
    let exclude = vec!["*.rs".to_string()];
    assert!(is_excluded(Path::new("/project/src/main.rs"), &exclude));
}

#[test]
fn test_is_excluded_multiple_patterns() {
    let exclude = vec![
        "node_modules".to_string(),
        "target".to_string(),
        "*.log".to_string(),
    ];
    assert!(is_excluded(Path::new("/project/node_modules"), &exclude));
    assert!(is_excluded(Path::new("/project/target"), &exclude));
    assert!(is_excluded(Path::new("/project/debug.log"), &exclude));
    assert!(!is_excluded(Path::new("/project/src"), &exclude));
}

#[test]
fn test_is_excluded_glob_with_middle_star() {
    // The is_excluded implementation splits on '*' and checks
    // path_str.starts_with(parts[0]) && path_str.ends_with(parts[1])
    let exclude = vec!["test*file".to_string()];
    // Path string must start with "test" and end with "file"
    assert!(is_excluded(Path::new("testABCfile"), &exclude));
    // Path with leading slash doesn't match since it doesn't start with "test"
    assert!(!is_excluded(Path::new("/testABCfile"), &exclude));
}

// ============================================================================
// is_hidden edge cases
// ============================================================================

#[test]
fn test_is_hidden_multiple_dots() {
    assert!(is_hidden(Path::new("..hidden")));
    assert!(is_hidden(Path::new(".config.old")));
}

#[test]
fn test_is_hidden_just_dot() {
    assert!(!is_hidden(Path::new(".")));
    assert!(!is_hidden(Path::new("..")));
}

// ============================================================================
// CleanupCandidate field tests
// ============================================================================

#[test]
fn test_cleanup_candidate_all_fields() {
    let candidate = CleanupCandidate {
        path: PathBuf::from("/very/long/nested/path/to/target"),
        size_bytes: 1_000_000_000, // 1GB
        category: "rust".to_string(),
        description: "Large Rust build artifacts".to_string(),
        age_days: 365,
    };

    assert_eq!(candidate.size_bytes, 1_000_000_000);
    assert_eq!(candidate.age_days, 365);
    assert_eq!(candidate.category, "rust");
    assert!(candidate.description.contains("Large"));
}

#[test]
fn test_cleanup_candidate_zero_values() {
    let candidate = CleanupCandidate {
        path: PathBuf::from("/"),
        size_bytes: 0,
        category: "".to_string(),
        description: "".to_string(),
        age_days: 0,
    };

    assert_eq!(candidate.size_bytes, 0);
    assert_eq!(candidate.age_days, 0);
    assert!(candidate.category.is_empty());
}

// ============================================================================
// print_results tests (testing via Result construction)
// ============================================================================

#[test]
fn test_print_results_json_format() {
    let mut result = CleanupResult::default();
    result.candidates.push(CleanupCandidate {
        path: PathBuf::from("/test"),
        size_bytes: 1024 * 1024, // 1 MB
        category: "rust".to_string(),
        description: "test".to_string(),
        age_days: 0,
    });
    result.total_size_bytes = 1024 * 1024;
    result.items_found = 1;

    // Just test that print_results doesn't panic with Json format
    let res = print_results(&result, OutputFormat::Json);
    assert!(res.is_ok());
}

#[test]
fn test_print_results_table_format() {
    let mut result = CleanupResult::default();
    result.candidates.push(CleanupCandidate {
        path: PathBuf::from("/test"),
        size_bytes: 1024,
        category: "node".to_string(),
        description: "test".to_string(),
        age_days: 0,
    });
    result.items_found = 1;

    // Test Table format doesn't panic
    let res = print_results(&result, OutputFormat::Table);
    assert!(res.is_ok());
}

#[test]
fn test_print_results_yaml_format() {
    let result = CleanupResult::default();

    // Test Yaml format (falls through to default branch)
    let res = print_results(&result, OutputFormat::Yaml);
    assert!(res.is_ok());
}

#[test]
fn test_print_results_empty() {
    let result = CleanupResult::default();
    let res = print_results(&result, OutputFormat::Table);
    assert!(res.is_ok());
}

#[test]
fn test_print_results_many_candidates() {
    let mut result = CleanupResult::default();
    // Add more than 20 candidates to test the "... and N more" branch
    for i in 0..25 {
        result.candidates.push(CleanupCandidate {
            path: PathBuf::from(format!("/test/{}", i)),
            size_bytes: 1024,
            category: "rust".to_string(),
            description: format!("candidate {}", i),
            age_days: 0,
        });
        result.items_found += 1;
    }

    let res = print_results(&result, OutputFormat::Table);
    assert!(res.is_ok());
}

// ============================================================================
// calculate_dir_size edge cases
// ============================================================================

#[test]
fn test_calculate_dir_size_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("a.txt"), "a").unwrap();
    std::fs::write(temp_dir.path().join("b.txt"), "bb").unwrap();
    std::fs::write(temp_dir.path().join("c.txt"), "ccc").unwrap();

    let size = calculate_dir_size(temp_dir.path());
    assert_eq!(size, 6); // 1 + 2 + 3 = 6 bytes
}

#[test]
fn test_calculate_dir_size_deeply_nested() {
    let temp_dir = TempDir::new().unwrap();
    let deep = temp_dir.path().join("a").join("b").join("c");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("file.txt"), "deep content").unwrap();

    let size = calculate_dir_size(temp_dir.path());
    assert_eq!(size, 12); // "deep content"
}

// ============================================================================
// count_loose_objects edge cases
// ============================================================================

#[test]
fn test_count_loose_objects_multiple_hex_dirs() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple valid hex directories
    for hex in ["ab", "cd", "ef", "12"] {
        let hex_dir = temp_dir.path().join(hex);
        std::fs::create_dir(&hex_dir).unwrap();
        std::fs::write(hex_dir.join("object1"), "content").unwrap();
    }

    let count = count_loose_objects(temp_dir.path());
    assert_eq!(count, 4);
}

#[test]
fn test_count_loose_objects_mixed_dirs() {
    let temp_dir = TempDir::new().unwrap();

    // Valid hex dir
    let hex_dir = temp_dir.path().join("ff");
    std::fs::create_dir(&hex_dir).unwrap();
    std::fs::write(hex_dir.join("obj"), "x").unwrap();

    // Invalid: not hex
    let non_hex = temp_dir.path().join("zz");
    std::fs::create_dir(&non_hex).unwrap();
    std::fs::write(non_hex.join("obj"), "x").unwrap();

    // Invalid: too long
    let long_dir = temp_dir.path().join("abc");
    std::fs::create_dir(&long_dir).unwrap();
    std::fs::write(long_dir.join("obj"), "x").unwrap();

    let count = count_loose_objects(temp_dir.path());
    assert_eq!(count, 1); // Only the "ff" directory counts
}

// ============================================================================
// is_old_enough + scan_* coverage (cleanup_resources_handler 37 uncov,
// cleanup_scanners 217 uncov on broad)
// ============================================================================

#[test]
fn test_is_old_enough_zero_threshold_always_true() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("a.txt");
    std::fs::write(&f, "x").unwrap();
    assert!(is_old_enough(&f, 0));
}

#[test]
fn test_is_old_enough_freshly_created_file_below_high_threshold() {
    let temp = TempDir::new().unwrap();
    let f = temp.path().join("a.txt");
    std::fs::write(&f, "x").unwrap();
    // Just-created → age ≈ 0 days → ≥ 30 is false.
    assert!(!is_old_enough(&f, 30));
}

#[test]
fn test_is_old_enough_missing_path_returns_true_via_unwrap_or() {
    // Missing path → metadata Err → unwrap_or(true).
    let missing = std::path::Path::new("/tmp/pmat_nope_oldenough_xyz.txt");
    assert!(is_old_enough(missing, 30));
}

#[test]
fn test_scan_rust_targets_empty_project_finds_nothing() {
    let temp = TempDir::new().unwrap();
    let mut result = CleanupResult::default();
    scan_rust_targets(temp.path(), &[], 0, &mut result).unwrap();
    assert!(result.candidates.iter().all(|c| c.category != "rust"));
}

#[test]
fn test_scan_rust_targets_finds_target_with_cargo_toml_sibling() {
    let temp = TempDir::new().unwrap();
    // tempfile dirs often start with "." (e.g. .tmpXYZ), which is_hidden
    // filters out at depth 0. Nest into a non-hidden subdir.
    let project = temp.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
    let target = project.join("target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("dummy"), "x").unwrap();

    let mut result = CleanupResult::default();
    scan_rust_targets(&project, &[], 0, &mut result).unwrap();
    let rust_count = result
        .candidates
        .iter()
        .filter(|c| c.category == "rust")
        .count();
    assert_eq!(rust_count, 1, "must detect target/ next to Cargo.toml");
}

#[test]
fn test_scan_rust_targets_skips_target_without_cargo_toml() {
    let temp = TempDir::new().unwrap();
    let target = temp.path().join("target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("dummy"), "x").unwrap();

    let mut result = CleanupResult::default();
    scan_rust_targets(temp.path(), &[], 0, &mut result).unwrap();
    assert!(result.candidates.iter().all(|c| c.category != "rust"));
}

#[test]
fn test_scan_node_targets_runs_without_panic() {
    // NOTE: scan_node_targets has a known limitation — its filter_entry
    // skips node_modules dirs from the walker entirely, so the inner
    // detection branch can't fire from this entry point. Just verify
    // the scanner runs without panic on a populated tree.
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("package.json"), "{}").unwrap();
    let nm = project.join("node_modules");
    std::fs::create_dir(&nm).unwrap();
    std::fs::write(nm.join("placeholder"), "x").unwrap();

    let mut result = CleanupResult::default();
    let res = scan_node_targets(&project, &[], 0, &mut result);
    assert!(res.is_ok());
}

#[test]
fn test_scan_git_targets_no_git_dir_no_candidates() {
    let temp = TempDir::new().unwrap();
    let mut result = CleanupResult::default();
    scan_git_targets(temp.path(), &mut result).unwrap();
    assert!(result.candidates.iter().all(|c| c.category != "git"));
}

#[test]
fn test_scan_log_targets_empty_project_no_logs() {
    let temp = TempDir::new().unwrap();
    let mut result = CleanupResult::default();
    scan_log_targets(temp.path(), &[], 0, &mut result).unwrap();
    assert!(result.candidates.iter().all(|c| c.category != "logs"));
}
