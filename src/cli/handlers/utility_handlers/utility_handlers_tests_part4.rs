//! Utility handlers tests - Part 4: Comprehensive coverage tests
//! Extracted for file health compliance (CB-040)

#[allow(unused_imports)]
use super::*;

/// NOTE: Temporarily disabled due to struct definition mismatches
#[cfg(all(test, feature = "broken-tests"))]
mod comprehensive_coverage_tests {
    use super::*;
    use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
    use crate::models::dead_code::{
        ConfidenceLevel as DeadCodeConfidence, DeadCodeAnalysisConfig, DeadCodeItem,
        DeadCodeRankingResult, DeadCodeSummary, DeadCodeType, FileDeadCodeMetrics,
    };
    use crate::services::complexity::{
        ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
    };
    use crate::services::context::{AstItem, FileContext, ProjectContext, ProjectSummary};
    use crate::services::deep_context::{
        AnalysisResults, DeepContext, DefectAnnotations, EnhancedFileContext, Impact,
        PrioritizedRecommendation, Priority, QualityScorecard,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::TempDir;

    // Helper Functions for Creating Test Data

    fn create_test_file_context(path: &str, language: &str) -> FileContext {
        FileContext {
            path: path.to_string(),
            language: language.to_string(),
            items: vec![
                AstItem::Function {
                    name: "test_function".to_string(),
                    visibility: "pub".to_string(),
                    is_async: false,
                    line: 10,
                },
                AstItem::Struct {
                    name: "TestStruct".to_string(),
                    visibility: "pub".to_string(),
                    fields_count: 3,
                    derives: vec!["Debug".to_string(), "Clone".to_string()],
                    line: 20,
                },
                AstItem::Enum {
                    name: "TestEnum".to_string(),
                    visibility: "pub".to_string(),
                    variants_count: 4,
                    line: 30,
                },
                AstItem::Trait {
                    name: "TestTrait".to_string(),
                    visibility: "pub".to_string(),
                    line: 40,
                },
                AstItem::Impl {
                    type_name: "TestStruct".to_string(),
                    trait_name: Some("TestTrait".to_string()),
                    line: 50,
                },
                AstItem::Impl {
                    type_name: "TestStruct".to_string(),
                    trait_name: None,
                    line: 60,
                },
            ],
            complexity_metrics: None,
        }
    }

    fn create_test_file_context_with_complexity(path: &str) -> FileContext {
        let mut ctx = create_test_file_context(path, "rust");
        ctx.complexity_metrics = Some(FileComplexityMetrics {
            path: path.to_string(),
            functions: vec![FunctionComplexity {
                name: "test_function".to_string(),
                start_line: 10,
                end_line: 25,
                metrics: ComplexityMetrics {
                    cyclomatic: 5,
                    cognitive: 8,
                    nesting_depth: 2,
                    halstead_volume: Some(150.0),
                    halstead_difficulty: Some(3.5),
                    parameter_count: Some(2),
                },
            }],
            total_cyclomatic: 5,
            total_cognitive: 8,
            total_lines: 100,
        });
        ctx
    }

    fn create_test_analysis_results() -> AnalysisResults {
        AnalysisResults {
            ast_contexts: vec![EnhancedFileContext {
                base: create_test_file_context("src/lib.rs", "rust"),
                complexity_metrics: None,
                churn_metrics: None,
                defects: DefectAnnotations {
                    dead_code: None,
                    technical_debt: vec![],
                    complexity_violations: vec![],
                    tdg_score: None,
                },
                symbol_id: "lib_rs".to_string(),
            }],
            complexity_report: None,
            churn_analysis: None,
            dependency_graph: None,
            dead_code_results: None,
            duplicate_code_results: None,
            satd_results: None,
            provability_results: None,
            cross_language_refs: vec![],
            big_o_analysis: None,
        }
    }

    fn create_test_analysis_results_with_churn() -> AnalysisResults {
        let mut results = create_test_analysis_results();
        results.churn_analysis = Some(CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("/test/repo"),
            files: vec![FileChurnMetrics {
                path: PathBuf::from("src/lib.rs"),
                relative_path: "src/lib.rs".to_string(),
                commit_count: 15,
                unique_authors: vec!["author1".to_string()],
                additions: 200,
                deletions: 50,
                churn_score: 0.75,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            }],
            summary: ChurnSummary {
                total_commits: 15,
                total_files_changed: 1,
                hotspot_files: vec![PathBuf::from("src/lib.rs")],
                stable_files: vec![],
                author_contributions: HashMap::new(),
                mean_churn_score: 0.75,
                variance_churn_score: 0.1,
                stddev_churn_score: 0.32,
            },
        });
        results
    }

    fn create_test_analysis_results_with_dead_code() -> AnalysisResults {
        let mut results = create_test_analysis_results();
        results.dead_code_results = Some(DeadCodeRankingResult {
            summary: DeadCodeSummary {
                total_files_analyzed: 1,
                files_with_dead_code: 1,
                total_dead_lines: 10,
                dead_percentage: 5.0,
                dead_functions: 1,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
            },
            ranked_files: vec![FileDeadCodeMetrics {
                path: "src/lib.rs".to_string(),
                dead_lines: 10,
                total_lines: 200,
                dead_percentage: 5.0,
                dead_functions: 1,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 0.25,
                confidence: DeadCodeConfidence::High,
                items: vec![DeadCodeItem {
                    item_type: DeadCodeType::Function,
                    name: "unused_function".to_string(),
                    line: 150,
                    reason: "No callers found".to_string(),
                }],
            }],
            analysis_timestamp: Utc::now(),
            config: DeadCodeAnalysisConfig {
                include_unreachable: true,
                include_tests: false,
                min_dead_lines: 1,
            },
        });
        results
    }

    fn create_test_deep_context() -> DeepContext {
        DeepContext {
            analyses: create_test_analysis_results_with_dead_code(),
            quality_scorecard: QualityScorecard {
                overall_health: Some(85.0),
                complexity_score: Some(75.0),
                maintainability_index: Some(80.0),
                modularity_score: Some(70.0),
                test_coverage: Some(65.0),
                technical_debt_hours: Some(4.5),
            },
            recommendations: vec![
                PrioritizedRecommendation {
                    title: "Reduce Complexity".to_string(),
                    description: "Consider refactoring complex functions".to_string(),
                    priority: Priority::High,
                    estimated_effort: Duration::from_secs(7200),
                    impact: Impact::High,
                    prerequisites: vec![],
                },
                PrioritizedRecommendation {
                    title: "Add Tests".to_string(),
                    description: "Increase test coverage".to_string(),
                    priority: Priority::Medium,
                    estimated_effort: Duration::from_secs(3600),
                    impact: Impact::Medium,
                    prerequisites: vec![],
                },
            ],
            ..Default::default()
        }
    }

    fn create_test_project_context_with_files() -> ProjectContext {
        ProjectContext {
            project_type: "rust".to_string(),
            files: vec![
                create_test_file_context_with_complexity("src/main.rs"),
                create_test_file_context("src/lib.rs", "rust"),
            ],
            graph: None,
            summary: ProjectSummary {
                total_files: 2,
                total_functions: 4,
                total_structs: 2,
                total_enums: 2,
                total_traits: 2,
                total_impls: 4,
                dependencies: vec!["serde".to_string(), "tokio".to_string()],
            },
        }
    }

    // Annotation Function Tests

    #[test]
    fn test_add_complexity_annotation_without_data() {
        let file = create_test_file_context("src/test.rs", "rust");
        let analyses = create_test_analysis_results();
        let mut annotations = String::new();

        add_complexity_annotation(&mut annotations, "test_function", &file, &analyses);

        // Should add fallback annotations
        assert!(annotations.contains("[complexity: 3]"));
        assert!(annotations.contains("[cognitive: 2]"));
        assert!(annotations.contains("[big-o: O(n)]"));
    }

    #[test]
    fn test_add_provability_annotation_without_data() {
        let analyses = create_test_analysis_results();
        let mut annotations = String::new();

        add_provability_annotation(&mut annotations, &analyses);

        // Should use default 0.75
        assert!(annotations.contains("[provability: 75%]"));
    }

    #[test]
    fn test_add_satd_annotation_no_items() {
        let file = create_test_file_context("src/other.rs", "rust");
        let analyses = create_test_analysis_results();
        let mut annotations = String::new();

        add_satd_annotation(&mut annotations, &file, &analyses);

        assert!(annotations.contains("[satd: 0]"));
    }

    #[test]
    fn test_add_pagerank_annotation_no_graph() {
        let file = create_test_file_context("src/lib.rs", "rust");
        let analyses = create_test_analysis_results();
        let mut annotations = String::new();

        add_pagerank_annotation(&mut annotations, "test_function", &file, &analyses);

        // No graph, so no annotation
        assert!(annotations.is_empty());
    }

    #[test]
    fn test_add_churn_annotation_high_churn() {
        let file = create_test_file_context("src/lib.rs", "rust");
        let analyses = create_test_analysis_results_with_churn();
        let mut annotations = String::new();

        add_churn_annotation(&mut annotations, &file, &analyses);

        // With 15 commits, should show high churn
        assert!(annotations.contains("[churn: high(15)]"));
    }

    #[test]
    fn test_add_churn_annotation_no_churn_data() {
        let file = create_test_file_context("src/lib.rs", "rust");
        let analyses = create_test_analysis_results();
        let mut annotations = String::new();

        add_churn_annotation(&mut annotations, &file, &analyses);

        // Default fallback
        assert!(annotations.contains("[churn: low(1)]"));
    }

    // Format Function Tests

    #[test]
    fn test_format_markdown_output() {
        let project_context = create_test_project_context_with_files();
        let deep_context = create_test_deep_context();

        let output = format_markdown_output(&project_context, &deep_context, "rust");

        assert!(output.contains("# Project Context"));
        assert!(output.contains("## Project Structure"));
        assert!(output.contains("## Quality Scorecard"));
    }

    #[test]
    fn test_simple_llm_format_with_recommendations() {
        let ctx = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            graph: None,
            summary: ProjectSummary {
                total_files: 0,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
        };

        let output = simple_llm_format(&ctx, "rust", Path::new("/test"));

        assert!(output.contains("Recommendations:"));
        assert!(output.contains("No functions detected"));
    }

    #[test]
    fn test_simple_llm_format_high_average_functions() {
        let ctx = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            graph: None,
            summary: ProjectSummary {
                total_files: 5,
                total_functions: 100, // 20 functions per file average
                total_structs: 10,
                total_enums: 5,
                total_traits: 3,
                total_impls: 15,
                dependencies: vec![],
            },
        };

        let output = simple_llm_format(&ctx, "rust", Path::new("/test"));

        assert!(output.contains("splitting large files"));
    }

    // Builder Function Tests

    #[test]
    fn test_add_project_structure() {
        let mut builder = MarkdownBuilder::new();
        let ctx = create_test_project_context_with_files();

        add_project_structure(&mut builder, &ctx, "rust");

        let content = builder.build();
        assert!(content.contains("**Language**: rust"));
        assert!(content.contains("**Total Files**: 2"));
        assert!(content.contains("**Total Functions**: 4"));
    }

    #[test]
    fn test_add_quality_scorecard() {
        let mut builder = MarkdownBuilder::new();
        let scorecard = QualityScorecard {
            overall_health: Some(85.0),
            complexity_score: Some(75.0),
            maintainability_index: Some(80.0),
            modularity_score: Some(70.0),
            test_coverage: Some(65.0),
            technical_debt_hours: Some(4.5),
        };

        add_quality_scorecard(&mut builder, &scorecard);

        let content = builder.build();
        assert!(content.contains("**Overall Health**: 85.0%"));
        assert!(content.contains("**Complexity Score**: 75.0%"));
        assert!(content.contains("**Test Coverage**: 65.0%"));
        assert!(content.contains("**Technical Debt Hours**: 4.5"));
    }

    #[test]
    fn test_add_recommendations() {
        let mut builder = MarkdownBuilder::new();
        let recommendations = vec![
            PrioritizedRecommendation {
                title: "Fix Bug".to_string(),
                description: "Important fix".to_string(),
                priority: Priority::Critical,
                estimated_effort: Duration::from_secs(3600),
                impact: Impact::High,
                prerequisites: vec![],
            },
            PrioritizedRecommendation {
                title: "Refactor".to_string(),
                description: "Code cleanup".to_string(),
                priority: Priority::Low,
                estimated_effort: Duration::from_secs(1800),
                impact: Impact::Low,
                prerequisites: vec![],
            },
        ];

        add_recommendations(&mut builder, &recommendations);

        let content = builder.build();
        assert!(content.contains("**Fix Bug**"));
        assert!(content.contains("Priority: Critical"));
        assert!(content.contains("**Refactor**"));
        assert!(content.contains("Priority: Low"));
    }
}
