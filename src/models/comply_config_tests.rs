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

    // ── PMAT-630: a partial `checks:` map is a PATCH, never a replacement ──
    //
    // `checks` carried plain `#[serde(default)]`, so serde built the map from
    // the YAML alone and `ComplyConfig::default()`'s 22 declared severities were
    // never consulted. A `.pmat.yaml` naming seven ids therefore demoted every
    // other rule to `Warning` — including CB-2100's own `Error` — and
    // `error_severity_rules` came back empty, which is the "absence rendered as
    // success" defect one level down from the rule that hunts it.

    /// An id ABSENT from a partial `checks:` map keeps its declared severity.
    #[test]
    fn test_partial_checks_map_merges_over_declared_defaults() {
        let yaml = r#"
comply:
  checks:
    cb-125:
      enabled: false
"#;
        let cfg: PmatYamlConfig = serde_yaml_ng::from_str(yaml).expect("partial map must parse");

        // The listed id still takes effect.
        assert!(!cfg.comply.is_check_enabled("cb-125"));

        // Everything the map does NOT name keeps what the code declared.
        assert_eq!(
            cfg.comply.get_severity("cb-2100"),
            CheckSeverity::Error,
            "listing one unrelated id must not demote CB-2100 to Warning"
        );
        assert_eq!(cfg.comply.get_severity("cb-050"), CheckSeverity::Critical);
        assert_eq!(cfg.comply.get_severity("cb-200"), CheckSeverity::Error);
        assert_eq!(cfg.comply.get_severity("cb-124"), CheckSeverity::Error);
        assert_eq!(cfg.comply.get_severity("cb-123"), CheckSeverity::Info);

        // Declared thresholds survive too.
        assert_eq!(cfg.comply.get_threshold("cb-124"), Some(80.0));

        // The whole declared roster is still there.
        assert_eq!(
            cfg.comply.checks.len(),
            ComplyConfig::default().checks.len(),
            "a partial map must not shrink the roster"
        );
    }

    /// A check ENTRY that sets only `enabled:` is also a patch: omitting
    /// `severity:` keeps the declared one rather than resetting it to the
    /// `CheckSeverity` enum default.
    #[test]
    fn test_partial_check_entry_keeps_declared_severity_and_threshold() {
        let yaml = r#"
comply:
  checks:
    cb-2100:
      enabled: false
    cb-124:
      enabled: true
"#;
        let cfg: PmatYamlConfig = serde_yaml_ng::from_str(yaml).expect("partial entry must parse");

        assert!(!cfg.comply.is_check_enabled("cb-2100"));
        assert_eq!(
            cfg.comply.get_severity("cb-2100"),
            CheckSeverity::Error,
            "`enabled: false` alone must not also silently demote the severity"
        );
        assert_eq!(
            cfg.comply.get_threshold("cb-124"),
            Some(80.0),
            "`enabled: true` alone must not silently erase the declared threshold"
        );
    }

    /// An explicit `severity:` still wins — merging must not make the config
    /// unwritable.
    #[test]
    fn test_explicit_severity_still_overrides_the_declared_default() {
        let yaml = r#"
comply:
  checks:
    cb-2100:
      severity: info
    cb-070:
      severity: critical
      threshold: 3.0
"#;
        let cfg: PmatYamlConfig = serde_yaml_ng::from_str(yaml).expect("explicit map must parse");
        assert_eq!(cfg.comply.get_severity("cb-2100"), CheckSeverity::Info);
        assert_eq!(cfg.comply.get_severity("cb-070"), CheckSeverity::Critical);
        assert_eq!(cfg.comply.get_threshold("cb-070"), Some(3.0));
        // Omitting `enabled:` leaves it enabled, as declared.
        assert!(cfg.comply.is_check_enabled("cb-2100"));
    }

    /// A `comply:` section with no `checks:` key at all kept NOTHING before:
    /// field-level `#[serde(default)]` yields an empty map, not the declared
    /// roster. Same demotion, reached without writing a single check id.
    #[test]
    fn test_comply_section_without_checks_keeps_declared_severities() {
        let yaml = "comply:\n  fail_fast: true\n";
        let cfg: PmatYamlConfig = serde_yaml_ng::from_str(yaml).expect("must parse");
        assert!(cfg.comply.fail_fast);
        assert_eq!(cfg.comply.get_severity("cb-2100"), CheckSeverity::Error);
        assert_eq!(
            cfg.comply.checks.len(),
            ComplyConfig::default().checks.len()
        );
    }

    /// An id the roster does not declare is still accepted and still patchable —
    /// merging adds, it does not restrict.
    #[test]
    fn test_undeclared_id_is_added_by_the_patch() {
        let yaml = r#"
comply:
  checks:
    cb-1703:
      severity: error
"#;
        let cfg: PmatYamlConfig = serde_yaml_ng::from_str(yaml).expect("must parse");
        assert_eq!(cfg.comply.get_severity("cb-1703"), CheckSeverity::Error);
        assert!(cfg.comply.is_check_enabled("cb-1703"));
        assert_eq!(cfg.comply.get_severity("cb-2100"), CheckSeverity::Error);
    }

    /// A misspelled key inside a check entry must be a parse ERROR. Ignoring it
    /// is how `severty: error` silently leaves a rule at Warning — the same
    /// silent demotion this whole change exists to remove.
    #[test]
    fn test_unknown_key_in_a_check_entry_is_a_parse_error() {
        let yaml = r#"
comply:
  checks:
    cb-2100:
      severty: error
"#;
        let result: Result<PmatYamlConfig, _> = serde_yaml_ng::from_str(yaml);
        assert!(
            result.is_err(),
            "a typo'd key must fail loudly, not resolve to the default severity"
        );
    }

    /// The shape this repository's own `.pmat.yaml` has: several ids, every one
    /// of them only disabling a check. Loaded from disk, through
    /// `PmatYamlConfig::load`, because that is the path `pmat comply` takes.
    #[test]
    fn test_repo_shaped_partial_yaml_keeps_the_error_severities() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join(".pmat.yaml"),
            "comply:\n  checks:\n    cb-125:\n      enabled: false\n    cb-081:\n      enabled: false\n    cb-120:\n      enabled: false\n",
        )
        .expect("write fixture");

        let cfg = PmatYamlConfig::load(tmp.path()).expect("fixture must load");
        let error_ids: Vec<&str> = ["cb-050", "cb-060", "cb-124", "cb-200", "cb-302", "cb-1100", "cb-2100"]
            .into_iter()
            .filter(|id| {
                matches!(
                    cfg.comply.get_severity(id),
                    CheckSeverity::Error | CheckSeverity::Critical
                )
            })
            .collect();
        assert_eq!(
            error_ids.len(),
            7,
            "disabling three unrelated checks emptied the severity=error roster: {error_ids:?}"
        );
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

    /// comply_config_impls.rs:87 — `std::fs::read_to_string(path).map_err(IoError)`
    /// fires when the target path does not exist. Task #140 (roundtrip) only
    /// exercises the success arm of read_to_string.
    #[test]
    fn test_load_from_path_returns_io_error_for_missing_file() {
        let missing =
            std::path::Path::new("/tmp/pmat_load_from_path_nonexistent_0xC0FFEE_xyz.yaml");
        let result = PmatYamlConfig::load_from_path(missing);
        match result {
            Err(ConfigError::IoError(_)) => {}
            other => panic!("expected IoError for missing path, got {other:?}"),
        }
    }

    /// comply_config_impls.rs:89 — `serde_yaml_ng::from_str(...).map_err(ParseError)`
    /// fires when the file content is syntactically invalid YAML.
    #[test]
    fn test_load_from_path_returns_parse_error_for_malformed_yaml() {
        let tmp = tempfile::NamedTempFile::new().expect("create tempfile");
        // Unbalanced braces + tab indent: not valid YAML.
        std::fs::write(tmp.path(), "comply: {checks: [cb-050\n\tbroken: :")
            .expect("write malformed yaml");

        let result = PmatYamlConfig::load_from_path(tmp.path());
        match result {
            Err(ConfigError::ParseError(_)) => {}
            other => panic!("expected ParseError for malformed yaml, got {other:?}"),
        }
    }

    /// comply_config_impls.rs:94 — `PmatYamlConfig::save` happy path.
    /// Writes the serialized YAML to `<project_path>/.pmat.yaml` and a fresh
    /// `load_from_path` of the same file returns an equivalent config.
    #[test]
    fn test_save_roundtrip_to_temp_dir() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let cfg = PmatYamlConfig::default();

        cfg.save(tmp.path()).expect("save must succeed in tempdir");

        let yaml_path = tmp.path().join(".pmat.yaml");
        assert!(yaml_path.exists(), ".pmat.yaml must be created by save()");

        let reloaded = PmatYamlConfig::load_from_path(&yaml_path)
            .expect("saved file must be parseable as PmatYamlConfig");
        assert_eq!(
            reloaded.comply.thresholds.coverage,
            cfg.comply.thresholds.coverage,
            "round-trip must preserve coverage threshold",
        );
        assert_eq!(
            reloaded.comply.thresholds.complexity,
            cfg.comply.thresholds.complexity,
            "round-trip must preserve complexity threshold",
        );
    }

    /// comply_config_impls.rs:99 — `std::fs::write(...).map_err(IoError)` fires
    /// when the target path cannot be created (e.g. parent directory is missing
    /// and not auto-created by `save`). `save` does not create the parent, so
    /// pointing at a non-existent nested directory triggers the IoError arm.
    #[test]
    fn test_save_returns_io_error_when_parent_missing() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let missing_parent = tmp.path().join("does").join("not").join("exist");
        let cfg = PmatYamlConfig::default();

        let result = cfg.save(&missing_parent);
        match result {
            Err(ConfigError::IoError(_)) => {}
            other => panic!("expected IoError when parent missing, got {other:?}"),
        }
    }
}
