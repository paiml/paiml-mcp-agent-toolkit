// Handle function tests

#[test]
fn test_print_installed_status() {
    let status = HookStatus {
        installed: true,
        is_pmat_managed: true,
        config_up_to_date: true,
        last_updated: Some("2025-01-01 12:00:00".to_string()),
        hook_content_preview: Some("#!/bin/bash\necho test".to_string()),
    };
    // Just verify it doesn't panic
    print_installed_status(&status);
}

#[test]
fn test_print_installed_status_not_managed() {
    let status = HookStatus {
        installed: true,
        is_pmat_managed: false,
        config_up_to_date: false,
        last_updated: None,
        hook_content_preview: None,
    };
    // Just verify it doesn't panic
    print_installed_status(&status);
}

#[test]
fn test_print_verification_issues_empty() {
    let result = HookVerificationResult {
        is_valid: true,
        issues: vec![],
        fixes_applied: vec![],
    };
    // Just verify it doesn't panic
    print_verification_issues(&result);
}

#[test]
fn test_print_verification_issues_with_issues() {
    let result = HookVerificationResult {
        is_valid: false,
        issues: vec!["Issue 1".to_string(), "Issue 2".to_string()],
        fixes_applied: vec![],
    };
    // Just verify it doesn't panic
    print_verification_issues(&result);
}

#[test]
fn test_print_verification_fixes_empty() {
    let result = HookVerificationResult {
        is_valid: true,
        issues: vec![],
        fixes_applied: vec![],
    };
    // Just verify it doesn't panic
    print_verification_fixes(&result);
}

#[test]
fn test_print_verification_fixes_with_fixes() {
    let result = HookVerificationResult {
        is_valid: true,
        issues: vec![],
        fixes_applied: vec!["Fix 1".to_string(), "Fix 2".to_string()],
    };
    // Just verify it doesn't panic
    print_verification_fixes(&result);
}

// Handle command integration tests

#[tokio::test]
async fn test_handle_hooks_command_init() {
    let cmd = HooksCommands::Init {
        interactive: false,
        force: false,
        backup: false,
        tdg_enforcement: false,
    };
    // This may fail depending on environment, but shouldn't panic
    let _ = handle_hooks_command(&cmd).await;
}

#[tokio::test]
async fn test_handle_hooks_command_uninstall() {
    let cmd = HooksCommands::Uninstall {
        restore_backup: false,
    };
    // This may fail depending on environment, but shouldn't panic
    let _ = handle_hooks_command(&cmd).await;
}

#[tokio::test]
async fn test_handle_hooks_command_refresh() {
    let cmd = HooksCommands::Refresh;
    // This may fail depending on environment, but shouldn't panic
    let _ = handle_hooks_command(&cmd).await;
}

#[tokio::test]
async fn test_handle_hooks_command_run() {
    let cmd = HooksCommands::Run {
        all_files: false,
        verbose: false,
        cache: true,
    };
    // This may fail depending on environment, but shouldn't panic
    let _ = handle_hooks_command(&cmd).await;
}

#[tokio::test]
async fn test_handle_hooks_command_run_verbose() {
    let cmd = HooksCommands::Run {
        all_files: true,
        verbose: true,
        cache: false,
    };
    // This may fail depending on environment, but shouldn't panic
    let _ = handle_hooks_command(&cmd).await;
}

// Normalize content tests

#[test]
fn test_normalize_hook_content_empty() {
    let normalized = HooksCommand::normalize_hook_content("");
    assert_eq!(normalized, "");
}

#[test]
fn test_normalize_hook_content_no_timestamp() {
    let content = "#!/bin/bash\necho test\nexit 0";
    let normalized = HooksCommand::normalize_hook_content(content);
    assert_eq!(normalized, content);
}

#[test]
fn test_normalize_hook_content_multiple_timestamps() {
    let content = "#!/bin/bash\n# Generated at: 2025-01-01\necho test\n# Generated at: 2025-01-02\nexit 0";
    let normalized = HooksCommand::normalize_hook_content(content);
    assert!(!normalized.contains("Generated at:"));
    assert!(normalized.contains("#!/bin/bash"));
    assert!(normalized.contains("echo test"));
}

// Detect project type tests

#[test]
#[ignore] // Flaky - CWD changes in parallel tests
fn test_detect_project_type_unknown() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    // Change to temp directory to test detection
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let project_type = cmd.detect_project_type();

    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(project_type, "Unknown");
}

#[test]
#[ignore] // Flaky - CWD changes in parallel tests
fn test_detect_project_type_rust() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    // Create Cargo.toml
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"",
    )
    .unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let project_type = cmd.detect_project_type();

    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(project_type, "Rust");
}

#[test]
#[ignore] // Flaky - CWD changes in parallel tests
fn test_detect_project_type_javascript() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    // Create package.json
    fs::write(temp_dir.path().join("package.json"), "{}").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let project_type = cmd.detect_project_type();

    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(project_type, "JavaScript/TypeScript");
}

#[test]
#[ignore] // Flaky - CWD changes in parallel tests
fn test_detect_project_type_python_pyproject() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    // Create pyproject.toml
    fs::write(
        temp_dir.path().join("pyproject.toml"),
        "[project]\nname = \"test\"",
    )
    .unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let project_type = cmd.detect_project_type();

    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(project_type, "Python");
}

#[test]
#[ignore] // Flaky - CWD changes in parallel tests
fn test_detect_project_type_python_setup() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    // Create setup.py
    fs::write(
        temp_dir.path().join("setup.py"),
        "from setuptools import setup",
    )
    .unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let project_type = cmd.detect_project_type();

    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(project_type, "Python");
}

#[test]
#[ignore] // Flaky - CWD changes in parallel tests
fn test_detect_project_type_go() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    // Create go.mod
    fs::write(temp_dir.path().join("go.mod"), "module example.com/test").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let project_type = cmd.detect_project_type();

    std::env::set_current_dir(original_dir).unwrap();

    assert_eq!(project_type, "Go");
}

// Edge case tests

#[tokio::test]
async fn test_install_existing_pmat_hook_no_force() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    // Already a PMAT hook
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\n# auto-managed by PMAT\n# DO NOT EDIT\nold content",
    )
    .unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.install(false, false, false).await.unwrap();

    // Should succeed because it's already PMAT-managed
    assert!(result.success);
    assert!(result.hook_created);
}

#[tokio::test]
async fn test_verify_valid_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    // First install
    let cmd = HooksCommand::new(hooks_dir.clone(), config_path.clone());
    let _ = cmd.install(false, false, false).await.unwrap();

    // Then verify
    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.verify(false).await.unwrap();

    assert!(result.is_valid);
    assert!(result.issues.is_empty());
}

#[tokio::test]
async fn test_refresh_already_up_to_date() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    // Install fresh
    let cmd = HooksCommand::new(hooks_dir.clone(), config_path.clone());
    let _ = cmd.install(false, false, false).await.unwrap();

    // Refresh immediately - content should be "the same" (except timestamp)
    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.refresh().await.unwrap();

    assert!(result.success);
    // Timestamps will differ so it'll detect changes, but that's expected
}
