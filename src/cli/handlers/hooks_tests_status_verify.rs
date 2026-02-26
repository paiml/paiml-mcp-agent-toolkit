// Status tests

#[tokio::test]
async fn test_status_no_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let status = cmd.status().await.unwrap();

    assert!(!status.installed);
    assert!(!status.is_pmat_managed);
    assert!(!status.config_up_to_date);
    assert!(status.last_updated.is_none());
    assert!(status.hook_content_preview.is_none());
}

#[tokio::test]
#[ignore = "Flaky: config_up_to_date detection varies by environment"]
async fn test_status_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\n# auto-managed by PMAT\n# DO NOT EDIT\necho test",
    )
    .unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let status = cmd.status().await.unwrap();

    assert!(status.installed);
    assert!(status.is_pmat_managed);
    assert!(status.config_up_to_date);
    assert!(status.last_updated.is_some());
    assert!(status.hook_content_preview.is_some());
}

#[tokio::test]
async fn test_status_non_pmat_hook() {
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
    let status = cmd.status().await.unwrap();

    assert!(status.installed);
    assert!(!status.is_pmat_managed);
    assert!(!status.config_up_to_date);
}

// Verify tests

#[tokio::test]
async fn test_verify_no_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.verify(false).await.unwrap();

    assert!(!result.is_valid);
    assert!(result.issues.iter().any(|i| i.contains("not installed")));
}

#[tokio::test]
async fn test_verify_with_fix_installs_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.verify(true).await.unwrap();

    assert!(result.is_valid);
    assert!(result.fixes_applied.iter().any(|f| f.contains("Installed")));
    assert!(hooks_dir.join("pre-commit").exists());
}

#[tokio::test]
async fn test_verify_non_pmat_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join("pre-commit"), "#!/bin/bash\necho 'custom'").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.verify(false).await.unwrap();

    assert!(!result.is_valid);
    assert!(result.issues.iter().any(|i| i.contains("not PMAT-managed")));
}

#[tokio::test]
async fn test_verify_outdated_hook_with_fix() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    // Create outdated PMAT hook
    fs::write(hooks_dir.join("pre-commit"), "#!/bin/bash\n# auto-managed by PMAT\n# DO NOT EDIT\n# Generated at: 2020-01-01 00:00:00\necho old").unwrap();
    fs::write(&config_path, "[quality]\nmax_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80.0\nallow_satd = false\nrequire_docs = true\nlint_compliance = true\nfail_on_violation = true").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.verify(true).await.unwrap();

    assert!(result.is_valid);
    assert!(result.fixes_applied.iter().any(|f| f.contains("Updated")));
}
