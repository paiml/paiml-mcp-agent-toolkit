#![allow(unused)]
//! Advanced Analysis Tool Handlers
//! Split for file health compliance (CB-040)
#![cfg_attr(coverage_nightly, coverage(off))]

// R22-2 / D102: Glob-aware `project_path` resolution lives in the shared
// `crate::services::path_glob` module so both MCP dispatcher trees call the
// same implementation.
use crate::services::path_glob::{resolve_project_path_with_globs, ResolvedProjectPath};

include!("tools_advanced_part1.rs");
include!("tools_advanced_part2.rs");
include!("tools_advanced_part3.rs");
include!("tools_advanced_part4.rs");

#[cfg(test)]
mod part2_tests {
    //! PMAT-648: cover tools_advanced_part2.rs pure helpers.
    use super::*;
    use crate::models::dead_code::{
        ConfidenceLevel, DeadCodeAnalysisConfig, DeadCodeItem, DeadCodeRankingResult,
        DeadCodeSummary, DeadCodeType, FileDeadCodeMetrics,
    };
    use crate::models::tdg::{TDGHotspot, TDGSummary};
    use chrono::Utc;
    use serde_json::json;

    fn empty_dc_summary() -> DeadCodeSummary {
        DeadCodeSummary {
            total_files_analyzed: 0,
            files_with_dead_code: 0,
            total_dead_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        }
    }

    fn tdg_summary(total: usize, critical: usize, warning: usize) -> TDGSummary {
        TDGSummary {
            total_files: total,
            critical_files: critical,
            warning_files: warning,
            average_tdg: 1.5,
            p95_tdg: 2.5,
            p99_tdg: 3.0,
            estimated_debt_hours: 42.0,
            hotspots: Vec::new(),
        }
    }

    // --- get_confidence_level_text / format_confidence_emoji ---

    #[test]
    fn test_get_confidence_level_text_all_variants() {
        assert_eq!(get_confidence_level_text(ConfidenceLevel::High), "HIGH ");
        assert_eq!(
            get_confidence_level_text(ConfidenceLevel::Medium),
            "MEDIUM "
        );
        assert_eq!(get_confidence_level_text(ConfidenceLevel::Low), "LOW ");
    }

    #[test]
    fn test_format_confidence_emoji_all_variants() {
        assert_eq!(format_confidence_emoji(ConfidenceLevel::High), "🔴 High");
        assert_eq!(
            format_confidence_emoji(ConfidenceLevel::Medium),
            "🟡 Medium"
        );
        assert_eq!(format_confidence_emoji(ConfidenceLevel::Low), "🟢 Low");
    }

    // --- calculate_percentage / calculate_dead_files_percentage ---

    #[test]
    fn test_calculate_percentage_zero_total_returns_zero() {
        assert_eq!(calculate_percentage(5, 0), 0.0);
    }

    #[test]
    fn test_calculate_percentage_basic_ratio() {
        assert!((calculate_percentage(1, 4) - 25.0).abs() < 1e-10);
        assert!((calculate_percentage(7, 10) - 70.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_dead_files_percentage_zero_total() {
        let s = empty_dc_summary();
        assert_eq!(calculate_dead_files_percentage(&s), 0.0);
    }

    #[test]
    fn test_calculate_dead_files_percentage_ratio() {
        let mut s = empty_dc_summary();
        s.total_files_analyzed = 10;
        s.files_with_dead_code = 3;
        assert!((calculate_dead_files_percentage(&s) - 30.0).abs() < 1e-3);
    }

    // --- write_dead_code_header ---

    #[test]
    fn test_write_dead_code_header_includes_title_and_date() {
        let mut out = String::new();
        let ts = Utc::now();
        write_dead_code_header(&mut out, &ts);
        assert!(out.contains("# Dead Code Analysis Report"));
        assert!(out.contains("**Analysis Date:**"));
    }

    // --- write_dead_code_metrics ---

    #[test]
    fn test_write_dead_code_metrics_has_all_fields() {
        let mut s = empty_dc_summary();
        s.total_dead_lines = 100;
        s.dead_percentage = 12.5;
        s.dead_functions = 7;
        s.dead_classes = 2;
        s.dead_modules = 1;
        s.unreachable_blocks = 3;
        let mut out = String::new();
        write_dead_code_metrics(&mut out, &s);
        for key in [
            "Total dead lines",
            "100",
            "12.5%",
            "Dead functions:** 7",
            "Dead classes:** 2",
            "Dead modules:** 1",
            "Unreachable blocks:** 3",
        ] {
            assert!(out.contains(key), "missing {key} in: {out}");
        }
    }

    // --- write_dead_code_summary_section ---

    #[test]
    fn test_write_dead_code_summary_section_integration() {
        let mut s = empty_dc_summary();
        s.total_files_analyzed = 100;
        s.files_with_dead_code = 25;
        let mut out = String::new();
        write_dead_code_summary_section(&mut out, &s);
        assert!(out.contains("## Summary"));
        assert!(out.contains("Total files analyzed:** 100"));
        assert!(out.contains("Files with dead code:** 25 (25.0%)"));
    }

    // --- write_dead_code_top_files_section ---

    #[test]
    fn test_write_dead_code_top_files_section_empty_is_no_op() {
        let mut out = String::new();
        write_dead_code_top_files_section(&mut out, &[]);
        assert!(out.is_empty());
    }

    fn dc_metric(path: &str, lines: usize, conf: ConfidenceLevel) -> FileDeadCodeMetrics {
        FileDeadCodeMetrics {
            path: path.to_string(),
            dead_lines: lines,
            total_lines: 100,
            dead_percentage: lines as f32,
            dead_functions: 1,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
            dead_score: 50.0,
            confidence: conf,
            items: Vec::new(),
        }
    }

    #[test]
    fn test_write_dead_code_top_files_section_populated_has_table() {
        let files = vec![
            dc_metric("src/a.rs", 10, ConfidenceLevel::High),
            dc_metric("src/b.rs", 5, ConfidenceLevel::Medium),
        ];
        let mut out = String::new();
        write_dead_code_top_files_section(&mut out, &files);
        assert!(out.contains("## Top Files with Dead Code"));
        assert!(out.contains("| Rank |"));
        assert!(out.contains("src/a.rs"));
        assert!(out.contains("src/b.rs"));
        assert!(out.contains("🔴 High"));
        assert!(out.contains("🟡 Medium"));
    }

    // --- format_dead_code_as_sarif_mcp ---

    #[test]
    fn test_format_dead_code_as_sarif_mcp_basic_shape() {
        let file = FileDeadCodeMetrics {
            path: "src/x.rs".to_string(),
            dead_lines: 5,
            total_lines: 50,
            dead_percentage: 10.0,
            dead_functions: 1,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
            dead_score: 30.0,
            confidence: ConfidenceLevel::High,
            items: vec![DeadCodeItem {
                item_type: DeadCodeType::Function,
                name: "unused_fn".to_string(),
                line: 42,
                reason: "No call sites".to_string(),
            }],
        };
        let result = DeadCodeRankingResult {
            summary: empty_dc_summary(),
            ranked_files: vec![file],
            analysis_timestamp: Utc::now(),
            config: DeadCodeAnalysisConfig {
                include_unreachable: false,
                include_tests: false,
                min_dead_lines: 0,
            },
        };
        let sarif = format_dead_code_as_sarif_mcp(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        assert_eq!(
            v.get("$schema").and_then(|s| s.as_str()),
            Some("https://json.schemastore.org/sarif-2.1.0.json")
        );
        assert!(v.get("runs").is_some());
        assert!(sarif.contains("dead-code-function"));
        assert!(sarif.contains("unused_fn"));
    }

    #[test]
    fn test_format_dead_code_as_sarif_mcp_empty_has_zero_results() {
        let result = DeadCodeRankingResult {
            summary: empty_dc_summary(),
            ranked_files: Vec::new(),
            analysis_timestamp: Utc::now(),
            config: DeadCodeAnalysisConfig {
                include_unreachable: false,
                include_tests: false,
                min_dead_lines: 0,
            },
        };
        let sarif = format_dead_code_as_sarif_mcp(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        let results_len = v["runs"][0]["results"].as_array().unwrap().len();
        assert_eq!(results_len, 0);
    }

    // --- format_dead_code_as_markdown_mcp ---

    #[test]
    fn test_format_dead_code_as_markdown_mcp_integration() {
        let result = DeadCodeRankingResult {
            summary: empty_dc_summary(),
            ranked_files: Vec::new(),
            analysis_timestamp: Utc::now(),
            config: DeadCodeAnalysisConfig {
                include_unreachable: false,
                include_tests: false,
                min_dead_lines: 0,
            },
        };
        let out = format_dead_code_as_markdown_mcp(&result).unwrap();
        assert!(out.contains("# Dead Code Analysis Report"));
        assert!(out.contains("## Summary"));
    }

    // --- parse_tdg_args ---

    #[test]
    fn test_parse_tdg_args_valid() {
        let args = parse_tdg_args(json!({
            "project_path": "/tmp",
            "format": "json",
            "threshold": 1.5,
        }))
        .unwrap();
        assert_eq!(args.project_path.as_deref(), Some("/tmp"));
        assert_eq!(args.format.as_deref(), Some("json"));
        assert_eq!(args.threshold, Some(1.5));
    }

    #[test]
    fn test_parse_tdg_args_invalid_type_returns_err() {
        let result = parse_tdg_args(json!({"threshold": "not-a-number"}));
        assert!(result.is_err());
    }

    // --- extract_tdg_project_path ---

    #[test]
    fn test_extract_tdg_project_path_none_is_err() {
        let args = AnalyzeTdgArgs {
            project_path: None,
            format: None,
            threshold: None,
            include_components: None,
            max_results: None,
        };
        assert!(extract_tdg_project_path(&args).is_err());
    }

    #[test]
    fn test_extract_tdg_project_path_empty_is_err() {
        let args = AnalyzeTdgArgs {
            project_path: Some(String::new()),
            format: None,
            threshold: None,
            include_components: None,
            max_results: None,
        };
        assert!(extract_tdg_project_path(&args).is_err());
    }

    // --- format_tdg_summary / append_tdg_* helpers ---

    #[test]
    fn test_format_tdg_summary_integration_empty_hotspots() {
        let s = tdg_summary(10, 1, 2);
        let out = format_tdg_summary(&s);
        assert!(out.contains("# Technical Debt Gradient Analysis"));
        assert!(out.contains("## Summary"));
        assert!(out.contains("Total files:** 10"));
        assert!(out.contains("Critical files:** 1 (10.0%)"));
        assert!(out.contains("Warning files:** 2 (20.0%)"));
        assert!(out.contains("## Severity Distribution"));
        assert!(out.contains("🔴 Critical"));
        assert!(out.contains("🟡 Warning"));
        assert!(out.contains("🟢 Normal"));
        // No Top Hotspots section when empty.
        assert!(!out.contains("## Top Hotspots"));
    }

    #[test]
    fn test_append_tdg_hotspots_section_empty_no_op() {
        let mut out = String::new();
        let s = tdg_summary(5, 0, 0);
        append_tdg_hotspots_section(&mut out, &s);
        assert!(out.is_empty());
    }

    #[test]
    fn test_append_tdg_hotspots_section_populated_has_rows() {
        let mut s = tdg_summary(10, 1, 0);
        s.hotspots = vec![
            TDGHotspot {
                path: "src/a.rs".to_string(),
                tdg_score: 3.15,
                primary_factor: "Complexity".to_string(),
                estimated_hours: 4.5,
            },
            TDGHotspot {
                path: "src/b.rs".to_string(),
                tdg_score: 2.5,
                primary_factor: "Duplication".to_string(),
                estimated_hours: 2.0,
            },
        ];
        let mut out = String::new();
        append_tdg_hotspots_section(&mut out, &s);
        assert!(out.contains("## Top Hotspots"));
        assert!(out.contains("src/a.rs"));
        assert!(out.contains("3.15"));
        assert!(out.contains("Complexity"));
        assert!(out.contains("src/b.rs"));
        assert!(out.contains("Duplication"));
    }

    #[test]
    fn test_append_tdg_severity_section_computes_normal_via_saturating_sub() {
        let mut out = String::new();
        // critical + warning > total → saturating sub produces 0.
        let s = tdg_summary(3, 5, 10);
        append_tdg_severity_section(&mut out, &s);
        assert!(out.contains("🟢 Normal (<1.5): 0 files"));
    }

    // --- format_and_respond_tdg ---

    #[test]
    fn test_format_and_respond_tdg_json_format_emits_json_content() {
        let s = tdg_summary(5, 1, 1);
        let resp = format_and_respond_tdg(json!(42), s, Some("json".to_string()));
        assert!(resp.error.is_none());
        let result = resp.result.as_ref().unwrap();
        // Content text should be valid JSON (TDGSummary serialized).
        let content = result["content"][0]["text"].as_str().unwrap();
        let _: serde_json::Value = serde_json::from_str(content).unwrap();
        assert_eq!(result["format"], "json");
    }

    #[test]
    fn test_format_and_respond_tdg_default_format_is_summary() {
        let s = tdg_summary(5, 0, 1);
        let resp = format_and_respond_tdg(json!(99), s, None);
        let result = resp.result.as_ref().unwrap();
        assert_eq!(result["format"], "summary");
        let content = result["content"][0]["text"].as_str().unwrap();
        assert!(content.contains("# Technical Debt Gradient Analysis"));
    }
}
