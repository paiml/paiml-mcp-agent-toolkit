// Serialization, Debug, Clone trait tests and batch/integration tests

#[test]
fn test_fix_serialization() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::DeadCodeRemoval,
        change: Change {
            before: "old".to_string(),
            after: "new".to_string(),
            line_range: (1, 10),
        },
        verify_command: Some("cargo check".to_string()),
        branch_name: "fix/test".to_string(),
    };

    let json = serde_json::to_string(&fix).unwrap();
    assert!(json.contains("test.rs"));
    assert!(json.contains("DeadCodeRemoval"));

    let deserialized: Fix = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.file, PathBuf::from("test.rs"));
    assert_eq!(deserialized.fix_type, FixType::DeadCodeRemoval);
}

#[test]
fn test_fix_debug() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::Formatting,
        change: Change {
            before: "".to_string(),
            after: "".to_string(),
            line_range: (1, 1),
        },
        verify_command: None,
        branch_name: "fix/test".to_string(),
    };
    let debug_str = format!("{:?}", fix);
    assert!(debug_str.contains("Fix"));
    assert!(debug_str.contains("test.rs"));
}

#[test]
fn test_fix_clone() {
    let fix = Fix {
        file: PathBuf::from("src/lib.rs"),
        fix_type: FixType::SimpleRefactor,
        change: Change {
            before: "code".to_string(),
            after: "better code".to_string(),
            line_range: (5, 15),
        },
        verify_command: Some("cargo test".to_string()),
        branch_name: "fix/refactor".to_string(),
    };
    let cloned = fix.clone();
    assert_eq!(cloned.file, fix.file);
    assert_eq!(cloned.fix_type, fix.fix_type);
    assert_eq!(cloned.branch_name, fix.branch_name);
}

#[test]
fn test_change_serialization() {
    let change = Change {
        before: "old code".to_string(),
        after: "new code".to_string(),
        line_range: (5, 10),
    };

    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("old code"));
    assert!(json.contains("new code"));

    let deserialized: Change = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.before, "old code");
    assert_eq!(deserialized.after, "new code");
}

#[test]
fn test_change_debug() {
    let change = Change {
        before: "before".to_string(),
        after: "after".to_string(),
        line_range: (1, 5),
    };
    let debug_str = format!("{:?}", change);
    assert!(debug_str.contains("Change"));
    assert!(debug_str.contains("before"));
}

#[test]
fn test_change_clone() {
    let change = Change {
        before: "original".to_string(),
        after: "modified".to_string(),
        line_range: (10, 20),
    };
    let cloned = change.clone();
    assert_eq!(cloned.before, change.before);
    assert_eq!(cloned.after, change.after);
    assert_eq!(cloned.line_range, change.line_range);
}

#[test]
fn test_fix_type_serialization() {
    let types = vec![
        FixType::DeadCodeRemoval,
        FixType::UnusedImportRemoval,
        FixType::Formatting,
        FixType::SimpleRefactor,
        FixType::DocumentationFix,
    ];

    for fix_type in types {
        let json = serde_json::to_string(&fix_type).unwrap();
        let deserialized: FixType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, fix_type);
    }
}

#[test]
fn test_fix_type_debug() {
    let fix_type = FixType::DocumentationFix;
    let debug_str = format!("{:?}", fix_type);
    assert!(debug_str.contains("DocumentationFix"));
}

#[test]
fn test_fix_type_clone() {
    let fix_type = FixType::UnusedImportRemoval;
    let cloned = fix_type.clone();
    assert_eq!(cloned, fix_type);
}

#[test]
fn test_automator_config_serialization() {
    let config = AutomatorConfig {
        enabled: true,
        require_review: false,
        safe_only: true,
        create_branches: false,
        auto_commit: true,
        max_batch_size: 20,
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("enabled"));
    assert!(json.contains("max_batch_size"));

    let deserialized: AutomatorConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.enabled);
    assert_eq!(deserialized.max_batch_size, 20);
}

#[test]
fn test_automator_config_debug() {
    let config = AutomatorConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("AutomatorConfig"));
    assert!(debug_str.contains("enabled"));
}

#[test]
fn test_automator_config_clone() {
    let config = AutomatorConfig {
        enabled: true,
        require_review: false,
        safe_only: false,
        create_branches: false,
        auto_commit: true,
        max_batch_size: 5,
    };
    let cloned = config.clone();
    assert_eq!(cloned.enabled, config.enabled);
    assert_eq!(cloned.max_batch_size, config.max_batch_size);
}

#[test]
fn test_automation_result_serialization() {
    let result = AutomationResult {
        successful: vec![],
        failed: vec![],
        pending_review: vec![],
        branch_name: Some("fix/auto".to_string()),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("successful"));
    assert!(json.contains("fix/auto"));

    let deserialized: AutomationResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.branch_name, Some("fix/auto".to_string()));
}

#[test]
fn test_automation_result_debug() {
    let result = AutomationResult {
        successful: vec![],
        failed: vec![],
        pending_review: vec![],
        branch_name: None,
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("AutomationResult"));
}

#[test]
fn test_automation_result_clone() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::Formatting,
        change: Change {
            before: "".to_string(),
            after: "".to_string(),
            line_range: (1, 1),
        },
        verify_command: None,
        branch_name: "fix".to_string(),
    };
    let result = AutomationResult {
        successful: vec![AppliedFix {
            fix: fix.clone(),
            verification_passed: true,
            commit_hash: None,
        }],
        failed: vec![],
        pending_review: vec![fix],
        branch_name: Some("test".to_string()),
    };
    let cloned = result.clone();
    assert_eq!(cloned.successful.len(), result.successful.len());
    assert_eq!(cloned.branch_name, result.branch_name);
}

#[test]
fn test_applied_fix_serialization() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::DeadCodeRemoval,
        change: Change {
            before: "".to_string(),
            after: "".to_string(),
            line_range: (1, 1),
        },
        verify_command: None,
        branch_name: "test".to_string(),
    };

    let applied = AppliedFix {
        fix,
        verification_passed: true,
        commit_hash: Some("abc123".to_string()),
    };

    let json = serde_json::to_string(&applied).unwrap();
    assert!(json.contains("verification_passed"));
    assert!(json.contains("abc123"));

    let deserialized: AppliedFix = serde_json::from_str(&json).unwrap();
    assert!(deserialized.verification_passed);
}

#[test]
fn test_applied_fix_debug() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::Formatting,
        change: Change {
            before: "".to_string(),
            after: "".to_string(),
            line_range: (1, 1),
        },
        verify_command: None,
        branch_name: "fix".to_string(),
    };
    let applied = AppliedFix {
        fix,
        verification_passed: false,
        commit_hash: None,
    };
    let debug_str = format!("{:?}", applied);
    assert!(debug_str.contains("AppliedFix"));
}

#[test]
fn test_applied_fix_clone() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::SimpleRefactor,
        change: Change {
            before: "".to_string(),
            after: "".to_string(),
            line_range: (1, 1),
        },
        verify_command: None,
        branch_name: "fix".to_string(),
    };
    let applied = AppliedFix {
        fix,
        verification_passed: true,
        commit_hash: Some("def456".to_string()),
    };
    let cloned = applied.clone();
    assert_eq!(cloned.verification_passed, applied.verification_passed);
    assert_eq!(cloned.commit_hash, applied.commit_hash);
}

#[test]
fn test_failed_fix_serialization() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::DeadCodeRemoval,
        change: Change {
            before: "".to_string(),
            after: "".to_string(),
            line_range: (1, 1),
        },
        verify_command: None,
        branch_name: "test".to_string(),
    };

    let failed = FailedFix {
        fix,
        error: "Compilation failed".to_string(),
        can_retry: false,
    };

    let json = serde_json::to_string(&failed).unwrap();
    assert!(json.contains("Compilation failed"));
    assert!(json.contains("can_retry"));

    let deserialized: FailedFix = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.error, "Compilation failed");
    assert!(!deserialized.can_retry);
}

#[test]
fn test_failed_fix_debug() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::Formatting,
        change: Change {
            before: "".to_string(),
            after: "".to_string(),
            line_range: (1, 1),
        },
        verify_command: None,
        branch_name: "fix".to_string(),
    };
    let failed = FailedFix {
        fix,
        error: "Error".to_string(),
        can_retry: true,
    };
    let debug_str = format!("{:?}", failed);
    assert!(debug_str.contains("FailedFix"));
}

#[test]
fn test_failed_fix_clone() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::DocumentationFix,
        change: Change {
            before: "".to_string(),
            after: "".to_string(),
            line_range: (1, 1),
        },
        verify_command: None,
        branch_name: "fix".to_string(),
    };
    let failed = FailedFix {
        fix,
        error: "Some error".to_string(),
        can_retry: true,
    };
    let cloned = failed.clone();
    assert_eq!(cloned.error, failed.error);
    assert_eq!(cloned.can_retry, failed.can_retry);
}

#[test]
fn test_safe_transform_debug() {
    let transform = SafeTransform {
        id: "test_transform".to_string(),
        name: "Test Transform".to_string(),
        handles: vec![ViolationType::DeadCode],
        success_rate: 1.0,
        transform: |_| Err(anyhow!("Not implemented")),
    };
    let debug_str = format!("{:?}", transform);
    assert!(debug_str.contains("SafeTransform"));
    assert!(debug_str.contains("test_transform"));
}

#[test]
fn test_safe_transform_clone() {
    let transform = SafeTransform {
        id: "clone_test".to_string(),
        name: "Clone Test".to_string(),
        handles: vec![ViolationType::UnusedImport, ViolationType::DeadCode],
        success_rate: 0.95,
        transform: |_| Err(anyhow!("Not implemented")),
    };
    let cloned = transform.clone();
    assert_eq!(cloned.id, transform.id);
    assert_eq!(cloned.name, transform.name);
    assert_eq!(cloned.handles.len(), 2);
}

#[test]
fn test_safe_transform_handles_multiple_types() {
    let transform = SafeTransform {
        id: "multi".to_string(),
        name: "Multi Handler".to_string(),
        handles: vec![
            ViolationType::DeadCode,
            ViolationType::UnusedImport,
            ViolationType::Formatting,
        ],
        success_rate: 1.0,
        transform: |_| Err(anyhow!("Not implemented")),
    };
    assert_eq!(transform.handles.len(), 3);
}

#[test]
fn test_fix_with_no_verify_command() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::DocumentationFix,
        change: Change {
            before: "old doc".to_string(),
            after: "new doc".to_string(),
            line_range: (1, 5),
        },
        verify_command: None,
        branch_name: "fix/docs".to_string(),
    };
    assert!(fix.verify_command.is_none());
}

#[tokio::test]
async fn test_batch_fix_empty_violations() {
    let config = AutomatorConfig {
        enabled: true,
        create_branches: false,
        require_review: false,
        ..Default::default()
    };
    let automator = ConservativeAutomator::new(config);

    let result = automator.batch_fix(vec![]).await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.successful.is_empty());
    assert!(result.failed.is_empty());
    assert!(result.pending_review.is_empty());
    assert!(result.branch_name.is_none());
}

#[test]
fn test_rollback_with_automator() {
    let config = AutomatorConfig::default();
    let mut automator = ConservativeAutomator::new(config);

    // Empty rollback should fail
    let result = automator.rollback();
    assert!(result.is_err());
}

#[test]
fn test_initialize_safe_transforms() {
    let config = AutomatorConfig::default();
    let automator = ConservativeAutomator::new(config);

    // Should have at least 2 transforms (dead code and unused import)
    assert!(automator.safe_transforms.len() >= 2);

    // All transforms should be safe (100% success rate)
    for t in &automator.safe_transforms {
        assert!((t.success_rate - 1.0).abs() < f64::EPSILON);
    }
}
