// CB-950 YAML Best Practices tests (CB-950 through CB-954)
// Included from tests.rs via include!() - shares parent module scope

#[test]
fn test_cb950_no_yaml_files_empty() {
    let temp = TempDir::new().unwrap();
    let violations = detect_cb950_truthy_ambiguity(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb950_detects_truthy_string() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("config.yaml"),
        "name: my-app\nenabled: yes\nverbose: no\n",
    )
    .unwrap();
    let violations = detect_cb950_truthy_ambiguity(temp.path());
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].pattern_id, "CB-950");
}

#[test]
fn test_cb950_allows_quoted_truthy() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("config.yaml"),
        "name: my-app\nenabled: \"yes\"\nverbose: 'no'\n",
    )
    .unwrap();
    let violations = detect_cb950_truthy_ambiguity(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb951_detects_excessive_nesting() {
    let temp = TempDir::new().unwrap();
    // Create deeply nested YAML (10 levels)
    let mut content = String::new();
    for i in 0..10 {
        let indent = "  ".repeat(i);
        content.push_str(&format!("{}level{}:\n", indent, i));
    }
    fs::write(temp.path().join("deep.yaml"), &content).unwrap();
    let violations = detect_cb951_excessive_nesting(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-951");
}

#[test]
fn test_cb951_allows_moderate_nesting() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("config.yaml"),
        "level0:\n  level1:\n    level2:\n      value: ok\n",
    )
    .unwrap();
    let violations = detect_cb951_excessive_nesting(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb952_detects_missing_gha_fields() {
    let temp = TempDir::new().unwrap();
    let gha_dir = temp.path().join(".github").join("workflows");
    fs::create_dir_all(&gha_dir).unwrap();
    fs::write(
        gha_dir.join("ci.yml"),
        "# No name, no on, no jobs\nsteps:\n  - run: echo hi\n",
    )
    .unwrap();
    let violations = detect_cb952_missing_required_fields(temp.path());
    assert!(violations.len() >= 2); // missing name, on, jobs
}

#[test]
fn test_cb952_passes_valid_workflow() {
    let temp = TempDir::new().unwrap();
    let gha_dir = temp.path().join(".github").join("workflows");
    fs::create_dir_all(&gha_dir).unwrap();
    fs::write(
        gha_dir.join("ci.yml"),
        "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n",
    )
    .unwrap();
    let violations = detect_cb952_missing_required_fields(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb953_detects_unpinned_action() {
    let temp = TempDir::new().unwrap();
    let gha_dir = temp.path().join(".github").join("workflows");
    fs::create_dir_all(&gha_dir).unwrap();
    fs::write(
        gha_dir.join("ci.yml"),
        "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@main\n",
    )
    .unwrap();
    let violations = detect_cb953_unpinned_action(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-953");
}

#[test]
fn test_cb953_allows_pinned_action() {
    let temp = TempDir::new().unwrap();
    let gha_dir = temp.path().join(".github").join("workflows");
    fs::create_dir_all(&gha_dir).unwrap();
    fs::write(
        gha_dir.join("ci.yml"),
        "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n",
    )
    .unwrap();
    let violations = detect_cb953_unpinned_action(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb954_detects_plaintext_secret() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("config.yaml"),
        "database:\n  password: supersecret123\n  host: localhost\n",
    )
    .unwrap();
    let violations = detect_cb954_plaintext_secret(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-954");
    assert!(matches!(violations[0].severity, Severity::Error));
}

#[test]
fn test_cb954_allows_env_reference() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("config.yaml"),
        "database:\n  password: ${{ secrets.DB_PASSWORD }}\n  host: localhost\n",
    )
    .unwrap();
    let violations = detect_cb954_plaintext_secret(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_walkdir_yaml_files_skips_git() {
    let temp = TempDir::new().unwrap();
    let git_dir = temp.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(git_dir.join("config.yml"), "key: val\n").unwrap();
    fs::write(temp.path().join("real.yaml"), "key: val\n").unwrap();
    let files = walkdir_yaml_files(temp.path());
    assert_eq!(files.len(), 1);
}
