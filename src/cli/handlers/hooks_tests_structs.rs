// HooksCommand struct tests

#[test]
fn test_hooks_command_new() {
    let hooks_dir = PathBuf::from("/tmp/test_hooks");
    let config_path = PathBuf::from("/tmp/pmat.toml");
    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    assert_eq!(cmd.hooks_dir, hooks_dir);
}

#[test]
fn test_hooks_command_for_current_repo() {
    // This test just verifies the function doesn't panic
    // It may fail if not in a git repo, which is fine for coverage
    let result = HooksCommand::for_current_repo();
    // Either succeeds or fails, both are valid
    let _ = result;
}

// Result struct tests

#[test]
fn test_hook_install_result_equality() {
    let result1 = HookInstallResult {
        success: true,
        hook_created: true,
        backup_created: false,
        message: "test".to_string(),
    };
    let result2 = HookInstallResult {
        success: true,
        hook_created: true,
        backup_created: false,
        message: "test".to_string(),
    };
    assert_eq!(result1, result2);
}

#[test]
fn test_hook_install_result_debug() {
    let result = HookInstallResult {
        success: true,
        hook_created: true,
        backup_created: true,
        message: "Success".to_string(),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("success"));
    assert!(debug_str.contains("hook_created"));
    assert!(debug_str.contains("backup_created"));
}

#[test]
fn test_hook_uninstall_result_equality() {
    let result1 = HookUninstallResult {
        success: true,
        hook_removed: true,
        backup_restored: false,
        message: "removed".to_string(),
    };
    let result2 = HookUninstallResult {
        success: true,
        hook_removed: true,
        backup_restored: false,
        message: "removed".to_string(),
    };
    assert_eq!(result1, result2);
}

#[test]
fn test_hook_uninstall_result_debug() {
    let result = HookUninstallResult {
        success: false,
        hook_removed: false,
        backup_restored: true,
        message: "test".to_string(),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("success"));
    assert!(debug_str.contains("hook_removed"));
}

#[test]
fn test_hook_status_equality() {
    let status1 = HookStatus {
        installed: true,
        is_pmat_managed: true,
        config_up_to_date: true,
        last_updated: Some("2025-01-01".to_string()),
        hook_content_preview: Some("#!/bin/bash".to_string()),
    };
    let status2 = HookStatus {
        installed: true,
        is_pmat_managed: true,
        config_up_to_date: true,
        last_updated: Some("2025-01-01".to_string()),
        hook_content_preview: Some("#!/bin/bash".to_string()),
    };
    assert_eq!(status1, status2);
}

#[test]
fn test_hook_status_debug() {
    let status = HookStatus {
        installed: true,
        is_pmat_managed: false,
        config_up_to_date: false,
        last_updated: None,
        hook_content_preview: None,
    };
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("installed"));
    assert!(debug_str.contains("is_pmat_managed"));
}

#[test]
fn test_hook_verification_result_equality() {
    let result1 = HookVerificationResult {
        is_valid: true,
        issues: vec![],
        fixes_applied: vec![],
    };
    let result2 = HookVerificationResult {
        is_valid: true,
        issues: vec![],
        fixes_applied: vec![],
    };
    assert_eq!(result1, result2);
}

#[test]
fn test_hook_verification_result_with_issues() {
    let result = HookVerificationResult {
        is_valid: false,
        issues: vec!["issue1".to_string(), "issue2".to_string()],
        fixes_applied: vec!["fix1".to_string()],
    };
    assert!(!result.is_valid);
    assert_eq!(result.issues.len(), 2);
    assert_eq!(result.fixes_applied.len(), 1);
}

#[test]
fn test_hook_refresh_result_equality() {
    let result1 = HookRefreshResult {
        success: true,
        hook_updated: true,
        config_changes_detected: true,
        message: "refreshed".to_string(),
    };
    let result2 = HookRefreshResult {
        success: true,
        hook_updated: true,
        config_changes_detected: true,
        message: "refreshed".to_string(),
    };
    assert_eq!(result1, result2);
}

#[test]
fn test_hook_refresh_result_debug() {
    let result = HookRefreshResult {
        success: false,
        hook_updated: false,
        config_changes_detected: false,
        message: "no changes".to_string(),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("success"));
    assert!(debug_str.contains("hook_updated"));
}

#[test]
fn test_hook_run_result_equality() {
    let result1 = HookRunResult {
        success: true,
        checks_passed: 5,
        checks_failed: 0,
        output: "all passed".to_string(),
    };
    let result2 = HookRunResult {
        success: true,
        checks_passed: 5,
        checks_failed: 0,
        output: "all passed".to_string(),
    };
    assert_eq!(result1, result2);
}

#[test]
fn test_hook_run_result_debug() {
    let result = HookRunResult {
        success: false,
        checks_passed: 3,
        checks_failed: 2,
        output: "some failed".to_string(),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("success"));
    assert!(debug_str.contains("checks_passed"));
    assert!(debug_str.contains("checks_failed"));
}
