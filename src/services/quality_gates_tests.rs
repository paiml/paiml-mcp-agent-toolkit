// Quality gate tests - included from quality_gates.rs
// NO use imports or #! attributes - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::deep_context::{
        AnalysisResults, CacheStats, ComplexityMetricsForQA, ContextMetadata, DeadCodeAnalysis,
        DeadCodeSummary, DeepContextResult, DefectSummary, FileComplexityMetricsForQA,
        FunctionComplexityForQA, QualityScorecard,
    };
    use std::path::PathBuf;
    use std::time::Duration;

    // Helper function to create a test DeepContextResult
    fn create_test_deep_context_result() -> DeepContextResult {
        DeepContextResult {
            metadata: ContextMetadata {
                generated_at: chrono::Utc::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                project_root: PathBuf::from("."),
                cache_stats: CacheStats {
                    hit_rate: 0.8,
                    memory_efficiency: 0.9,
                    time_saved_ms: 1000,
                },
                analysis_duration: Duration::from_secs(1),
            },
            file_tree: vec![],
            analyses: AnalysisResults {
                ast_contexts: vec![],
                complexity_report: None,
                churn_analysis: None,
                dependency_graph: None,
                dead_code_results: None,
                satd_results: None,
                duplicate_code_results: None,
                provability_results: None,
                cross_language_refs: vec![],
                big_o_analysis: None,
            },
            quality_scorecard: QualityScorecard {
                overall_health: 0.8,
                complexity_score: 0.8,
                maintainability_index: 0.8,
                modularity_score: 0.8,
                test_coverage: Some(0.8),
                technical_debt_hours: 10.0,
            },
            template_provenance: None,
            defect_summary: DefectSummary {
                total_defects: 0,
                by_severity: FxHashMap::default(),
                by_type: FxHashMap::default(),
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
        }
    }

    #[test]
    fn test_qa_verification_dead_code() {
        let qa = QAVerification::new();

        // Test zero dead code with large codebase
        let mut result = create_test_deep_context_result();
        result.complexity_metrics = Some(ComplexityMetricsForQA {
            files: vec![FileComplexityMetricsForQA {
                path: PathBuf::from("src/main.rs"),
                functions: vec![],
                total_cyclomatic: 100,
                total_cognitive: 150,
                total_lines: 2000,
            }],
            summary: Default::default(),
        });

        result.dead_code_analysis = Some(DeadCodeAnalysis {
            summary: DeadCodeSummary {
                total_functions: 50,
                dead_functions: 0,
                total_lines: 2000,
                total_dead_lines: 0,
                dead_percentage: 0.0,
            },
            dead_functions: vec![],
            warnings: vec![],
        });

        let verification = qa.verify(&result);
        assert!(
            verification.get("dead_code_sanity").unwrap().is_ok(),
            "Pure Rust project with zero dead code should pass"
        );
    }

    #[test]
    fn test_qa_verification_complexity() {
        let qa = QAVerification::new();

        let mut result = create_test_deep_context_result();

        // Generate varied complexity distribution
        let functions: Vec<FunctionComplexityForQA> = (0..100)
            .map(|i| {
                FunctionComplexityForQA {
                    name: format!("func_{i}"),
                    cyclomatic: ((i % 20) + 1) as u32, // Varied complexity
                    cognitive: ((i % 15) + 1) as u32,
                    nesting_depth: ((i % 5) + 1) as u32,
                    start_line: (i * 10) as usize,
                    end_line: (i * 10 + 5) as usize,
                }
            })
            .collect();

        result.complexity_metrics = Some(ComplexityMetricsForQA {
            files: vec![FileComplexityMetricsForQA {
                path: PathBuf::from("src/lib.rs"),
                functions,
                total_cyclomatic: 1050,
                total_cognitive: 800,
                total_lines: 1500,
            }],
            summary: Default::default(),
        });

        let verification = qa.verify(&result);
        assert!(verification.get("complexity_distribution").unwrap().is_ok());
        assert!(verification.get("complexity_entropy").unwrap().is_ok());
    }

    #[test]
    fn test_qa_report_generation() {
        let qa = QAVerification::new();
        let result = create_test_deep_context_result();

        let report = qa.generate_verification_report(&result);

        assert!(!report.timestamp.is_empty());
        assert_eq!(report.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(report.dead_code.expected_range, [0.005, 0.15]);
    }
}

