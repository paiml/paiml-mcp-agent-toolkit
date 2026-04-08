// =============================================================================
// Muda waste score check tests (CB-300)
// =============================================================================

#[test]
fn test_check_muda_waste_score() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn main() {}").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_muda_waste_score(temp_dir.path());
    assert!(check.name.contains("Muda"));
    // Score should be in valid range
    assert!(check.message.contains("/100"));
}

// =============================================================================
// Reproducibility level check tests (CB-301)
// =============================================================================

#[test]
fn test_check_reproducibility_level_none() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_reproducibility_level(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Fail);
    assert!(check.name.contains("Reproducibility"));
}

#[test]
fn test_check_reproducibility_level_with_lockfile() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("Cargo.lock"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_reproducibility_level(temp_dir.path());
    // Should be at least Bronze with lockfile
    assert!(check.status == CheckStatus::Pass || check.status == CheckStatus::Warn);
}

// =============================================================================
// Golden trace drift check tests (CB-302)
// =============================================================================

#[test]
fn test_check_golden_trace_drift_no_config() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_golden_trace_drift(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
}

#[test]
fn test_check_golden_trace_drift_with_config() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("renacer.toml"), "[golden_traces]").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_golden_trace_drift(temp_dir.path());
    // Should pass or fail based on golden trace validation
    assert!(check.status == CheckStatus::Pass || check.status == CheckStatus::Fail);
}

// =============================================================================
// EDD compliance check tests (CB-303)
// =============================================================================

#[test]
fn test_check_edd_compliance_not_simulation() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_edd_compliance(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
    assert!(check.message.contains("Not a simulation project"));
}

// =============================================================================
// Load/create project config tests
// =============================================================================

#[test]
fn test_load_or_create_project_config_new() {
    let temp_dir = TempDir::new().unwrap();

    let config =
        crate::cli::handlers::comply_handlers::load_or_create_project_config(temp_dir.path())
            .unwrap();

    // Should create default config
    assert_eq!(config.pmat.version, PMAT_VERSION);

    // Verify file was created
    assert!(temp_dir.path().join(".pmat").join("project.toml").exists());
}

#[test]
fn test_load_or_create_project_config_existing() {
    let temp_dir = TempDir::new().unwrap();
    let pmat_dir = temp_dir.path().join(".pmat");
    std::fs::create_dir_all(&pmat_dir).unwrap();
    std::fs::write(
        pmat_dir.join("project.toml"),
        "[pmat]\nversion = \"1.0.0\"\nauto_update = true",
    )
    .unwrap();

    let config =
        crate::cli::handlers::comply_handlers::load_or_create_project_config(temp_dir.path())
            .unwrap();

    assert_eq!(config.pmat.version, "1.0.0");
    assert!(config.pmat.auto_update);
}

// =============================================================================
// Update timestamp tests
// =============================================================================

#[test]
fn test_update_last_check_timestamp() {
    let temp_dir = TempDir::new().unwrap();
    let pmat_dir = temp_dir.path().join(".pmat");
    std::fs::create_dir_all(&pmat_dir).unwrap();
    std::fs::write(pmat_dir.join("project.toml"), "[pmat]\nversion = \"1.0.0\"").unwrap();

    let result =
        crate::cli::handlers::comply_handlers::update_last_check_timestamp(temp_dir.path());
    assert!(result.is_ok());

    // Verify timestamp was updated
    let content = std::fs::read_to_string(pmat_dir.join("project.toml")).unwrap();
    assert!(content.contains("last_compliance_check"));
}

// =============================================================================
// PAIML workspace dependencies check tests
// =============================================================================

#[test]
fn test_check_paiml_deps_workspace() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_paiml_deps_workspace(temp_dir.path());
    // Should pass or warn based on workspace check
    assert!(
        check.status == CheckStatus::Pass
            || check.status == CheckStatus::Warn
            || check.status == CheckStatus::Skip
    );
}

// =============================================================================
// Sovereign stack patterns check tests
// =============================================================================

#[test]
fn test_check_sovereign_stack_patterns() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check =
        crate::cli::handlers::comply_handlers::check_sovereign_stack_patterns(temp_dir.path());
    assert!(check.name.contains("Sovereign") || check.name.contains("Stack"));
}

// =============================================================================
// File health check tests (CB-040)
// =============================================================================

#[test]
fn test_check_file_health() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn main() {}").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_file_health(temp_dir.path());
    // Should complete without panic
    assert!(
        check.status == CheckStatus::Pass
            || check.status == CheckStatus::Warn
            || check.status == CheckStatus::Fail
            || check.status == CheckStatus::Skip
    );
}

// =============================================================================
// Collect production rs files tests
// =============================================================================

#[test]
fn test_collect_production_rs_files_excludes_tests() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    let tests_dir = src_dir.join("tests");
    std::fs::create_dir_all(&tests_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "").unwrap();
    std::fs::write(src_dir.join("main_tests.rs"), "").unwrap();
    std::fs::write(tests_dir.join("integration.rs"), "").unwrap();

    let files = crate::cli::handlers::comply_handlers::collect_production_rs_files(&src_dir);

    // Should only include lib.rs, not test files
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().contains("lib.rs"));
}

#[test]
fn test_collect_production_rs_files_excludes_falsification() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    let falsification_dir = src_dir.join("falsification");
    std::fs::create_dir_all(&falsification_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "").unwrap();
    std::fs::write(falsification_dir.join("property_tests.rs"), "").unwrap();

    let files = crate::cli::handlers::comply_handlers::collect_production_rs_files(&src_dir);

    // Should only include lib.rs, not falsification files
    assert_eq!(files.len(), 1);
}

// =============================================================================
// Scan dead code indicators tests
// =============================================================================

#[test]
fn test_scan_dead_code_indicators_empty() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    let (total, dead, lines, dead_lines) =
        crate::cli::handlers::comply_handlers::scan_dead_code_indicators(&src_dir);

    assert_eq!(total, 0);
    assert_eq!(dead, 0);
    assert_eq!(lines, 0);
    assert_eq!(dead_lines, 0);
}

#[test]
fn test_scan_dead_code_indicators_with_code() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
/// Active.
pub fn active() {}

fn dead() {}
"#,
    )
    .unwrap();

    let (total, dead, lines, dead_lines) =
        crate::cli::handlers::comply_handlers::scan_dead_code_indicators(&src_dir);

    assert!(total >= 2);
    assert!(dead >= 1);
    assert!(lines > 0);
    assert!(dead_lines >= 10);
}

// =============================================================================
// PMAT_VERSION constant test
// =============================================================================

#[test]
fn test_pmat_version_is_set() {
    assert!(!PMAT_VERSION.is_empty());
    // Version should follow semver format
    let parts: Vec<&str> = PMAT_VERSION.split('.').collect();
    assert!(parts.len() >= 2);
}
