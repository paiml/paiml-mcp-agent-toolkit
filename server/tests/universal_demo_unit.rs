//! Unit tests for Universal Demo components
//! 
//! These tests verify individual components without network access.

#[cfg(test)]
mod universal_demo_unit_tests {
    use pmat::services::context::AstItem;
    use pmat::services::quality_gates::{QAVerification, VerificationStatus};
    use pmat::services::deep_context::{
        ComplexityMetricsForQA, FileComplexityMetricsForQA,
        DeadCodeAnalysis, DeadCodeSummary,
    };
    use std::path::PathBuf;

    #[test]
    fn test_import_ast_item_creation() {
        // Test Python-style imports
        let import1 = AstItem::Import {
            module: "numpy".to_string(),
            items: vec![],
            alias: Some("np".to_string()),
            line: 1,
        };
        assert_eq!(import1.display_name(), "numpy");

        // Test from...import style
        let import2 = AstItem::Import {
            module: "typing".to_string(),
            items: vec!["List".to_string(), "Dict".to_string()],
            alias: None,
            line: 2,
        };
        assert_eq!(import2.display_name(), "typing");

        // Test JavaScript-style imports
        let import3 = AstItem::Import {
            module: "react".to_string(),
            items: vec!["useState".to_string(), "useEffect".to_string()],
            alias: None,
            line: 3,
        };
        assert_eq!(import3.display_name(), "react");
    }

    #[test]
    fn test_quality_gate_with_no_complexity_metrics() {
        use pmat::services::deep_context::{
            DeepContextResult, ContextMetadata, AnalysisResults,
            QualityScorecard, DefectSummary, CacheStats,
        };
        use std::time::Duration;
        use chrono::Utc;

        // Create a result with no complexity metrics but with files
        let mut result = DeepContextResult {
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: "test".to_string(),
                project_root: PathBuf::from("/test"),
                cache_stats: CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: Duration::from_secs(1),
            },
            file_tree: vec![
                "main.py".to_string(),
                "utils.py".to_string(),
                "test.js".to_string(),
            ],
            analyses: AnalysisResults {
                complexity_report: None,
                churn_analysis: None,
                dependency_graph: None,
                dead_code_results: None,
                satd_results: None,
                duplicate_code_results: None,
                provability_results: None,
                cross_language_refs: vec![],
                big_o_analysis: None,
                ast_contexts: vec![],
            },
            quality_scorecard: QualityScorecard {
                overall_health: 75.0,
                complexity_score: 80.0,
                maintainability_index: 70.0,
                modularity_score: 85.0,
                test_coverage: Some(0.0),
                technical_debt_hours: 0.0,
            },
            template_provenance: None,
            defect_summary: DefectSummary {
                total_defects: 0,
                by_severity: Default::default(),
                by_type: Default::default(),
                defect_density: 0.0,
            },
            hotspots: vec![],
            recommendations: vec![],
            qa_verification: None,
            complexity_metrics: None,
            dead_code_analysis: None,
            ast_summaries: None,
            churn_analysis: None,
            language_stats: None,
            build_info: None,
            project_overview: None,
        };

        // Test with no metrics at all
        let qa = QAVerification::new();
        let verification = qa.verify(&result);
        
        // Should handle gracefully
        assert!(verification.contains_key("dead_code_sanity"));
        
        // Add dead code analysis with line counts
        result.dead_code_analysis = Some(DeadCodeAnalysis {
            summary: DeadCodeSummary {
                total_functions: 10,
                dead_functions: 1,
                total_lines: 500,
                total_dead_lines: 50,
                dead_percentage: 10.0,
            },
            dead_functions: vec![],
            warnings: vec![],
        });
        
        let verification2 = qa.verify(&result);
        assert!(verification2.contains_key("dead_code_sanity"));
    }

    #[test]
    fn test_complexity_metrics_fallback() {
        // Test that complexity metrics can be created from basic file info
        let metrics = ComplexityMetricsForQA {
            files: vec![
                FileComplexityMetricsForQA {
                    path: PathBuf::from("test.py"),
                    functions: vec![],
                    total_cyclomatic: 0,
                    total_cognitive: 0,
                    total_lines: 100,
                },
                FileComplexityMetricsForQA {
                    path: PathBuf::from("main.js"),
                    functions: vec![],
                    total_cyclomatic: 0,
                    total_cognitive: 0,
                    total_lines: 200,
                },
            ],
            summary: Default::default(),
        };

        // Should be able to get total line count
        let total_lines: usize = metrics.files.iter()
            .map(|f| f.total_lines)
            .sum();
        assert_eq!(total_lines, 300);
    }

    #[test]
    fn test_verification_status_ordering() {
        // Ensure verification statuses have expected properties
        use VerificationStatus::*;
        
        // Test equality
        assert_eq!(Pass, Pass);
        assert_eq!(Fail, Fail);
        assert_eq!(Partial, Partial);
        
        // Test inequality
        assert_ne!(Pass, Fail);
        assert_ne!(Pass, Partial);
        assert_ne!(Fail, Partial);
    }

    #[test]
    fn test_python_import_variations() {
        let test_cases = vec![
            (
                AstItem::Import {
                    module: "os".to_string(),
                    items: vec![],
                    alias: None,
                    line: 1,
                },
                "os",
            ),
            (
                AstItem::Import {
                    module: "os.path".to_string(),
                    items: vec![],
                    alias: None,
                    line: 2,
                },
                "os.path",
            ),
            (
                AstItem::Import {
                    module: "numpy".to_string(),
                    items: vec![],
                    alias: Some("np".to_string()),
                    line: 3,
                },
                "numpy",
            ),
            (
                AstItem::Import {
                    module: "matplotlib.pyplot".to_string(),
                    items: vec![],
                    alias: Some("plt".to_string()),
                    line: 4,
                },
                "matplotlib.pyplot",
            ),
            (
                AstItem::Import {
                    module: "typing".to_string(),
                    items: vec![
                        "List".to_string(),
                        "Dict".to_string(),
                        "Optional".to_string(),
                    ],
                    alias: None,
                    line: 5,
                },
                "typing",
            ),
        ];

        for (import, expected_name) in test_cases {
            assert_eq!(
                import.display_name(),
                expected_name,
                "Import {:?} should display as {}",
                import,
                expected_name
            );
        }
    }

    #[test]
    fn test_javascript_import_variations() {
        let test_cases = vec![
            (
                AstItem::Import {
                    module: "react".to_string(),
                    items: vec![],
                    alias: None,
                    line: 1,
                },
                "react",
            ),
            (
                AstItem::Import {
                    module: "react".to_string(),
                    items: vec!["useState".to_string()],
                    alias: None,
                    line: 2,
                },
                "react",
            ),
            (
                AstItem::Import {
                    module: "./utils".to_string(),
                    items: vec![],
                    alias: None,
                    line: 3,
                },
                "./utils",
            ),
            (
                AstItem::Import {
                    module: "@mui/material".to_string(),
                    items: vec!["Button".to_string(), "TextField".to_string()],
                    alias: None,
                    line: 4,
                },
                "@mui/material",
            ),
        ];

        for (import, expected_name) in test_cases {
            assert_eq!(
                import.display_name(),
                expected_name,
                "Import {:?} should display as {}",
                import,
                expected_name
            );
        }
    }
}