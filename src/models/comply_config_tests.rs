#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PmatYamlConfig::default();
        assert!(config.comply.is_check_enabled("cb-050"));
        assert!(config.comply.is_check_enabled("cb-060"));
        assert_eq!(config.comply.thresholds.coverage, 95.0);
        assert_eq!(config.comply.thresholds.complexity, 20);
    }

    #[test]
    fn test_severity_should_fail() {
        let config = ComplyConfig::default();

        // Critical always fails
        assert!(config.should_fail(CheckSeverity::Critical, false));
        assert!(config.should_fail(CheckSeverity::Critical, true));

        // Error always fails
        assert!(config.should_fail(CheckSeverity::Error, false));
        assert!(config.should_fail(CheckSeverity::Error, true));

        // Warning fails only in strict mode
        assert!(!config.should_fail(CheckSeverity::Warning, false));
        assert!(config.should_fail(CheckSeverity::Warning, true));

        // Info never fails
        assert!(!config.should_fail(CheckSeverity::Info, false));
        assert!(!config.should_fail(CheckSeverity::Info, true));
    }

    #[test]
    fn test_yaml_parsing() {
        let yaml = r#"
comply:
  checks:
    cb-050:
      enabled: false
      severity: warning
    cb-128:
      enabled: true
      threshold: 2.5
  thresholds:
    coverage: 90.0
    complexity: 15
"#;

        let config: PmatYamlConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(!config.comply.is_check_enabled("cb-050"));
        assert!(config.comply.is_check_enabled("cb-128"));
        assert_eq!(config.comply.get_threshold("cb-128"), Some(2.5));
        assert_eq!(config.comply.thresholds.coverage, 90.0);
        assert_eq!(config.comply.thresholds.complexity, 15);
    }

    #[test]
    fn test_unknown_check_defaults_to_enabled() {
        let config = ComplyConfig::default();
        // Unknown check should default to enabled
        assert!(config.is_check_enabled("cb-999"));
        assert_eq!(config.get_severity("cb-999"), CheckSeverity::Warning);
    }

    #[test]
    fn test_check_config_default() {
        let check = CheckConfig::default();
        assert!(check.enabled);
        assert_eq!(check.severity, CheckSeverity::Warning);
        assert!(check.threshold.is_none());
    }

    #[test]
    fn test_suppression_by_rule_id() {
        let config = ComplyConfig {
            suppressions: vec![SuppressionYamlRule {
                rules: vec!["CB-954".to_string()],
                files: vec![],
                reason: "max_tokens is an LLM parameter".to_string(),
                expires: None,
            }],
            ..Default::default()
        };
        // CB-954 should be suppressed regardless of file
        assert!(config
            .is_suppressed("CB-954", "playbooks/config.yaml")
            .is_some());
        // CB-950 should NOT be suppressed
        assert!(config
            .is_suppressed("CB-950", "playbooks/config.yaml")
            .is_none());
    }

    #[test]
    fn test_suppression_case_insensitive() {
        let config = ComplyConfig {
            suppressions: vec![SuppressionYamlRule {
                rules: vec!["cb-954".to_string()],
                files: vec![],
                reason: "test".to_string(),
                expires: None,
            }],
            ..Default::default()
        };
        assert!(config.is_suppressed("CB-954", "file.yaml").is_some());
    }

    #[test]
    fn test_suppression_with_file_glob() {
        let config = ComplyConfig {
            suppressions: vec![SuppressionYamlRule {
                rules: vec!["CB-501".to_string()],
                files: vec!["examples/**".to_string()],
                reason: "Examples use unwrap for brevity".to_string(),
                expires: None,
            }],
            ..Default::default()
        };
        // File matching glob should be suppressed
        assert!(config.is_suppressed("CB-501", "examples/demo.rs").is_some());
        // File NOT matching glob should NOT be suppressed
        assert!(config.is_suppressed("CB-501", "src/main.rs").is_none());
    }

    #[test]
    fn test_suppression_expired() {
        let config = ComplyConfig {
            suppressions: vec![SuppressionYamlRule {
                rules: vec!["CB-516".to_string()],
                files: vec![],
                reason: "Temporary suppression".to_string(),
                expires: Some("2020-01-01".to_string()), // Long expired
            }],
            ..Default::default()
        };
        // Expired suppression should NOT apply
        assert!(config.is_suppressed("CB-516", "src/lib.rs").is_none());
    }

    #[test]
    fn test_suppression_not_expired() {
        let config = ComplyConfig {
            suppressions: vec![SuppressionYamlRule {
                rules: vec!["CB-516".to_string()],
                files: vec![],
                reason: "Future suppression".to_string(),
                expires: Some("2099-12-31".to_string()),
            }],
            ..Default::default()
        };
        assert!(config.is_suppressed("CB-516", "src/lib.rs").is_some());
    }

    #[test]
    fn test_suppression_yaml_parsing() {
        let yaml = r#"
comply:
  suppressions:
    - rules: ["CB-954"]
      reason: "max_tokens is an LLM parameter"
    - rules: ["CB-501"]
      files: ["examples/**"]
      reason: "Examples use unwrap for brevity"
      expires: "2026-12-31"
"#;
        let config: PmatYamlConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.comply.suppressions.len(), 2);
        assert_eq!(config.comply.suppressions[0].rules, vec!["CB-954"]);
        assert_eq!(config.comply.suppressions[1].files, vec!["examples/**"]);
        assert_eq!(
            config.comply.suppressions[1].expires,
            Some("2026-12-31".to_string())
        );
    }

    #[test]
    fn test_suppression_returns_reason() {
        let config = ComplyConfig {
            suppressions: vec![SuppressionYamlRule {
                rules: vec!["CB-954".to_string()],
                files: vec![],
                reason: "LLM parameter, not a secret".to_string(),
                expires: None,
            }],
            ..Default::default()
        };
        let reason = config.is_suppressed("CB-954", "file.yaml");
        assert_eq!(reason, Some("LLM parameter, not a secret".to_string()));
    }

    #[test]
    fn test_suppression_multiple_rules() {
        let config = ComplyConfig {
            suppressions: vec![SuppressionYamlRule {
                rules: vec!["CB-501".to_string(), "CB-507".to_string()],
                files: vec![],
                reason: "Accepted risk".to_string(),
                expires: None,
            }],
            ..Default::default()
        };
        assert!(config.is_suppressed("CB-501", "any.rs").is_some());
        assert!(config.is_suppressed("CB-507", "any.rs").is_some());
        assert!(config.is_suppressed("CB-502", "any.rs").is_none());
    }

    #[test]
    fn test_scoring_plugin_yaml_parsing() {
        let yaml = r#"
scoring:
  custom_scores:
    - id: model-accuracy
      name: "APR Model Accuracy"
      command: "cargo test --test accuracy"
      max_score: 100.0
      min_score: 90.0
      severity: error
      weight: 2.0
    - id: inference-speed
      name: "Inference Speed"
      command: "cargo bench --bench inference"
      min_score: 50.0
"#;
        let config: PmatYamlConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.scoring.custom_scores.len(), 2);

        let first = &config.scoring.custom_scores[0];
        assert_eq!(first.id, "model-accuracy");
        assert_eq!(first.min_score, Some(90.0));
        assert_eq!(first.severity, CheckSeverity::Error);
        assert!((first.weight - 2.0).abs() < 0.001);

        let second = &config.scoring.custom_scores[1];
        assert_eq!(second.id, "inference-speed");
        assert_eq!(second.max_score, 100.0); // default
        assert!((second.weight - 1.0).abs() < 0.001); // default
    }

    #[test]
    fn test_default_config_has_scoring() {
        let config = PmatYamlConfig::default();
        assert!(config.scoring.custom_scores.is_empty());
    }

    #[test]
    fn test_default_min_tdg_grade_is_a() {
        let config = ComplyConfig::default();
        assert_eq!(config.thresholds.min_tdg_grade, "A");
    }

    #[test]
    fn test_cb200_default_severity_is_error() {
        let config = ComplyConfig::default();
        assert_eq!(config.get_severity("cb-200"), CheckSeverity::Error);
    }

    /// Display for ConfigError covers all three variants.
    #[test]
    fn test_config_error_display_all_variants() {
        let io = ConfigError::IoError("disk gone".to_string());
        assert_eq!(format!("{io}"), "IO error loading config: disk gone");

        let parse = ConfigError::ParseError("bad yaml".to_string());
        assert_eq!(format!("{parse}"), "Parse error in .pmat.yaml: bad yaml");

        let ser = ConfigError::SerializeError("non-utf8".to_string());
        assert_eq!(format!("{ser}"), "Serialization error: non-utf8");
    }

    /// ConfigError implements std::error::Error so it can be boxed as dyn Error.
    #[test]
    fn test_config_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(ConfigError::IoError("fail".to_string()));
        assert!(err.to_string().contains("fail"));
    }

    /// Private serde default helpers are exercised indirectly via deserialization
    /// of partial YAML that omits the matching fields. This forces serde to call
    /// each `default_*` helper to populate the missing value.
    #[test]
    fn test_serde_defaults_for_omitted_fields() {
        // YAML provides empty maps for each section so serde populates their
        // fields via the default_* helpers (rather than Default::default(),
        // which yields f64::default() == 0.0 and i64::default() == 0).
        let yaml = r#"
comply: {}
quality: {}
work: {}
"#;
        let cfg: PmatYamlConfig =
            serde_yaml_ng::from_str(yaml).expect("minimal yaml must deserialize");

        // default_tdg_score -> 70.0 (QualityConfig::min_tdg_score)
        assert!((cfg.quality.min_tdg_score - 70.0).abs() < 0.0001);
        // default_cache_warn_hours -> 1 (WorkConfig::cache_warn_hours)
        assert_eq!(cfg.work.cache_warn_hours, 1);
        // default_cache_block_hours -> 24 (WorkConfig::cache_block_hours)
        assert_eq!(cfg.work.cache_block_hours, 24);
    }

    /// PmatYamlConfig::default() flows through `default_true` and other helpers
    /// to produce the enabled-by-default check set. Spot-check several checks
    /// whose `enabled` field is populated via default_true.
    #[test]
    fn test_default_true_populates_check_enabled() {
        let cfg = PmatYamlConfig::default();
        // Every default check is enabled: default_true returns true.
        for id in ["cb-050", "cb-060", "cb-128", "cb-200"] {
            assert!(
                cfg.comply.is_check_enabled(id),
                "{id} should be enabled by default"
            );
        }
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cfg = PmatYamlConfig::default();

        cfg.save(temp_dir.path()).unwrap();

        let config_path = temp_dir.path().join(".pmat.yaml");
        assert!(config_path.exists());

        let loaded = PmatYamlConfig::load(temp_dir.path()).unwrap();
        assert_eq!(
            loaded.comply.thresholds.coverage,
            cfg.comply.thresholds.coverage
        );
    }

    #[test]
    fn test_load_from_path_missing_file_errors() {
        let temp_dir = tempfile::tempdir().unwrap();
        let missing = temp_dir.path().join("does-not-exist.yaml");

        let err = PmatYamlConfig::load_from_path(&missing).unwrap_err();
        assert!(matches!(err, ConfigError::IoError(_)));
    }

    #[test]
    fn test_load_from_path_malformed_yaml_errors() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("bad.yaml");
        // Non-mapping at top level fails PmatYamlConfig deserialization.
        std::fs::write(&path, "not: valid: yaml: here\n  broken").unwrap();

        let err = PmatYamlConfig::load_from_path(&path).unwrap_err();
        assert!(matches!(err, ConfigError::ParseError(_)));
    }

    #[test]
    fn test_load_prefers_pmat_yml_when_pmat_yaml_missing() {
        // Covers the `.pmat.yml` fallback branch in load().
        let temp_dir = tempfile::tempdir().unwrap();
        let yml_path = temp_dir.path().join(".pmat.yml");
        let cfg = PmatYamlConfig::default();
        std::fs::write(&yml_path, serde_yaml_ng::to_string(&cfg).unwrap()).unwrap();

        let loaded = PmatYamlConfig::load(temp_dir.path()).unwrap();
        assert_eq!(
            loaded.comply.thresholds.complexity,
            cfg.comply.thresholds.complexity
        );
    }
}
