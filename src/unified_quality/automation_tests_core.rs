// Core automation tests: creation, auto_fix, rollback manager

#[test]
fn test_conservative_automator_creation() {
    let config = AutomatorConfig::default();
    let automator = ConservativeAutomator::new(config);
    assert!(!automator.safe_transforms.is_empty());
}

#[tokio::test]
async fn test_auto_fix_dead_code() {
    let config = AutomatorConfig {
        enabled: true,
        ..Default::default()
    };
    let automator = ConservativeAutomator::new(config);

    let violation = Violation {
        file: "test.rs".to_string(),
        violation_type: ViolationType::DeadCode,
        severity: crate::unified_quality::metrics::Severity::Low,
        value: 1.0,
        threshold: 0.0,
    };

    let result = automator.auto_fix(&violation).await;
    assert!(result.is_ok());

    let fix = result.unwrap();
    assert_eq!(fix.fix_type, FixType::DeadCodeRemoval);
}

#[test]
fn test_rollback_manager() {
    let mut manager = RollbackManager::new();
    manager.add_rollback_point(
        "main".to_string(),
        "abc123".to_string(),
        vec![PathBuf::from("test.rs")],
    );
    assert_eq!(manager.rollback_points.len(), 1);
}

#[test]
fn test_automator_config_default() {
    let config = AutomatorConfig::default();

    assert!(!config.enabled);
    assert!(config.require_review);
    assert!(config.safe_only);
    assert!(!config.create_branches); // DISABLED per CLAUDE.md zero-branching policy
    assert!(!config.auto_commit);
    assert_eq!(config.max_batch_size, 10);
}

#[test]
fn test_fix_type_variants() {
    let dead_code = FixType::DeadCodeRemoval;
    let unused_import = FixType::UnusedImportRemoval;
    let formatting = FixType::Formatting;
    let refactor = FixType::SimpleRefactor;
    let doc_fix = FixType::DocumentationFix;

    assert_eq!(dead_code, FixType::DeadCodeRemoval);
    assert_eq!(unused_import, FixType::UnusedImportRemoval);
    assert_eq!(formatting, FixType::Formatting);
    assert_eq!(refactor, FixType::SimpleRefactor);
    assert_eq!(doc_fix, FixType::DocumentationFix);
}

#[test]
fn test_change_creation() {
    let change = Change {
        before: "old code".to_string(),
        after: "new code".to_string(),
        line_range: (5, 10),
    };

    assert_eq!(change.before, "old code");
    assert_eq!(change.after, "new code");
    assert_eq!(change.line_range, (5, 10));
}

#[test]
fn test_fix_creation() {
    let fix = Fix {
        file: PathBuf::from("test.rs"),
        fix_type: FixType::DeadCodeRemoval,
        change: Change {
            before: "old".to_string(),
            after: "new".to_string(),
            line_range: (1, 1),
        },
        verify_command: Some("cargo check".to_string()),
        branch_name: "fix/test".to_string(),
    };

    assert_eq!(fix.file, PathBuf::from("test.rs"));
    assert_eq!(fix.fix_type, FixType::DeadCodeRemoval);
    assert!(fix.verify_command.is_some());
}

#[tokio::test]
async fn test_auto_fix_disabled() {
    let config = AutomatorConfig {
        enabled: false,
        ..Default::default()
    };
    let automator = ConservativeAutomator::new(config);

    let violation = Violation {
        file: "test.rs".to_string(),
        violation_type: ViolationType::DeadCode,
        severity: crate::unified_quality::metrics::Severity::Low,
        value: 1.0,
        threshold: 0.0,
    };

    let result = automator.auto_fix(&violation).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("disabled"));
}

#[tokio::test]
async fn test_auto_fix_unused_import() {
    let config = AutomatorConfig {
        enabled: true,
        create_branches: false,
        ..Default::default()
    };
    let automator = ConservativeAutomator::new(config);

    let violation = Violation {
        file: "test.rs".to_string(),
        violation_type: ViolationType::UnusedImport,
        severity: crate::unified_quality::metrics::Severity::Low,
        value: 1.0,
        threshold: 0.0,
    };

    let result = automator.auto_fix(&violation).await;
    assert!(result.is_ok());

    let fix = result.unwrap();
    assert_eq!(fix.fix_type, FixType::UnusedImportRemoval);
}

#[tokio::test]
async fn test_auto_fix_formatting() {
    let config = AutomatorConfig {
        enabled: true,
        create_branches: false,
        ..Default::default()
    };
    let automator = ConservativeAutomator::new(config);

    let violation = Violation {
        file: "test.rs".to_string(),
        violation_type: ViolationType::Formatting,
        severity: crate::unified_quality::metrics::Severity::Low,
        value: 1.0,
        threshold: 0.0,
    };

    let result = automator.auto_fix(&violation).await;
    assert!(result.is_ok());

    let fix = result.unwrap();
    assert_eq!(fix.fix_type, FixType::Formatting);
}

#[tokio::test]
async fn test_auto_fix_requires_review() {
    let config = AutomatorConfig {
        enabled: true,
        create_branches: false,
        ..Default::default()
    };
    let automator = ConservativeAutomator::new(config);

    let violation = Violation {
        file: "test.rs".to_string(),
        violation_type: ViolationType::Complexity,
        severity: crate::unified_quality::metrics::Severity::High,
        value: 25.0,
        threshold: 20.0,
    };

    let result = automator.auto_fix(&violation).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("review"));
}

#[test]
fn test_get_safe_transforms() {
    let config = AutomatorConfig::default();
    let automator = ConservativeAutomator::new(config);

    let transforms = automator.get_safe_transforms();
    assert!(!transforms.is_empty());

    // Verify all transforms have 100% success rate
    for transform in &transforms {
        assert_eq!(transform.success_rate, 1.0);
    }
}

#[test]
fn test_safe_transform_creation() {
    let transform = SafeTransform {
        id: "test".to_string(),
        name: "Test Transform".to_string(),
        handles: vec![ViolationType::DeadCode],
        success_rate: 1.0,
        transform: |_violation| {
            Ok(Fix {
                file: PathBuf::from("test.rs"),
                fix_type: FixType::DeadCodeRemoval,
                change: Change {
                    before: "".to_string(),
                    after: "".to_string(),
                    line_range: (1, 1),
                },
                verify_command: None,
                branch_name: "test".to_string(),
            })
        },
    };

    assert_eq!(transform.id, "test");
    assert_eq!(transform.success_rate, 1.0);
    assert!(!transform.handles.is_empty());
}

#[test]
fn test_automation_result_creation() {
    let result = AutomationResult {
        successful: vec![],
        failed: vec![],
        pending_review: vec![],
        branch_name: Some("fix/test".to_string()),
    };

    assert!(result.successful.is_empty());
    assert!(result.failed.is_empty());
    assert!(result.pending_review.is_empty());
    assert_eq!(result.branch_name, Some("fix/test".to_string()));
}

#[test]
fn test_applied_fix_creation() {
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

    assert!(applied.verification_passed);
    assert_eq!(applied.commit_hash, Some("abc123".to_string()));
}

#[test]
fn test_failed_fix_creation() {
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
        error: "Test error".to_string(),
        can_retry: true,
    };

    assert_eq!(failed.error, "Test error");
    assert!(failed.can_retry);
}

#[test]
fn test_rollback_manager_empty() {
    let mut manager = RollbackManager::new();
    let result = manager.rollback_last();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No rollback points"));
}

#[test]
fn test_rollback_manager_max_history() {
    let mut manager = RollbackManager::new();

    // Add more than max_history points
    for i in 0..15 {
        manager.add_rollback_point(
            format!("branch-{}", i),
            format!("commit-{}", i),
            vec![PathBuf::from(format!("file-{}.rs", i))],
        );
    }

    // Should only keep max_history (10) points
    assert_eq!(manager.rollback_points.len(), 10);
}
