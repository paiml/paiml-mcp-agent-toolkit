// CB-130 Agent Context Adoption tests
// Included from tests.rs via include!() - shares parent module scope

#[test]
fn test_cb130_no_index() {
    let temp = TempDir::new().unwrap();
    let report = detect_cb130_agent_context_adoption(temp.path());

    assert!(!report.index_exists);
    assert!(report.index_age_hours.is_none());
    assert!(!report.index_stale);
    assert_eq!(report.function_count, 0);
    assert!(!report.claude_md_configured);
}

#[test]
fn test_cb130_with_claude_md_pmat_query() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("CLAUDE.md"),
        "# Instructions\n\nUse `pmat query` for code search.\n",
    )
    .unwrap();

    let report = detect_cb130_agent_context_adoption(temp.path());
    assert!(report.claude_md_configured);
}

#[test]
fn test_cb130_with_claude_md_mcp_tool() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("CLAUDE.md"),
        "# Instructions\n\nUse pmat_query_code tool for search.\n",
    )
    .unwrap();

    let report = detect_cb130_agent_context_adoption(temp.path());
    assert!(report.claude_md_configured);
}

#[test]
fn test_cb130_claude_md_no_mention() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("CLAUDE.md"),
        "# Instructions\n\nJust some generic instructions.\n",
    )
    .unwrap();

    let report = detect_cb130_agent_context_adoption(temp.path());
    assert!(!report.claude_md_configured);
}

#[test]
fn test_cb130_with_index_file() {
    let temp = TempDir::new().unwrap();

    // Create .pmat directory and a dummy index file
    let pmat_dir = temp.path().join(".pmat");
    fs::create_dir_all(&pmat_dir).unwrap();
    // Write some bytes - it won't deserialize but index_exists is checked first
    fs::write(pmat_dir.join("context.idx"), b"dummy").unwrap();

    let report = detect_cb130_agent_context_adoption(temp.path());
    assert!(report.index_exists);
    // function_count will be 0 because the index can't be loaded
    assert_eq!(report.function_count, 0);
}

#[test]
fn test_cb130_required_patterns_missing() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("CLAUDE.md"),
        "# Instructions\n\nUse pmat query for search.\n",
    )
    .unwrap();

    let report = detect_cb130_agent_context_adoption(temp.path());
    // "pmat query" is present but "NEVER use grep" is missing
    assert!(report.claude_md_configured);
    assert!(report
        .missing_required_patterns
        .contains(&"NEVER use grep".to_string()));
}

#[test]
fn test_cb130_all_required_patterns_present() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("CLAUDE.md"),
        "# Instructions\n\nNEVER use grep for code search.\nUse `pmat query --faults` instead.\n",
    )
    .unwrap();

    let report = detect_cb130_agent_context_adoption(temp.path());
    assert!(report.claude_md_configured);
    assert!(report.missing_required_patterns.is_empty());
}

#[test]
fn test_cb130_forbidden_pattern_detected() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("CLAUDE.md"),
        "# Instructions\n\nSearch with: grep -r \"error\" src/\n",
    )
    .unwrap();

    let report = detect_cb130_agent_context_adoption(temp.path());
    assert!(!report.forbidden_patterns_found.is_empty());
    assert_eq!(report.forbidden_patterns_found[0].pattern, "grep -r");
    assert_eq!(report.forbidden_patterns_found[0].line, 3);
}

#[test]
fn test_cb130_forbidden_pattern_in_negative_example_allowed() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("CLAUDE.md"),
        "# Instructions\n\n# BAD - Don't do this: grep -r \"error\" src/\n",
    )
    .unwrap();

    let report = detect_cb130_agent_context_adoption(temp.path());
    // Should not flag "grep -r" when it's in a negative example context
    assert!(report.forbidden_patterns_found.is_empty());
}
