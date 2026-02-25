// CB-700 SQL Best Practices tests (CB-700 through CB-705)
// Included from tests.rs via include!() - shares parent module scope

#[test]
fn test_cb700_no_sql_files_empty() {
    let temp = TempDir::new().unwrap();
    let violations = detect_cb700_select_star(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb700_detects_select_star() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("query.sql"),
        "SELECT * FROM users WHERE active = 1;\n",
    )
    .unwrap();
    let violations = detect_cb700_select_star(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-700");
}

#[test]
fn test_cb700_allows_count_star() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("query.sql"),
        "SELECT COUNT(*) FROM users;\n",
    )
    .unwrap();
    let violations = detect_cb700_select_star(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb700_allows_explicit_columns() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("query.sql"),
        "SELECT id, name, email FROM users;\n",
    )
    .unwrap();
    let violations = detect_cb700_select_star(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb701_detects_update_without_where() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("dangerous.sql"),
        "UPDATE users SET active = 0;\n",
    )
    .unwrap();
    let violations = detect_cb701_missing_where(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-701");
    assert!(matches!(violations[0].severity, Severity::Error));
}

#[test]
fn test_cb701_allows_update_with_where() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("safe.sql"),
        "UPDATE users SET active = 0 WHERE id = 5;\n",
    )
    .unwrap();
    let violations = detect_cb701_missing_where(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb701_detects_delete_without_where() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("dangerous.sql"), "DELETE FROM users;\n").unwrap();
    let violations = detect_cb701_missing_where(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-701");
}

#[test]
fn test_cb702_detects_implicit_join() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("query.sql"),
        "SELECT u.name FROM users u, orders o WHERE u.id = o.user_id;\n",
    )
    .unwrap();
    let violations = detect_cb702_implicit_join(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-702");
}

#[test]
fn test_cb702_allows_explicit_join() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("query.sql"),
        "SELECT u.name FROM users u JOIN orders o ON u.id = o.user_id;\n",
    )
    .unwrap();
    let violations = detect_cb702_implicit_join(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb704_detects_many_joins() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("query.sql"),
        "SELECT a.x FROM a JOIN b ON a.id = b.id JOIN c ON b.id = c.id JOIN d ON c.id = d.id JOIN e ON d.id = e.id;\n",
    )
    .unwrap();
    let violations = detect_cb704_missing_index_hint(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-704");
}

#[test]
fn test_sql_test_file_excluded() {
    let temp = TempDir::new().unwrap();
    let test_dir = temp.path().join("tests");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(test_dir.join("test_queries.sql"), "SELECT * FROM users;\n").unwrap();
    let violations = detect_cb700_select_star(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_compute_sql_production_lines() {
    let content = "-- This is a comment\nSELECT id FROM users; -- inline comment\n/* block */\nINSERT INTO t VALUES(1);\n";
    let lines = compute_sql_production_lines(content);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].1.contains("SELECT"));
    assert!(lines[1].1.contains("INSERT"));
}

#[test]
fn test_walkdir_sql_files_skips_git() {
    let temp = TempDir::new().unwrap();
    let git_dir = temp.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(git_dir.join("hooks.sql"), "SELECT 1;\n").unwrap();
    fs::write(temp.path().join("real.sql"), "SELECT 1;\n").unwrap();
    let files = walkdir_sql_files(temp.path());
    assert_eq!(files.len(), 1);
}

#[test]
fn test_cb705_detects_n_plus_1_query() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.py"),
        "users = db.query('SELECT * FROM users')\nfor user in users:\n    cursor.execute('SELECT * FROM orders WHERE user_id=' + str(user.id))\n",
    )
    .unwrap();
    let violations = detect_cb705_n_plus_1_query(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-705");
}

#[test]
fn test_cb705_no_false_positive_outside_loop() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.py"),
        "result = cursor.execute('SELECT * FROM users WHERE id = 1')\n",
    )
    .unwrap();
    let violations = detect_cb705_n_plus_1_query(temp.path());
    assert_eq!(violations.len(), 0);
}
