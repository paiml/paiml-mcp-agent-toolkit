// Formatter tests for format_text, format_text_with_code, format_markdown.
// Included via include!() from tests_part2.rs - no use imports allowed.

// ── Rich coverage metric tests (format_text_with_code) ──────────────────

#[test]
fn test_format_text_with_code_coverage_uncovered_rich() {
    let entry = create_test_entry("uncov_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "uncovered".to_string();
    result.lines_total = 25;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("0/25"), "missing uncovered line count");
}

#[test]
fn test_format_text_with_code_coverage_partial_low_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 30.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("30%"), "missing low coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_partial_mid_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 65.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("65%"), "missing mid coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_partial_high_rich() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 90.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("90%"), "missing high coverage pct");
}

#[test]
fn test_format_text_with_code_coverage_full_rich() {
    let entry = create_test_entry("full_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_status = "full".to_string();
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("100%"), "missing full coverage indicator");
}

#[test]
fn test_format_text_with_code_coverage_impact_rich() {
    let entry = create_test_entry("impact_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.impact_score = 4.2;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("4.2"), "missing impact score");
}

#[test]
fn test_format_text_with_code_coverage_diff_positive_rich() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_diff = 2.5;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("+2.5%"), "missing positive diff");
}

#[test]
fn test_format_text_with_code_coverage_diff_negative_rich() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, true);
    result.coverage_diff = -3.0;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("-3.0%"), "missing negative diff");
}

// ── Grade color branches in format_text ─────────────────────────────────

#[test]
fn test_format_text_grade_c() {
    let entry = create_test_entry("grade_c_fn", 15, 4.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "C".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("C"), "missing grade C");
}

#[test]
fn test_format_text_grade_d() {
    let entry = create_test_entry("grade_d_fn", 25, 6.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "D".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("D"), "missing grade D");
}

#[test]
fn test_format_text_grade_f() {
    let entry = create_test_entry("grade_f_fn", 50, 8.0);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.tdg_grade = "F".to_string();
    let text = format_text(&[result]);
    assert!(text.contains("F"), "missing grade F");
}

#[test]
fn test_format_text_with_satd() {
    let entry = create_test_entry("satd_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.satd_count = 3;
    let text = format_text(&[result]);
    assert!(text.contains("SATD: 3"), "missing SATD count");
}

#[test]
fn test_format_text_large_loc() {
    let entry = create_test_entry("big_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.loc = 100;
    let text = format_text(&[result]);
    assert!(text.contains("LOC: 100"), "missing LOC for large function");
}

#[test]
fn test_format_text_with_doc_comment() {
    let entry = create_test_entry("doc_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.doc_comment = Some("Important documentation".to_string());
    let text = format_text(&[result]);
    assert!(
        text.contains("Important documentation"),
        "missing doc comment"
    );
}

#[test]
fn test_format_text_summary_with_calls() {
    let entry = create_test_entry("caller_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.calls = vec!["foo".to_string(), "bar".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Calls:"), "missing calls label");
    assert!(text.contains("foo"), "missing call target");
}

#[test]
fn test_format_text_summary_with_called_by() {
    let entry = create_test_entry("callee_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.called_by = vec!["main".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Called by:"), "missing called_by label");
}

#[test]
fn test_format_text_summary_with_graph_metrics() {
    let entry = create_test_entry("graph_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.pagerank = 0.001;
    result.in_degree = 3;
    result.out_degree = 5;
    let text = format_text(&[result]);
    assert!(text.contains("PageRank"), "missing pagerank");
    assert!(text.contains("In-Degree: 3"), "missing in-degree");
}

#[test]
fn test_format_text_low_relevance() {
    let entry = create_test_entry("low_rel_fn", 5, 1.5);
    let result = QueryResult::from_entry(&entry, 0.1, false);
    let text = format_text(&[result]);
    assert!(text.contains("0.10"), "missing low relevance score");
}

#[test]
fn test_format_text_with_faults() {
    let entry = create_test_entry("fault_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.fault_annotations = vec!["BH001: Boundary at line 5".to_string()];
    let text = format_text(&[result]);
    assert!(text.contains("Boundary"), "missing fault annotation");
}

#[test]
fn test_format_text_summary_with_clones() {
    let entry = create_test_entry("clone_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.clone_count = 2;
    result.duplication_score = 0.85;
    let text = format_text(&[result]);
    assert!(text.contains("Clones: 2"), "missing clone count");
}

#[test]
fn test_format_text_with_repetitive_pattern() {
    let entry = create_test_entry("rep_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.pattern_diversity = 0.2;
    let text = format_text(&[result]);
    assert!(text.contains("Repetitive"), "missing repetitive label");
}

// ── Markdown detail branches ────────────────────────────────────────────

#[test]
fn test_format_markdown_with_doc_and_graph() {
    let entry = create_test_entry("md_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.doc_comment = Some("A documented function".to_string());
    result.calls = vec!["helper".to_string()];
    result.called_by = vec!["main".to_string()];
    result.pagerank = 0.002;
    result.in_degree = 4;
    result.out_degree = 1;
    let md = format_markdown(&[result]);
    assert!(md.contains("A documented function"), "missing doc in md");
    assert!(md.contains("Calls:"), "missing calls in md");
    assert!(md.contains("Called by:"), "missing called_by in md");
    assert!(md.contains("PageRank"), "missing graph in md");
}

// ── Call graph with only called_by (no calls) ───────────────────────────

#[test]
fn test_format_text_with_code_only_called_by() {
    let entry = create_test_entry("callee_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.called_by = vec!["a".to_string(), "b".to_string()];
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("← a, b"), "missing called_by in call graph");
}

// ── Highlight source with syntect (no highlight param) ──────────────────

#[test]
fn test_format_text_with_code_syntect_source() {
    let entry = create_test_entry("src_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.source = Some("fn src_fn() { let x = 42; }".to_string());
    let text = format_text_with_code(&[result], None);
    assert!(
        text.contains("src_fn"),
        "missing source content with syntect"
    );
    // This used to assert `text.contains("\x1b[")` and called it "ANSI codes
    // from syntect". `syntax-highlighting` is not a default feature, so what it
    // really pinned was the plain fallback's UNCONDITIONAL `\x1b[2m` line
    // number — the very leak that made `pmat query --color never` print
    // escapes. `cargo test` captures stdout, so colour is off and the printer
    // must be plain either way.
    assert!(
        !text.contains('\x1b'),
        "the code printer must be plain with colour off: {text:?}"
    );
}

// ── High churn in rich metrics (>0.7 branch) ────────────────────────────

#[test]
fn test_format_text_with_code_very_high_churn() {
    let entry = create_test_entry("hot_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.commit_count = 50;
    result.churn_score = 0.8;
    let text = format_text_with_code(&[result], None);
    assert!(text.contains("🔥"), "missing fire emoji for >0.7 churn");
    assert!(text.contains("50c"), "missing commit count");
}

// ── Pagerank metric with no star (below 1.0 threshold) ─────────────────

#[test]
fn test_format_text_with_code_low_pagerank_no_star() {
    let entry = create_test_entry("low_pr_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.9, true);
    result.pagerank = 0.00005; // scaled: 0.5 -> below 1.0 threshold
    let text = format_text_with_code(&[result], None);
    assert!(
        !text.contains("★"),
        "should not show star for very low pagerank"
    );
}
