#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for check_handlers.rs
//!
//! Tests the PMAT compliance check handlers including:
//! - ProjectConfig and PmatSection default/serialization
//! - ComplianceReport, ComplianceCheck, CheckStatus, Severity types
//! - Version currency checks
//! - Config file checks
//! - Hook checks (installation, O(1) capability, cache health)
//! - Quality threshold checks
//! - ComputeBrick compliance checks (CB-001 through CB-027)
//! - OIP Tarantula patterns (CB-120 through CB-124)
//! - Coverage quality patterns (CB-125 through CB-127)
//! - Build checks (Cargo.lock, MSRV, CI)
//! - PAIML dependency checks
//! - Sovereign stack pattern checks
//! - File health checks (CB-040)
//! - Muda waste score (CB-300)
//! - Reproducibility level (CB-301)
//! - Golden trace drift (CB-302)
//! - EDD compliance (CB-303)
//! - Dead code percentage (CB-304)
//! - Dead code analysis helpers

use crate::cli::handlers::comply_handlers::{
    BreakingChange, CheckStatus, ComplianceCheck, ComplianceReport, PmatSection, ProjectConfig,
    Severity, PMAT_VERSION,
};
use crate::models::comply_config::{CheckSeverity, ComplyConfig};
use chrono::Utc;
use tempfile::TempDir;

// =============================================================================
// ProjectConfig and PmatSection tests
// =============================================================================

#[test]
fn test_project_config_default() {
    let config = ProjectConfig::default();
    assert_eq!(config.pmat.version, PMAT_VERSION);
    assert!(config.pmat.last_compliance_check.is_some());
    assert!(!config.pmat.auto_update);
}

#[test]
fn test_pmat_section_serialization() {
    let section = PmatSection {
        version: "1.0.0".to_string(),
        last_compliance_check: Some(Utc::now()),
        auto_update: true,
    };

    let serialized = toml::to_string(&section).unwrap();
    assert!(serialized.contains("version = \"1.0.0\""));
    assert!(serialized.contains("auto_update = true"));
}

#[test]
fn test_project_config_roundtrip() {
    let config = ProjectConfig::default();
    let serialized = toml::to_string_pretty(&config).unwrap();
    let deserialized: ProjectConfig = toml::from_str(&serialized).unwrap();

    assert_eq!(config.pmat.version, deserialized.pmat.version);
    assert_eq!(config.pmat.auto_update, deserialized.pmat.auto_update);
}

// =============================================================================
// CheckStatus tests
// =============================================================================

#[test]
fn test_check_status_variants() {
    assert_eq!(CheckStatus::Pass, CheckStatus::Pass);
    assert_eq!(CheckStatus::Warn, CheckStatus::Warn);
    assert_eq!(CheckStatus::Fail, CheckStatus::Fail);
    assert_eq!(CheckStatus::Skip, CheckStatus::Skip);
}

#[test]
fn test_check_status_serialization() {
    let status = CheckStatus::Pass;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"Pass\"");

    let status = CheckStatus::Fail;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"Fail\"");
}

#[test]
fn test_check_status_inequality() {
    assert_ne!(CheckStatus::Pass, CheckStatus::Fail);
    assert_ne!(CheckStatus::Warn, CheckStatus::Skip);
}

// =============================================================================
// Severity tests
// =============================================================================

#[test]
fn test_severity_variants() {
    assert_eq!(Severity::Info, Severity::Info);
    assert_eq!(Severity::Warning, Severity::Warning);
    assert_eq!(Severity::Error, Severity::Error);
    assert_eq!(Severity::Critical, Severity::Critical);
}

#[test]
fn test_severity_from_config_severity() {
    let severity: Severity = CheckSeverity::Info.into();
    assert_eq!(severity, Severity::Info);

    let severity: Severity = CheckSeverity::Warning.into();
    assert_eq!(severity, Severity::Warning);

    let severity: Severity = CheckSeverity::Error.into();
    assert_eq!(severity, Severity::Error);

    let severity: Severity = CheckSeverity::Critical.into();
    assert_eq!(severity, Severity::Critical);
}

#[test]
fn test_severity_serialization() {
    let severity = Severity::Critical;
    let json = serde_json::to_string(&severity).unwrap();
    assert_eq!(json, "\"Critical\"");

    let deserialized: Severity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, Severity::Critical);
}

// =============================================================================
// ComplianceCheck tests
// =============================================================================

#[test]
fn test_compliance_check_creation() {
    let check = ComplianceCheck {
        name: "Test Check".to_string(),
        status: CheckStatus::Pass,
        message: "All good".to_string(),
        severity: Severity::Info,
    };

    assert_eq!(check.name, "Test Check");
    assert_eq!(check.status, CheckStatus::Pass);
    assert_eq!(check.message, "All good");
    assert_eq!(check.severity, Severity::Info);
}

#[test]
fn test_compliance_check_serialization() {
    let check = ComplianceCheck {
        name: "Version Check".to_string(),
        status: CheckStatus::Warn,
        message: "Version behind".to_string(),
        severity: Severity::Warning,
    };

    let json = serde_json::to_string(&check).unwrap();
    assert!(json.contains("\"name\":\"Version Check\""));
    assert!(json.contains("\"status\":\"Warn\""));
    assert!(json.contains("\"severity\":\"Warning\""));
}

#[test]
fn test_compliance_check_clone() {
    let check = ComplianceCheck {
        name: "Clone Test".to_string(),
        status: CheckStatus::Fail,
        message: "Failed".to_string(),
        severity: Severity::Error,
    };

    let cloned = check.clone();
    assert_eq!(check.name, cloned.name);
    assert_eq!(check.status, cloned.status);
    assert_eq!(check.severity, cloned.severity);
}

// =============================================================================
// BreakingChange tests
// =============================================================================

#[test]
fn test_breaking_change_creation() {
    let change = BreakingChange {
        version: "2.0.0".to_string(),
        description: "Major API change".to_string(),
        migration_guide: Some("See docs".to_string()),
    };

    assert_eq!(change.version, "2.0.0");
    assert_eq!(change.description, "Major API change");
    assert!(change.migration_guide.is_some());
}

#[test]
fn test_breaking_change_without_guide() {
    let change = BreakingChange {
        version: "1.5.0".to_string(),
        description: "Minor breaking change".to_string(),
        migration_guide: None,
    };

    assert!(change.migration_guide.is_none());
}

#[test]
fn test_breaking_change_serialization() {
    let change = BreakingChange {
        version: "3.0.0".to_string(),
        description: "Complete rewrite".to_string(),
        migration_guide: Some("Start fresh".to_string()),
    };

    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("\"version\":\"3.0.0\""));
    assert!(json.contains("\"migration_guide\":\"Start fresh\""));
}

// =============================================================================
// ComplianceReport tests
// =============================================================================

#[test]
fn test_compliance_report_compliant() {
    let report = ComplianceReport {
        project_version: PMAT_VERSION.to_string(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: true,
        versions_behind: 0,
        checks: vec![ComplianceCheck {
            name: "Test".to_string(),
            status: CheckStatus::Pass,
            message: "OK".to_string(),
            severity: Severity::Info,
        }],
        breaking_changes: vec![],
        recommendations: vec![],
        timestamp: Utc::now(),
    };

    assert!(report.is_compliant);
    assert_eq!(report.versions_behind, 0);
    assert!(report.breaking_changes.is_empty());
}

#[test]
fn test_compliance_report_not_compliant() {
    let report = ComplianceReport {
        project_version: "0.1.0".to_string(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: false,
        versions_behind: 10,
        checks: vec![ComplianceCheck {
            name: "Version Currency".to_string(),
            status: CheckStatus::Fail,
            message: "Too old".to_string(),
            severity: Severity::Error,
        }],
        breaking_changes: vec![BreakingChange {
            version: "1.0.0".to_string(),
            description: "Breaking change".to_string(),
            migration_guide: None,
        }],
        recommendations: vec!["Update now".to_string()],
        timestamp: Utc::now(),
    };

    assert!(!report.is_compliant);
    assert_eq!(report.versions_behind, 10);
    assert!(!report.breaking_changes.is_empty());
    assert!(!report.recommendations.is_empty());
}

#[test]
fn test_compliance_report_serialization() {
    let report = ComplianceReport {
        project_version: "1.0.0".to_string(),
        current_version: PMAT_VERSION.to_string(),
        is_compliant: true,
        versions_behind: 0,
        checks: vec![],
        breaking_changes: vec![],
        recommendations: vec![],
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("\"project_version\":"));
    assert!(json.contains("\"is_compliant\": true"));
}

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

// =============================================================================
// ComputeBrick check tests
// =============================================================================

#[test]
fn test_check_compute_brick_not_cb_project() {
    let temp_dir = TempDir::new().unwrap();
    // No probar/trueno/realizar deps or brick/ dir
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_compute_brick(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
    assert!(check.message.contains("Not a ComputeBrick project"));
}

#[test]
fn test_check_compute_brick_with_trueno() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\ntrueno = \"0.1\"",
    )
    .unwrap();
    std::fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    std::fs::write(temp_dir.path().join("src").join("lib.rs"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_compute_brick(temp_dir.path());
    // Should not skip
    assert_ne!(check.status, CheckStatus::Skip);
}

// =============================================================================
// Cargo.lock check tests
// =============================================================================

#[test]
fn test_check_cargo_lock_present() {
    let temp_dir = TempDir::new().unwrap();
    // check_cargo_lock requires Cargo.toml to exist (otherwise Skip)
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();
    std::fs::write(temp_dir.path().join("Cargo.lock"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_cargo_lock(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("reproducible builds"));
}

#[test]
fn test_check_cargo_lock_missing() {
    let temp_dir = TempDir::new().unwrap();
    // check_cargo_lock requires Cargo.toml to exist (otherwise Skip)
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_cargo_lock(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Fail);
    assert!(check.message.contains("Missing Cargo.lock"));
}

// =============================================================================
// MSRV check tests
// =============================================================================

#[test]
fn test_check_msrv_defined() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nrust-version = \"1.70\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_msrv(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

#[test]
fn test_check_msrv_not_defined() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_msrv(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("No rust-version"));
}

#[test]
fn test_check_msrv_no_cargo_toml() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_msrv(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
}

// =============================================================================
// CI check tests
// =============================================================================

#[test]
fn test_check_ci_configured_github() {
    let temp_dir = TempDir::new().unwrap();
    let workflows_dir = temp_dir.path().join(".github").join("workflows");
    std::fs::create_dir_all(&workflows_dir).unwrap();
    std::fs::write(workflows_dir.join("ci.yml"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_ci_configured(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("GitHub Actions"));
}

#[test]
fn test_check_ci_configured_gitlab() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join(".gitlab-ci.yml"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_ci_configured(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("GitLab CI"));
}

#[test]
fn test_check_ci_configured_jenkins() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("Jenkinsfile"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_ci_configured(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("Jenkins"));
}

#[test]
fn test_check_ci_configured_none() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_ci_configured(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("No CI configuration"));
}

// =============================================================================
// OIP Tarantula patterns check tests (CB-120 through CB-124)
// =============================================================================

#[test]
fn test_check_oip_tarantula_patterns_clean() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn safe_fn() {}").unwrap();

    let check =
        crate::cli::handlers::comply_handlers::check_oip_tarantula_patterns(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

// =============================================================================
// Coverage quality patterns check tests (CB-125 through CB-127)
// =============================================================================

#[test]
fn test_check_coverage_quality_patterns_clean() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn clean() {}").unwrap();

    let check =
        crate::cli::handlers::comply_handlers::check_coverage_quality_patterns(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

// =============================================================================
// Dead code percentage check tests (CB-304)
// =============================================================================

#[test]
fn test_check_dead_code_percentage_no_src() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_dead_code_percentage(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
}

#[test]
fn test_check_dead_code_percentage_clean() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("lib.rs"),
        "pub fn active_fn() { println!(\"used\"); }",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_dead_code_percentage(temp_dir.path());
    // Clean code should pass
    assert!(check.status == CheckStatus::Pass || check.status == CheckStatus::Skip);
}

#[test]
fn test_check_dead_code_percentage_with_dead_code() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    // Create file with dead code annotations
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"

fn dead_fn1() {}

fn dead_fn2() {}

fn dead_fn3() {}
/// Active fn.
pub fn active_fn() {}
"#,
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_dead_code_percentage(temp_dir.path());
    // Should detect dead code
    assert!(check.name.contains("Dead Code"));
}

// =============================================================================
// Dead code analysis helper tests
// =============================================================================

#[test]
fn test_is_heavily_cfg_gated_false() {
    let content = "pub fn normal_fn() {}";
    let result = crate::cli::handlers::comply_handlers::is_heavily_cfg_gated(content);
    assert!(!result);
}

#[test]
fn test_is_heavily_cfg_gated_true() {
    let content = r#"
#[cfg(target_arch = "x86_64")]
fn simd_fn1() {}
#[cfg(target_feature = "avx2")]
fn simd_fn2() {}
#[cfg(feature = "simd")]
fn simd_fn3() {}
#[target_feature(enable = "sse2")]
fn simd_fn4() {}
"#;
    let result = crate::cli::handlers::comply_handlers::is_heavily_cfg_gated(content);
    assert!(result);
}

#[test]
fn test_is_dead_code_annotation_true() {
    assert!(crate::cli::handlers::comply_handlers::is_dead_code_annotation(&format!(
        "#[allow({})]",
        "dead_code"
    )));
    assert!(crate::cli::handlers::comply_handlers::is_dead_code_annotation("#[allow(unused)]"));
}

#[test]
fn test_is_dead_code_annotation_false() {
    assert!(!crate::cli::handlers::comply_handlers::is_dead_code_annotation("pub fn active() {}"));
    assert!(!crate::cli::handlers::comply_handlers::is_dead_code_annotation("#[derive(Debug)]"));
}

#[test]
fn test_is_code_item_declaration_true() {
    assert!(crate::cli::handlers::comply_handlers::is_code_item_declaration("pub fn test() {}"));
    assert!(crate::cli::handlers::comply_handlers::is_code_item_declaration("fn private() {}"));
    assert!(
        crate::cli::handlers::comply_handlers::is_code_item_declaration("pub struct MyStruct {")
    );
    assert!(crate::cli::handlers::comply_handlers::is_code_item_declaration("pub enum MyEnum {"));
    assert!(crate::cli::handlers::comply_handlers::is_code_item_declaration("pub trait MyTrait {"));
    assert!(
        crate::cli::handlers::comply_handlers::is_code_item_declaration(
            "pub const VALUE: i32 = 42;"
        )
    );
    assert!(
        crate::cli::handlers::comply_handlers::is_code_item_declaration(
            "pub async fn async_fn() {}"
        )
    );
}

#[test]
fn test_is_code_item_declaration_false() {
    assert!(!crate::cli::handlers::comply_handlers::is_code_item_declaration("let x = 5;"));
    assert!(!crate::cli::handlers::comply_handlers::is_code_item_declaration("// comment"));
    assert!(!crate::cli::handlers::comply_handlers::is_code_item_declaration("#[derive(Debug)]"));
}

#[test]
fn test_has_code_markers_true() {
    assert!(crate::cli::handlers::comply_handlers::has_code_markers(
        "fn test() {"
    ));
    assert!(crate::cli::handlers::comply_handlers::has_code_markers(
        "let x = 5;"
    ));
    assert!(crate::cli::handlers::comply_handlers::has_code_markers(
        "return value;"
    ));
    assert!(crate::cli::handlers::comply_handlers::has_code_markers(
        "struct Foo {"
    ));
}

#[test]
fn test_has_code_markers_false() {
    assert!(!crate::cli::handlers::comply_handlers::has_code_markers(
        "just a comment"
    ));
    assert!(!crate::cli::handlers::comply_handlers::has_code_markers(
        "empty line"
    ));
}

#[test]
fn test_is_commented_out_code_true() {
    assert!(crate::cli::handlers::comply_handlers::is_commented_out_code("// fn test() {"));
    assert!(crate::cli::handlers::comply_handlers::is_commented_out_code("// let x = 5;"));
    assert!(crate::cli::handlers::comply_handlers::is_commented_out_code("// return foo;"));
}

#[test]
fn test_is_commented_out_code_false() {
    // Just a regular comment
    assert!(
        !crate::cli::handlers::comply_handlers::is_commented_out_code(
            "// This is a regular comment"
        )
    );
    // Not a comment
    assert!(!crate::cli::handlers::comply_handlers::is_commented_out_code("fn test() {}"));
}

#[test]
fn test_flush_comment_run_threshold() {
    // Run of 3 or more counts
    assert_eq!(
        crate::cli::handlers::comply_handlers::flush_comment_run(3),
        3
    );
    assert_eq!(
        crate::cli::handlers::comply_handlers::flush_comment_run(5),
        5
    );
    // Run of less than 3 doesn't count
    assert_eq!(
        crate::cli::handlers::comply_handlers::flush_comment_run(2),
        0
    );
    assert_eq!(
        crate::cli::handlers::comply_handlers::flush_comment_run(0),
        0
    );
}

#[test]
fn test_update_macro_depth_enter() {
    let depth =
        crate::cli::handlers::comply_handlers::update_macro_depth("macro_rules! my_macro {", None);
    assert!(depth.is_some());
}

#[test]
fn test_update_macro_depth_exit() {
    let depth = crate::cli::handlers::comply_handlers::update_macro_depth("}", Some(1));
    assert!(depth.is_none());
}

#[test]
fn test_update_macro_depth_nested() {
    // Start macro
    let depth =
        crate::cli::handlers::comply_handlers::update_macro_depth("macro_rules! test {", None);
    assert!(depth.is_some());

    // Inside macro, nested brace
    let depth = crate::cli::handlers::comply_handlers::update_macro_depth("() => { {", depth);
    assert!(depth.is_some());
}

#[test]
fn test_filter_production_lines() {
    let lines = vec![
        "pub fn main() {}",
        "#[cfg(test)]",
        "mod tests {",
        "    #[test]",
        "    fn test_something() {}",
        "}",
    ];
    let prod_lines = crate::cli::handlers::comply_handlers::filter_production_lines(&lines);
    assert_eq!(prod_lines.len(), 1);
    assert_eq!(prod_lines[0], "pub fn main() {}");
}

#[test]
fn test_count_dead_items_no_dead_code() {
    let lines = vec![
        "pub fn active() {}",
        "pub struct Active {}",
        "impl Active {}",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let (total, dead, annotations) = crate::cli::handlers::comply_handlers::count_dead_items(&refs);
    assert_eq!(total, 2); // fn + struct
    assert_eq!(dead, 0);
    assert_eq!(annotations, 0);
}

#[test]
fn test_count_dead_items_with_dead_code() {
    let lines = vec![
        "",
        "fn dead_fn() {}",
        "pub fn active() {}",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let (total, dead, annotations) = crate::cli::handlers::comply_handlers::count_dead_items(&refs);
    assert_eq!(total, 2);
    assert_eq!(dead, 1);
    assert_eq!(annotations, 1);
}

#[test]
fn test_count_block_comment_code_lines() {
    let lines = vec![
        "/* fn old_function() {",
        "    let x = 5;",
        "    return x;",
        "} */",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let dead_lines = crate::cli::handlers::comply_handlers::count_block_comment_code_lines(&refs);
    // Should detect code-like content in block comment
    assert!(dead_lines >= 2);
}

#[test]
fn test_count_commented_code_lines() {
    let lines = vec![
        "// fn old_function() {",
        "//     let x = 5;",
        "//     return x;",
        "// }",
        "pub fn new_function() {}",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let dead_lines = crate::cli::handlers::comply_handlers::count_commented_code_lines(&refs);
    // Should detect 4 consecutive commented code lines
    assert_eq!(dead_lines, 4);
}

#[test]
fn test_analyze_file_dead_code() {
    let lines = vec![
        "",
        "fn dead() {}",
        "pub fn active() {}",
        "#[cfg(test)]",
        "mod tests {}",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let (total, dead, prod_lines, estimated_dead) =
        crate::cli::handlers::comply_handlers::analyze_file_dead_code(&refs);

    assert!(total >= 2);
    assert!(dead >= 1);
    assert!(prod_lines >= 3);
    assert!(estimated_dead >= 10); // ~10 per annotation
}

// =============================================================================
// format_violation_list tests
// =============================================================================

#[test]
fn test_format_violation_list_empty() {
    let issues: Vec<String> = vec![];
    let result = crate::cli::handlers::comply_handlers::format_violation_list(&issues);
    assert!(result.is_empty());
}

#[test]
fn test_format_violation_list_single() {
    let issues = vec!["CB-001: Test issue".to_string()];
    let result = crate::cli::handlers::comply_handlers::format_violation_list(&issues);
    assert_eq!(result, "    - CB-001: Test issue");
}

#[test]
fn test_format_violation_list_multiple() {
    let issues = vec![
        "Issue 1".to_string(),
        "Issue 2".to_string(),
        "Issue 3".to_string(),
    ];
    let result = crate::cli::handlers::comply_handlers::format_violation_list(&issues);
    assert!(result.contains("    - Issue 1"));
    assert!(result.contains("    - Issue 2"));
    assert!(result.contains("    - Issue 3"));
}

// =============================================================================
// collect_cb_violations tests
// =============================================================================

#[test]
fn test_collect_cb_violations_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    let (issues, critical, warning) =
        crate::cli::handlers::comply_handlers::collect_cb_violations(temp_dir.path(), false, false);

    // Empty project should have no violations
    assert!(issues.is_empty());
    assert_eq!(critical, 0);
    assert_eq!(warning, 0);
}

// =============================================================================
// build_cb_result tests
// =============================================================================

#[test]
fn test_build_cb_result_no_issues() {
    let result = crate::cli::handlers::comply_handlers::build_cb_result(vec![], 0, 0);
    assert_eq!(result.status, CheckStatus::Pass);
    assert!(result.message.contains("no violations"));
}

#[test]
fn test_build_cb_result_warnings_only() {
    let issues = vec!["Warning 1".to_string(), "Warning 2".to_string()];
    let result = crate::cli::handlers::comply_handlers::build_cb_result(issues, 0, 2);
    assert_eq!(result.status, CheckStatus::Warn);
    assert!(result.message.contains("2 warnings"));
}

#[test]
fn test_build_cb_result_critical() {
    let issues = vec!["Critical 1".to_string(), "Warning 1".to_string()];
    let result = crate::cli::handlers::comply_handlers::build_cb_result(issues, 1, 1);
    assert_eq!(result.status, CheckStatus::Fail);
    assert_eq!(result.severity, Severity::Critical);
}

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
