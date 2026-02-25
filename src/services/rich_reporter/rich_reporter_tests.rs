// RichReporter unit tests: reporter construction, finding management,
// analysis pipeline, rendering (text/JSON/Markdown), Andon status, and trend integration.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_finding(id: &str, category: &str, severity: Severity) -> Finding {
        Finding {
            id: id.to_string(),
            category: category.to_string(),
            severity,
            location: SourceLocation {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
                scope: None,
            },
            message: "Test finding".to_string(),
            confidence: 0.9,
            cluster_id: None,
            pagerank: None,
            community: None,
            anomaly_score: None,
            fix_suggestion: None,
        }
    }

    #[test]
    fn test_rich_reporter_new() {
        let config = ReportConfig::default();
        let reporter = RichReporter::new(config);
        assert_eq!(reporter.report.findings.len(), 0);
    }

    #[test]
    fn test_rich_reporter_add_finding() {
        let config = ReportConfig::default();
        let mut reporter = RichReporter::new(config);
        reporter.add_finding(create_test_finding("1", "TypeMismatch", Severity::High));
        assert_eq!(reporter.report.findings.len(), 1);
    }

    #[test]
    fn test_rich_reporter_analyze() {
        let config = ReportConfig::default();
        let mut reporter = RichReporter::new(config);

        for i in 0..5 {
            reporter.add_finding(create_test_finding(
                &format!("{}", i),
                "TypeMismatch",
                Severity::High,
            ));
        }

        reporter.analyze();

        // Should have assigned cluster IDs
        assert!(reporter
            .report
            .findings
            .iter()
            .all(|f| f.cluster_id.is_some()));
    }

    #[test]
    fn test_rich_reporter_render_text() {
        let config = ReportConfig::default();
        let mut reporter = RichReporter::new(config)
            .with_title("Test Report")
            .with_project("test-project");

        reporter.add_finding(create_test_finding("1", "TypeMismatch", Severity::High));
        reporter.set_quality_score(85.0);
        reporter.analyze();

        let output = reporter.render_text();
        assert!(output.contains("Test Report"));
        assert!(output.contains("Quality Score"));
        assert!(output.contains("85.0%"));
    }

    #[test]
    fn test_rich_reporter_render_json() {
        let config = ReportConfig {
            format: OutputFormat::Json,
            ..Default::default()
        };
        let mut reporter = RichReporter::new(config);
        reporter.add_finding(create_test_finding("1", "TypeMismatch", Severity::High));
        reporter.analyze();

        let output = reporter.render_json();
        assert!(output.contains("\"findings\""));
        assert!(output.contains("TypeMismatch"));
    }

    #[test]
    fn test_rich_reporter_render_markdown() {
        let config = ReportConfig {
            format: OutputFormat::Markdown,
            ..Default::default()
        };
        let mut reporter = RichReporter::new(config)
            .with_title("Test Report")
            .with_project("test-project");

        reporter.add_finding(create_test_finding("1", "TypeMismatch", Severity::High));
        reporter.analyze();

        let output = reporter.render_markdown();
        assert!(output.contains("# Test Report"));
        assert!(output.contains("## Summary"));
    }

    #[test]
    fn test_andon_status_calculation() {
        let mut report = RichReport::new("Test", "test");

        // No findings = Green
        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Green);

        // High finding = Yellow
        report
            .findings
            .push(create_test_finding("1", "Test", Severity::High));
        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Yellow);

        // Critical finding = Red
        report
            .findings
            .push(create_test_finding("2", "Test", Severity::Critical));
        report.calculate_andon_status();
        assert_eq!(report.andon_status, AndonStatus::Red);
    }

    #[test]
    fn test_trend_analysis_integration() {
        let config = ReportConfig::default();
        let mut reporter = RichReporter::new(config);

        // Add metric history
        let data: Vec<(i64, f64)> = (0..10).map(|i| (i as i64, 50.0 + i as f64 * 2.0)).collect();
        reporter.add_metric_history("coverage", data);

        reporter.analyze();

        assert_eq!(reporter.report.trends.len(), 1);
        assert_eq!(reporter.report.trends[0].name, "coverage");
    }
}
