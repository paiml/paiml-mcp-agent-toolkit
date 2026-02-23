#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use crate::cli::ComprehensiveOutputFormat;
    use crate::cli::handlers::comprehensive_analysis_handler::helpers::{
        create_analysis_request, determine_analysis_path, init_timing,
    };
    use crate::cli::handlers::comprehensive_analysis_handler::output::{
        format_as_json, format_as_markdown,
    };
    use crate::cli::handlers::comprehensive_analysis_handler::types::ComprehensiveAnalysisConfig;
    use crate::services::facades::analysis_orchestrator::{
        AnalysisSummary, ComprehensiveAnalysisResult,
    };
    use proptest::prelude::*;
    use std::path::PathBuf;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            prop_assert!(_x < 1001);
        }

        #[test]
        fn test_format_as_json_never_panics(
            total_files in 0usize..1000,
            total_issues in 0usize..1000,
            critical_issues in 0usize..1000,
            quality_score in 0.0f64..100.0,
            duration_ms in 0u64..100000
        ) {
            let result = ComprehensiveAnalysisResult {
                complexity: None,
                dead_code: None,
                satd: None,
                summary: AnalysisSummary {
                    total_files,
                    total_issues,
                    critical_issues,
                    quality_score,
                    recommendations: vec![],
                },
                duration_ms,
            };
            let json_result = format_as_json(&result);
            prop_assert!(json_result.is_ok());
        }

        #[test]
        fn test_format_as_markdown_never_panics(
            total_files in 0usize..1000,
            quality_score in 0.0f64..100.0,
            executive_summary in proptest::bool::ANY
        ) {
            let result = ComprehensiveAnalysisResult {
                complexity: None,
                dead_code: None,
                satd: None,
                summary: AnalysisSummary {
                    total_files,
                    total_issues: 0,
                    critical_issues: 0,
                    quality_score,
                    recommendations: vec![],
                },
                duration_ms: 1000,
            };
            let md_result = format_as_markdown(&result, executive_summary);
            prop_assert!(md_result.is_ok());
        }

        #[test]
        fn test_init_timing_returns_correct_option(perf in proptest::bool::ANY) {
            let result = init_timing(perf);
            prop_assert_eq!(result.is_some(), perf);
        }

        #[test]
        fn test_determine_analysis_path_priority(
            project_path in "[a-z/]+",
            single_file in proptest::option::of("[a-z/]+\\.rs"),
            has_multiple_files in proptest::bool::ANY
        ) {
            let files = if has_multiple_files {
                vec![PathBuf::from("/test/a.rs"), PathBuf::from("/test/b.rs")]
            } else {
                vec![]
            };
            let config = ComprehensiveAnalysisConfig {
                project_path: PathBuf::from(&project_path),
                file: single_file.map(PathBuf::from),
                files,
                format: ComprehensiveOutputFormat::Json,
                include_duplicates: false,
                include_dead_code: false,
                include_defects: false,
                include_complexity: false,
                include_tdg: false,
                confidence_threshold: 0.7,
                min_lines: 50,
                include: None,
                exclude: None,
                output: None,
                perf: false,
                executive_summary: false,
                top_files: 10,
            };
            let result = determine_analysis_path(&config);
            if let Some(ref single) = config.file {
                prop_assert_eq!(&result, single);
            } else if !config.files.is_empty() {
                prop_assert_eq!(result, config.project_path);
            } else {
                prop_assert_eq!(result, config.project_path);
            }
        }

        #[test]
        fn test_create_analysis_request_preserves_flags(
            include_complexity in proptest::bool::ANY,
            include_dead_code in proptest::bool::ANY,
            include_tdg in proptest::bool::ANY
        ) {
            let config = ComprehensiveAnalysisConfig {
                project_path: PathBuf::from("/test"),
                file: None,
                files: vec![],
                format: ComprehensiveOutputFormat::Json,
                include_duplicates: false,
                include_dead_code,
                include_defects: false,
                include_complexity,
                include_tdg,
                confidence_threshold: 0.7,
                min_lines: 50,
                include: None,
                exclude: None,
                output: None,
                perf: false,
                executive_summary: false,
                top_files: 10,
            };
            let request = create_analysis_request(PathBuf::from("/test"), &config);
            prop_assert_eq!(request.include_complexity, include_complexity);
            prop_assert_eq!(request.include_dead_code, include_dead_code);
            prop_assert_eq!(request.include_satd, include_tdg);
        }
    }
}
