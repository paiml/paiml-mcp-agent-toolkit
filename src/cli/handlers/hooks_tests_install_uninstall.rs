// Install tests with temp directory

#[tokio::test]
async fn test_install_creates_hooks_directory() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    // Create minimal config file
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.install(false, false, false).await.unwrap();

    assert!(result.success);
    assert!(result.hook_created);
    assert!(hooks_dir.exists());
}

#[tokio::test]
async fn test_install_with_existing_non_pmat_hook_no_force() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\necho 'custom hook'",
    )
    .unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.install(false, false, false).await.unwrap();

    assert!(!result.success);
    assert!(!result.hook_created);
    assert!(result.message.contains("not PMAT-managed"));
}

#[tokio::test]
async fn test_install_with_force_overwrites() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\necho 'custom hook'",
    )
    .unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.install(true, false, false).await.unwrap();

    assert!(result.success);
    assert!(result.hook_created);
}

#[tokio::test]
async fn test_install_with_backup() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\n# auto-managed by PMAT\n# DO NOT EDIT",
    )
    .unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.install(false, true, false).await.unwrap();

    assert!(result.success);
    assert!(result.backup_created);
    assert!(hooks_dir.join("pre-commit.pmat-backup").exists());
}

#[tokio::test]
async fn test_install_backup_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\n# auto-managed by PMAT\n# DO NOT EDIT",
    )
    .unwrap();
    fs::write(
        hooks_dir.join("pre-commit.pmat-backup"),
        "#!/bin/bash\nold backup",
    )
    .unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.install(false, true, false).await.unwrap();

    assert!(result.success);
    assert!(!result.backup_created); // Backup already existed
}

// Uninstall tests

#[tokio::test]
async fn test_uninstall_no_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.uninstall(false).await.unwrap();

    assert!(result.success);
    assert!(!result.hook_removed);
    assert!(result.message.contains("No hook to uninstall"));
}

#[tokio::test]
async fn test_uninstall_non_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\necho 'custom hook'",
    )
    .unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.uninstall(false).await.unwrap();

    assert!(!result.success);
    assert!(!result.hook_removed);
    assert!(result.message.contains("not PMAT-managed"));
}

#[tokio::test]
async fn test_uninstall_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\n# auto-managed by PMAT\n# DO NOT EDIT",
    )
    .unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.uninstall(false).await.unwrap();

    assert!(result.success);
    assert!(result.hook_removed);
    assert!(!hooks_dir.join("pre-commit").exists());
}

#[tokio::test]
async fn test_uninstall_with_restore_backup() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\n# auto-managed by PMAT\n# DO NOT EDIT",
    )
    .unwrap();
    fs::write(
        hooks_dir.join("pre-commit.pmat-backup"),
        "#!/bin/bash\noriginal hook",
    )
    .unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.uninstall(true).await.unwrap();

    assert!(result.success);
    assert!(result.hook_removed);
    assert!(result.backup_restored);

    let content = fs::read_to_string(hooks_dir.join("pre-commit")).unwrap();
    assert!(content.contains("original hook"));
}
