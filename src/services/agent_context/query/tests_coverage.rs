// Coverage metric formatter tests.
// Lines 1357–1492 of the original tests.rs.

#[test]
fn test_format_markdown_coverage_uncovered() {
    let entry = create_test_entry("uncov_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_status = "uncovered".to_string();
    result.lines_total = 20;
    let md = format_markdown(&[result]);
    assert!(md.contains("Uncovered"), "missing uncovered label");
    assert!(md.contains("0/20"), "missing line counts");
}

#[test]
fn test_format_markdown_coverage_partial() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 60.0;
    result.lines_covered = 12;
    result.lines_total = 20;
    result.missed_lines = 8;
    let md = format_markdown(&[result]);
    assert!(md.contains("Coverage: 60%"), "missing partial coverage pct");
    assert!(md.contains("8 missed lines"), "missing missed lines");
}

#[test]
fn test_format_markdown_coverage_full() {
    let entry = create_test_entry("full_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_status = "full".to_string();
    result.lines_total = 15;
    let md = format_markdown(&[result]);
    assert!(md.contains("Fully covered"), "missing full coverage");
    assert!(md.contains("15 lines"), "missing line count");
}

#[test]
fn test_format_markdown_coverage_impact() {
    let entry = create_test_entry("impact_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.impact_score = 5.2;
    let md = format_markdown(&[result]);
    assert!(md.contains("Impact: 5.2"), "missing impact score");
}

#[test]
fn test_format_markdown_coverage_diff_positive() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_diff = 3.5;
    let md = format_markdown(&[result]);
    assert!(md.contains("+3.5%"), "missing positive coverage diff");
}

#[test]
fn test_format_markdown_coverage_diff_negative() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.8, false);
    result.coverage_diff = -2.1;
    let md = format_markdown(&[result]);
    assert!(md.contains("-2.1%"), "missing negative coverage diff");
}

#[test]
fn test_format_text_coverage_uncovered() {
    let entry = create_test_entry("uncov_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.1, false);
    result.coverage_status = "uncovered".to_string();
    result.lines_total = 30;
    let text = format_text(&[result]);
    assert!(text.contains("Uncovered"), "missing uncovered text");
    assert!(text.contains("0/30"), "missing line count");
}

#[test]
fn test_format_text_coverage_partial_low() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.1, false);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 30.0;
    result.lines_covered = 6;
    result.lines_total = 20;
    let text = format_text(&[result]);
    assert!(text.contains("Cov: 30%"), "missing partial low coverage");
}

#[test]
fn test_format_text_coverage_partial_mid() {
    let entry = create_test_entry("partial_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.coverage_status = "partial".to_string();
    result.line_coverage_pct = 65.0;
    result.lines_covered = 13;
    result.lines_total = 20;
    let text = format_text(&[result]);
    assert!(text.contains("Cov: 65%"), "missing partial mid coverage");
}

#[test]
fn test_format_text_coverage_full() {
    let entry = create_test_entry("full_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.coverage_status = "full".to_string();
    result.lines_total = 10;
    let text = format_text(&[result]);
    assert!(text.contains("Covered"), "missing full coverage text");
}

#[test]
fn test_format_text_coverage_impact() {
    let entry = create_test_entry("impact_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.impact_score = 3.7;
    let text = format_text(&[result]);
    assert!(text.contains("Impact: 3.7"), "missing impact score text");
}

#[test]
fn test_format_text_coverage_diff_positive() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.coverage_diff = 5.0;
    let text = format_text(&[result]);
    assert!(text.contains("+5.0%"), "missing positive diff text");
}

#[test]
fn test_format_text_coverage_diff_negative() {
    let entry = create_test_entry("diff_fn", 5, 1.5);
    let mut result = QueryResult::from_entry(&entry, 0.5, false);
    result.coverage_diff = -1.5;
    let text = format_text(&[result]);
    assert!(text.contains("-1.5%"), "missing negative diff text");
}
