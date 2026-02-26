// =============================================================================
// ComputeBrick check tests
// =============================================================================

#[test]
fn test_check_compute_brick_not_cb_project() {
    let temp_dir = TempDir::new().unwrap();
    // No probar/trueno/realizar deps or brick/ dir
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_compute_brick(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
    assert!(check.message.contains("Not a ComputeBrick project"));
}

#[test]
fn test_check_compute_brick_with_trueno() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\ntrueno = \"0.1\"",
    )
    .unwrap();
    std::fs::create_dir_all(temp_dir.path().join("src")).unwrap();
    std::fs::write(temp_dir.path().join("src").join("lib.rs"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_compute_brick(temp_dir.path());
    // Should not skip
    assert_ne!(check.status, CheckStatus::Skip);
}

// =============================================================================
// Cargo.lock check tests
// =============================================================================

#[test]
fn test_check_cargo_lock_present() {
    let temp_dir = TempDir::new().unwrap();
    // check_cargo_lock requires Cargo.toml to exist (otherwise Skip)
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();
    std::fs::write(temp_dir.path().join("Cargo.lock"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_cargo_lock(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("reproducible builds"));
}

#[test]
fn test_check_cargo_lock_missing() {
    let temp_dir = TempDir::new().unwrap();
    // check_cargo_lock requires Cargo.toml to exist (otherwise Skip)
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_cargo_lock(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Fail);
    assert!(check.message.contains("Missing Cargo.lock"));
}

// =============================================================================
// MSRV check tests
// =============================================================================

#[test]
fn test_check_msrv_defined() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nrust-version = \"1.70\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_msrv(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

#[test]
fn test_check_msrv_not_defined() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_msrv(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("No rust-version"));
}

#[test]
fn test_check_msrv_no_cargo_toml() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_msrv(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
}

// =============================================================================
// CI check tests
// =============================================================================

#[test]
fn test_check_ci_configured_github() {
    let temp_dir = TempDir::new().unwrap();
    let workflows_dir = temp_dir.path().join(".github").join("workflows");
    std::fs::create_dir_all(&workflows_dir).unwrap();
    std::fs::write(workflows_dir.join("ci.yml"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_ci_configured(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("GitHub Actions"));
}

#[test]
fn test_check_ci_configured_gitlab() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join(".gitlab-ci.yml"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_ci_configured(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("GitLab CI"));
}

#[test]
fn test_check_ci_configured_jenkins() {
    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("Jenkinsfile"), "").unwrap();

    let check = crate::cli::handlers::comply_handlers::check_ci_configured(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
    assert!(check.message.contains("Jenkins"));
}

#[test]
fn test_check_ci_configured_none() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_ci_configured(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Warn);
    assert!(check.message.contains("No CI configuration"));
}
