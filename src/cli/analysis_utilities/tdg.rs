// TDG handlers - extracted for file health (CB-040)
// See handle_analyze_tdg() in tdg_analysis.rs for full documentation.

// Helper utilities: percentile, primary factor identification, filtering
include!("tdg_helpers.rs");

// Output formatting: table, json, markdown, sarif
include!("tdg_formatting.rs");

// Core analysis: single file, multiple files, project
include!("tdg_analysis.rs");

// Watch mode (feature-gated)
include!("tdg_watch.rs");

// Main entry point and orchestration
include!("tdg_handler.rs");

#[cfg(test)]
mod tdg_formatting_tests {
    //! PMAT-649: cover tdg_formatting.rs pure helpers.
    use super::*;
    use crate::cli::TdgOutputFormat;
    use crate::models::tdg::{TDGComponents, TDGHotspot, TDGScore, TDGSeverity, TDGSummary};

    fn summary_zero() -> TDGSummary {
        TDGSummary {
            total_files: 0,
            critical_files: 0,
            warning_files: 0,
            average_tdg: 0.0,
            p95_tdg: 0.0,
            p99_tdg: 0.0,
            estimated_debt_hours: 0.0,
            hotspots: Vec::new(),
        }
    }

    fn summary_with(total: usize, critical: usize, warning: usize, avg: f64) -> TDGSummary {
        TDGSummary {
            total_files: total,
            critical_files: critical,
            warning_files: warning,
            average_tdg: avg,
            p95_tdg: avg * 1.2,
            p99_tdg: avg * 1.5,
            estimated_debt_hours: 100.0,
            hotspots: Vec::new(),
        }
    }

    fn hotspot(path: &str, tdg: f64) -> TDGHotspot {
        TDGHotspot {
            path: path.to_string(),
            tdg_score: tdg,
            primary_factor: "Complexity".to_string(),
            estimated_hours: 5.0,
        }
    }

    // --- format_empty_results: all 4 format arms ---

    #[test]
    fn test_format_empty_results_table() {
        let out = format_empty_results(TdgOutputFormat::Table);
        assert!(out.contains("No files found"));
    }

    #[test]
    fn test_format_empty_results_json_is_valid_json() {
        let out = format_empty_results(TdgOutputFormat::Json);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["summary"]["total_files"], 0);
    }

    #[test]
    fn test_format_empty_results_markdown_has_header() {
        let out = format_empty_results(TdgOutputFormat::Markdown);
        assert!(out.starts_with("# Technical Debt Gradient Analysis"));
        assert!(out.contains("No files found"));
    }

    #[test]
    fn test_format_empty_results_sarif_is_valid_json() {
        let out = format_empty_results(TdgOutputFormat::Sarif);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["version"], "2.1.0");
    }

    // --- format_output_from_summary dispatches to each format ---

    #[test]
    fn test_format_output_from_summary_table_dispatch() {
        let s = summary_zero();
        let out = format_output_from_summary(&s, TdgOutputFormat::Table, false, false).unwrap();
        assert!(out.contains("# Technical Debt Gradient"));
    }

    #[test]
    fn test_format_output_from_summary_json_dispatch() {
        let s = summary_zero();
        let out = format_output_from_summary(&s, TdgOutputFormat::Json, false, false).unwrap();
        let _: serde_json::Value = serde_json::from_str(&out).unwrap();
    }

    #[test]
    fn test_format_output_from_summary_markdown_dispatch() {
        let s = summary_zero();
        let out = format_output_from_summary(&s, TdgOutputFormat::Markdown, false, false).unwrap();
        assert!(out.contains("# Technical Debt Gradient Analysis"));
    }

    #[test]
    fn test_format_output_from_summary_sarif_dispatch() {
        let s = summary_zero();
        let out = format_output_from_summary(&s, TdgOutputFormat::Sarif, false, false).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["runs"][0]["tool"]["driver"]["name"].as_str() == Some("pmat-tdg"));
    }

    // --- format_table_output ---

    #[test]
    fn test_format_table_output_no_files_skips_percentage_lines() {
        let s = summary_zero();
        let out = format_table_output(&s, false, false);
        assert!(out.contains("Total Files Analyzed"));
        // When total_files==0, percentage lines are omitted.
        assert!(!out.contains("Critical Files"));
    }

    #[test]
    fn test_format_table_output_with_files_emits_percentages() {
        let s = summary_with(10, 1, 2, 1.5);
        let out = format_table_output(&s, false, false);
        assert!(out.contains("Critical Files**: 1 (10.0%)"));
        assert!(out.contains("Warning Files**: 2 (20.0%)"));
        assert!(out.contains("Average TDG**: 1.50"));
    }

    #[test]
    fn test_format_table_output_hotspots_emitted_when_present() {
        let mut s = summary_with(5, 1, 0, 1.0);
        s.hotspots = vec![hotspot("src/a.rs", 2.5)];
        let out = format_table_output(&s, false, false);
        assert!(out.contains("## Top Hotspots"));
        assert!(out.contains("src/a.rs"));
        assert!(out.contains("2.50"));
    }

    #[test]
    fn test_format_table_output_components_only_when_include_and_verbose() {
        let s = summary_with(5, 0, 0, 1.0);
        let with = format_table_output(&s, true, true);
        assert!(with.contains("## Component Weights"));
        let without_verbose = format_table_output(&s, true, false);
        assert!(!without_verbose.contains("## Component Weights"));
        let without_include = format_table_output(&s, false, true);
        assert!(!without_include.contains("## Component Weights"));
    }

    // --- format_json_output ---

    #[test]
    fn test_format_json_output_without_components_has_null_components_field() {
        let s = summary_with(3, 0, 0, 1.0);
        let out = format_json_output(&s, false);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["summary"]["total_files"], 3);
        assert!(v["components"].is_null());
    }

    #[test]
    fn test_format_json_output_with_components_has_weights() {
        let s = summary_with(3, 0, 0, 1.0);
        let out = format_json_output(&s, true);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["components"]["complexity_weight"], 0.30);
        assert_eq!(v["components"]["churn_weight"], 0.35);
    }

    // --- format_markdown_output ---

    #[test]
    fn test_format_markdown_output_without_components() {
        let s = summary_with(5, 1, 2, 1.2);
        let out = format_markdown_output(&s, false);
        assert!(out.contains("# Technical Debt Gradient Analysis"));
        assert!(out.contains("## Summary"));
        assert!(out.contains("Total Files**: 5"));
        assert!(!out.contains("## TDG Components"));
    }

    #[test]
    fn test_format_markdown_output_with_components_includes_section() {
        let s = summary_with(5, 0, 0, 1.0);
        let out = format_markdown_output(&s, true);
        assert!(out.contains("## TDG Components"));
        assert!(out.contains("Complexity** (30%)"));
    }

    #[test]
    fn test_format_markdown_output_zero_total_skips_file_stats() {
        let s = summary_zero();
        let out = format_markdown_output(&s, false);
        assert!(!out.contains("Critical Files"));
        // But still has TDG stats and header.
        assert!(out.contains("Average TDG"));
    }

    #[test]
    fn test_format_markdown_output_hotspots_enumerated() {
        let mut s = summary_with(5, 1, 0, 1.0);
        s.hotspots = vec![hotspot("src/a.rs", 3.0), hotspot("src/b.rs", 2.0)];
        let out = format_markdown_output(&s, false);
        assert!(out.contains("## Hotspots"));
        assert!(out.contains("### 1. src/a.rs"));
        assert!(out.contains("### 2. src/b.rs"));
    }

    // --- format_sarif_output ---

    #[test]
    fn test_format_sarif_output_empty_hotspots_has_zero_results() {
        let s = summary_zero();
        let out = format_sarif_output(&s);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["runs"][0]["results"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_format_sarif_output_high_tdg_is_error_level() {
        let mut s = summary_with(2, 1, 0, 3.0);
        s.hotspots = vec![hotspot("src/critical.rs", 3.5)];
        let out = format_sarif_output(&s);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let level = v["runs"][0]["results"][0]["level"].as_str().unwrap();
        assert_eq!(level, "error");
    }

    #[test]
    fn test_format_sarif_output_low_tdg_is_warning_level() {
        let mut s = summary_with(2, 0, 1, 2.0);
        s.hotspots = vec![hotspot("src/warn.rs", 2.0)];
        let out = format_sarif_output(&s);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let level = v["runs"][0]["results"][0]["level"].as_str().unwrap();
        assert_eq!(level, "warning");
    }

    #[test]
    fn test_format_sarif_output_contains_rule_metadata() {
        let s = summary_zero();
        let out = format_sarif_output(&s);
        assert!(out.contains("TDG001"));
        assert!(out.contains("HighTechnicalDebtGradient"));
    }

    // --- format_tdg_single_file_output ---

    fn score_with(value: f64, severity: TDGSeverity) -> TDGScore {
        TDGScore {
            value,
            components: TDGComponents::default(),
            severity,
            percentile: 50.0,
            confidence: 0.8,
        }
    }

    #[test]
    fn test_format_tdg_single_file_output_critical_flags_critical_count() {
        let score = score_with(3.0, TDGSeverity::Critical);
        let path = std::path::Path::new("src/x.rs");
        let out =
            format_tdg_single_file_output(&score, path, TdgOutputFormat::Json, false, false)
                .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["summary"]["critical_files"], 1);
        assert_eq!(v["summary"]["warning_files"], 0);
    }

    #[test]
    fn test_format_tdg_single_file_output_warning_flags_warning_count() {
        let score = score_with(2.0, TDGSeverity::Warning);
        let path = std::path::Path::new("src/y.rs");
        let out =
            format_tdg_single_file_output(&score, path, TdgOutputFormat::Json, false, false)
                .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["summary"]["critical_files"], 0);
        assert_eq!(v["summary"]["warning_files"], 1);
    }

    #[test]
    fn test_format_tdg_single_file_output_normal_flags_neither() {
        let score = score_with(1.0, TDGSeverity::Normal);
        let path = std::path::Path::new("src/z.rs");
        let out =
            format_tdg_single_file_output(&score, path, TdgOutputFormat::Json, false, false)
                .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["summary"]["critical_files"], 0);
        assert_eq!(v["summary"]["warning_files"], 0);
    }
}
