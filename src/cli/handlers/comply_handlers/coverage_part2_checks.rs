    #[test]
    fn test_check_compute_brick_with_probar_dep() {
        let temp = create_temp_project();
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
probar = "0.1"
"#;
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");

        let check = check_compute_brick(temp.path());
        // Has probar but no .pmat-gates.toml
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("[compute-brick]"));
    }

    #[test]
    fn test_check_compute_brick_with_brick_dir() {
        let temp = create_temp_project();
        fs::create_dir_all(temp.path().join("src").join("brick"))
            .expect("Failed to create brick dir");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_compute_brick_fully_configured() {
        let temp = create_temp_project();
        fs::create_dir_all(temp.path().join("src").join("brick"))
            .expect("Failed to create brick dir");

        let gates_content = r#"
[compute-brick]
enabled = true
"#;
        fs::write(temp.path().join(".pmat-gates.toml"), gates_content)
            .expect("Failed to write gates");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_compute_brick_probar_without_coverage() {
        let temp = create_temp_project();
        let cargo_content = r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
probar = "0.1"
"#;
        fs::write(temp.path().join("Cargo.toml"), cargo_content)
            .expect("Failed to write Cargo.toml");

        let gates_content = r#"
[compute-brick]
enabled = true
"#;
        fs::write(temp.path().join(".pmat-gates.toml"), gates_content)
            .expect("Failed to write gates");

        let check = check_compute_brick(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("GUI coverage"));
    }

    // check_cargo_lock Tests

    #[test]
    fn test_check_cargo_lock_missing() {
        let temp = create_rust_project(false, false);
        let check = check_cargo_lock(temp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("Missing Cargo.lock"));
    }

    #[test]
    fn test_check_cargo_lock_present() {
        let temp = create_rust_project(false, true);
        let check = check_cargo_lock(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("reproducible builds"));
    }

    // check_msrv Tests

    #[test]
    fn test_check_msrv_no_cargo_toml() {
        let temp = create_temp_project();
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No Cargo.toml"));
    }

    #[test]
    fn test_check_msrv_missing() {
        let temp = create_rust_project(false, false);
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No rust-version"));
    }

    #[test]
    fn test_check_msrv_present() {
        let temp = create_rust_project(true, false);
        let check = check_msrv(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("rust-version field present"));
    }

    // check_ci_configured Tests

    #[test]
    fn test_check_ci_not_configured() {
        let temp = create_temp_project();
        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("No CI configuration"));
    }

    #[test]
    fn test_check_ci_github_actions() {
        let temp = create_temp_project();
        let workflows_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create workflows dir");
        fs::write(workflows_dir.join("ci.yml"), "name: CI").expect("Failed to write workflow");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GitHub Actions"));
    }

    #[test]
    fn test_check_ci_github_actions_empty() {
        let temp = create_temp_project();
        let workflows_dir = temp.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).expect("Failed to create workflows dir");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Warn);
    }

    #[test]
    fn test_check_ci_gitlab() {
        let temp = create_temp_project();
        fs::write(temp.path().join(".gitlab-ci.yml"), "stages:\n  - build")
            .expect("Failed to write gitlab-ci");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GitLab CI"));
    }

    #[test]
    fn test_check_ci_jenkins() {
        let temp = create_temp_project();
        fs::write(temp.path().join("Jenkinsfile"), "pipeline { }")
            .expect("Failed to write Jenkinsfile");

        let check = check_ci_configured(temp.path());
        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("Jenkins"));
    }

    // check_paiml_deps_workspace Tests

    #[test]
    fn test_check_paiml_deps_no_cargo_toml() {
        let temp = tempfile::tempdir().expect("Failed to create temp dir");
        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No Cargo.toml"));
    }

    #[test]
    fn test_check_paiml_deps_no_paiml_deps() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.status, CheckStatus::Skip);
        assert!(check.message.contains("No PAIML stack dependencies"));
    }

    #[test]
    fn test_check_paiml_deps_with_trueno() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
trueno = "0.11"
serde = "1.0"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        // Status depends on whether ~/src/trueno exists and its git state
        // But check name should always be correct
        assert_eq!(check.name, "PAIML Deps Workspace");
    }

    #[test]
    fn test_check_paiml_deps_with_multiple_paiml_deps() {
        let temp = create_temp_project();
        fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
trueno = "0.11"
trueno-graph = "0.1"
aprender = "0.24"
"#,
        )
        .expect("Failed to write Cargo.toml");

        let check = check_paiml_deps_workspace(temp.path());
        assert_eq!(check.name, "PAIML Deps Workspace");
        // Message should mention PAIML deps count or dirty status
        assert!(
            check.message.contains("PAIML") || check.message.contains("dirty"),
            "Expected message about PAIML deps, got: {}",
            check.message
        );
    }
