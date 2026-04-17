// CB-900 Markdown Best Practices tests (CB-900 through CB-904)
// Included from tests.rs via include!() - shares parent module scope

#[test]
fn test_cb900_no_md_files_empty() {
    let temp = TempDir::new().unwrap();
    let violations = detect_cb900_broken_internal_link(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb900_detects_broken_link() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("README.md"),
        "# Hello\n\nSee [docs](./nonexistent.md) for more.\n",
    )
    .unwrap();
    let violations = detect_cb900_broken_internal_link(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-900");
}

#[test]
fn test_cb900_allows_valid_link() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("other.md"), "# Other\n").unwrap();
    fs::write(
        temp.path().join("README.md"),
        "# Hello\n\nSee [docs](./other.md) for more.\n",
    )
    .unwrap();
    let violations = detect_cb900_broken_internal_link(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb900_skips_http_links() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("README.md"),
        "# Hello\n\n[link](https://example.com)\n",
    )
    .unwrap();
    let violations = detect_cb900_broken_internal_link(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb901_detects_heading_skip() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("doc.md"),
        "# Title\n\n### Subsection\n\nContent here.\n",
    )
    .unwrap();
    let violations = detect_cb901_heading_hierarchy_skip(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-901");
    assert!(violations[0].description.contains("h1 to h3"));
}

#[test]
fn test_cb901_allows_proper_hierarchy() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("doc.md"),
        "# Title\n\n## Section\n\n### Subsection\n",
    )
    .unwrap();
    let violations = detect_cb901_heading_hierarchy_skip(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb902_detects_missing_alt_text() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("doc.md"), "# Title\n\n![](image.png)\n").unwrap();
    let violations = detect_cb902_missing_alt_text(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-902");
}

#[test]
fn test_cb902_allows_alt_text() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("doc.md"),
        "# Title\n\n![A diagram](image.png)\n",
    )
    .unwrap();
    let violations = detect_cb902_missing_alt_text(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb903_detects_bare_url() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("doc.md"),
        "# Title\n\nhttps://example.com\n",
    )
    .unwrap();
    let violations = detect_cb903_bare_url(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-903");
}

#[test]
fn test_cb903_allows_markdown_link() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("doc.md"),
        "# Title\n\n[Example](https://example.com)\n",
    )
    .unwrap();
    let violations = detect_cb903_bare_url(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb904_detects_long_line() {
    let temp = TempDir::new().unwrap();
    let long_line = "x".repeat(150);
    fs::write(
        temp.path().join("doc.md"),
        format!("# Title\n\n{}\n", long_line),
    )
    .unwrap();
    let violations = detect_cb904_long_line(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-904");
}

#[test]
fn test_cb904_allows_code_blocks() {
    let temp = TempDir::new().unwrap();
    let long_line = "x".repeat(150);
    fs::write(
        temp.path().join("doc.md"),
        format!("# Title\n\n```\n{}\n```\n", long_line),
    )
    .unwrap();
    let violations = detect_cb904_long_line(temp.path());
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_cb904_allows_tables() {
    let temp = TempDir::new().unwrap();
    let long_table = format!("| {} | {} |", "cell".repeat(30), "data".repeat(30));
    fs::write(
        temp.path().join("doc.md"),
        format!("# Title\n\n{}\n", long_table),
    )
    .unwrap();
    let violations = detect_cb904_long_line(temp.path());
    assert_eq!(violations.len(), 0);
}

/// GH-278: `.pmatignore` with `assistant/docs/**` must skip vendored
/// dependency documentation so CB-900 does not flag their internal links.
#[test]
fn test_cb900_pmatignore_excludes_vendored_docs() {
    let temp = TempDir::new().unwrap();
    let vendor = temp.path().join("assistant/docs/dependencies/lock_api");
    fs::create_dir_all(&vendor).unwrap();
    fs::write(
        vendor.join("index.md"),
        "# Lock\n\nSee [Mutex](./crate::Mutex) for more.\n",
    )
    .unwrap();
    fs::write(temp.path().join(".pmatignore"), "assistant/docs/**\n").unwrap();
    let violations = detect_cb900_broken_internal_link(temp.path());
    assert!(
        violations.is_empty(),
        "expected no violations under .pmatignore, got {violations:?}"
    );
}

/// GH-278: `.pmat-gates.toml [exclude] paths` provides an alternative to
/// `.pmatignore` for users who prefer keeping all comply configuration
/// in the gates file.
#[test]
fn test_cb900_gates_exclude_paths_skips_vendored_docs() {
    let temp = TempDir::new().unwrap();
    let vendor = temp.path().join("assistant/docs/foldhash");
    fs::create_dir_all(&vendor).unwrap();
    fs::write(
        vendor.join("quality.md"),
        "# FoldHasher\n\nSee [Hasher](./FoldHasher.md).\n",
    )
    .unwrap();
    fs::write(
        temp.path().join(".pmat-gates.toml"),
        "[exclude]\npaths = [\"assistant/docs/**\"]\n",
    )
    .unwrap();
    let violations = detect_cb900_broken_internal_link(temp.path());
    assert!(violations.is_empty());
}

/// GH-278: `.pmat.yaml comply.thresholds.file_health_exclude` (added in
/// GH-292) should also apply to CB-9xx Markdown checks so one list covers
/// both file-health and broken-link analyses.
#[test]
fn test_cb900_pmat_yaml_file_health_exclude_skips_vendored_docs() {
    let temp = TempDir::new().unwrap();
    let vendor = temp.path().join("assistant/docs/foldhash");
    fs::create_dir_all(&vendor).unwrap();
    fs::write(
        vendor.join("index.md"),
        "# FoldHasher\n\nSee [Hasher](./missing.md).\n",
    )
    .unwrap();
    fs::write(
        temp.path().join(".pmat.yaml"),
        "comply:\n  thresholds:\n    file_health_exclude:\n      - \"assistant/docs/**\"\n",
    )
    .unwrap();
    let violations = detect_cb900_broken_internal_link(temp.path());
    assert!(violations.is_empty());
}
