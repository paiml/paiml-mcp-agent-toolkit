#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::facades::satd_facade::{SatdSeverity as FacadeSeverity, SatdViolation};

    /// Strip ANSI escape codes from a string for assertion comparisons.
    fn strip_ansi(s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(s, "").to_string()
    }

    #[test]
    fn test_format_summary() {
        let result = SatdAnalysisResult {
            census: Default::default(),
            total_files: 10,
            violations: vec![SatdViolation {
                file_path: "test.rs".to_string(),
                line_number: 42,
                violation_type: "TODO".to_string(),
                message: "Implement feature".to_string(),
                severity: FacadeSeverity::Medium,
            }],
            summary: "Test summary".to_string(),
        };

        let output = strip_ansi(&format_summary(&result, 10));
        assert!(output.contains("Test summary"));
        assert!(output.contains("Total violations:"));
        assert!(output.contains("1"));
    }

    fn make_violation(severity: FacadeSeverity, vtype: &str) -> SatdViolation {
        SatdViolation {
            file_path: format!("src/{vtype}.rs"),
            line_number: 1,
            violation_type: vtype.to_string(),
            message: format!("{vtype} message"),
            severity,
        }
    }

    fn result_with_all_severities() -> SatdAnalysisResult {
        SatdAnalysisResult {
            census: Default::default(),
            total_files: 4,
            violations: vec![
                make_violation(FacadeSeverity::Critical, "FIXME"),
                make_violation(FacadeSeverity::High, "TODO"),
                make_violation(FacadeSeverity::Medium, "HACK"),
                make_violation(FacadeSeverity::Low, "XXX"),
            ],
            summary: "All severities".to_string(),
        }
    }

    fn empty_result() -> SatdAnalysisResult {
        SatdAnalysisResult {
            census: Default::default(),
            total_files: 0,
            violations: vec![],
            summary: "Empty".to_string(),
        }
    }

    // ── format_output dispatcher ──

    #[test]
    fn test_format_output_summary_arm() {
        let r = format_output(
            &result_with_all_severities(),
            SatdOutputFormat::Summary,
            false,
            10,
        );
        let s = strip_ansi(&r);
        assert!(s.contains("SATD Analysis Summary"));
    }

    #[test]
    fn test_format_output_json_arm() {
        let r = format_output(
            &result_with_all_severities(),
            SatdOutputFormat::Json,
            false,
            10,
        );
        assert!(r.contains("\"total_violations\": 4"));
    }

    #[test]
    fn test_format_output_sarif_arm() {
        let r = format_output(
            &result_with_all_severities(),
            SatdOutputFormat::Sarif,
            false,
            10,
        );
        assert!(r.contains("\"version\":\"2.1.0\""));
        assert!(r.contains("satd-violation"));
    }

    #[test]
    fn test_format_output_markdown_arm() {
        let r = format_output(
            &result_with_all_severities(),
            SatdOutputFormat::Markdown,
            false,
            10,
        );
        assert!(r.contains("# SATD Analysis Report"));
    }

    // ── format_summary: severity-loop and Top-Violations gating ──

    #[test]
    fn test_format_summary_with_all_severities_lists_each_count() {
        let r = strip_ansi(&format_summary(&result_with_all_severities(), 10));
        assert!(r.contains("Critical:"));
        assert!(r.contains("High:"));
        assert!(r.contains("Medium:"));
        assert!(r.contains("Low:"));
        assert!(r.contains("Top Violations"));
    }

    /// `--color never` (and NO_COLOR, and a redirected stdout) must leave no
    /// escape sequence in the summary. The Severity Distribution and Top
    /// Violations blocks interpolated the raw `c::BOLD` / `c::RED` / `c::RESET`
    /// consts, which are unconditional, so five sequences survived
    /// `analyze satd --color never` and landed in redirected files.
    #[test]
    fn test_format_summary_emits_no_ansi_when_colors_are_disabled() {
        // Under `cargo test` stdout is captured, so colour resolves to off —
        // unless the operator forced it on, in which case there is nothing to
        // assert.
        if crate::cli::colors::colors_enabled() {
            return;
        }

        let r = format_summary(&result_with_all_severities(), 10);
        assert!(
            !r.contains('\x1b'),
            "no ANSI escape may survive with colour disabled, got: {r:?}"
        );
        // And the content itself is untouched by the migration.
        assert!(r.contains("Critical:"));
        assert!(r.contains("Top Violations"));
    }

    #[test]
    fn test_format_summary_empty_skips_top_violations() {
        let r = strip_ansi(&format_summary(&empty_result(), 10));
        assert!(!r.contains("Top Violations"));
    }

    /// Sixteen violations, and the row count is whatever `--top-files` says.
    ///
    /// This test used to be `test_format_summary_top_violations_caps_at_10`,
    /// and it asserted the defect: the cap was the literal `.take(10)` in
    /// `format_summary`, so `analyze satd --top-files 1` and
    /// `--top-files 50` printed the same ten rows over a 63-violation corpus.
    /// A test that pins a hardcoded limit cannot fail when the flag that is
    /// supposed to set that limit does nothing.
    #[test]
    fn test_format_summary_top_violations_row_count_is_top_files() {
        let mut res = result_with_all_severities();
        for i in 0..12 {
            res.violations
                .push(make_violation(FacadeSeverity::Low, &format!("V{i}")));
        }
        assert_eq!(res.violations.len(), 16, "fixture must exceed every probe");

        let rows = |top_files: usize| {
            strip_ansi(&format_summary(&res, top_files))
                .lines()
                .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()))
                .filter(|l| l.contains(".rs:"))
                .count()
        };

        assert_eq!(rows(1), 1, "--top-files 1 must list one violation");
        assert_eq!(rows(5), 5, "--top-files 5 must list five violations");
        assert_eq!(rows(50), 16, "a limit above the total lists every row");
        assert_eq!(rows(0), 16, "--top-files 0 means all");
    }

    /// Truncation must say it truncated: a capped list that looks complete is
    /// how "10 of 63" gets read as "63".
    #[test]
    fn test_format_summary_discloses_what_the_limit_hid() {
        let mut res = result_with_all_severities();
        for i in 0..12 {
            res.violations
                .push(make_violation(FacadeSeverity::Low, &format!("V{i}")));
        }
        let capped = strip_ansi(&format_summary(&res, 3));
        assert!(
            capped.contains("13 more not shown"),
            "expected a disclosure of the 13 hidden rows, got:\n{capped}"
        );
        let uncapped = strip_ansi(&format_summary(&res, 0));
        assert!(
            !uncapped.contains("more not shown"),
            "nothing was hidden, so nothing to disclose:\n{uncapped}"
        );
    }

    // ── format_json: metrics flag toggle ──

    #[test]
    fn test_format_json_metrics_flag_adds_metrics_block() {
        let r = format_json(&result_with_all_severities(), true, 10);
        assert!(r.contains("\"metrics\""));
        assert!(r.contains("\"critical_count\": 1"));
        assert!(r.contains("\"high_count\": 1"));
    }

    #[test]
    fn test_format_json_no_metrics_flag_omits_metrics() {
        let r = format_json(&result_with_all_severities(), false, 10);
        assert!(!r.contains("\"metrics\""));
    }

    /// This test used to assert the opposite: that `--evolution` added
    /// `"evolution": {"message": "Evolution tracking would show SATD trends
    /// over time"}` to the document. Nothing computed a trend, so that block
    /// was a sentence shaped like a measurement. `--evolution` is now refused
    /// by the handler and no such key can be emitted.
    #[test]
    fn test_format_json_never_emits_an_evolution_placeholder() {
        let r = format_json(&result_with_all_severities(), true, 10);
        assert!(!r.contains("\"evolution\""));
        assert!(!r.contains("Evolution tracking"));
    }

    /// `analyze satd --evolution --days 90` produced output byte-identical to
    /// `analyze satd` in the summary format and a placeholder sentence in the
    /// others. A flag that measures nothing must be refused.
    #[test]
    fn test_evolution_flag_is_rejected() {
        assert!(reject_unimplemented_evolution(false).is_ok());
        let err = reject_unimplemented_evolution(true)
            .expect_err("--evolution computes nothing and must be refused");
        assert!(
            err.to_string().contains("--evolution"),
            "the error must name the flag: {err}"
        );
    }

    // ── format_sarif: severity → level mapping ──

    #[test]
    fn test_format_sarif_maps_each_severity_to_correct_level() {
        let r = format_sarif(&result_with_all_severities());
        // Critical + High → "error", Medium → "warning", Low → "note".
        assert!(r.contains("\"error\""));
        assert!(r.contains("\"warning\""));
        assert!(r.contains("\"note\""));
    }

    #[test]
    fn test_format_sarif_empty_violations_emits_empty_results() {
        let r = format_sarif(&empty_result());
        assert!(r.contains("\"results\":[]"));
    }

    // ── format_markdown: violations table gating ──

    /// This pair used to assert that `--evolution` emitted a
    /// `## Evolution (Last 30 Days)` heading over "*Evolution tracking would
    /// show SATD trends over time*" — a heading and a sentence, with no trend
    /// behind either, and `--days` feeding only the heading. No evolution
    /// section may be emitted at all now.
    #[test]
    fn test_format_markdown_never_emits_an_evolution_section() {
        let r = format_markdown(&result_with_all_severities(), 10);
        assert!(!r.contains("## Evolution"));
        assert!(!r.contains("Evolution tracking"));
    }

    #[test]
    fn test_format_markdown_violations_table_emitted_only_when_non_empty() {
        let with = format_markdown(&result_with_all_severities(), 10);
        assert!(with.contains("## Violations"));
        let without = format_markdown(&empty_result(), 10);
        assert!(!without.contains("## Violations"));
    }

    #[test]
    fn test_format_markdown_metrics_table_includes_critical_high_counts() {
        let r = format_markdown(&result_with_all_severities(), 10);
        assert!(r.contains("Critical Violations | 1"));
        assert!(r.contains("High Violations | 1"));
    }

    // ── print_metrics smoke (writes to stderr; verify no panic) ──

    #[test]
    fn test_print_metrics_with_violations_no_panic() {
        print_metrics(&result_with_all_severities());
    }

    #[test]
    fn test_print_metrics_empty_no_panic() {
        print_metrics(&empty_result());
    }

    // ── #676: summary/total_files must describe the FILTERED result set ──

    fn config_with_severity(severity: Option<SatdSeverity>) -> SatdAnalysisConfig {
        SatdAnalysisConfig {
            path: std::path::PathBuf::from("."),
            format: SatdOutputFormat::Json,
            severity,
            critical_only: false,
            include_tests: false,
            strict: false,
            evolution: false,
            days: 30,
            metrics: false,
            output: None,
            top_files: 0,
            fail_on_violation: false,
            timeout: 30,
            include: vec![],
            exclude: vec![],
            extended: false,
        }
    }

    #[test]
    fn test_severity_filter_restates_summary_and_total_files() {
        // Observed on 3.29.0: {"total_files":1,"total_violations":0,
        // "summary":"Found 7 SATD violations in 1 files","violations":[]}
        let filtered = apply_analysis_filters(
            result_with_all_severities(),
            &config_with_severity(Some(SatdSeverity::Critical)),
        )
        .unwrap();

        assert_eq!(filtered.violations.len(), 1, "only Critical survives");
        assert_eq!(filtered.total_files, 1, "one file holds the survivor");
        assert_eq!(
            filtered.summary, "Found 1 SATD violations in 1 files",
            "summary must describe the filtered set, not the pre-filter one"
        );
    }

    #[test]
    fn test_severity_filter_removing_everything_reports_zero_files() {
        let mut only_low = result_with_all_severities();
        only_low.violations.retain(|v| {
            matches!(
                v.severity,
                crate::services::facades::satd_facade::SatdSeverity::Low
            )
        });

        let filtered = apply_analysis_filters(
            only_low,
            &config_with_severity(Some(SatdSeverity::Critical)),
        )
        .unwrap();

        assert_eq!(filtered.violations.len(), 0);
        assert_eq!(filtered.total_files, 0, "no violations => no files");
        assert_eq!(filtered.summary, "Found 0 SATD violations in 0 files");
    }

    #[test]
    fn test_no_filter_summary_still_matches_payload() {
        let filtered =
            apply_analysis_filters(result_with_all_severities(), &config_with_severity(None))
                .unwrap();

        assert_eq!(filtered.violations.len(), 4);
        assert_eq!(filtered.summary, "Found 4 SATD violations in 4 files");
    }
}
