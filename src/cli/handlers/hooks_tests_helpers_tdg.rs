// Helper function tests

#[test]
fn test_is_pmat_managed_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.is_pmat_managed(&hooks_dir.join("pre-commit")).unwrap();

    assert!(!result);
}

#[test]
fn test_is_pmat_managed_custom_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join("pre-commit"), "#!/bin/bash\necho custom").unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.is_pmat_managed(&hooks_dir.join("pre-commit")).unwrap();

    assert!(!result);
}

#[test]
fn test_is_pmat_managed_pmat_hook() {
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
    let result = cmd.is_pmat_managed(&hooks_dir.join("pre-commit")).unwrap();

    assert!(result);
}

#[test]
fn test_is_pmat_managed_partial_marker() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    fs::create_dir_all(&hooks_dir).unwrap();
    // Only has one of the two required markers
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/bin/bash\n# auto-managed by PMAT\necho test",
    )
    .unwrap();

    let cmd = HooksCommand::new(hooks_dir.clone(), config_path);
    let result = cmd.is_pmat_managed(&hooks_dir.join("pre-commit")).unwrap();

    assert!(!result);
}

#[test]
fn test_generate_hook_header() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let header = cmd.generate_hook_header();

    assert!(header.contains("#!/bin/bash"));
    assert!(header.contains("auto-managed by PMAT"));
    assert!(header.contains("DO NOT EDIT"));
    assert!(header.contains("set -e"));
}

#[test]
fn test_generate_quality_checks() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let checks = cmd.generate_quality_checks();

    assert!(checks.contains("pmat analyze complexity"));
    assert!(checks.contains("pmat analyze satd"));
    assert!(checks.contains("exit 0"));
}

#[test]
fn test_generate_config_content() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    let cmd = HooksCommand::new(hooks_dir, config_path);
    let config = cmd.generate_config_content(15, 20, 85, 3);

    assert!(config.contains("max_complexity = 15"));
    assert!(config.contains("max_cognitive_complexity = 20"));
    assert!(config.contains("min_coverage = 85"));
    assert!(config.contains("max_satd_comments = 3"));
    assert!(config.contains("[quality]"));
    assert!(config.contains("[hooks]"));
}

#[test]
fn test_extract_current_value() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    let cmd = HooksCommand::new(hooks_dir, config_path);

    let content = "max_complexity = 20\nmax_cognitive_complexity = 25";
    assert_eq!(cmd.extract_current_value(content, "max_complexity"), "20");
    assert_eq!(
        cmd.extract_current_value(content, "max_cognitive_complexity"),
        "25"
    );
    assert_eq!(cmd.extract_current_value(content, "nonexistent"), "10"); // default
}

#[test]
fn test_update_config_values() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    let config_path = temp_dir.path().join("pmat.toml");

    let cmd = HooksCommand::new(hooks_dir, config_path);

    let content = "max_complexity = 10\nmax_cognitive_complexity = 15\nmin_coverage = 80";
    let updated = cmd.update_config_values(content, 20, 30, 90, 5);

    assert!(updated.contains("max_complexity = 20"));
    assert!(updated.contains("max_cognitive_complexity = 30"));
    assert!(updated.contains("min_coverage = 90"));
}

// TDG hooks tests

#[tokio::test]
async fn test_install_tdg_hooks_no_git() {
    let temp_dir = TempDir::new().unwrap();
    // No .git directory

    let result = install_tdg_hooks(temp_dir.path()).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Not a git repository"));
}

#[tokio::test]
async fn test_install_tdg_hooks_success() {
    let temp_dir = TempDir::new().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();

    let result = install_tdg_hooks(temp_dir.path()).await;
    assert!(result.is_ok());

    // Check hooks were created
    assert!(git_dir.join("hooks").join("pre-commit").exists());
    assert!(git_dir.join("hooks").join("post-commit").exists());

    // Config is loaded from defaults when file doesn't exist, so file may not be created
    // The function uses TdgHooksConfig::load which returns Ok(default) when file is missing
}

#[test]
fn test_install_tdg_pre_commit_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    let config = TdgHooksConfig::default();
    let result = install_tdg_pre_commit_hook(&hooks_dir, &config);
    assert!(result.is_ok());

    let hook_path = hooks_dir.join("pre-commit");
    assert!(hook_path.exists());

    let content = fs::read_to_string(&hook_path).unwrap();
    assert!(content.contains("PMAT TDG Enforcement"));
    assert!(content.contains("B+")); // Default min grade
}

#[test]
fn test_install_tdg_post_commit_hook() {
    let temp_dir = TempDir::new().unwrap();
    let hooks_dir = temp_dir.path().join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    let config = TdgHooksConfig::default();
    let result = install_tdg_post_commit_hook(&hooks_dir, &config);
    assert!(result.is_ok());

    let hook_path = hooks_dir.join("post-commit");
    assert!(hook_path.exists());

    let content = fs::read_to_string(&hook_path).unwrap();
    assert!(content.contains("PMAT TDG Baseline Auto-Update"));
    assert!(content.contains(".pmat/baseline.json")); // Default baseline path
}
