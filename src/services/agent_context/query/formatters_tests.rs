// Unit tests for query result formatters.
// Included into formatters.rs -- do NOT add `use` imports or `#!` inner attributes here.

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(name: &str, doc: Option<&str>) -> QueryResult {
        QueryResult {
            function_name: name.to_string(),
            file_path: "src/test.rs".to_string(),
            signature: format!("fn {}()", name),
            definition_type: "function".to_string(),
            doc_comment: doc.map(|s| s.to_string()),
            start_line: 1,
            end_line: 10,
            language: "Rust".to_string(),
            tdg_score: 80.0,
            tdg_grade: "A".to_string(),
            complexity: 5,
            big_o: "O(1)".to_string(),
            satd_count: 0,
            loc: 10,
            relevance_score: 0.95,
            source: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            pagerank: 0.0,
            in_degree: 0,
            out_degree: 0,
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            duplication_score: 0.0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
            line_coverage_pct: 0.0,
            lines_covered: 0,
            lines_total: 0,
            missed_lines: 0,
            impact_score: 0.0,
            coverage_status: String::new(),
            coverage_diff: 0.0,
            coverage_exclusion: Default::default(),
            coverage_excluded: false,
            cross_project_callers: 0,
            io_classification: String::new(),
            io_patterns: Vec::new(),
            suggested_module: String::new(),
            contract_level: None,
            contract_equation: None,
        }
    }

    /// Regression test for #157: UTF-8 multi-byte char boundary panic
    #[test]
    fn test_format_text_with_code_multibyte_doc_comment() {
        let result = make_result(
            "verify_output",
            Some("Verify output is correct: not empty, no garbage, contains expected answer (PMAT-QA-PROTOCOL-001 §7.5)  Order of checks is CRITICAL for safety"),
        );
        let output = format_text_with_code(&[result], None);
        assert!(output.contains("verify_output"));
        assert!(output.contains("..."));
    }

    #[test]
    fn test_format_text_short_doc_no_truncation() {
        let result = make_result("foo", Some("Short doc"));
        let output = format_text_with_code(&[result], None);
        assert!(output.contains("Short doc"));
    }

    // These highlight/colour tests used to assert that a raw escape appeared in
    // the output — i.e. they pinned the defect that `--color never` could not
    // turn colour off. Colour SELECTION (which sequence a tier picks) is now
    // asserted on the pure constants; colour EMISSION is asserted to be absent,
    // because `cargo test` captures stdout so `colors_enabled()` is false.

    #[test]
    fn test_highlight_matches_literal() {
        let line = "let result = unwrap();";
        let out = highlight_matches_in_line(line, "unwrap()", false);
        assert!(out.contains("unwrap()"), "missing matched text");
        assert!(
            !out.contains('\x1b'),
            "highlight must be plain with colour off, got {out:?}"
        );
        // …and the sequence it would use when colour is on.
        assert_eq!(BG_YELLOW_BOLD.raw(), "\x1b[1;43m");
        assert_eq!(RESET.raw(), "\x1b[0m");
    }

    #[test]
    fn test_highlight_matches_literal_case_insensitive() {
        let line = "fn HandleRequest() {}";
        let out = highlight_matches_in_line(line, "handlerequest", false);
        // Should highlight preserving original case
        assert!(out.contains("HandleRequest"));
        assert!(!out.contains('\x1b'), "plain with colour off: {out:?}");
    }

    #[test]
    fn test_highlight_matches_regex() {
        let line = "fn handle_request(ctx: Context) {}";
        let out = highlight_matches_in_line(line, r"fn\s+handle_\w+", true);
        assert!(out.contains("fn handle_request"));
        assert!(!out.contains('\x1b'), "plain with colour off: {out:?}");
    }

    #[test]
    fn test_highlight_matches_no_match() {
        let line = "let x = 42;";
        let out = highlight_matches_in_line(line, "nonexistent", false);
        assert_eq!(out, line);
    }

    #[test]
    fn test_highlight_matches_invalid_regex() {
        let line = "some text here";
        let out = highlight_matches_in_line(line, "[invalid", true);
        assert_eq!(out, line);
    }

    #[test]
    fn test_format_text_with_code_literal_highlight() {
        let mut result = make_result("test_fn", None);
        result.source = Some("fn test_fn() {\n    unwrap();\n}".to_string());
        let output = format_text_with_code(&[result], Some(("unwrap()", false)));
        assert!(
            output.contains("unwrap()"),
            "missing matched text in output"
        );
        assert!(
            !output.contains('\x1b'),
            "the code printer must be plain with colour off: {output:?}"
        );
        // Should have line numbers in highlight mode
        assert!(output.contains("\u{2502}"), "missing line number separator");
    }

    // ─────────────────────────────────────────────────────────────────────
    // formatters_helpers.rs: coverage metric + truncation + rich-metric
    // helpers (387 lines, 0 prior tests).
    // ─────────────────────────────────────────────────────────────────────

    fn result_with_coverage(status: &str, pct: f32, total: u32) -> QueryResult {
        let mut r = make_result("f", None);
        r.coverage_status = status.to_string();
        r.line_coverage_pct = pct;
        r.lines_total = total;
        r.lines_covered = ((pct as u32) * total) / 100;
        r.missed_lines = total - r.lines_covered;
        r
    }

    // ── format_coverage_metrics_md (3 status arms + impact + diff) ──

    #[test]
    fn test_format_coverage_metrics_md_uncovered_arm() {
        let r = result_with_coverage("uncovered", 0.0, 50);
        let mut out = String::new();
        format_coverage_metrics_md(&r, &mut out);
        assert!(out.contains("Uncovered (0/50 lines)"));
    }

    #[test]
    fn test_format_coverage_metrics_md_partial_arm() {
        let r = result_with_coverage("partial", 50.0, 100);
        let mut out = String::new();
        format_coverage_metrics_md(&r, &mut out);
        assert!(out.contains("Coverage: 50%"));
        assert!(out.contains("missed lines"));
    }

    #[test]
    fn test_format_coverage_metrics_md_full_arm() {
        let r = result_with_coverage("full", 100.0, 30);
        let mut out = String::new();
        format_coverage_metrics_md(&r, &mut out);
        assert!(out.contains("Fully covered"));
        assert!(out.contains("(30 lines)"));
    }

    #[test]
    fn test_format_coverage_metrics_md_unknown_status_emits_nothing() {
        let r = result_with_coverage("xyz", 0.0, 0);
        let mut out = String::new();
        format_coverage_metrics_md(&r, &mut out);
        // Unknown status branches to `_ => {}` so output stays empty.
        assert!(out.is_empty());
    }

    #[test]
    fn test_format_coverage_metrics_md_high_impact_appends_marker() {
        let mut r = result_with_coverage("full", 100.0, 10);
        r.impact_score = 5.5;
        let mut out = String::new();
        format_coverage_metrics_md(&r, &mut out);
        assert!(out.contains("Impact: 5.5"));
    }

    // ── format_coverage_diff_md / _text ──

    #[test]
    fn test_format_coverage_diff_md_positive_arm() {
        let mut out = String::new();
        format_coverage_diff_md(2.5, &mut out);
        assert!(out.contains("+2.5% coverage"));
    }

    #[test]
    fn test_format_coverage_diff_md_negative_arm() {
        let mut out = String::new();
        format_coverage_diff_md(-1.0, &mut out);
        assert!(out.contains("-1.0% coverage"));
    }

    #[test]
    fn test_format_coverage_diff_md_zero_writes_nothing() {
        let mut out = String::new();
        format_coverage_diff_md(0.0, &mut out);
        assert!(out.is_empty());
    }

    // ── format_coverage_metrics_text (3 status arms + cov_color tiers) ──

    #[test]
    fn test_format_coverage_metrics_text_partial_low_uses_red_color() {
        let r = result_with_coverage("partial", 30.0, 100);
        let mut out = String::new();
        format_coverage_metrics_text(&r, &mut out);
        // < 50% → red. Selection on the pure tier fn; emission is off here.
        assert_eq!(coverage_tier_color(30.0), BOLD_RED);
        assert!(out.contains("Cov: 30%"));
        assert!(!out.contains('\x1b'), "plain with colour off: {out:?}");
    }

    #[test]
    fn test_format_coverage_metrics_text_partial_mid_uses_yellow_color() {
        let r = result_with_coverage("partial", 70.0, 100);
        let mut out = String::new();
        format_coverage_metrics_text(&r, &mut out);
        // 50-80% → yellow.
        assert_eq!(coverage_tier_color(70.0), YELLOW);
        assert!(out.contains("Cov: 70%"));
        assert!(!out.contains('\x1b'), "plain with colour off: {out:?}");
    }

    #[test]
    fn test_format_coverage_metrics_text_partial_high_uses_green_color() {
        let r = result_with_coverage("partial", 90.0, 100);
        let mut out = String::new();
        format_coverage_metrics_text(&r, &mut out);
        // ≥ 80% → green.
        assert_eq!(coverage_tier_color(90.0), GREEN);
        assert!(out.contains("Cov: 90%"));
        assert!(!out.contains('\x1b'), "plain with colour off: {out:?}");
    }

    // ── truncate_doc ──

    #[test]
    fn test_truncate_doc_short_returns_unchanged() {
        let doc = "short doc";
        assert_eq!(truncate_doc(doc), "short doc");
    }

    #[test]
    fn test_truncate_doc_long_truncates_with_ellipsis() {
        let long = "x".repeat(150);
        let r = truncate_doc(&long);
        assert!(r.ends_with("..."));
        assert!(r.len() <= 100);
    }

    #[test]
    fn test_truncate_doc_takes_only_first_line() {
        let multi = "first line is short\nsecond line is long";
        let r = truncate_doc(multi);
        assert_eq!(r, "first line is short");
    }

    // ── push_pagerank_metric (3 tier arms) ──

    #[test]
    fn test_push_pagerank_metric_zero_no_metric() {
        let r = make_result("f", None);
        let mut metrics = vec![];
        push_pagerank_metric(&r, &mut metrics);
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_push_pagerank_metric_high_tier_emits_star() {
        let mut r = make_result("f", None);
        r.pagerank = 0.005; // pr_scaled = 50.0 (>= 10.0)
        let mut metrics = vec![];
        push_pagerank_metric(&r, &mut metrics);
        assert_eq!(metrics.len(), 1);
        assert!(metrics[0].contains("★"));
    }

    #[test]
    fn test_push_pagerank_metric_low_tier_omits() {
        let mut r = make_result("f", None);
        r.pagerank = 0.00005; // pr_scaled = 0.5 (< 1.0)
        let mut metrics = vec![];
        push_pagerank_metric(&r, &mut metrics);
        // Below threshold → no metric.
        assert!(metrics.is_empty());
    }

    // ── push_indegree_metric ──

    #[test]
    fn test_push_indegree_metric_zero_no_metric() {
        let r = make_result("f", None);
        let mut metrics = vec![];
        push_indegree_metric(&r, &mut metrics);
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_push_indegree_metric_low_uses_plain() {
        let mut r = make_result("f", None);
        r.in_degree = 3;
        let mut metrics = vec![];
        push_indegree_metric(&r, &mut metrics);
        assert_eq!(metrics.len(), 1);
        // Plain ↓ without ANSI color.
        assert!(metrics[0].contains("↓3"));
    }

    #[test]
    fn test_push_indegree_metric_high_uses_green() {
        let mut r = make_result("f", None);
        r.in_degree = 10;
        let mut metrics = vec![];
        push_indegree_metric(&r, &mut metrics);
        assert_eq!(metrics.len(), 1);
        // ≥ 5 → bold green, and the payload survives with colour off.
        assert_eq!(BOLD_GREEN.raw(), "\x1b[1;32m");
        assert!(metrics[0].contains("10"));
        assert!(!metrics[0].contains('\x1b'), "plain: {:?}", metrics[0]);
    }

    // ── push_churn_metric_rich (3 tier arms) ──

    #[test]
    fn test_push_churn_metric_rich_zero_commit_no_metric() {
        let r = make_result("f", None);
        let mut metrics = vec![];
        push_churn_metric_rich(&r, &mut metrics);
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_push_churn_metric_rich_high_tier_uses_fire() {
        let mut r = make_result("f", None);
        r.commit_count = 10;
        r.churn_score = 0.8;
        let mut metrics = vec![];
        push_churn_metric_rich(&r, &mut metrics);
        assert!(metrics[0].contains("🔥"));
    }

    // ── push_entropy_metric ──

    #[test]
    fn test_push_entropy_metric_zero_no_metric() {
        let r = make_result("f", None);
        let mut metrics = vec![];
        push_entropy_metric(&r, &mut metrics);
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_push_entropy_metric_low_diversity_emits_repetitive() {
        let mut r = make_result("f", None);
        r.pattern_diversity = 0.2;
        let mut metrics = vec![];
        push_entropy_metric(&r, &mut metrics);
        assert!(metrics[0].contains("🔄"));
    }

    #[test]
    fn test_push_entropy_metric_high_diversity_emits_h_marker() {
        let mut r = make_result("f", None);
        r.pattern_diversity = 0.9;
        let mut metrics = vec![];
        push_entropy_metric(&r, &mut metrics);
        assert!(metrics[0].contains("H:"));
    }

    #[test]
    fn test_push_entropy_metric_mid_diversity_omits() {
        let mut r = make_result("f", None);
        r.pattern_diversity = 0.5;
        let mut metrics = vec![];
        push_entropy_metric(&r, &mut metrics);
        // 0.3 ≤ d ≤ 0.8 → no metric.
        assert!(metrics.is_empty());
    }

    // ── build_rich_metrics integration ──

    #[test]
    fn test_build_rich_metrics_baseline_has_complexity_and_loc() {
        let r = make_result("f", None);
        let metrics = build_rich_metrics(&r);
        // Always present: C: and L: prefixes.
        assert!(metrics.iter().any(|m| m.starts_with("C:")));
        assert!(metrics.iter().any(|m| m.starts_with("L:")));
    }

    #[test]
    fn test_build_rich_metrics_with_satd_appends_warn_marker() {
        let mut r = make_result("f", None);
        r.satd_count = 3;
        let metrics = build_rich_metrics(&r);
        assert!(metrics.iter().any(|m| m.contains("⚠3")));
    }

    #[test]
    fn test_build_rich_metrics_with_clone_count_appends_clipboard() {
        let mut r = make_result("f", None);
        r.clone_count = 2;
        let metrics = build_rich_metrics(&r);
        assert!(metrics.iter().any(|m| m.contains("📋2")));
    }
}
