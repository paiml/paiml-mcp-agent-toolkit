// Unit tests for hooks_config types.
// Included from hooks_config.rs mod tests — no `use` imports or inner attributes allowed.

#[test]
fn test_tdg_hooks_config_default() {
    let config = TdgHooksConfig::default();
    assert_eq!(config.quality_gates.max_score_drop, 5.0);
    assert_eq!(config.quality_gates.mode, EnforcementMode::Strict);
    assert!(config.baseline.auto_update_on_commit);
    assert!(config.ci_cd.generate_reports);
}

#[test]
fn test_tdg_hooks_config_load_nonexistent() {
    let temp_dir = tempdir().unwrap();
    let config = TdgHooksConfig::load(temp_dir.path()).unwrap();

    // Should return default when file doesn't exist
    assert_eq!(config.quality_gates.max_score_drop, 5.0);
}

#[test]
fn test_tdg_hooks_config_create_default() {
    let temp_dir = tempdir().unwrap();

    // Create default config
    TdgHooksConfig::create_default(temp_dir.path()).unwrap();

    // Verify file was created
    let config_path = temp_dir.path().join(".pmat").join("tdg-rules.toml");
    assert!(config_path.exists());

    // Verify config can be loaded
    let loaded = TdgHooksConfig::load(temp_dir.path()).unwrap();
    assert_eq!(loaded.quality_gates.max_score_drop, 5.0);
}

#[test]
fn test_tdg_hooks_config_create_default_idempotent() {
    let temp_dir = tempdir().unwrap();

    // Create once
    TdgHooksConfig::create_default(temp_dir.path()).unwrap();

    // Create again - should not error
    TdgHooksConfig::create_default(temp_dir.path()).unwrap();
}

#[test]
fn test_quality_gates_config_default() {
    let config = QualityGatesConfig::default();

    assert_eq!(config.min_grades.get("rust"), Some(&"B+".to_string()));
    assert_eq!(config.min_grades.get("typescript"), Some(&"B+".to_string()));
    assert_eq!(config.min_grades.get("python"), Some(&"B".to_string()));
    assert_eq!(config.max_score_drop, 5.0);
    assert!(!config.allow_grade_drop);
    assert_eq!(config.mode, EnforcementMode::Strict);
    assert!(config.block_on_regression);
    assert!(config.block_on_new_files_below_threshold);
}

#[test]
fn test_quality_gates_get_min_grade() {
    let config = QualityGatesConfig::default();

    assert_eq!(config.get_min_grade("rust"), Some("B+"));
    assert_eq!(config.get_min_grade("typescript"), Some("B+"));
    assert_eq!(config.get_min_grade("python"), Some("B"));
    assert_eq!(config.get_min_grade("unknown"), None);
}

#[test]
fn test_quality_gates_get_min_grade_fallback() {
    let mut config = QualityGatesConfig::default();
    config.min_grades.clear(); // Clear the new format

    // Should fallback to deprecated fields
    assert_eq!(config.get_min_grade("rust"), Some("B+"));
    assert_eq!(config.get_min_grade("javascript"), Some("B+")); // Uses typescript fallback
}

#[test]
fn test_quality_gates_get_default_min_grade() {
    let config = QualityGatesConfig::default();
    assert_eq!(config.get_default_min_grade(), "B+");
}

#[test]
fn test_baseline_config_default() {
    let config = BaselineConfig::default();

    assert!(config.auto_update_on_commit);
    assert!(config.auto_update_on_merge);
    assert_eq!(config.baseline_path, ".pmat/baseline.json");
    assert!(config.store_in_git);
}

#[test]
fn test_cicd_config_default() {
    let config = CiCdConfig::default();

    assert!(!config.fail_fast);
    assert!(config.generate_reports);
    assert!(config.comment_on_pr);
}

#[test]
fn test_enforcement_mode_variants() {
    assert!(matches!(EnforcementMode::Strict, EnforcementMode::Strict));
    assert!(matches!(EnforcementMode::Warning, EnforcementMode::Warning));
    assert!(matches!(
        EnforcementMode::Disabled,
        EnforcementMode::Disabled
    ));
}

#[test]
fn test_enforcement_mode_display() {
    assert_eq!(EnforcementMode::Strict.to_string(), "strict");
    assert_eq!(EnforcementMode::Warning.to_string(), "warning");
    assert_eq!(EnforcementMode::Disabled.to_string(), "disabled");
}

#[test]
fn test_enforcement_mode_default() {
    let mode = EnforcementMode::default();
    assert_eq!(mode, EnforcementMode::Strict);
}

#[test]
fn test_enforcement_mode_equality() {
    assert_eq!(EnforcementMode::Strict, EnforcementMode::Strict);
    assert_ne!(EnforcementMode::Strict, EnforcementMode::Warning);
}

#[test]
fn test_enforcement_mode_clone() {
    let mode = EnforcementMode::Warning;
    let cloned = mode.clone();
    assert_eq!(cloned, EnforcementMode::Warning);
}

#[test]
fn test_default_helper_functions() {
    assert_eq!(default_max_score_drop(), 5.0);
    assert_eq!(default_mode(), EnforcementMode::Strict);
    assert!(default_true());
    assert_eq!(default_baseline_path(), ".pmat/baseline.json");
}

#[test]
fn test_config_serialization() {
    let config = TdgHooksConfig::default();
    let toml_str = toml::to_string(&config).unwrap();

    assert!(toml_str.contains("max_score_drop"));
    assert!(toml_str.contains("baseline_path"));
}

#[test]
fn test_config_deserialization() {
    let toml_str = r#"
[quality_gates]
max_score_drop = 10.0
mode = "warning"

[baseline]
auto_update_on_commit = false

[ci_cd]
fail_fast = true
"#;
    let config: TdgHooksConfig = toml::from_str(toml_str).unwrap();

    assert_eq!(config.quality_gates.max_score_drop, 10.0);
    assert_eq!(config.quality_gates.mode, EnforcementMode::Warning);
    assert!(!config.baseline.auto_update_on_commit);
    assert!(config.ci_cd.fail_fast);
}

#[test]
fn test_tdg_hooks_config_clone() {
    let config = TdgHooksConfig::default();
    let cloned = config.clone();

    assert_eq!(
        cloned.quality_gates.max_score_drop,
        config.quality_gates.max_score_drop
    );
}

#[test]
fn test_quality_gates_config_clone() {
    let config = QualityGatesConfig::default();
    let cloned = config.clone();

    assert_eq!(cloned.max_score_drop, config.max_score_drop);
}

#[test]
fn test_baseline_config_clone() {
    let config = BaselineConfig::default();
    let cloned = config.clone();

    assert_eq!(cloned.baseline_path, config.baseline_path);
}

#[test]
fn test_cicd_config_clone() {
    let config = CiCdConfig::default();
    let cloned = config.clone();

    assert_eq!(cloned.fail_fast, config.fail_fast);
}
