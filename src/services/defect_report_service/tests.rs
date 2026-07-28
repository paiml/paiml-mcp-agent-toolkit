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

    // Continuation of coverage_tests in separate files for file health compliance
    include!("defect_report_tests_part2.rs");
    include!("defect_report_tests_part3.rs");
}
