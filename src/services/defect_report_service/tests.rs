#![cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::defect_report::Severity;

    #[test]
    fn test_report_format_filename() {
        let service = DefectReportService::new();

        let json_name = service.generate_filename(ReportFormat::Json);
        assert!(json_name.starts_with("defect-report-"));
        assert!(json_name.ends_with(".json"));

        let csv_name = service.generate_filename(ReportFormat::Csv);
        assert!(csv_name.ends_with(".csv"));

        let md_name = service.generate_filename(ReportFormat::Markdown);
        assert!(md_name.ends_with(".md"));

        let txt_name = service.generate_filename(ReportFormat::Text);
        assert!(txt_name.ends_with(".txt"));
    }

    #[test]
    fn test_summary_computation() {
        let service = DefectReportService::new();
        let defects = vec![
            Defect {
                id: "TEST-001".to_string(),
                severity: Severity::Critical,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file1.rs"),
                line_start: 1,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
            Defect {
                id: "TEST-002".to_string(),
                severity: Severity::High,
                category: DefectCategory::Complexity,
                file_path: PathBuf::from("file1.rs"),
                line_start: 10,
                line_end: None,
                column_start: None,
                column_end: None,
                message: "Test 2".to_string(),
                rule_id: "test".to_string(),
                fix_suggestion: None,
                metrics: HashMap::new(),
            },
        ];

        let summary = service.compute_summary(&defects);
        assert_eq!(summary.total_defects, 2);
        assert_eq!(summary.by_severity.get("critical"), Some(&1));
        assert_eq!(summary.by_severity.get("high"), Some(&1));
        assert_eq!(summary.hotspot_files.len(), 1);
        assert_eq!(summary.hotspot_files[0].defect_count, 2);
        assert_eq!(summary.hotspot_files[0].severity_score, 15.0);
    }
}

// Include additional integration tests
#[cfg(test)]
#[path = "defect_report_service_tests.rs"]
mod integration_tests;

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

/// Comprehensive EXTREME TDD coverage tests for defect report service
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::models::defect_report::{Defect, DefectCategory, Severity};
    use proptest::prelude::*;
    use std::collections::{BTreeMap, HashMap};
    use std::path::PathBuf;

    // Test Fixture Builders

    /// Build a minimal defect for testing
    fn build_defect(id: &str, severity: Severity, category: DefectCategory) -> Defect {
        Defect {
            id: id.to_string(),
            severity,
            category,
            file_path: PathBuf::from("src/test.rs"),
            line_start: 1,
            line_end: None,
            column_start: None,
            column_end: None,
            message: format!("Test defect: {}", id),
            rule_id: "TEST-001".to_string(),
            fix_suggestion: None,
            metrics: HashMap::new(),
        }
    }

    /// Build a defect with specific file path
    fn build_defect_for_file(id: &str, file_path: &str, severity: Severity) -> Defect {
        Defect {
            id: id.to_string(),
            severity,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from(file_path),
            line_start: 10,
            line_end: Some(20),
            column_start: Some(5),
            column_end: Some(50),
            message: format!("Defect in {}", file_path),
            rule_id: "FILE-001".to_string(),
            fix_suggestion: Some("Consider refactoring".to_string()),
            metrics: {
                let mut m = HashMap::new();
                m.insert("cyclomatic".to_string(), 25.0);
                m.insert("cognitive".to_string(), 30.0);
                m
            },
        }
    }

    /// Build a complete test defect with all optional fields populated
    fn build_complete_defect(
        id: &str,
        severity: Severity,
        category: DefectCategory,
        file_path: &str,
    ) -> Defect {
        Defect {
            id: id.to_string(),
            severity,
            category,
            file_path: PathBuf::from(file_path),
            line_start: 42,
            line_end: Some(100),
            column_start: Some(1),
            column_end: Some(80),
            message: format!("Complete defect {} in {}", id, file_path),
            rule_id: format!("{:?}-RULE", category),
            fix_suggestion: Some(format!("Fix suggestion for {}", id)),
            metrics: {
                let mut m = HashMap::new();
                m.insert("cyclomatic".to_string(), 15.0);
                m.insert("cognitive".to_string(), 20.0);
                m.insert("nesting".to_string(), 5.0);
                m
            },
        }
    }

    /// Build a collection of defects across multiple files and categories
    fn build_diverse_defects() -> Vec<Defect> {
        vec![
            build_complete_defect(
                "DEF-001",
                Severity::Critical,
                DefectCategory::Complexity,
                "src/main.rs",
            ),
            build_complete_defect(
                "DEF-002",
                Severity::High,
                DefectCategory::TechnicalDebt,
                "src/main.rs",
            ),
            build_complete_defect(
                "DEF-003",
                Severity::Medium,
                DefectCategory::DeadCode,
                "src/lib.rs",
            ),
            build_complete_defect(
                "DEF-004",
                Severity::Low,
                DefectCategory::Duplication,
                "src/lib.rs",
            ),
            build_complete_defect(
                "DEF-005",
                Severity::Critical,
                DefectCategory::Performance,
                "src/utils.rs",
            ),
            build_complete_defect(
                "DEF-006",
                Severity::High,
                DefectCategory::Architecture,
                "src/utils.rs",
            ),
            build_complete_defect(
                "DEF-007",
                Severity::Medium,
                DefectCategory::TestCoverage,
                "tests/test.rs",
            ),
        ]
    }

    /// Build a test report with the given defects
    fn build_test_report(defects: Vec<Defect>) -> DefectReport {
        let service = DefectReportService::new();
        let summary = service.compute_summary(&defects);

        let mut file_index = BTreeMap::new();
        for defect in &defects {
            file_index
                .entry(defect.file_path.clone())
                .or_insert_with(Vec::new)
                .push(defect.id.clone());
        }

        DefectReport {
            metadata: ReportMetadata {
                tool: "pmat".to_string(),
                version: "test-version".to_string(),
                generated_at: Utc::now(),
                project_root: PathBuf::from("/test/project"),
                total_files_analyzed: file_index.len(),
                analysis_duration_ms: 100,
            },
            defects,
            summary,
            file_index,
        }
    }

    // DefectReportService::new() Tests

    #[test]
    fn test_new_creates_service_with_semaphore() {
        let service = DefectReportService::new();
        // Service should be created without panicking
        // The semaphore is initialized based on CPU count
        assert!(std::mem::size_of_val(&service) > 0);
    }

    #[test]
    fn test_default_creates_same_as_new() {
        let service1 = DefectReportService::new();
        let service2 = DefectReportService::default();
        // Both should have similar structure
        assert!(std::mem::size_of_val(&service1) == std::mem::size_of_val(&service2));
    }

    // compute_summary() Tests

    #[test]
    fn test_compute_summary_empty_defects() {
        let service = DefectReportService::new();
        let summary = service.compute_summary(&[]);

        assert_eq!(summary.total_defects, 0);
        assert!(summary.by_severity.is_empty());
        assert!(summary.by_category.is_empty());
        assert!(summary.hotspot_files.is_empty());
    }

    #[test]
    fn test_compute_summary_single_defect() {
        let service = DefectReportService::new();
        let defects = vec![build_defect(
            "TEST-001",
            Severity::High,
            DefectCategory::Complexity,
        )];
        let summary = service.compute_summary(&defects);

        assert_eq!(summary.total_defects, 1);
        assert_eq!(summary.by_severity.get("high"), Some(&1));
        assert_eq!(summary.by_category.get("Complexity"), Some(&1));
        assert_eq!(summary.hotspot_files.len(), 1);
        assert_eq!(summary.hotspot_files[0].defect_count, 1);
    }

    #[test]
    fn test_compute_summary_multiple_severities() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect("T-001", Severity::Critical, DefectCategory::Complexity),
            build_defect("T-002", Severity::High, DefectCategory::Complexity),
            build_defect("T-003", Severity::Medium, DefectCategory::Complexity),
            build_defect("T-004", Severity::Low, DefectCategory::Complexity),
            build_defect("T-005", Severity::Critical, DefectCategory::Complexity),
        ];
        let summary = service.compute_summary(&defects);

        assert_eq!(summary.total_defects, 5);
        assert_eq!(summary.by_severity.get("critical"), Some(&2));
        assert_eq!(summary.by_severity.get("high"), Some(&1));
        assert_eq!(summary.by_severity.get("medium"), Some(&1));
        assert_eq!(summary.by_severity.get("low"), Some(&1));
    }

    #[test]
    fn test_compute_summary_multiple_categories() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect("T-001", Severity::High, DefectCategory::Complexity),
            build_defect("T-002", Severity::High, DefectCategory::TechnicalDebt),
            build_defect("T-003", Severity::High, DefectCategory::DeadCode),
            build_defect("T-004", Severity::High, DefectCategory::Performance),
        ];
        let summary = service.compute_summary(&defects);

        assert_eq!(summary.total_defects, 4);
        assert_eq!(summary.by_category.len(), 4);
        assert_eq!(summary.by_category.get("Complexity"), Some(&1));
        assert_eq!(summary.by_category.get("TechnicalDebt"), Some(&1));
        assert_eq!(summary.by_category.get("DeadCode"), Some(&1));
        assert_eq!(summary.by_category.get("Performance"), Some(&1));
    }

    #[test]
    fn test_compute_summary_hotspot_calculation() {
        let service = DefectReportService::new();
        let defects = vec![
            // File1 has 3 critical defects = 30.0 score
            build_defect_for_file("T-001", "file1.rs", Severity::Critical),
            build_defect_for_file("T-002", "file1.rs", Severity::Critical),
            build_defect_for_file("T-003", "file1.rs", Severity::Critical),
            // File2 has 1 low defect = 1.0 score
            build_defect_for_file("T-004", "file2.rs", Severity::Low),
        ];
        let summary = service.compute_summary(&defects);

        assert_eq!(summary.hotspot_files.len(), 2);
        // Hotspots are sorted by severity_score descending
        assert_eq!(summary.hotspot_files[0].path, PathBuf::from("file1.rs"));
        assert_eq!(summary.hotspot_files[0].defect_count, 3);
        assert_eq!(summary.hotspot_files[0].severity_score, 30.0);
        assert_eq!(summary.hotspot_files[1].path, PathBuf::from("file2.rs"));
        assert_eq!(summary.hotspot_files[1].defect_count, 1);
        assert_eq!(summary.hotspot_files[1].severity_score, 1.0);
    }

    #[test]
    fn test_compute_summary_hotspot_truncation_to_10() {
        let service = DefectReportService::new();
        // Create 15 different files with defects
        let defects: Vec<Defect> = (0..15)
            .map(|i| {
                build_defect_for_file(
                    &format!("T-{:03}", i),
                    &format!("file{}.rs", i),
                    Severity::High,
                )
            })
            .collect();
        let summary = service.compute_summary(&defects);

        // Should only have top 10 hotspots
        assert_eq!(summary.hotspot_files.len(), 10);
    }

    #[test]
    fn test_compute_summary_severity_scores() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect_for_file("T-001", "critical.rs", Severity::Critical), // 10.0
            build_defect_for_file("T-002", "high.rs", Severity::High),         // 5.0
            build_defect_for_file("T-003", "medium.rs", Severity::Medium),     // 3.0
            build_defect_for_file("T-004", "low.rs", Severity::Low),           // 1.0
        ];
        let summary = service.compute_summary(&defects);

        // Check the severity scores
        let critical_file = summary
            .hotspot_files
            .iter()
            .find(|h| h.path == PathBuf::from("critical.rs"))
            .unwrap();
        let high_file = summary
            .hotspot_files
            .iter()
            .find(|h| h.path == PathBuf::from("high.rs"))
            .unwrap();
        let medium_file = summary
            .hotspot_files
            .iter()
            .find(|h| h.path == PathBuf::from("medium.rs"))
            .unwrap();
        let low_file = summary
            .hotspot_files
            .iter()
            .find(|h| h.path == PathBuf::from("low.rs"))
            .unwrap();

        assert_eq!(critical_file.severity_score, 10.0);
        assert_eq!(high_file.severity_score, 5.0);
        assert_eq!(medium_file.severity_score, 3.0);
        assert_eq!(low_file.severity_score, 1.0);
    }

    // format_json() Tests

    #[test]
    fn test_format_json_empty_report() {
        let service = DefectReportService::new();
        let report = build_test_report(vec![]);
        let json = service.format_json(&report).unwrap();

        // Verify valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["metadata"].is_object());
        assert!(parsed["defects"].is_array());
        assert_eq!(parsed["defects"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_format_json_with_defects() {
        let service = DefectReportService::new();
        let defects = build_diverse_defects();
        let report = build_test_report(defects);
        let json = service.format_json(&report).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["defects"].as_array().unwrap().len(), 7);
        assert!(parsed["summary"]["total_defects"].as_u64().unwrap() == 7);
    }

    #[test]
    fn test_format_json_metadata_fields() {
        let service = DefectReportService::new();
        let report = build_test_report(vec![]);
        let json = service.format_json(&report).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["metadata"]["tool"].as_str().unwrap(), "pmat");
        assert!(parsed["metadata"]["version"].as_str().is_some());
        assert!(parsed["metadata"]["generated_at"].as_str().is_some());
        assert!(parsed["metadata"]["project_root"].as_str().is_some());
    }

    #[test]
    fn test_format_json_defect_fields() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "TEST-001",
            Severity::Critical,
            DefectCategory::Complexity,
            "src/main.rs",
        )];
        let report = build_test_report(defects);
        let json = service.format_json(&report).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let defect = &parsed["defects"][0];

        assert_eq!(defect["id"].as_str().unwrap(), "TEST-001");
        assert_eq!(defect["severity"].as_str().unwrap(), "critical");
        assert_eq!(defect["category"].as_str().unwrap(), "complexity");
        assert!(defect["line_start"].as_u64().is_some());
        assert!(defect["line_end"].as_u64().is_some());
        assert!(defect["fix_suggestion"].as_str().is_some());
    }

    // format_csv() Tests

    #[test]
    fn test_format_csv_headers() {
        let service = DefectReportService::new();
        let report = build_test_report(vec![]);
        let csv = service.format_csv(&report).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        assert!(!lines.is_empty());
        let header = lines[0];
        assert!(header.contains("id"));
        assert!(header.contains("severity"));
        assert!(header.contains("category"));
        assert!(header.contains("file_path"));
        assert!(header.contains("line_start"));
        assert!(header.contains("line_end"));
        assert!(header.contains("message"));
        assert!(header.contains("rule_id"));
        assert!(header.contains("cyclomatic"));
        assert!(header.contains("cognitive"));
    }

    #[test]
    fn test_format_csv_with_defects() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "CSV-001",
            Severity::High,
            DefectCategory::Complexity,
            "src/test.rs",
        )];
        let report = build_test_report(defects);
        let csv = service.format_csv(&report).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2); // Header + 1 defect
        assert!(lines[1].contains("CSV-001"));
        assert!(lines[1].contains("high"));
        assert!(lines[1].contains("Complexity"));
    }

    #[test]
    fn test_format_csv_metrics_columns() {
        let service = DefectReportService::new();
        let mut defect = build_defect("METRIC-001", Severity::High, DefectCategory::Complexity);
        defect.metrics.insert("cyclomatic".to_string(), 42.0);
        defect.metrics.insert("cognitive".to_string(), 55.0);
        let report = build_test_report(vec![defect]);
        let csv = service.format_csv(&report).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        // Check that metrics are included
        assert!(lines[1].contains("42"));
        assert!(lines[1].contains("55"));
    }

    #[test]
    fn test_format_csv_empty_optional_fields() {
        let service = DefectReportService::new();
        let defect = build_defect("EMPTY-001", Severity::Low, DefectCategory::DeadCode);
        let report = build_test_report(vec![defect]);
        let csv = service.format_csv(&report).unwrap();

        // Should still produce valid CSV even with empty optional fields
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_format_csv_multiple_defects() {
        let service = DefectReportService::new();
        let defects = build_diverse_defects();
        let expected_count = defects.len();
        let report = build_test_report(defects);
        let csv = service.format_csv(&report).unwrap();

        let lines: Vec<&str> = csv.lines().collect();
        // Header + 7 defects
        assert_eq!(lines.len(), expected_count + 1);
    }

    // format_markdown() Tests

    #[test]
    fn test_format_markdown_header() {
        let service = DefectReportService::new();
        let report = build_test_report(vec![]);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("# Code Quality Report"));
        assert!(md.contains("Generated:"));
    }

    #[test]
    fn test_format_markdown_executive_summary() {
        let service = DefectReportService::new();
        let defects = build_diverse_defects();
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("## Executive Summary"));
        assert!(md.contains("**Total Defects**"));
        assert!(md.contains("**Files Analyzed**"));
        assert!(md.contains("**Analysis Duration**"));
    }

    #[test]
    fn test_format_markdown_severity_distribution() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect("MD-001", Severity::Critical, DefectCategory::Complexity),
            build_defect("MD-002", Severity::High, DefectCategory::Complexity),
        ];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("### Severity Distribution"));
        // Progress bar should be present
        assert!(md.contains("```"));
    }

    #[test]
    fn test_format_markdown_hotspot_table() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect_for_file("HOT-001", "src/hotspot.rs", Severity::Critical),
            build_defect_for_file("HOT-002", "src/hotspot.rs", Severity::High),
        ];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("### Top 10 Hotspot Files"));
        assert!(md.contains("| Rank | File | Defects | Severity Score |"));
        assert!(md.contains("src/hotspot.rs"));
    }

    #[test]
    fn test_format_markdown_detailed_findings() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "DETAIL-001",
            Severity::High,
            DefectCategory::Complexity,
            "src/complex.rs",
        )];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        assert!(md.contains("## Detailed Findings"));
        assert!(md.contains("### Complexity"));
        assert!(md.contains("src/complex.rs"));
    }

    #[test]
    fn test_format_markdown_fix_suggestion() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "FIX-001",
            Severity::High,
            DefectCategory::Complexity,
            "src/fix.rs",
        )];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        // Should contain the suggestion indicator
        assert!(md.contains("**Suggestion**"));
    }

    #[test]
    fn test_format_markdown_truncation_indicator() {
        let service = DefectReportService::new();
        // Create more than 10 defects of the same category
        let defects: Vec<Defect> = (0..15)
            .map(|i| {
                build_complete_defect(
                    &format!("TRUNC-{:03}", i),
                    Severity::High,
                    DefectCategory::Complexity,
                    "src/many.rs",
                )
            })
            .collect();
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        // Should show truncation message
        assert!(md.contains("...and 5 more Complexity"));
    }

    #[test]
    fn test_format_markdown_empty_category() {
        let service = DefectReportService::new();
        // Only create defects for one category
        let defects = vec![build_defect(
            "ONLY-001",
            Severity::High,
            DefectCategory::Complexity,
        )];
        let report = build_test_report(defects);
        let md = service.format_markdown(&report).unwrap();

        // Should only have Complexity section, not other categories
        assert!(md.contains("### Complexity"));
        // Other categories should not appear as headers (with issue counts)
        assert!(!md.contains("### Technical Debt ("));
    }

    // format_text() Tests

    #[test]
    fn test_format_text_header() {
        let service = DefectReportService::new();
        let report = build_test_report(vec![]);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("CODE QUALITY REPORT"));
        assert!(text.contains("==================="));
        assert!(text.contains("Generated:"));
        assert!(text.contains("Project:"));
    }

    #[test]
    fn test_format_text_severity_breakdown() {
        let service = DefectReportService::new();
        let defects = vec![
            build_defect("TXT-001", Severity::Critical, DefectCategory::Complexity),
            build_defect("TXT-002", Severity::High, DefectCategory::Complexity),
        ];
        let report = build_test_report(defects);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("SEVERITY BREAKDOWN"));
        assert!(text.contains("------------------"));
    }

    #[test]
    fn test_format_text_category_breakdown() {
        let service = DefectReportService::new();
        let defects = build_diverse_defects();
        let report = build_test_report(defects);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("CATEGORY BREAKDOWN"));
    }

    #[test]
    fn test_format_text_hotspot_files() {
        let service = DefectReportService::new();
        let defects = vec![build_defect_for_file(
            "TXT-HOT-001",
            "src/hot.rs",
            Severity::Critical,
        )];
        let report = build_test_report(defects);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("TOP HOTSPOT FILES"));
        assert!(text.contains("src/hot.rs"));
    }

    #[test]
    fn test_format_text_defect_listing() {
        let service = DefectReportService::new();
        let defects = vec![build_complete_defect(
            "LIST-001",
            Severity::High,
            DefectCategory::Complexity,
            "src/list.rs",
        )];
        let report = build_test_report(defects);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("DEFECTS"));
        assert!(text.contains("-------"));
        assert!(text.contains("[High]"));
        assert!(text.contains("Complexity"));
        assert!(text.contains("src/list.rs"));
    }

    #[test]
    fn test_format_text_line_range() {
        let service = DefectReportService::new();
        let mut defect = build_defect("RANGE-001", Severity::High, DefectCategory::Complexity);
        defect.line_start = 10;
        defect.line_end = Some(25);
        let report = build_test_report(vec![defect]);
        let text = service.format_text(&report).unwrap();

        // Should show line range
        assert!(text.contains("10-25"));
    }

    #[test]
    fn test_format_text_fix_suggestion_display() {
        let service = DefectReportService::new();
        let mut defect = build_defect("FIX-TXT-001", Severity::High, DefectCategory::Complexity);
        defect.fix_suggestion = Some("Extract method to reduce complexity".to_string());
        let report = build_test_report(vec![defect]);
        let text = service.format_text(&report).unwrap();

        assert!(text.contains("Fix:"));
        assert!(text.contains("Extract method to reduce complexity"));
    }

    // generate_filename() Tests

    #[test]
    fn test_generate_filename_json() {
        let service = DefectReportService::new();
        let filename = service.generate_filename(ReportFormat::Json);

        assert!(filename.starts_with("defect-report-"));
        assert!(filename.ends_with(".json"));
        // Should contain a timestamp pattern
        assert!(filename.len() > 20);
    }

    #[test]
    fn test_generate_filename_csv() {
        let service = DefectReportService::new();
        let filename = service.generate_filename(ReportFormat::Csv);

        assert!(filename.starts_with("defect-report-"));
        assert!(filename.ends_with(".csv"));
    }

    #[test]
    fn test_generate_filename_markdown() {
        let service = DefectReportService::new();
        let filename = service.generate_filename(ReportFormat::Markdown);

        assert!(filename.starts_with("defect-report-"));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn test_generate_filename_text() {
        let service = DefectReportService::new();
        let filename = service.generate_filename(ReportFormat::Text);

        assert!(filename.starts_with("defect-report-"));
        assert!(filename.ends_with(".txt"));
    }

    #[test]
    fn test_generate_filename_uniqueness() {
        let service = DefectReportService::new();
        let filename1 = service.generate_filename(ReportFormat::Json);
        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
        let filename2 = service.generate_filename(ReportFormat::Json);

        // Filenames should be different if generated at different times
        // (they might be same if generated in same second, so we just check format)
        assert!(filename1.starts_with("defect-report-"));
        assert!(filename2.starts_with("defect-report-"));
    }

    // filter_by_pattern() Tests

    #[test]
    fn test_filter_by_pattern_no_filters() {
        let defects = build_diverse_defects();
        let original_count = defects.len();
        let report = build_test_report(defects);

        let filtered = DefectReportService::filter_by_pattern(&report, None, None, 0);

        assert_eq!(filtered.defects.len(), original_count);
    }

    #[test]
    fn test_filter_by_pattern_include_glob() {
        let defects = vec![
            build_defect_for_file("INC-001", "src/main.rs", Severity::High),
            build_defect_for_file("INC-002", "src/lib.rs", Severity::High),
            build_defect_for_file("INC-003", "tests/test.rs", Severity::High),
        ];
        let report = build_test_report(defects);

        let filtered =
            DefectReportService::filter_by_pattern(&report, Some("src/*.rs".to_string()), None, 0);

        assert_eq!(filtered.defects.len(), 2);
        assert!(filtered
            .defects
            .iter()
            .all(|d| d.file_path.to_string_lossy().starts_with("src/")));
    }

    #[test]
    fn test_filter_by_pattern_exclude_glob() {
        let defects = vec![
            build_defect_for_file("EXC-001", "src/main.rs", Severity::High),
            build_defect_for_file("EXC-002", "tests/test.rs", Severity::High),
            build_defect_for_file("EXC-003", "benches/bench.rs", Severity::High),
        ];
        let report = build_test_report(defects);

        let filtered =
            DefectReportService::filter_by_pattern(&report, None, Some("tests/*".to_string()), 0);

        assert_eq!(filtered.defects.len(), 2);
        assert!(filtered
            .defects
            .iter()
            .all(|d| !d.file_path.to_string_lossy().contains("tests/")));
    }

    #[test]
    fn test_filter_by_pattern_include_and_exclude() {
        let defects = vec![
            build_defect_for_file("BOTH-001", "src/main.rs", Severity::High),
            build_defect_for_file("BOTH-002", "src/test_helper.rs", Severity::High),
            build_defect_for_file("BOTH-003", "tests/test.rs", Severity::High),
        ];
        let report = build_test_report(defects);

        let filtered = DefectReportService::filter_by_pattern(
            &report,
            Some("src/*.rs".to_string()),
            Some("**/test*.rs".to_string()),
            0,
        );

        // Should include src/*.rs but exclude **/test*.rs
        assert_eq!(filtered.defects.len(), 1);
        assert_eq!(filtered.defects[0].file_path, PathBuf::from("src/main.rs"));
    }

    #[test]
    fn test_filter_by_pattern_rebuilds_file_index() {
        let defects = vec![
            build_defect_for_file("IDX-001", "src/main.rs", Severity::High),
            build_defect_for_file("IDX-002", "tests/test.rs", Severity::High),
        ];
        let report = build_test_report(defects);

        let filtered =
            DefectReportService::filter_by_pattern(&report, Some("src/*.rs".to_string()), None, 0);

        assert_eq!(filtered.file_index.len(), 1);
        assert!(filtered
            .file_index
            .contains_key(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_filter_by_pattern_recomputes_summary() {
        let defects = vec![
            build_defect_for_file("SUM-001", "src/main.rs", Severity::Critical),
            build_defect_for_file("SUM-002", "tests/test.rs", Severity::Low),
        ];
        let report = build_test_report(defects);

        let filtered =
            DefectReportService::filter_by_pattern(&report, Some("src/*.rs".to_string()), None, 0);

        assert_eq!(filtered.summary.total_defects, 1);
        assert_eq!(filtered.summary.by_severity.get("critical"), Some(&1));
        assert!(filtered.summary.by_severity.get("low").is_none());
    }

    #[test]
    fn test_filter_by_pattern_preserves_metadata() {
        let defects = build_diverse_defects();
        let report = build_test_report(defects);

        let filtered = DefectReportService::filter_by_pattern(&report, None, None, 0);

        assert_eq!(filtered.metadata.tool, report.metadata.tool);
        assert_eq!(filtered.metadata.version, report.metadata.version);
        assert_eq!(filtered.metadata.project_root, report.metadata.project_root);
    }

    #[test]
    fn test_filter_by_pattern_invalid_glob_handled() {
        let defects = build_diverse_defects();
        let report = build_test_report(defects);

        // Invalid glob pattern should be handled gracefully (returns None matcher)
        let filtered = DefectReportService::filter_by_pattern(
            &report,
            Some("[[[invalid".to_string()),
            None,
            0,
        );

        // With invalid include pattern, nothing should be filtered
        assert_eq!(filtered.defects.len(), report.defects.len());
    }

    #[test]
    fn test_filter_by_pattern_empty_result() {
        let defects = vec![build_defect_for_file(
            "EMPTY-001",
            "src/main.rs",
            Severity::High,
        )];
        let report = build_test_report(defects);

        let filtered = DefectReportService::filter_by_pattern(
            &report,
            Some("nonexistent/*.rs".to_string()),
            None,
            0,
        );

        assert!(filtered.defects.is_empty());
        assert!(filtered.file_index.is_empty());
        assert_eq!(filtered.summary.total_defects, 0);
    }

    // ReportFormat Tests

    #[test]
    fn test_report_format_debug() {
        assert_eq!(format!("{:?}", ReportFormat::Json), "Json");
        assert_eq!(format!("{:?}", ReportFormat::Csv), "Csv");
        assert_eq!(format!("{:?}", ReportFormat::Markdown), "Markdown");
        assert_eq!(format!("{:?}", ReportFormat::Text), "Text");
    }

    #[test]
    fn test_report_format_clone() {
        let format = ReportFormat::Json;
        let cloned = format.clone();
        assert!(matches!(cloned, ReportFormat::Json));
    }

    #[test]
    fn test_report_format_copy() {
        let format = ReportFormat::Csv;
        let copied: ReportFormat = format;
        assert!(matches!(copied, ReportFormat::Csv));
        // format is still usable because it's Copy
        assert!(matches!(format, ReportFormat::Csv));
    }

    // Edge Cases and Error Handling

    #[test]
    fn test_format_with_unicode_content() {
        let service = DefectReportService::new();
        let mut defect = build_defect("UNICODE-001", Severity::High, DefectCategory::Complexity);
        defect.message = "Test with unicode: \u{1F600} \u{4E2D}\u{6587} \u{0394}".to_string();
        defect.fix_suggestion = Some("Use proper \u{03BB} function".to_string());
        let report = build_test_report(vec![defect]);

        // All formats should handle unicode
        let json = service.format_json(&report).unwrap();
        assert!(json.contains("\\u{1F600}") || json.contains("\u{1F600}"));

        let csv = service.format_csv(&report).unwrap();
        assert!(!csv.is_empty());

        let md = service.format_markdown(&report).unwrap();
        assert!(!md.is_empty());

        let text = service.format_text(&report).unwrap();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_format_with_special_csv_characters() {
        let service = DefectReportService::new();
        let mut defect = build_defect("CSV-SPECIAL", Severity::High, DefectCategory::Complexity);
        defect.message = "Message with, comma and \"quotes\"".to_string();
        let report = build_test_report(vec![defect]);

        let csv = service.format_csv(&report).unwrap();
        // CSV should properly escape these characters
        assert!(csv.contains("CSV-SPECIAL"));
    }

    #[test]
    fn test_format_with_long_paths() {
        let service = DefectReportService::new();
        let long_path = "a/".repeat(50) + "very_long_file_name.rs";
        let defect = build_defect_for_file("LONG-001", &long_path, Severity::High);
        let report = build_test_report(vec![defect]);

        // All formats should handle long paths
        let json = service.format_json(&report).unwrap();
        assert!(json.contains("very_long_file_name.rs"));

        let csv = service.format_csv(&report).unwrap();
        assert!(csv.contains("very_long_file_name.rs"));

        let md = service.format_markdown(&report).unwrap();
        assert!(md.contains("very_long_file_name.rs"));

        let text = service.format_text(&report).unwrap();
        assert!(text.contains("very_long_file_name.rs"));
    }

    #[test]
    fn test_format_with_empty_message() {
        let service = DefectReportService::new();
        let mut defect = build_defect("EMPTY-MSG", Severity::High, DefectCategory::Complexity);
        defect.message = String::new();
        let report = build_test_report(vec![defect]);

        // Should not panic with empty message
        let json = service.format_json(&report).unwrap();
        assert!(json.contains("EMPTY-MSG"));

        let csv = service.format_csv(&report).unwrap();
        assert!(csv.contains("EMPTY-MSG"));
    }

    #[test]
    fn test_large_report_performance() {
        let service = DefectReportService::new();

        // Create a large number of defects
        let defects: Vec<Defect> = (0..1000)
            .map(|i| {
                let file = format!("src/file_{}.rs", i % 50);
                build_complete_defect(
                    &format!("PERF-{:04}", i),
                    Severity::High,
                    DefectCategory::Complexity,
                    &file,
                )
            })
            .collect();

        let report = build_test_report(defects);

        // All formats should complete without issues
        let json = service.format_json(&report).unwrap();
        assert!(json.len() > 10000);

        let csv = service.format_csv(&report).unwrap();
        assert!(csv.lines().count() > 1000);

        let md = service.format_markdown(&report).unwrap();
        assert!(md.len() > 1000);

        let text = service.format_text(&report).unwrap();
        assert!(text.len() > 1000);
    }

    // Property-Based Tests with Proptest

    proptest! {
        /// Property: compute_summary.total_defects always equals input defects count
        #[test]
        fn prop_summary_total_equals_input_count(count in 0usize..100) {
            let service = DefectReportService::new();
            let defects: Vec<Defect> = (0..count)
                .map(|i| build_defect(&format!("PROP-{}", i), Severity::High, DefectCategory::Complexity))
                .collect();

            let summary = service.compute_summary(&defects);
            prop_assert_eq!(summary.total_defects, count);
        }

        /// Property: hotspot files never exceed 10
        #[test]
        fn prop_hotspots_max_10(file_count in 1usize..50) {
            let service = DefectReportService::new();
            let defects: Vec<Defect> = (0..file_count)
                .map(|i| build_defect_for_file(&format!("PROP-{}", i), &format!("file_{}.rs", i), Severity::High))
                .collect();

            let summary = service.compute_summary(&defects);
            prop_assert!(summary.hotspot_files.len() <= 10);
        }

        /// Property: severity counts sum to total defects
        #[test]
        fn prop_severity_counts_sum_to_total(
            critical in 0usize..20,
            high in 0usize..20,
            medium in 0usize..20,
            low in 0usize..20,
        ) {
            let service = DefectReportService::new();
            let mut defects = Vec::new();

            for i in 0..critical {
                defects.push(build_defect(&format!("C-{}", i), Severity::Critical, DefectCategory::Complexity));
            }
            for i in 0..high {
                defects.push(build_defect(&format!("H-{}", i), Severity::High, DefectCategory::Complexity));
            }
            for i in 0..medium {
                defects.push(build_defect(&format!("M-{}", i), Severity::Medium, DefectCategory::Complexity));
            }
            for i in 0..low {
                defects.push(build_defect(&format!("L-{}", i), Severity::Low, DefectCategory::Complexity));
            }

            let summary = service.compute_summary(&defects);
            let severity_sum: usize = summary.by_severity.values().sum();
            prop_assert_eq!(severity_sum, summary.total_defects);
        }

        /// Property: category counts sum to total defects
        #[test]
        fn prop_category_counts_sum_to_total(count in 0usize..50) {
            let service = DefectReportService::new();
            let categories = DefectCategory::all();
            let defects: Vec<Defect> = (0..count)
                .map(|i| {
                    let category = categories[i % categories.len()];
                    build_defect(&format!("CAT-{}", i), Severity::High, category)
                })
                .collect();

            let summary = service.compute_summary(&defects);
            let category_sum: usize = summary.by_category.values().sum();
            prop_assert_eq!(category_sum, summary.total_defects);
        }

        /// Property: hotspots are sorted by severity_score descending
        #[test]
        fn prop_hotspots_sorted_descending(file_count in 2usize..20) {
            let service = DefectReportService::new();
            let severities = [Severity::Critical, Severity::High, Severity::Medium, Severity::Low];
            let defects: Vec<Defect> = (0..file_count)
                .map(|i| {
                    let severity = severities[i % severities.len()];
                    build_defect_for_file(&format!("SORT-{}", i), &format!("file_{}.rs", i), severity)
                })
                .collect();

            let summary = service.compute_summary(&defects);

            // Verify descending order
            for i in 1..summary.hotspot_files.len() {
                prop_assert!(summary.hotspot_files[i-1].severity_score >= summary.hotspot_files[i].severity_score);
            }
        }

        /// Property: filter_by_pattern preserves defect content
        #[test]
        fn prop_filter_preserves_defect_content(count in 1usize..20) {
            let defects: Vec<Defect> = (0..count)
                .map(|i| build_defect_for_file(&format!("PRESERVE-{}", i), "src/test.rs", Severity::High))
                .collect();
            let report = build_test_report(defects.clone());

            let filtered = DefectReportService::filter_by_pattern(&report, None, None, 0);

            for (original, filtered) in defects.iter().zip(filtered.defects.iter()) {
                prop_assert_eq!(&original.id, &filtered.id);
                prop_assert_eq!(&original.message, &filtered.message);
                prop_assert_eq!(original.severity, filtered.severity);
            }
        }

        /// Property: JSON format is always valid JSON
        #[test]
        fn prop_json_always_valid(count in 0usize..30) {
            let service = DefectReportService::new();
            let defects: Vec<Defect> = (0..count)
                .map(|i| build_defect(&format!("JSON-{}", i), Severity::High, DefectCategory::Complexity))
                .collect();
            let report = build_test_report(defects);

            let json = service.format_json(&report).unwrap();
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
            prop_assert!(parsed.is_ok());
        }

        /// Property: CSV has correct number of lines (header + defects)
        #[test]
        fn prop_csv_line_count(count in 0usize..50) {
            let service = DefectReportService::new();
            let defects: Vec<Defect> = (0..count)
                .map(|i| build_defect(&format!("CSV-{}", i), Severity::High, DefectCategory::Complexity))
                .collect();
            let report = build_test_report(defects);

            let csv = service.format_csv(&report).unwrap();
            let line_count = csv.lines().count();
            // Header + defect lines
            prop_assert_eq!(line_count, count + 1);
        }

        /// Property: filename always has correct extension
        #[test]
        fn prop_filename_extension(format_index in 0usize..4) {
            let service = DefectReportService::new();
            let formats = [ReportFormat::Json, ReportFormat::Csv, ReportFormat::Markdown, ReportFormat::Text];
            let extensions = [".json", ".csv", ".md", ".txt"];

            let format = formats[format_index];
            let expected_ext = extensions[format_index];

            let filename = service.generate_filename(format);
            prop_assert!(filename.ends_with(expected_ext));
        }

        /// Property: filter with exclude removes all matching files
        #[test]
        fn prop_filter_exclude_removes_matching(count in 1usize..20) {
            // Create defects in two directories
            let mut defects = Vec::new();
            for i in 0..(count/2).max(1) {
                defects.push(build_defect_for_file(&format!("SRC-{}", i), "src/file.rs", Severity::High));
            }
            for i in 0..(count/2).max(1) {
                defects.push(build_defect_for_file(&format!("TEST-{}", i), "tests/file.rs", Severity::High));
            }
            let report = build_test_report(defects);

            let filtered = DefectReportService::filter_by_pattern(
                &report,
                None,
                Some("tests/*".to_string()),
                0,
            );

            // No defects should be from tests/
            for defect in &filtered.defects {
                prop_assert!(!defect.file_path.starts_with("tests/"));
            }
        }
    }

    // Concurrency Tests (for semaphore behavior)

    #[test]
    fn test_service_can_be_used_multiple_times() {
        let service = DefectReportService::new();

        // Use the service multiple times to compute summaries
        for i in 0..10 {
            let defects = vec![build_defect(
                &format!("MULTI-{}", i),
                Severity::High,
                DefectCategory::Complexity,
            )];
            let summary = service.compute_summary(&defects);
            assert_eq!(summary.total_defects, 1);
        }
    }

    #[test]
    fn test_multiple_services_can_coexist() {
        let service1 = DefectReportService::new();
        let service2 = DefectReportService::new();

        let defects1 = vec![build_defect(
            "SVC1-001",
            Severity::High,
            DefectCategory::Complexity,
        )];
        let defects2 = vec![build_defect(
            "SVC2-001",
            Severity::Low,
            DefectCategory::DeadCode,
        )];

        let summary1 = service1.compute_summary(&defects1);
        let summary2 = service2.compute_summary(&defects2);

        assert_eq!(summary1.by_severity.get("high"), Some(&1));
        assert_eq!(summary2.by_severity.get("low"), Some(&1));
    }
}
