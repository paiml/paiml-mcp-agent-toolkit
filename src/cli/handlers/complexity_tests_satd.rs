#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod satd_format_tests {
    use super::*;
    use crate::cli::{SatdOutputFormat, SatdSeverity};
    use crate::services::satd_detector::{
        DebtCategory, SATDAnalysisResult, SATDSummary, Severity, TechnicalDebt,
    };
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_comprehensive_satd_result() -> SATDAnalysisResult {
        let mut by_severity = HashMap::new();
        by_severity.insert("Critical".to_string(), 2);
        by_severity.insert("High".to_string(), 5);
        by_severity.insert("Medium".to_string(), 8);
        by_severity.insert("Low".to_string(), 10);

        let mut by_category = HashMap::new();
        by_category.insert("Defect".to_string(), 7);
        by_category.insert("Requirement".to_string(), 10);
        by_category.insert("Design".to_string(), 5);
        by_category.insert("Security".to_string(), 3);

        SATDAnalysisResult {
            items: vec![
                TechnicalDebt {
                    category: DebtCategory::Security,
                    severity: Severity::Critical,
                    text: "SECURITY: Validate user input".to_string(),
                    file: PathBuf::from("src/auth.rs"),
                    line: 45,
                    column: 8,
                    context_hash: [0u8; 16],
                },
                TechnicalDebt {
                    category: DebtCategory::Security,
                    severity: Severity::Critical,
                    text: "VULN: SQL injection risk".to_string(),
                    file: PathBuf::from("src/db.rs"),
                    line: 120,
                    column: 4,
                    context_hash: [1u8; 16],
                },
                TechnicalDebt {
                    category: DebtCategory::Defect,
                    severity: Severity::High,
                    text: "BUG: Race condition in cache".to_string(),
                    file: PathBuf::from("src/cache.rs"),
                    line: 88,
                    column: 12,
                    context_hash: [2u8; 16],
                },
                TechnicalDebt {
                    category: DebtCategory::Requirement,
                    severity: Severity::Medium,
                    text: "TODO: Add pagination support".to_string(),
                    file: PathBuf::from("src/api.rs"),
                    line: 200,
                    column: 4,
                    context_hash: [3u8; 16],
                },
                TechnicalDebt {
                    category: DebtCategory::Requirement,
                    severity: Severity::Low,
                    text: "TODO: Nice to have feature".to_string(),
                    file: PathBuf::from("src/utils.rs"),
                    line: 30,
                    column: 4,
                    context_hash: [4u8; 16],
                },
            ],
            summary: SATDSummary {
                total_items: 25,
                by_severity,
                by_category,
                files_with_satd: 8,
                avg_age_days: 45.5,
            },
            total_files_analyzed: 50,
            files_with_debt: 8,
            analysis_timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_generate_satd_sarif() {
        let result = create_comprehensive_satd_result();

        let sarif = generate_satd_sarif(&result);

        // Verify SARIF structure
        assert_eq!(sarif["version"], "2.1.0");
        assert!(sarif["$schema"].as_str().unwrap().contains("sarif-schema"));

        // Check tool information
        let driver = &sarif["runs"][0]["tool"]["driver"];
        assert_eq!(driver["name"], "pmat");
        assert!(driver["rules"].as_array().unwrap().len() > 0);

        // Check results
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 5);

        // Verify severity mapping (Critical -> error)
        let critical_results: Vec<_> = results
            .iter()
            .filter(|r| r["level"] == "error")
            .collect();
        assert!(critical_results.len() >= 2); // We have 2 Critical items
    }

    #[test]
    fn test_format_satd_output_json() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_output(&result, SatdOutputFormat::Json, false, false, 30)
            .expect("JSON formatting should work");

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Should be valid JSON");

        assert!(parsed.get("items").is_some());
        assert!(parsed.get("summary").is_some());
        assert_eq!(parsed["total_files_analyzed"], 50);
    }

    #[test]
    fn test_format_satd_output_sarif() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_output(&result, SatdOutputFormat::Sarif, false, false, 30)
            .expect("SARIF formatting should work");

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Should be valid JSON");
        assert_eq!(parsed["version"], "2.1.0");
    }

    #[test]
    fn test_format_satd_output_summary() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_output(&result, SatdOutputFormat::Summary, false, false, 30)
            .expect("Summary formatting should work");

        assert!(output.contains("SATD Analysis Summary"));
        assert!(output.contains("Files analyzed**: 50"));
        assert!(output.contains("Files with SATD**: 8"));
    }

    #[test]
    fn test_format_satd_output_markdown() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_output(&result, SatdOutputFormat::Markdown, true, false, 30)
            .expect("Markdown formatting should work");

        assert!(output.contains("# Self-Admitted Technical Debt Report"));
        assert!(output.contains("## Summary"));
        assert!(output.contains("| Metric | Value |"));
        assert!(output.contains("## Distribution"));
        assert!(output.contains("### By Severity"));
        assert!(output.contains("### By Category"));
    }

    #[test]
    fn test_format_satd_markdown_groups_by_file() {
        let result = create_comprehensive_satd_result();

        let output = format_satd_markdown(&result, true, false, 30);

        // Check that file grouping is present
        assert!(output.contains("## SATD Items by File"));
        assert!(output.contains("### src/auth.rs"));
        assert!(output.contains("| Line | Severity | Category | Text |"));
    }

    #[test]
    fn test_apply_satd_filters_severity_high() {
        let mut result = create_comprehensive_satd_result();
        let original_count = result.items.len();

        apply_satd_filters(&mut result, Some(SatdSeverity::High), false, 0);

        // Should only keep Critical and High items
        assert!(result.items.len() < original_count);
        assert!(result.items.iter().all(|i| i.severity >= Severity::High));
    }

    #[test]
    fn test_apply_satd_filters_critical_only() {
        let mut result = create_comprehensive_satd_result();

        apply_satd_filters(&mut result, None, true, 0);

        // Should only keep Critical items
        assert_eq!(result.items.len(), 2);
        assert!(result
            .items
            .iter()
            .all(|i| i.severity == Severity::Critical));
    }

    #[test]
    fn test_apply_satd_filters_top_files() {
        let mut result = create_comprehensive_satd_result();

        // Apply top files = 2
        apply_satd_filters(&mut result, None, false, 2);

        // Should only have items from top 2 files
        let unique_files: std::collections::HashSet<_> =
            result.items.iter().map(|i| &i.file).collect();
        assert!(unique_files.len() <= 2);
    }

    #[test]
    fn test_filter_top_files_helper() {
        let mut result = create_comprehensive_satd_result();
        let original_item_count = result.items.len();

        filter_top_files(&mut result, 1);

        // Should have fewer items (only from top file)
        assert!(result.items.len() <= original_item_count);
        assert!(!result.items.is_empty());
    }

    #[test]
    fn test_write_top_files_with_satd_section() {
        let result = create_comprehensive_satd_result();
        let mut output = String::new();

        write_top_files_with_satd_section(&mut output, &result);

        assert!(output.contains("## Top Files with SATD"));
        assert!(output.contains("SATD items"));
    }

    #[test]
    fn test_write_critical_items_section() {
        let result = create_comprehensive_satd_result();
        let mut output = String::new();

        write_critical_items_section(&mut output, &result);

        assert!(output.contains("## Critical Items"));
        // Should contain the critical items' file names
        assert!(output.contains("auth.rs") || output.contains("db.rs"));
    }
}

// Extended Coverage Tests - Churn Analysis

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod churn_tests {
    use super::*;
    use crate::utils::file_filter::FileFilter;

    #[test]
    fn test_create_and_report_file_filter_empty() {
        let filter = create_and_report_file_filter(vec![], vec![]).expect("Should create filter");

        assert!(!filter.has_filters());
    }

    #[test]
    fn test_create_and_report_file_filter_with_include() {
        let filter = create_and_report_file_filter(
            vec!["src/**/*.rs".to_string()],
            vec![],
        )
        .expect("Should create filter");

        assert!(filter.has_filters());
        assert!(filter.should_include(Path::new("src/main.rs")));
    }

    #[test]
    fn test_create_and_report_file_filter_with_exclude() {
        let filter = create_and_report_file_filter(
            vec![],
            vec!["target/**".to_string()],
        )
        .expect("Should create filter");

        assert!(filter.has_filters());
        assert!(!filter.should_include(Path::new("target/debug/main")));
    }

    #[test]
    fn test_create_and_report_file_filter_combined() {
        let filter = create_and_report_file_filter(
            vec!["**/*.rs".to_string()],
            vec!["target/**".to_string()],
        )
        .expect("Should create filter");

        assert!(filter.has_filters());
        assert!(filter.should_include(Path::new("src/main.rs")));
        assert!(!filter.should_include(Path::new("target/debug/main.rs")));
    }
}

// Extended Coverage Tests - Watch Mode Helpers

