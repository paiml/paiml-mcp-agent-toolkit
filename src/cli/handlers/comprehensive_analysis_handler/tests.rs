#![cfg_attr(coverage_nightly, coverage(off))]

/// Shared test helpers used by test submodules
#[cfg(test)]
pub(super) mod helpers {
    use crate::cli::handlers::comprehensive_analysis_handler::types::ComprehensiveAnalysisConfig;
    use crate::cli::ComprehensiveOutputFormat;
    use crate::services::facades::analysis_orchestrator::{
        AnalysisSummary, ComprehensiveAnalysisResult,
    };
    use crate::services::facades::complexity_facade::{
        ComplexityAnalysisResult, ComplexityViolation,
    };
    use crate::services::facades::dead_code_facade::{
        DeadCodeAnalysisResult, DeadCodeItem, DeadCodeType,
    };
    use crate::services::facades::satd_facade::{SatdAnalysisResult, SatdSeverity, SatdViolation};
    use std::path::PathBuf;

    pub(crate) fn create_basic_result() -> ComprehensiveAnalysisResult {
        ComprehensiveAnalysisResult {
            complexity: None,
            dead_code: None,
            satd: None,
            summary: AnalysisSummary {
                total_files: 10,
                total_issues: 5,
                critical_issues: 2,
                quality_score: Some(85.0),
                recommendations: vec!["Test recommendation".to_string()],
            },
            duration_ms: 1000,
        }
    }

    pub(crate) fn create_full_result() -> ComprehensiveAnalysisResult {
        ComprehensiveAnalysisResult {
            complexity: Some(ComplexityAnalysisResult {
                total_files: 5,
                violations: vec![
                    ComplexityViolation {
                        file_path: "src/main.rs".to_string(),
                        function_name: "complex_function".to_string(),
                        line_number: 42,
                        complexity: 35,
                        complexity_type: "cyclomatic".to_string(),
                    },
                    ComplexityViolation {
                        file_path: "src/lib.rs".to_string(),
                        function_name: "another_function".to_string(),
                        line_number: 100,
                        complexity: 22,
                        complexity_type: "cyclomatic".to_string(),
                    },
                ],
                average_complexity: 15.5,
                max_complexity: 35,
                summary: "Found 2 complexity violations".to_string(),
            }),
            dead_code: Some(DeadCodeAnalysisResult {
                total_files: 3,
                dead_items: vec![DeadCodeItem {
                    file_path: "src/unused.rs".to_string(),
                    item_name: "unused_function".to_string(),
                    item_type: DeadCodeType::Function,
                    line_number: 10,
                    reason: "Never called".to_string(),
                }],
                dead_percentage: 5.0,
                summary: "Found 1 dead code item".to_string(),
            }),
            satd: Some(SatdAnalysisResult {
                skipped: Default::default(),
                total_files: 2,
                violations: vec![SatdViolation {
                    file_path: "src/todo.rs".to_string(),
                    line_number: 25,
                    violation_type: "TODO".to_string(),
                    message: "Fix this later".to_string(),
                    severity: SatdSeverity::Medium,
                }],
                summary: "Found 1 SATD violation".to_string(),
            }),
            summary: AnalysisSummary {
                total_files: 10,
                total_issues: 4,
                critical_issues: 1,
                quality_score: Some(75.0),
                recommendations: vec![
                    "Refactor complex functions".to_string(),
                    "Remove dead code".to_string(),
                ],
            },
            duration_ms: 2500,
        }
    }

    pub(crate) fn create_default_config() -> ComprehensiveAnalysisConfig {
        ComprehensiveAnalysisConfig {
            project_path: PathBuf::from("/test/project"),
            file: None,
            files: Vec::new(),
            format: ComprehensiveOutputFormat::Json,
            include_duplicates: false,
            include_dead_code: true,
            include_defects: false,
            include_complexity: true,
            include_tdg: false,
            confidence_threshold: 0.7,
            min_lines: 50,
            include: None,
            exclude: None,
            output: None,
            perf: false,
            executive_summary: true,
            top_files: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use crate::cli::handlers::comprehensive_analysis_handler::helpers::{
        create_additional_config, create_analysis_request, determine_analysis_path, init_timing,
        print_performance_breakdown,
    };
    use crate::cli::handlers::comprehensive_analysis_handler::output::{
        format_as_json, format_as_markdown, format_as_sarif, format_executive_summary,
        format_result,
    };
    use crate::cli::handlers::comprehensive_analysis_handler::types::ComprehensiveAnalysisConfig;
    use crate::cli::ComprehensiveOutputFormat;
    use crate::services::facades::analysis_orchestrator::{
        AnalysisSummary, ComprehensiveAnalysisResult,
    };
    use crate::services::facades::complexity_facade::{
        ComplexityAnalysisResult, ComplexityViolation,
    };
    use std::path::PathBuf;

    fn strip_ansi(s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(s, "").to_string()
    }

    #[test]
    fn test_format_as_json() {
        let result = create_basic_result();
        let json = format_as_json(&result).unwrap();
        assert!(json.contains("\"total_files\": 10"));
        assert!(json.contains("\"quality_score\": 85.0"));
        assert!(json.contains("\"duration_ms\": 1000"));
    }

    #[test]
    fn test_format_as_json_with_full_result() {
        let result = create_full_result();
        let json = format_as_json(&result).unwrap();
        assert!(json.contains("\"function_name\": \"complex_function\""));
        assert!(json.contains("\"complexity\": 35"));
        assert!(json.contains("\"item_name\": \"unused_function\""));
        assert!(json.contains("\"violation_type\": \"TODO\""));
    }

    #[test]
    fn test_format_as_markdown_with_executive_summary() {
        let result = create_basic_result();
        let markdown = format_as_markdown(&result, true, 10).unwrap();
        assert!(markdown.contains("# Comprehensive Code Analysis Report"));
        assert!(markdown.contains("## Executive Summary"));
        assert!(markdown.contains("**Quality Score**: 85.0%"));
        assert!(markdown.contains("**Total Files**: 10"));
        assert!(markdown.contains("Test recommendation"));
    }

    #[test]
    fn test_format_as_markdown_without_executive_summary() {
        let result = create_basic_result();
        let markdown = format_as_markdown(&result, false, 10).unwrap();
        assert!(markdown.contains("# Comprehensive Code Analysis Report"));
        assert!(!markdown.contains("## Executive Summary"));
    }

    #[test]
    fn test_format_as_markdown_with_complexity() {
        let result = create_full_result();
        let markdown = format_as_markdown(&result, false, 10).unwrap();
        assert!(markdown.contains("## Complexity Analysis"));
        assert!(markdown.contains("**Files Analyzed**: 5"));
        assert!(markdown.contains("**Max Complexity**: 35"));
        assert!(markdown.contains("complex_function"));
    }

    #[test]
    fn test_format_as_markdown_with_dead_code() {
        let result = create_full_result();
        let markdown = format_as_markdown(&result, false, 10).unwrap();
        assert!(markdown.contains("## Dead Code Analysis"));
        assert!(markdown.contains("**Dead Items**: 1"));
        assert!(markdown.contains("unused_function"));
    }

    #[test]
    fn test_format_as_markdown_with_satd() {
        let result = create_full_result();
        let markdown = format_as_markdown(&result, false, 10).unwrap();
        assert!(markdown.contains("## Technical Debt (SATD) Analysis"));
        assert!(markdown.contains("**Violations**: 1"));
        assert!(markdown.contains("TODO"));
    }

    #[test]
    fn test_format_as_sarif() {
        let result = create_full_result();
        let sarif = format_as_sarif(&result).unwrap();
        assert!(sarif.contains("\"$schema\": \"https://json.schemastore.org/sarif-2.1.0.json\""));
        assert!(sarif.contains("\"version\": \"2.1.0\""));
        assert!(sarif.contains("\"name\": \"pmat-comprehensive\""));
        assert!(sarif.contains("\"ruleId\": \"high-complexity\""));
        assert!(sarif.contains("\"ruleId\": \"technical-debt\""));
    }

    #[test]
    fn test_format_as_sarif_complexity_levels() {
        let result = ComprehensiveAnalysisResult {
            complexity: Some(ComplexityAnalysisResult {
                total_files: 2,
                violations: vec![
                    ComplexityViolation {
                        file_path: "high.rs".to_string(),
                        function_name: "very_complex".to_string(),
                        line_number: 1,
                        complexity: 35,
                        complexity_type: "cyclomatic".to_string(),
                    },
                    ComplexityViolation {
                        file_path: "medium.rs".to_string(),
                        function_name: "moderately_complex".to_string(),
                        line_number: 1,
                        complexity: 25,
                        complexity_type: "cyclomatic".to_string(),
                    },
                    ComplexityViolation {
                        file_path: "low.rs".to_string(),
                        function_name: "simple".to_string(),
                        line_number: 1,
                        complexity: 15,
                        complexity_type: "cyclomatic".to_string(),
                    },
                ],
                average_complexity: 25.0,
                max_complexity: 35,
                summary: "Test".to_string(),
            }),
            dead_code: None,
            satd: None,
            summary: AnalysisSummary {
                total_files: 2,
                total_issues: 2,
                critical_issues: 1,
                quality_score: Some(70.0),
                recommendations: vec![],
            },
            duration_ms: 500,
        };
        let sarif = format_as_sarif(&result).unwrap();
        assert!(sarif.contains("\"level\": \"error\""));
        assert!(sarif.contains("\"level\": \"warning\""));
    }

    #[test]
    fn test_format_as_sarif_empty_results() {
        let result = create_basic_result();
        let sarif = format_as_sarif(&result).unwrap();
        assert!(sarif.contains("\"results\": []"));
    }

    #[test]
    fn test_init_timing_with_perf() {
        let start = init_timing(true);
        assert!(start.is_some());
    }

    #[test]
    fn test_init_timing_without_perf() {
        let start = init_timing(false);
        assert!(start.is_none());
    }

    #[test]
    fn test_determine_analysis_path_with_single_file() {
        let config = ComprehensiveAnalysisConfig {
            file: Some(PathBuf::from("/test/file.rs")),
            ..create_default_config()
        };
        let path = determine_analysis_path(&config);
        assert_eq!(path, PathBuf::from("/test/file.rs"));
    }

    #[test]
    fn test_determine_analysis_path_with_multiple_files() {
        let config = ComprehensiveAnalysisConfig {
            files: vec![
                PathBuf::from("/test/file1.rs"),
                PathBuf::from("/test/file2.rs"),
            ],
            ..create_default_config()
        };
        let path = determine_analysis_path(&config);
        assert_eq!(path, config.project_path);
    }

    #[test]
    fn test_determine_analysis_path_project_only() {
        let config = create_default_config();
        let path = determine_analysis_path(&config);
        assert_eq!(path, PathBuf::from("/test/project"));
    }

    #[test]
    fn test_create_analysis_request() {
        let config = create_default_config();
        let path = PathBuf::from("/test/project");
        let request = create_analysis_request(path.clone(), &config);
        assert_eq!(request.path, path);
        assert!(request.include_complexity);
        assert!(!request.include_satd);
        assert!(!request.include_tests);
        assert!(request.parallel);
        assert!(request.language.is_none());
    }

    #[test]
    fn test_create_analysis_request_with_tdg() {
        let config = ComprehensiveAnalysisConfig {
            include_tdg: true,
            ..create_default_config()
        };
        let path = PathBuf::from("/test/project");
        let request = create_analysis_request(path, &config);
        assert!(request.include_satd);
    }

    #[test]
    fn test_create_additional_config() {
        let config = ComprehensiveAnalysisConfig {
            include_duplicates: true,
            include_defects: true,
            confidence_threshold: 0.8,
            min_lines: 100,
            include: Some("src/".to_string()),
            exclude: Some("test/".to_string()),
            top_files: 20,
            ..create_default_config()
        };
        let additional = create_additional_config(&config);
        assert_eq!(additional.project_path, config.project_path.as_path());
        assert!(additional.include_duplicates);
        assert!(additional.include_defects);
        assert!((additional.confidence_threshold - 0.8).abs() < f32::EPSILON);
        assert_eq!(additional.min_lines, 100);
        assert_eq!(additional.include.as_ref().unwrap(), "src/");
        assert_eq!(additional.exclude.as_ref().unwrap(), "test/");
        assert_eq!(additional.top_files, 20);
    }

    #[test]
    fn test_format_result_json() {
        let result = create_basic_result();
        let formatted = format_result(result, ComprehensiveOutputFormat::Json, false, 10).unwrap();
        assert!(formatted.contains("\"total_files\""));
        assert!(formatted.contains("\"quality_score\""));
    }

    #[test]
    fn test_format_result_markdown() {
        let result = create_basic_result();
        let formatted =
            format_result(result, ComprehensiveOutputFormat::Markdown, true, 10).unwrap();
        assert!(formatted.contains("# Comprehensive Code Analysis Report"));
        assert!(formatted.contains("## Executive Summary"));
    }

    #[test]
    fn test_format_result_sarif() {
        let result = create_basic_result();
        let formatted = format_result(result, ComprehensiveOutputFormat::Sarif, false, 10).unwrap();
        assert!(formatted.contains("\"$schema\""));
        assert!(formatted.contains("sarif-2.1.0.json"));
    }

    #[test]
    fn test_format_result_summary() {
        let result = create_basic_result();
        let formatted = strip_ansi(
            &format_result(result, ComprehensiveOutputFormat::Summary, false, 10).unwrap(),
        );
        assert!(formatted.contains("Executive Summary"));
    }

    #[test]
    fn test_format_result_detailed() {
        let result = create_basic_result();
        let formatted =
            format_result(result, ComprehensiveOutputFormat::Detailed, true, 10).unwrap();
        assert!(!formatted.contains("## Executive Summary"));
    }

    #[test]
    fn test_format_executive_summary() {
        let summary = AnalysisSummary {
            total_files: 50,
            total_issues: 10,
            critical_issues: 3,
            quality_score: Some(90.5),
            recommendations: vec![
                "First recommendation".to_string(),
                "Second recommendation".to_string(),
            ],
        };
        let mut output = String::new();
        format_executive_summary(&mut output, &summary).unwrap();
        assert!(output.contains("## Executive Summary"));
        assert!(output.contains("50 total files analyzed"));
        assert!(output.contains("**Quality Score**: 90.5%"));
        assert!(output.contains("**Total Issues**: 10"));
        assert!(output.contains("**Critical Issues**: 3"));
        assert!(output.contains("### Key Recommendations"));
        assert!(output.contains("First recommendation"));
        assert!(output.contains("Second recommendation"));
    }

    #[test]
    fn test_format_executive_summary_no_recommendations() {
        let summary = AnalysisSummary {
            total_files: 10,
            total_issues: 0,
            critical_issues: 0,
            quality_score: Some(100.0),
            recommendations: vec![],
        };
        let mut output = String::new();
        format_executive_summary(&mut output, &summary).unwrap();
        assert!(!output.contains("### Key Recommendations"));
    }

    #[test]
    fn test_print_performance_breakdown_captures_output() {
        let result = create_full_result();
        print_performance_breakdown(&result, 1500);
    }

    #[test]
    fn test_print_performance_breakdown_with_zero_files() {
        let result = ComprehensiveAnalysisResult {
            complexity: None,
            dead_code: None,
            satd: None,
            summary: AnalysisSummary {
                total_files: 0,
                total_issues: 0,
                critical_issues: 0,
                quality_score: Some(100.0),
                recommendations: vec![],
            },
            duration_ms: 100,
        };
        print_performance_breakdown(&result, 100);
    }

    #[test]
    fn test_comprehensive_output_format_variants() {
        let json = format_result(
            create_basic_result(),
            ComprehensiveOutputFormat::Json,
            false,
            10,
        );
        assert!(json.is_ok());

        let md = format_result(
            create_basic_result(),
            ComprehensiveOutputFormat::Markdown,
            true,
            10,
        );
        assert!(md.is_ok());

        let sarif = format_result(
            create_basic_result(),
            ComprehensiveOutputFormat::Sarif,
            false,
            10,
        );
        assert!(sarif.is_ok());

        let summary = format_result(
            create_basic_result(),
            ComprehensiveOutputFormat::Summary,
            false,
            10,
        );
        assert!(summary.is_ok());

        let detailed = format_result(
            create_basic_result(),
            ComprehensiveOutputFormat::Detailed,
            true,
            10,
        );
        assert!(detailed.is_ok());
    }

    #[test]
    fn test_comprehensive_analysis_config_clone() {
        let config = create_default_config();
        let cloned = config.clone();
        assert_eq!(config.project_path, cloned.project_path);
        assert_eq!(config.format, cloned.format);
        assert_eq!(config.include_duplicates, cloned.include_duplicates);
    }

    #[test]
    fn test_comprehensive_analysis_config_debug() {
        let config = create_default_config();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ComprehensiveAnalysisConfig"));
        assert!(debug_str.contains("project_path"));
    }
}
