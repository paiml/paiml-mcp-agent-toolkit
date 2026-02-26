// =============================================================================
// filter_check_by_config tests
// =============================================================================

#[test]
fn test_filter_check_enabled() {
    let config = ComplyConfig::default();
    let check = ComplianceCheck {
        name: "Test Check".to_string(),
        status: CheckStatus::Pass,
        message: "Passed".to_string(),
        severity: Severity::Info,
    };

    // cb-050 should be enabled by default
    let filtered = crate::cli::handlers::comply_handlers::filter_check_by_config(
        check.clone(),
        "cb-050",
        &config,
    );
    assert_eq!(filtered.status, CheckStatus::Pass);
}

#[test]
fn test_filter_check_disabled() {
    let mut config = ComplyConfig::default();
    // Disable a check
    if let Some(check_config) = config.checks.get_mut("cb-050") {
        check_config.enabled = false;
    }

    let check = ComplianceCheck {
        name: "Test Check".to_string(),
        status: CheckStatus::Pass,
        message: "Passed".to_string(),
        severity: Severity::Warning,
    };

    let filtered = crate::cli::handlers::comply_handlers::filter_check_by_config(
        check.clone(),
        "cb-050",
        &config,
    );
    assert_eq!(filtered.status, CheckStatus::Skip);
    assert!(filtered.message.contains("disabled"));
}

// =============================================================================
// Version currency check tests
// =============================================================================

#[test]
fn test_check_version_currency_current() {
    let check = crate::cli::handlers::comply_handlers::check_version_currency(PMAT_VERSION);
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("latest version"));
}

#[test]
fn test_check_version_currency_slightly_behind() {
    // Simulate being slightly behind (this may vary based on actual versioning)
    let check = crate::cli::handlers::comply_handlers::check_version_currency("0.99.0");
    // Should be Warn or Fail depending on versions_behind calculation
    assert!(check.status == CheckStatus::Warn || check.status == CheckStatus::Fail);
}

// =============================================================================
// Config files check tests
// =============================================================================

#[test]
fn test_check_config_files_all_present() {
    let temp_dir = TempDir::new().unwrap();
    let pmat_dir = temp_dir.path().join(".pmat");
    std::fs::create_dir_all(&pmat_dir).unwrap();
    std::fs::write(pmat_dir.join("project.toml"), "[pmat]\nversion = \"1.0.0\"").unwrap();
    std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_config_files(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

#[test]
fn test_check_config_files_missing() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_config_files(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("Missing"));
}

// =============================================================================
// Hooks check tests
// =============================================================================

#[test]
fn test_check_hooks_installed_no_hooks() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_installed(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("No pre-commit hook"));
}

#[test]
fn test_check_hooks_installed_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    std::fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/sh\npmat hooks validate",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_installed(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

#[test]
fn test_check_hooks_installed_non_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    std::fs::write(hooks_dir.join("pre-commit"), "#!/bin/sh\necho hello").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_installed(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("may not be PMAT"));
}

// =============================================================================
// O(1) hooks check tests
// =============================================================================

#[test]
fn test_check_hooks_o1_capable_no_cache() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_o1_capable(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("O(1)"));
}

#[test]
fn test_check_hooks_o1_capable_with_tree_hash() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".pmat").join("hooks-cache");
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(cache_dir.join("tree-hash.json"), "{}").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_o1_capable(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

#[test]
fn test_check_hooks_o1_capable_with_gates() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".pmat").join("hooks-cache");
    let gates_dir = cache_dir.join("gates");
    std::fs::create_dir_all(&gates_dir).unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_o1_capable(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

// =============================================================================
// Hooks cache health check tests
// =============================================================================

#[test]
fn test_check_hooks_cache_health_no_metrics() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_cache_health(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
}

#[test]
fn test_check_hooks_cache_health_insufficient_data() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".pmat").join("hooks-cache");
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(
        cache_dir.join("metrics.json"),
        r#"{"total_runs": 2, "cache_hits": 1}"#,
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_cache_health(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
    assert!(check.message.contains("Insufficient data"));
}

#[test]
fn test_check_hooks_cache_health_good_hit_rate() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".pmat").join("hooks-cache");
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(
        cache_dir.join("metrics.json"),
        r#"{"total_runs": 100, "cache_hits": 80}"#,
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_cache_health(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("80.0%"));
}

#[test]
fn test_check_hooks_cache_health_poor_hit_rate() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join(".pmat").join("hooks-cache");
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(
        cache_dir.join("metrics.json"),
        r#"{"total_runs": 100, "cache_hits": 30}"#,
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_hooks_cache_health(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("below 60%"));
}

// =============================================================================
// Quality thresholds check tests
// =============================================================================

#[test]
fn test_check_quality_thresholds_present() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join(".pmat-metrics.toml"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_quality_thresholds(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

#[test]
fn test_check_quality_thresholds_missing() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_quality_thresholds(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("using defaults"));
}

// =============================================================================
// Deprecated features check tests
// =============================================================================

#[test]
fn test_check_deprecated_features() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_deprecated_features(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("No deprecated features"));
}
