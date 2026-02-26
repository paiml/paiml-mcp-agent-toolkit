// Refresh tests

#[tokio::test]
async fn test_refresh_no_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.refresh().await.unwrap();

    assert!(!result.success);
    assert!(!result.hook_updated);
    assert!(result.message.contains("No hook to refresh"));
}

#[tokio::test]
async fn test_refresh_non_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join("pre-commit"), "#!/bin/bash\necho 'custom'").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.refresh().await.unwrap();

    assert!(!result.success);
    assert!(!result.hook_updated);
    assert!(result.message.contains("not PMAT-managed"));
}

#[tokio::test]
async fn test_refresh_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join("pre-commit"), "#!/bin/bash\n# auto-managed by PMAT\n# DO NOT EDIT\n# Generated at: 2020-01-01 00:00:00\nold content").unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.refresh().await.unwrap();

    assert!(result.success);
    // Content will differ because of timestamp
    assert!(result.config_changes_detected);
}

// Run tests

#[tokio::test]
async fn test_run_no_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.run(false, false).await.unwrap();

    assert!(!result.success);
    assert!(result.output.contains("not installed"));
}

#[tokio::test]
async fn test_run_with_hook_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    // Create a simple hook that exits 0
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\necho '✅ passed'\nexit 0",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(hooks_dir.join("pre-commit"))
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(hooks_dir.join("pre-commit"), perms).unwrap();
    }

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.run(false, true).await.unwrap();

    assert!(result.success);
    assert_eq!(result.checks_passed, 1); // One ✅ in output
    assert_eq!(result.checks_failed, 0);
}

#[tokio::test]
async fn test_run_all_files_mode() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\necho 'running all'\nexit 0",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(hooks_dir.join("pre-commit"))
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(hooks_dir.join("pre-commit"), perms).unwrap();
    }

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.run(true, true).await.unwrap();

    assert!(result.success);
}
