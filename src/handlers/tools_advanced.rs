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

#[cfg(test)]
mod part1_pure_helpers_tests {
    //! Cover the pure-compute helpers in tools_advanced_part1.rs that
    //! were 0%-covered on broad (259 missed lines). Handlers themselves are
    //! async and hit disk/analysis pipelines — skipped here.
    use super::*;

    // ── require_project_path_advanced (D101 guard) ──

    #[test]
    fn test_require_project_path_advanced_none_is_rejected() {
        let err = require_project_path_advanced(None).unwrap_err();
        assert!(err.contains("'project_path' is required"), "got: {err}");
        assert!(err.contains("R22-1"), "must tag R22-1/D101: {err}");
    }

    #[test]
    fn test_require_project_path_advanced_empty_string_is_rejected() {
        let err = require_project_path_advanced(Some(String::new())).unwrap_err();
        assert!(err.contains("non-empty string"), "got: {err}");
    }

    #[test]
    fn test_require_project_path_advanced_whitespace_is_rejected() {
        let err = require_project_path_advanced(Some("   \t\n  ".to_string())).unwrap_err();
        assert!(err.contains("non-empty string"), "got: {err}");
    }

    #[test]
    fn test_require_project_path_advanced_valid_returns_pathbuf() {
        let path = require_project_path_advanced(Some("/tmp/proj".to_string())).unwrap();
        assert_eq!(path, std::path::PathBuf::from("/tmp/proj"));
    }

    // ── require_non_empty_path ──

    #[test]
    fn test_require_non_empty_path_empty_rejected() {
        let err = require_non_empty_path("", "output").unwrap_err();
        assert!(err.contains("'output'"), "field name must appear: {err}");
        assert!(err.contains("non-empty string"), "got: {err}");
    }

    #[test]
    fn test_require_non_empty_path_whitespace_rejected() {
        let err = require_non_empty_path("   ", "project_path").unwrap_err();
        assert!(err.contains("'project_path'"), "got: {err}");
    }

    #[test]
    fn test_require_non_empty_path_valid_returns_pathbuf() {
        let path = require_non_empty_path("/tmp/x", "project_path").unwrap();
        assert_eq!(path, std::path::PathBuf::from("/tmp/x"));
    }

    // ── get_relative_path ──

    #[test]
    fn test_get_relative_path_strips_prefix() {
        let base = std::path::Path::new("/home/noah/project");
        let file = std::path::Path::new("/home/noah/project/src/main.rs");
        assert_eq!(get_relative_path(file, base), "src/main.rs");
    }

    #[test]
    fn test_get_relative_path_no_prefix_match_returns_original() {
        let base = std::path::Path::new("/home/noah/project");
        let file = std::path::Path::new("/tmp/other.rs");
        // strip_prefix returns Err → unwrap_or(path) → original.
        assert_eq!(get_relative_path(file, base), "/tmp/other.rs");
    }

    // ── calculate_cyclomatic_complexity + calculate_cognitive_complexity ──

    #[test]
    fn test_calculate_cyclomatic_complexity_trivial() {
        // No control flow → base complexity 1.
        assert_eq!(calculate_cyclomatic_complexity("fn f() {}"), 1);
    }

    #[test]
    fn test_calculate_cyclomatic_complexity_counts_control_flow() {
        let src = "if cond { for x in xs { while y { match z {} } } }";
        let c = calculate_cyclomatic_complexity(src);
        assert!(c >= 5, "if+for+while+match should count: got {c}");
    }

    #[test]
    fn test_calculate_cognitive_complexity_is_1_5x_cyclomatic() {
        assert_eq!(calculate_cognitive_complexity(10), 15);
        assert_eq!(calculate_cognitive_complexity(1), 1); // (1 * 1.5) as u32 == 1
        assert_eq!(calculate_cognitive_complexity(0), 0);
    }

    // ── calculate_duplicate_ratio ──

    #[test]
    fn test_calculate_duplicate_ratio_empty_returns_zero() {
        let lines: Vec<&str> = vec![];
        let r = calculate_duplicate_ratio(&lines);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn test_calculate_duplicate_ratio_all_unique_is_zero() {
        let lines = vec!["let a = 1;", "let b = 2;", "let c = 3;"];
        assert_eq!(calculate_duplicate_ratio(&lines), 0.0);
    }

    #[test]
    fn test_calculate_duplicate_ratio_duplicates_counted() {
        let lines = vec!["let x = 1;", "let x = 1;", "let y = 2;"];
        // line_counts: "let x = 1;" → 2, "let y = 2;" → 1
        // duplicates = (2-1) = 1; total lines = 3; ratio = 1/3
        let r = calculate_duplicate_ratio(&lines);
        assert!((r - (1.0f32 / 3.0f32)).abs() < 1e-5, "got {r}");
    }

    #[test]
    fn test_calculate_duplicate_ratio_skips_comments_and_blanks() {
        // // comment lines and blanks are ignored in the count but still
        // contribute to the denominator (lines.len()).
        let lines = vec!["// comment", "", "let x = 1;", "let x = 1;", "// another"];
        let r = calculate_duplicate_ratio(&lines);
        // duplicates = 1 (one extra "let x = 1;"); total lines = 5; ratio = 0.2
        assert!((r - 0.2).abs() < 1e-5, "got {r}");
    }

    // ── calculate_efferent_coupling ──

    #[test]
    fn test_calculate_efferent_coupling_counts_use_lines() {
        let src = "use foo::bar;\nuse x::y::z;\nfn main() {}\n// use commented";
        assert_eq!(calculate_efferent_coupling(src), 2.0);
    }

    #[test]
    fn test_calculate_efferent_coupling_zero_when_no_imports() {
        assert_eq!(calculate_efferent_coupling("fn main() {}"), 0.0);
    }

    // ── is_public_declaration ──

    #[test]
    fn test_is_public_declaration_recognizes_pub_items() {
        assert!(is_public_declaration("pub fn foo() {}"));
        assert!(is_public_declaration("    pub struct Bar;"));
        assert!(is_public_declaration("pub enum E { A }"));
    }

    #[test]
    fn test_is_public_declaration_rejects_non_pub() {
        assert!(!is_public_declaration("fn foo() {}"));
        assert!(!is_public_declaration("struct S;"));
        assert!(!is_public_declaration("// pub fn comment"));
    }

    // ── get_churn_score ──

    #[test]
    fn test_get_churn_score_found() {
        let mut m = std::collections::HashMap::new();
        m.insert("src/foo.rs".to_string(), 0.75_f32);
        assert!((get_churn_score("src/foo.rs", &m) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_get_churn_score_not_found_returns_default_floor() {
        // Fallback is 0.1 (not 0.0) — get(key).copied().unwrap_or(0.1).
        let m: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        assert!((get_churn_score("src/missing.rs", &m) - 0.1).abs() < 1e-6);
    }

    // ── calculate_afferent_coupling (constant in this implementation) ──

    #[test]
    fn test_calculate_afferent_coupling_is_deterministic() {
        // Both calls on the same content produce the same value.
        let a = calculate_afferent_coupling("fn x() {}");
        let b = calculate_afferent_coupling("fn x() {}");
        assert!((a - b).abs() < 1e-6);
    }
}

#[cfg(test)]
mod part3_pure_helpers_tests {
    //! Wave 36 PR4 — cover the pure-compute helpers in tools_advanced_part3.rs
    //! (deep_context arg parsing + makefile lint helpers). Async handlers and
    //! the analyzer wrappers are not tested here.
    use super::*;
    use crate::services::deep_context::{AnalysisType, CacheStrategy, DagType};
    use crate::services::makefile_linter::Severity;

    // ── default_project_path / default_top_files ────────────────────────────

    #[test]
    fn test_default_project_path_is_empty() {
        // Sentinel value: serde default that downstream rejects via require_*.
        assert_eq!(default_project_path(), "");
    }

    #[test]
    fn test_default_top_files_is_ten() {
        assert_eq!(default_top_files(), 10);
    }

    // ── get_default_analysis_types ──────────────────────────────────────────

    #[test]
    fn test_get_default_analysis_types_includes_ast_complexity_churn() {
        let types = get_default_analysis_types();
        assert_eq!(types.len(), 3);
        assert!(matches!(types[0], AnalysisType::Ast));
        assert!(matches!(types[1], AnalysisType::Complexity));
        assert!(matches!(types[2], AnalysisType::Churn));
    }

    // ── parse_analysis_type_string ──────────────────────────────────────────

    #[test]
    fn test_parse_analysis_type_string_ast() {
        assert!(matches!(
            parse_analysis_type_string("ast"),
            Some(AnalysisType::Ast)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_complexity() {
        assert!(matches!(
            parse_analysis_type_string("complexity"),
            Some(AnalysisType::Complexity)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_churn() {
        assert!(matches!(
            parse_analysis_type_string("churn"),
            Some(AnalysisType::Churn)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_dag() {
        assert!(matches!(
            parse_analysis_type_string("dag"),
            Some(AnalysisType::Dag)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_dead_code() {
        assert!(matches!(
            parse_analysis_type_string("dead_code"),
            Some(AnalysisType::DeadCode)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_satd() {
        assert!(matches!(
            parse_analysis_type_string("satd"),
            Some(AnalysisType::Satd)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_tdg() {
        assert!(matches!(
            parse_analysis_type_string("tdg"),
            Some(AnalysisType::TechnicalDebtGradient)
        ));
    }

    #[test]
    fn test_parse_analysis_type_string_unknown_returns_none() {
        assert!(parse_analysis_type_string("nonsense").is_none());
        assert!(parse_analysis_type_string("").is_none());
    }

    // ── parse_analysis_types ────────────────────────────────────────────────

    #[test]
    fn test_parse_analysis_types_none_returns_defaults() {
        let result = parse_analysis_types(None);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_analysis_types_some_filters_unknown() {
        // PIN: filter_map drops unknown strings silently — no error surfaced.
        let result = parse_analysis_types(Some(vec![
            "ast".to_string(),
            "bogus".to_string(),
            "satd".to_string(),
        ]));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_analysis_types_empty_vec_returns_empty() {
        // PIN: explicit empty vec ≠ None — returns empty, not defaults.
        let result = parse_analysis_types(Some(vec![]));
        assert!(result.is_empty());
    }

    // ── parse_deep_context_dag_type ─────────────────────────────────────────

    #[test]
    fn test_parse_dag_type_import_graph() {
        assert!(matches!(
            parse_deep_context_dag_type(Some("import-graph".to_string())),
            DagType::ImportGraph
        ));
    }

    #[test]
    fn test_parse_dag_type_inheritance() {
        assert!(matches!(
            parse_deep_context_dag_type(Some("inheritance".to_string())),
            DagType::Inheritance
        ));
    }

    #[test]
    fn test_parse_dag_type_full_dependency() {
        assert!(matches!(
            parse_deep_context_dag_type(Some("full-dependency".to_string())),
            DagType::FullDependency
        ));
    }

    #[test]
    fn test_parse_dag_type_call_graph_explicit() {
        assert!(matches!(
            parse_deep_context_dag_type(Some("call-graph".to_string())),
            DagType::CallGraph
        ));
    }

    #[test]
    fn test_parse_dag_type_none_defaults_to_call_graph() {
        assert!(matches!(
            parse_deep_context_dag_type(None),
            DagType::CallGraph
        ));
    }

    #[test]
    fn test_parse_dag_type_unknown_defaults_to_call_graph() {
        // PIN: silent fallback — unknown values do NOT error, they coerce to CallGraph.
        assert!(matches!(
            parse_deep_context_dag_type(Some("garbage".to_string())),
            DagType::CallGraph
        ));
    }

    // ── parse_cache_strategy ────────────────────────────────────────────────

    #[test]
    fn test_parse_cache_strategy_force_refresh() {
        assert!(matches!(
            parse_cache_strategy(Some("force-refresh".to_string())),
            CacheStrategy::ForceRefresh
        ));
    }

    #[test]
    fn test_parse_cache_strategy_offline() {
        assert!(matches!(
            parse_cache_strategy(Some("offline".to_string())),
            CacheStrategy::Offline
        ));
    }

    #[test]
    fn test_parse_cache_strategy_normal_explicit() {
        assert!(matches!(
            parse_cache_strategy(Some("normal".to_string())),
            CacheStrategy::Normal
        ));
    }

    #[test]
    fn test_parse_cache_strategy_none_defaults_to_normal() {
        assert!(matches!(parse_cache_strategy(None), CacheStrategy::Normal));
    }

    #[test]
    fn test_parse_cache_strategy_unknown_defaults_to_normal() {
        // PIN: silent fallback — unknown values do NOT error.
        assert!(matches!(
            parse_cache_strategy(Some("xyzzy".to_string())),
            CacheStrategy::Normal
        ));
    }

    // ── parse_deep_context_args ─────────────────────────────────────────────

    #[test]
    fn test_parse_deep_context_args_minimal() {
        let v = serde_json::json!({});
        let args = parse_deep_context_args(v).unwrap();
        assert!(args.project_path.is_none());
    }

    #[test]
    fn test_parse_deep_context_args_with_path() {
        let v = serde_json::json!({"project_path": "/tmp/x"});
        let args = parse_deep_context_args(v).unwrap();
        assert_eq!(args.project_path.as_deref(), Some("/tmp/x"));
    }

    #[test]
    fn test_parse_deep_context_args_invalid_returns_err() {
        // Wrong type for project_path → serde error.
        let v = serde_json::json!({"project_path": 42});
        let err = parse_deep_context_args(v).unwrap_err();
        assert!(err.contains("Invalid analyze_deep_context"));
    }

    // ── build_deep_context_config ───────────────────────────────────────────

    #[test]
    fn test_build_deep_context_config_uses_defaults_when_unset() {
        let args = AnalyzeDeepContextArgs {
            project_path: None,
            format: None,
            include_analyses: None,
            exclude_analyses: None,
            period_days: None,
            dag_type: None,
            max_depth: None,
            include_pattern: None,
            exclude_pattern: None,
            cache_strategy: None,
            parallel: None,
        };
        let cfg = build_deep_context_config(&args);
        assert_eq!(cfg.period_days, 30); // default
        assert_eq!(cfg.parallel, 4); // default
        assert!(matches!(cfg.dag_type, DagType::CallGraph));
        assert!(matches!(cfg.cache_strategy, CacheStrategy::Normal));
        assert_eq!(cfg.include_analyses.len(), 3); // default analyses
    }

    #[test]
    fn test_build_deep_context_config_threshold_constants_pinned() {
        // PIN: hardcoded ComplexityThresholds {10, 15} — only place these
        // particular numbers live for the deep_context entrypoint.
        let args = AnalyzeDeepContextArgs {
            project_path: None,
            format: None,
            include_analyses: None,
            exclude_analyses: None,
            period_days: None,
            dag_type: None,
            max_depth: None,
            include_pattern: None,
            exclude_pattern: None,
            cache_strategy: None,
            parallel: None,
        };
        let cfg = build_deep_context_config(&args);
        let t = cfg.complexity_thresholds.unwrap();
        assert_eq!(t.max_cyclomatic, 10);
        assert_eq!(t.max_cognitive, 15);
    }

    // ── format_deep_context_as_sarif ────────────────────────────────────────

    #[test]
    fn test_format_deep_context_as_sarif_emits_v2_1_0_skeleton() {
        // Function ignores its argument; safe to pass a shoddy mock context
        // by constructing via Default if available, but easier: use unsafe
        // transmute? No — pass a valid via the function signature.
        // Instead, since the body never reads the context, we can call it
        // with any &DeepContext. Build via the public constructor path is
        // expensive; the function is independent so we test its output shape
        // by parsing the JSON.
        // Minimal approach: assert the function name itself is a constant
        // string-builder by constructing a fake DeepContext only if cheap.
        // It is cheap:
        let dc = stub_deep_context();
        let sarif = format_deep_context_as_sarif(&dc);
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(parsed["runs"][0]["tool"]["driver"]["name"], "pmat");
        // results array is always empty in this stub formatter.
        assert!(parsed["runs"][0]["results"].as_array().unwrap().is_empty());
    }

    fn stub_deep_context() -> crate::services::deep_context::DeepContext {
        // Build a minimal DeepContext via Default or its public ctor.
        // The SARIF formatter doesn't read fields, but the type system requires &DeepContext.
        crate::services::deep_context::DeepContext::default()
    }

    // ── map_severity ────────────────────────────────────────────────────────

    #[test]
    fn test_map_severity_error() {
        assert_eq!(map_severity(&Severity::Error), "error");
    }

    #[test]
    fn test_map_severity_warning() {
        assert_eq!(map_severity(&Severity::Warning), "warning");
    }

    #[test]
    fn test_map_severity_performance() {
        assert_eq!(map_severity(&Severity::Performance), "performance");
    }

    #[test]
    fn test_map_severity_info() {
        assert_eq!(map_severity(&Severity::Info), "info");
    }

    // ── format_violation ────────────────────────────────────────────────────

    #[test]
    fn test_format_violation_emits_all_fields() {
        use crate::services::makefile_linter::ast::SourceSpan;
        use crate::services::makefile_linter::Violation;
        let v = Violation {
            rule: "MAKE001".to_string(),
            severity: Severity::Warning,
            span: SourceSpan {
                start: 0,
                end: 5,
                line: 7,
                column: 3,
            },
            message: "uses :=".to_string(),
            fix_hint: Some("change to =".to_string()),
        };
        let json = format_violation(&v);
        assert_eq!(json["rule"], "MAKE001");
        assert_eq!(json["severity"], "warning");
        assert_eq!(json["line"], 7);
        assert_eq!(json["column"], 3);
        assert_eq!(json["message"], "uses :=");
        assert_eq!(json["fix_hint"], "change to =");
    }

    #[test]
    fn test_format_violation_with_no_fix_hint() {
        use crate::services::makefile_linter::ast::SourceSpan;
        use crate::services::makefile_linter::Violation;
        let v = Violation {
            rule: "MAKE002".to_string(),
            severity: Severity::Error,
            span: SourceSpan {
                start: 0,
                end: 0,
                line: 1,
                column: 1,
            },
            message: "broken".to_string(),
            fix_hint: None,
        };
        let json = format_violation(&v);
        assert!(json["fix_hint"].is_null());
    }

    // ── count_violations_by_severity ────────────────────────────────────────

    #[test]
    fn test_count_violations_by_severity_pin_buggy_pattern_counts_all() {
        // PIN BUG: the function uses `matches!(&v.severity, _target_severity)`
        // where `_target_severity` is a *binding pattern*, not a value match.
        // So this counts ALL violations regardless of severity passed in.
        // Tests pin the current behavior so any future fix must update the
        // tests intentionally rather than silently regressing the metric.
        use crate::services::makefile_linter::ast::SourceSpan;
        use crate::services::makefile_linter::Violation;
        let span = SourceSpan {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        };
        let vs = vec![
            Violation {
                rule: "a".into(),
                severity: Severity::Error,
                span,
                message: "".into(),
                fix_hint: None,
            },
            Violation {
                rule: "b".into(),
                severity: Severity::Warning,
                span,
                message: "".into(),
                fix_hint: None,
            },
            Violation {
                rule: "c".into(),
                severity: Severity::Info,
                span,
                message: "".into(),
                fix_hint: None,
            },
        ];
        // Whatever severity we pass, count == vs.len() because of the binding bug.
        assert_eq!(count_violations_by_severity(&vs, Severity::Error), 3);
        assert_eq!(count_violations_by_severity(&vs, Severity::Warning), 3);
        assert_eq!(count_violations_by_severity(&vs, Severity::Info), 3);
        assert_eq!(count_violations_by_severity(&vs, Severity::Performance), 3);
    }

    #[test]
    fn test_count_violations_by_severity_empty_returns_zero() {
        assert_eq!(count_violations_by_severity(&[], Severity::Error), 0);
    }

    // ── parse_makefile_lint_args ────────────────────────────────────────────

    #[test]
    fn test_parse_makefile_lint_args_none_rejected() {
        // PIN: MakefileLintArgs lacks `Debug`, so `unwrap_err` won't compile —
        // use match-expression error extraction instead.
        let err = match parse_makefile_lint_args(None) {
            Err(e) => e,
            Ok(_) => panic!("expected Err"),
        };
        assert!(err.contains("Missing required arguments"));
    }

    #[test]
    fn test_parse_makefile_lint_args_minimal() {
        let v = serde_json::json!({"path": "/tmp/Makefile"});
        let args = parse_makefile_lint_args(Some(v)).unwrap();
        assert_eq!(args.path, "/tmp/Makefile");
        assert!(args.rules.is_empty());
        assert!(!args.fix);
    }

    #[test]
    fn test_parse_makefile_lint_args_full() {
        let v = serde_json::json!({
            "path": "/x",
            "rules": ["MAKE001", "MAKE002"],
            "fix": true,
            "gnu_version": "4.3",
        });
        let args = parse_makefile_lint_args(Some(v)).unwrap();
        assert_eq!(args.rules, vec!["MAKE001", "MAKE002"]);
        assert!(args.fix);
        assert_eq!(args.gnu_version, "4.3");
    }

    #[test]
    fn test_parse_makefile_lint_args_invalid_returns_err() {
        let v = serde_json::json!({"path": 42}); // path must be string
        let err = match parse_makefile_lint_args(Some(v)) {
            Err(e) => e,
            Ok(_) => panic!("expected Err"),
        };
        assert!(err.contains("Invalid analyze_makefile_lint"));
    }
}
