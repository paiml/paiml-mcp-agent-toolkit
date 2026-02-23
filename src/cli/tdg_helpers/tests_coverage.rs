#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use super::markdown_format::{
    calculate_component_score, write_component_breakdown, write_hotspot_basic_info,
    write_single_hotspot, write_tdg_header, write_tdg_hotspots, write_tdg_summary,
};
use super::sarif_format::generate_tdg_rules;
use crate::models::tdg::{TDGHotspot, TDGSummary};
use std::path::PathBuf;

// ============================================================
// Helper functions to create test data
// ============================================================

fn create_test_hotspot(path: &str, score: f64, factor: &str, hours: f64) -> TDGHotspot {
    TDGHotspot {
        path: path.to_string(),
        tdg_score: score,
        primary_factor: factor.to_string(),
        estimated_hours: hours,
    }
}

fn create_test_summary() -> TDGSummary {
    TDGSummary {
        total_files: 100,
        critical_files: 5,
        warning_files: 15,
        average_tdg: 1.5,
        p95_tdg: 2.8,
        p99_tdg: 3.5,
        estimated_debt_hours: 80.0,
        hotspots: vec![],
    }
}

fn create_test_hotspots() -> Vec<TDGHotspot> {
    vec![
        create_test_hotspot("src/complex.rs", 3.5, "complexity", 12.0),
        create_test_hotspot("src/churn.rs", 2.8, "churn", 8.0),
        create_test_hotspot("src/coupling.rs", 2.0, "coupling", 5.0),
        create_test_hotspot("src/normal.rs", 1.2, "none", 2.0),
        create_test_hotspot("src/low.rs", 0.5, "none", 0.5),
    ]
}

// ============================================================
// Tests for filter_tdg_hotspots
// ============================================================

mod filter_tests {
    use super::*;

    #[test]
    fn test_filter_no_filters() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots.clone(), 0.0, 0, false);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filter_by_threshold() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots, 2.0, 0, false);
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|h| h.tdg_score >= 2.0));
    }

    #[test]
    fn test_filter_by_threshold_high() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots, 3.0, 0, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, "src/complex.rs");
    }

    #[test]
    fn test_filter_critical_only() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots, 0.0, 0, true);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|h| h.tdg_score > 2.5));
    }

    #[test]
    fn test_filter_top_limit() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots, 0.0, 3, false);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_filter_top_limit_larger_than_list() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots, 0.0, 100, false);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filter_combined_threshold_and_critical() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots, 1.0, 0, true);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_combined_all_filters() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots, 1.0, 1, true);
        assert_eq!(result.len(), 1);
        assert!(result[0].tdg_score > 2.5);
    }

    #[test]
    fn test_filter_empty_hotspots() {
        let hotspots: Vec<TDGHotspot> = vec![];
        let result = filter_tdg_hotspots(hotspots, 1.0, 10, true);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_zero_threshold() {
        let hotspots = create_test_hotspots();
        let result = filter_tdg_hotspots(hotspots, 0.0, 0, false);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filter_negative_threshold_treated_as_no_filter() {
        let hotspots = create_test_hotspots();
        // Negative threshold should not filter (threshold > 0.0 check)
        let result = filter_tdg_hotspots(hotspots, -1.0, 0, false);
        assert_eq!(result.len(), 5);
    }
}

// ============================================================
// Tests for format_tdg_json
// ============================================================

mod json_format_tests {
    use super::*;

    #[test]
    fn test_format_json_basic() {
        let summary = create_test_summary();
        let hotspots = create_test_hotspots();
        let result = format_tdg_json(&summary, &hotspots, false);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["summary"]["total_files"], 100);
        assert_eq!(parsed["hotspots"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn test_format_json_with_components() {
        let summary = create_test_summary();
        let hotspots = create_test_hotspots();
        let result = format_tdg_json(&summary, &hotspots, true).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["components"].is_object());
        assert_eq!(parsed["components"]["complexity_weight"], 0.4);
    }

    #[test]
    fn test_format_json_without_components() {
        let summary = create_test_summary();
        let hotspots = create_test_hotspots();
        let result = format_tdg_json(&summary, &hotspots, false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["components"].is_null());
    }

    #[test]
    fn test_format_json_empty_hotspots() {
        let summary = create_test_summary();
        let result = format_tdg_json(&summary, &[], false).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["hotspots"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_format_json_is_valid_json() {
        let summary = create_test_summary();
        let hotspots = vec![create_test_hotspot(
            "path/with spaces/file.rs",
            2.5,
            "special\"chars",
            5.0,
        )];
        let result = format_tdg_json(&summary, &hotspots, true).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }
}

// ============================================================
// Tests for format_tdg_table
// ============================================================

mod table_format_tests {
    use super::*;

    #[test]
    fn test_format_table_basic() {
        let hotspots = create_test_hotspots();
        let result = format_tdg_table(&hotspots, false);
        assert!(result.is_ok());
        let table = result.unwrap();
        assert!(table.contains("| File | TDG Score |"));
        assert!(table.contains("|------|-----------|"));
    }

    #[test]
    fn test_format_table_contains_hotspot_data() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "complexity", 5.0)];
        let result = format_tdg_table(&hotspots, false).unwrap();
        assert!(result.contains("test.rs"));
        assert!(result.contains("2.50"));
        assert!(result.contains("complexity"));
        assert!(result.contains("5.0"));
    }

    #[test]
    fn test_format_table_verbose_mode() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "complexity", 5.0)];
        let result = format_tdg_table(&hotspots, true).unwrap();
        assert!(result.contains("Components:"));
        assert!(result.contains("complexity="));
        assert!(result.contains("churn="));
    }

    #[test]
    fn test_format_table_non_verbose_no_components() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "complexity", 5.0)];
        let result = format_tdg_table(&hotspots, false).unwrap();
        assert!(!result.contains("Components:"));
    }

    #[test]
    fn test_format_table_empty_hotspots() {
        let result = format_tdg_table(&[], false).unwrap();
        assert!(result.contains("| File | TDG Score |"));
        // Should only have header
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2); // header + separator
    }

    #[test]
    fn test_format_table_multiple_hotspots() {
        let hotspots = create_test_hotspots();
        let result = format_tdg_table(&hotspots, false).unwrap();
        let data_lines = result.lines().count() - 2; // minus header and separator
        assert_eq!(data_lines, 5);
    }

    #[test]
    fn test_format_table_extracts_filename_from_path() {
        let hotspots = vec![create_test_hotspot(
            "very/deep/path/to/file.rs",
            1.0,
            "none",
            1.0,
        )];
        let result = format_tdg_table(&hotspots, false).unwrap();
        assert!(result.contains("file.rs"));
        // Should not contain the full path in the table
        assert!(!result.contains("very/deep/path/to/file.rs"));
    }
}

// ============================================================
// Tests for format_tdg_markdown
// ============================================================

mod markdown_format_tests {
    use super::*;

    #[test]
    fn test_format_markdown_basic() {
        let summary = create_test_summary();
        let hotspots = create_test_hotspots();
        let result = format_tdg_markdown(&summary, &hotspots, false);
        assert!(result.is_ok());
        let md = result.unwrap();
        assert!(md.contains("# Technical Debt Gradient Analysis"));
    }

    #[test]
    fn test_format_markdown_summary_content() {
        let summary = create_test_summary();
        let result = format_tdg_markdown(&summary, &[], false).unwrap();
        assert!(result.contains("**Total Files**: 100"));
        assert!(result.contains("**Critical Files**: 5"));
        assert!(result.contains("**Warning Files**: 15"));
        assert!(result.contains("**Average TDG**: 1.500"));
        assert!(result.contains("**95th Percentile**: 2.800"));
    }

    #[test]
    fn test_format_markdown_with_hotspots() {
        let summary = create_test_summary();
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "complexity", 5.0)];
        let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();
        assert!(result.contains("## Top Hotspots"));
        assert!(result.contains("### 1. src/test.rs"));
        assert!(result.contains("**TDG Score**: 2.500"));
        assert!(result.contains("**Primary Factor**: complexity"));
        assert!(result.contains("**Estimated Hours**: 5.0"));
    }

    #[test]
    fn test_format_markdown_with_components() {
        let summary = create_test_summary();
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.0, "complexity", 5.0)];
        let result = format_tdg_markdown(&summary, &hotspots, true).unwrap();
        assert!(result.contains("#### Component Breakdown:"));
        assert!(result.contains("Complexity: 0.800"));
        assert!(result.contains("Churn: 0.600"));
        assert!(result.contains("Duplication: 0.400"));
        assert!(result.contains("Coupling: 0.200"));
    }

    #[test]
    fn test_format_markdown_without_components() {
        let summary = create_test_summary();
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.0, "complexity", 5.0)];
        let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();
        assert!(!result.contains("Component Breakdown"));
    }

    #[test]
    fn test_format_markdown_empty_hotspots_no_section() {
        let summary = create_test_summary();
        let result = format_tdg_markdown(&summary, &[], false).unwrap();
        assert!(!result.contains("## Top Hotspots"));
    }

    #[test]
    fn test_format_markdown_multiple_hotspots_indexed() {
        let summary = create_test_summary();
        let hotspots = vec![
            create_test_hotspot("src/a.rs", 3.0, "complexity", 10.0),
            create_test_hotspot("src/b.rs", 2.0, "churn", 5.0),
            create_test_hotspot("src/c.rs", 1.0, "coupling", 2.0),
        ];
        let result = format_tdg_markdown(&summary, &hotspots, false).unwrap();
        assert!(result.contains("### 1. src/a.rs"));
        assert!(result.contains("### 2. src/b.rs"));
        assert!(result.contains("### 3. src/c.rs"));
    }
}

// ============================================================
// Tests for format_tdg_sarif
// ============================================================

mod sarif_format_tests {
    use super::*;

    #[test]
    fn test_format_sarif_basic_structure() {
        let hotspots = create_test_hotspots();
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/test"));
        assert!(result.is_ok());
        let sarif_str = result.unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
        assert_eq!(sarif["version"], "2.1.0");
    }

    #[test]
    fn test_format_sarif_tool_info() {
        let hotspots = create_test_hotspots();
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/test")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let tool = &sarif["runs"][0]["tool"]["driver"];
        assert_eq!(tool["name"], "paiml-tdg-analyzer");
        assert!(!tool["version"].as_str().unwrap().is_empty());
        assert!(tool["informationUri"]
            .as_str()
            .unwrap()
            .contains("github.com"));
    }

    #[test]
    fn test_format_sarif_rules_present() {
        let hotspots = create_test_hotspots();
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/test")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let rules = sarif["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(rules.len(), 3);

        let rule_ids: Vec<&str> = rules
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert!(rule_ids.contains(&"critical-tdg"));
        assert!(rule_ids.contains(&"high-tdg"));
        assert!(rule_ids.contains(&"moderate-tdg"));
    }

    #[test]
    fn test_format_sarif_critical_hotspot() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 3.0, "complexity", 10.0)];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[0]["ruleId"], "critical-tdg");
    }

    #[test]
    fn test_format_sarif_warning_hotspot() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.0, "churn", 5.0)];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "warning");
        assert_eq!(results[0]["ruleId"], "high-tdg");
    }

    #[test]
    fn test_format_sarif_note_hotspot() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 1.0, "none", 2.0)];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "note");
        assert_eq!(results[0]["ruleId"], "moderate-tdg");
    }

    #[test]
    fn test_format_sarif_message_format() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.5, "complexity", 5.0)];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let msg = sarif["runs"][0]["results"][0]["message"]["text"]
            .as_str()
            .unwrap();
        assert!(msg.contains("2.50"));
        assert!(msg.contains("complexity"));
        assert!(msg.contains("5.0 hours"));
    }

    #[test]
    fn test_format_sarif_location() {
        let hotspots = vec![create_test_hotspot("/test/src/file.rs", 2.0, "churn", 5.0)];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/test")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let uri = sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
            ["artifactLocation"]["uri"]
            .as_str()
            .unwrap();
        assert_eq!(uri, "src/file.rs");
    }

    #[test]
    fn test_format_sarif_location_no_strip() {
        let hotspots = vec![create_test_hotspot("other/path/file.rs", 2.0, "churn", 5.0)];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/test")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let uri = sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
            ["artifactLocation"]["uri"]
            .as_str()
            .unwrap();
        assert_eq!(uri, "other/path/file.rs");
    }

    #[test]
    fn test_format_sarif_empty_hotspots() {
        let result = format_tdg_sarif(&[], &PathBuf::from("/test")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert!(results.is_empty());
        // Rules should still be present
        let rules = sarif["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_format_sarif_multiple_severity_levels() {
        let hotspots = vec![
            create_test_hotspot("src/critical.rs", 3.0, "complexity", 10.0),
            create_test_hotspot("src/warning.rs", 2.0, "churn", 5.0),
            create_test_hotspot("src/note.rs", 1.0, "none", 2.0),
        ];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[1]["level"], "warning");
        assert_eq!(results[2]["level"], "note");
    }

    #[test]
    fn test_format_sarif_boundary_values() {
        // Test exact boundary values for severity levels
        let hotspots = vec![
            create_test_hotspot("src/exactly_2.5.rs", 2.5, "boundary", 5.0),
            create_test_hotspot("src/exactly_1.5.rs", 1.5, "boundary", 3.0),
        ];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&result).unwrap();
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        // 2.5 is NOT > 2.5, so should be warning
        assert_eq!(results[0]["level"], "warning");
        assert_eq!(results[0]["ruleId"], "high-tdg");
        // 1.5 is NOT > 1.5, so should be note
        assert_eq!(results[1]["level"], "note");
        assert_eq!(results[1]["ruleId"], "moderate-tdg");
    }
}

// ============================================================
// Tests for internal helpers
// ============================================================

mod internal_helpers_tests {
    use super::*;

    #[test]
    fn test_calculate_component_score() {
        assert!((calculate_component_score(10.0, 0.4) - 4.0).abs() < f64::EPSILON);
        assert!((calculate_component_score(10.0, 0.3) - 3.0).abs() < f64::EPSILON);
        assert!((calculate_component_score(10.0, 0.2) - 2.0).abs() < f64::EPSILON);
        assert!((calculate_component_score(10.0, 0.1) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_component_score_zero() {
        assert!((calculate_component_score(0.0, 0.5)).abs() < f64::EPSILON);
        assert!((calculate_component_score(5.0, 0.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_write_tdg_header() {
        let mut output = String::new();
        write_tdg_header(&mut output).unwrap();
        assert!(output.contains("# Technical Debt Gradient Analysis"));
    }

    #[test]
    fn test_write_tdg_summary() {
        let mut output = String::new();
        let summary = create_test_summary();
        write_tdg_summary(&mut output, &summary).unwrap();
        assert!(output.contains("## Summary"));
        assert!(output.contains("**Total Files**: 100"));
    }

    #[test]
    fn test_write_hotspot_basic_info() {
        let mut output = String::new();
        let hotspot = create_test_hotspot("src/test.rs", 2.5, "complexity", 5.0);
        write_hotspot_basic_info(&mut output, &hotspot).unwrap();
        assert!(output.contains("**TDG Score**: 2.500"));
        assert!(output.contains("**Primary Factor**: complexity"));
    }

    #[test]
    fn test_write_component_breakdown() {
        let mut output = String::new();
        let hotspot = create_test_hotspot("src/test.rs", 2.0, "complexity", 5.0);
        write_component_breakdown(&mut output, &hotspot).unwrap();
        assert!(output.contains("#### Component Breakdown:"));
        assert!(output.contains("Complexity: 0.800"));
    }

    #[test]
    fn test_write_single_hotspot_with_components() {
        let mut output = String::new();
        let hotspot = create_test_hotspot("src/test.rs", 2.0, "complexity", 5.0);
        write_single_hotspot(&mut output, 1, &hotspot, true).unwrap();
        assert!(output.contains("### 1. src/test.rs"));
        assert!(output.contains("Component Breakdown"));
    }

    #[test]
    fn test_write_single_hotspot_without_components() {
        let mut output = String::new();
        let hotspot = create_test_hotspot("src/test.rs", 2.0, "complexity", 5.0);
        write_single_hotspot(&mut output, 1, &hotspot, false).unwrap();
        assert!(output.contains("### 1. src/test.rs"));
        assert!(!output.contains("Component Breakdown"));
    }

    #[test]
    fn test_write_tdg_hotspots() {
        let mut output = String::new();
        let hotspots = vec![
            create_test_hotspot("src/a.rs", 3.0, "complexity", 10.0),
            create_test_hotspot("src/b.rs", 2.0, "churn", 5.0),
        ];
        write_tdg_hotspots(&mut output, &hotspots, false).unwrap();
        assert!(output.contains("## Top Hotspots"));
        assert!(output.contains("### 1. src/a.rs"));
        assert!(output.contains("### 2. src/b.rs"));
    }

    #[test]
    fn test_generate_tdg_rules() {
        let rules = generate_tdg_rules();
        assert_eq!(rules.len(), 3);

        let rule_ids: Vec<&str> = rules
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert!(rule_ids.contains(&"critical-tdg"));
        assert!(rule_ids.contains(&"high-tdg"));
        assert!(rule_ids.contains(&"moderate-tdg"));

        // Check severity levels
        assert_eq!(
            rules[0]["defaultConfiguration"]["level"]
                .as_str()
                .unwrap(),
            "error"
        );
        assert_eq!(
            rules[1]["defaultConfiguration"]["level"]
                .as_str()
                .unwrap(),
            "warning"
        );
        assert_eq!(
            rules[2]["defaultConfiguration"]["level"]
                .as_str()
                .unwrap(),
            "note"
        );
    }
}

// ============================================================
// Edge case tests
// ============================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_hotspot_with_empty_path() {
        let hotspots = vec![create_test_hotspot("", 2.0, "complexity", 5.0)];
        let result = format_tdg_table(&hotspots, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hotspot_with_special_characters_in_path() {
        let hotspots = vec![create_test_hotspot(
            "src/my file (1).rs",
            2.0,
            "complexity",
            5.0,
        )];
        let result = format_tdg_json(
            &create_test_summary(),
            &hotspots,
            false,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_hotspot_with_unicode_path() {
        let hotspots = vec![create_test_hotspot("src/日本語.rs", 2.0, "complexity", 5.0)];
        let result = format_tdg_table(&hotspots, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_summary_with_zero_values() {
        let summary = TDGSummary {
            total_files: 0,
            critical_files: 0,
            warning_files: 0,
            average_tdg: 0.0,
            p95_tdg: 0.0,
            p99_tdg: 0.0,
            estimated_debt_hours: 0.0,
            hotspots: vec![],
        };
        let result = format_tdg_markdown(&summary, &[], false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hotspot_with_very_high_score() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 999.99, "complexity", 1000.0)];
        let result = format_tdg_sarif(&hotspots, &PathBuf::from("/"));
        assert!(result.is_ok());
        let sarif_str = result.unwrap();
        let sarif: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
        assert_eq!(sarif["runs"][0]["results"][0]["level"], "error");
    }

    #[test]
    fn test_hotspot_with_zero_score() {
        let hotspots = vec![create_test_hotspot("src/test.rs", 0.0, "none", 0.0)];
        let result = format_tdg_table(&hotspots, true).unwrap();
        assert!(result.contains("0.00"));
    }

    #[test]
    fn test_hotspot_with_negative_score() {
        let hotspots = vec![create_test_hotspot("src/test.rs", -1.0, "none", 0.0)];
        let result = format_tdg_table(&hotspots, false).unwrap();
        assert!(result.contains("-1.00"));
    }

    #[test]
    fn test_very_long_primary_factor() {
        let long_factor = "a".repeat(1000);
        let hotspots = vec![create_test_hotspot("src/test.rs", 2.0, &long_factor, 5.0)];
        let result = format_tdg_table(&hotspots, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_special_chars_in_factor() {
        let hotspots = vec![
            create_test_hotspot("src/test.rs", 2.0, "factor<>\"'&", 5.0),
        ];
        // JSON should properly escape special characters
        let result = format_tdg_json(
            &create_test_summary(),
            &hotspots,
            false,
        );
        assert!(result.is_ok());
        let json_str = result.unwrap();
        // Verify it's still valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&json_str).is_ok());
    }
}

// ============================================================
// Integration tests
// ============================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow_json() {
        let summary = create_test_summary();
        let hotspots = create_test_hotspots();

        // Filter
        let filtered = filter_tdg_hotspots(hotspots, 2.0, 3, false);
        assert_eq!(filtered.len(), 3);

        // Format
        let json = format_tdg_json(&summary, &filtered, true).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["hotspots"].as_array().unwrap().len(), 3);
        assert!(parsed["components"].is_object());
    }

    #[test]
    fn test_full_workflow_markdown() {
        let summary = create_test_summary();
        let hotspots = create_test_hotspots();

        let filtered = filter_tdg_hotspots(hotspots, 1.0, 0, false);
        let md = format_tdg_markdown(&summary, &filtered, true).unwrap();

        assert!(md.contains("# Technical Debt Gradient Analysis"));
        assert!(md.contains("## Summary"));
        assert!(md.contains("## Top Hotspots"));
        assert!(md.contains("Component Breakdown"));
    }

    #[test]
    fn test_full_workflow_sarif() {
        let hotspots = create_test_hotspots();
        let filtered = filter_tdg_hotspots(hotspots, 0.0, 0, false);
        let sarif = format_tdg_sarif(&filtered, &PathBuf::from("/project")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        assert_eq!(parsed["runs"][0]["results"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn test_all_formats_produce_valid_output() {
        let summary = create_test_summary();
        let hotspots = create_test_hotspots();

        // All format functions should succeed
        assert!(format_tdg_json(&summary, &hotspots, false).is_ok());
        assert!(format_tdg_json(&summary, &hotspots, true).is_ok());
        assert!(format_tdg_table(&hotspots, false).is_ok());
        assert!(format_tdg_table(&hotspots, true).is_ok());
        assert!(format_tdg_markdown(&summary, &hotspots, false).is_ok());
        assert!(format_tdg_markdown(&summary, &hotspots, true).is_ok());
        assert!(format_tdg_sarif(&hotspots, &PathBuf::from("/")).is_ok());

        // Verify JSON outputs are parseable
        let json = format_tdg_json(&summary, &hotspots, true).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());

        let sarif = format_tdg_sarif(&hotspots, &PathBuf::from("/")).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&sarif).is_ok());
    }
}
