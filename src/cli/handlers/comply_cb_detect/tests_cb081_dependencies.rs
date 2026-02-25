// CB-081 Dependency Count + CB-400/401/402 bashrs integration tests
// Included from tests.rs via include!() - shares parent module scope

#[test]
fn test_cb081_detects_excessive_direct_deps() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml with many dependencies (>50)
    let mut deps =
        String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
    for i in 0..60 {
        deps.push_str(&format!("dep{} = \"1.0\"\n", i));
    }
    fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();
    fs::write(
        temp.path().join("Cargo.lock"),
        "[[package]]\nname = \"test\"",
    )
    .unwrap();

    let report = detect_cb081_dependency_count(temp.path());
    assert_eq!(report.direct_count, 60);
    assert_eq!(report.score, 0); // >50 direct = score 0
    assert!(!report.violations.is_empty());
    assert_eq!(report.violations[0].pattern_id, "CB-081-A");
}

#[test]
fn test_cb081_moderate_deps() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml with moderate dependencies (30-40)
    let mut deps =
        String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
    for i in 0..35 {
        deps.push_str(&format!("dep{} = \"1.0\"\n", i));
    }
    fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();

    // Create Cargo.lock with 180 packages (between 150-200)
    let mut lock = String::new();
    for _ in 0..180 {
        lock.push_str("[[package]]\nname = \"pkg\"\n");
    }
    fs::write(temp.path().join("Cargo.lock"), &lock).unwrap();

    let report = detect_cb081_dependency_count(temp.path());
    assert_eq!(report.direct_count, 35);
    assert_eq!(report.transitive_count, 180);
    assert_eq!(report.score, 3); // 30-40 direct, 150-200 transitive = 3
}

#[test]
fn test_cb081_low_deps_excellent() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml with few dependencies (<=20)
    let mut deps =
        String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
    for i in 0..15 {
        deps.push_str(&format!("dep{} = \"1.0\"\n", i));
    }
    fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();

    // Create Cargo.lock with few packages (<=100)
    let mut lock = String::new();
    for _ in 0..80 {
        lock.push_str("[[package]]\nname = \"pkg\"\n");
    }
    fs::write(temp.path().join("Cargo.lock"), &lock).unwrap();

    let report = detect_cb081_dependency_count(temp.path());
    assert_eq!(report.direct_count, 15);
    assert_eq!(report.transitive_count, 80);
    assert_eq!(report.score, 5); // <=20 direct, <=100 transitive = 5
    assert!(report.violations.is_empty());
}

#[test]
fn test_cb081_excludes_dev_dependencies() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml with few regular deps but many dev-deps
    let deps = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
anyhow = "1.0"

[dev-dependencies]
criterion = "0.5"
tempfile = "3.0"
proptest = "1.0"
quickcheck = "1.0"
tokio-test = "0.4"
"#;
    fs::write(temp.path().join("Cargo.toml"), deps).unwrap();
    fs::write(
        temp.path().join("Cargo.lock"),
        "[[package]]\nname = \"test\"",
    )
    .unwrap();

    let report = detect_cb081_dependency_count(temp.path());
    // Only counts [dependencies], not [dev-dependencies]
    assert_eq!(report.direct_count, 2);
}

#[test]
fn test_cb081_no_cargo_toml() {
    let temp = TempDir::new().unwrap();
    // No Cargo.toml

    let report = detect_cb081_dependency_count(temp.path());
    assert_eq!(report.direct_count, 0);
    assert_eq!(report.transitive_count, 0);
}

// =========================================================================
// CB-400/401/402 bashrs integration tests
// =========================================================================

#[test]
fn test_cb400_no_git_hooks_dir() {
    let temp = TempDir::new().unwrap();
    // No .git/hooks directory
    let violations = detect_cb400_git_hooks_quality(temp.path());
    assert!(violations.is_empty(), "No hooks dir should return empty");
}

#[test]
fn test_cb400_empty_git_hooks_dir() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".git/hooks")).unwrap();
    // Empty hooks dir - no hook files
    let violations = detect_cb400_git_hooks_quality(temp.path());
    assert!(violations.is_empty(), "Empty hooks dir should return empty");
}

#[test]
fn test_cb400_sample_hooks_ignored() {
    let temp = TempDir::new().unwrap();
    let hooks_dir = temp.path().join(".git/hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    // Sample hooks should be ignored
    fs::write(
        hooks_dir.join("pre-commit.sample"),
        "#!/bin/bash\necho test",
    )
    .unwrap();
    let violations = detect_cb400_git_hooks_quality(temp.path());
    assert!(violations.is_empty(), "Sample hooks should be ignored");
}

#[test]
fn test_cb401_no_makefile() {
    let temp = TempDir::new().unwrap();
    // No Makefile
    let violations = detect_cb401_makefile_quality(temp.path());
    assert!(violations.is_empty(), "No Makefile should return empty");
}

#[test]
fn test_cb402_no_shell_scripts() {
    let temp = TempDir::new().unwrap();
    // No shell scripts
    let violations = detect_cb402_shell_script_quality(temp.path());
    assert!(
        violations.is_empty(),
        "No shell scripts should return empty"
    );
}

#[test]
fn test_cb402_target_dir_excluded() {
    let temp = TempDir::new().unwrap();
    // Shell script in target/ should be ignored
    let target_dir = temp.path().join("target");
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(target_dir.join("test.sh"), "#!/bin/bash\necho test").unwrap();
    let violations = detect_cb402_shell_script_quality(temp.path());
    assert!(
        violations.is_empty(),
        "Scripts in target/ should be ignored"
    );
}

#[test]
fn test_parse_bashrs_json_array() {
    let json = r#"[{"code":"SC2086","message":"Double quote","line":5,"severity":"warning"}]"#;
    let result = parse_bashrs_json_output(json).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, "SC2086");
    assert_eq!(result[0].line, 5);
}

#[test]
fn test_parse_bashrs_json_object() {
    let json = r#"{"diagnostics":[{"code":"SC2046","message":"Quote this","line":3,"severity":"error"}]}"#;
    let result = parse_bashrs_json_output(json).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, "SC2046");
    assert_eq!(result[0].severity, "error");
}

#[test]
fn test_parse_bashrs_json_invalid() {
    let json = "not valid json";
    let result = parse_bashrs_json_output(json).unwrap();
    assert!(result.is_empty(), "Invalid JSON should return empty");
}

#[test]
fn test_parse_bashrs_json_empty_array() {
    let json = "[]";
    let result = parse_bashrs_json_output(json).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_bashrs_json_multiple_issues() {
    let json = r#"[
        {"code":"SC2086","message":"Double quote","line":5,"severity":"warning"},
        {"code":"SC2046","message":"Quote this","line":10,"severity":"error"},
        {"code":"SC2116","message":"Useless echo","line":15,"severity":"info"}
    ]"#;
    let result = parse_bashrs_json_output(json).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].code, "SC2086");
    assert_eq!(result[1].code, "SC2046");
    assert_eq!(result[2].code, "SC2116");
}
