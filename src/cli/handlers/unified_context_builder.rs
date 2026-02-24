// UnifiedContextBuilder - Integrates all advanced annotations into unified context output
// use crate::services::simple_deep_context::SimpleDeepContext;
#![allow(dead_code)]

use crate::services::context::ProjectContext;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

pub struct UnifiedContextBuilder {
    output: String,
    project_path: PathBuf,
    #[allow(dead_code)]
    annotations: HashMap<String, String>,
}

impl UnifiedContextBuilder {
    pub fn new(project_path: &Path) -> Self {
        Self {
            output: String::new(),
            project_path: project_path.to_path_buf(),
            annotations: HashMap::new(),
        }
    }
}

impl Display for UnifiedContextBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.output)
    }
}

include!("unified_context_sync_methods.rs");
include!("unified_context_async_methods.rs");
include!("unified_context_analysis.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_project() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    // ============================================================================
    // UnifiedContextBuilder basic tests
    // ============================================================================

    #[test]
    fn test_builder_new() {
        let temp = create_temp_project();
        let builder = UnifiedContextBuilder::new(temp.path());
        assert!(builder.output.is_empty());
    }

    #[test]
    fn test_builder_add_basic_structure() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_basic_structure();

        let output = builder.build();
        assert!(output.contains("# Project Context"));
        assert!(output.contains("## Project Structure"));
        assert!(output.contains("**Language**"));
        assert!(output.contains("**Total Files**"));
        assert!(output.contains("**Total Functions**"));
        assert!(output.contains("**Total Structs**"));
        assert!(output.contains("**Total Enums**"));
        assert!(output.contains("**Total Traits**"));
    }

    #[test]
    fn test_builder_add_big_o_analysis() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_big_o_analysis();

        let output = builder.build();
        assert!(output.contains("## Big-O Complexity Analysis"));
        assert!(output.contains("O(n)"));
        assert!(output.contains("O(n log n)"));
        assert!(output.contains("O(n²)"));
    }

    #[test]
    fn test_builder_add_entropy_analysis() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_entropy_analysis();

        let output = builder.build();
        assert!(output.contains("## Entropy Analysis"));
        assert!(output.contains("Pattern Entropy"));
        assert!(output.contains("Code Duplication"));
        assert!(output.contains("Structural Entropy"));
        assert!(output.contains("Actionable Improvements"));
    }

    #[test]
    fn test_builder_add_tdg_analysis() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_tdg_analysis();

        let output = builder.build();
        assert!(output.contains("## Technical Debt Gradient (TDG)"));
        assert!(output.contains("Overall TDG Score"));
        assert!(output.contains("File-level TDG"));
        assert!(output.contains("Debt Hotspots"));
        assert!(output.contains("Refactoring Priority"));
    }

    #[test]
    fn test_builder_chaining() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder
            .add_basic_structure()
            .add_big_o_analysis()
            .add_entropy_analysis()
            .add_tdg_analysis();
        let output = builder.build();

        assert!(output.contains("# Project Context"));
        assert!(output.contains("## Big-O Complexity Analysis"));
        assert!(output.contains("## Entropy Analysis"));
        assert!(output.contains("## Technical Debt Gradient"));
    }

    #[test]
    fn test_builder_display() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_basic_structure();

        let display_output = format!("{}", builder);
        assert!(display_output.contains("# Project Context"));
    }

    // ============================================================================
    // DeadCodeAnalysis tests
    // ============================================================================

    #[test]
    fn test_dead_code_analysis_is_empty_when_all_empty() {
        let analysis = DeadCodeAnalysis {
            unreachable_functions: vec![],
            unused_variables: vec![],
            unused_imports: vec![],
            dead_branches: vec![],
        };
        assert!(analysis.is_empty());
    }

    #[test]
    fn test_dead_code_analysis_is_not_empty_with_unreachable_functions() {
        let analysis = DeadCodeAnalysis {
            unreachable_functions: vec!["unused_fn".to_string()],
            unused_variables: vec![],
            unused_imports: vec![],
            dead_branches: vec![],
        };
        assert!(!analysis.is_empty());
    }

    #[test]
    fn test_dead_code_analysis_is_not_empty_with_unused_variables() {
        let analysis = DeadCodeAnalysis {
            unreachable_functions: vec![],
            unused_variables: vec!["x".to_string()],
            unused_imports: vec![],
            dead_branches: vec![],
        };
        assert!(!analysis.is_empty());
    }

    #[test]
    fn test_dead_code_analysis_is_not_empty_with_unused_imports() {
        let analysis = DeadCodeAnalysis {
            unreachable_functions: vec![],
            unused_variables: vec![],
            unused_imports: vec!["std::io".to_string()],
            dead_branches: vec![],
        };
        assert!(!analysis.is_empty());
    }

    #[test]
    fn test_dead_code_analysis_is_not_empty_with_dead_branches() {
        let analysis = DeadCodeAnalysis {
            unreachable_functions: vec![],
            unused_variables: vec![],
            unused_imports: vec![],
            dead_branches: vec!["line 42".to_string()],
        };
        assert!(!analysis.is_empty());
    }

    // ============================================================================
    // Analysis struct tests
    // ============================================================================

    #[test]
    fn test_big_o_analysis_creation() {
        let mut complexities = HashMap::new();
        complexities.insert("sort".to_string(), "O(n log n)".to_string());
        let analysis = BigOAnalysis { complexities };
        assert_eq!(analysis.complexities.len(), 1);
    }

    #[test]
    fn test_entropy_analysis_creation() {
        let analysis = EntropyAnalysis {
            pattern_entropy: 0.75,
            duplication_percentage: 15.0,
            structural_entropy: 0.65,
            actionable_improvements: vec!["Improve code".to_string()],
        };
        assert_eq!(analysis.pattern_entropy, 0.75);
        assert_eq!(analysis.duplication_percentage, 15.0);
        assert_eq!(analysis.structural_entropy, 0.65);
        assert_eq!(analysis.actionable_improvements.len(), 1);
    }

    #[test]
    fn test_provability_analysis_creation() {
        let analysis = ProvabilityAnalysis {
            invariants: vec!["x > 0".to_string()],
            preconditions: vec!["input != null".to_string()],
            postconditions: vec!["result >= 0".to_string()],
            is_sound: true,
            is_complete: false,
        };
        assert!(analysis.is_sound);
        assert!(!analysis.is_complete);
        assert_eq!(analysis.invariants.len(), 1);
    }

    #[test]
    fn test_graph_metrics_analysis_creation() {
        let analysis = GraphMetricsAnalysis {
            betweenness: 0.5,
            closeness: 0.7,
            degree: 0.3,
            node_count: 100,
            edge_count: 200,
            cyclomatic: 15,
            critical_paths: vec!["A -> B -> C".to_string()],
        };
        assert_eq!(analysis.node_count, 100);
        assert_eq!(analysis.edge_count, 200);
        assert_eq!(analysis.cyclomatic, 15);
    }

    #[test]
    fn test_tdg_analysis_creation() {
        let analysis = TdgAnalysis {
            overall_score: 3.5,
            file_scores: HashMap::new(),
            hotspots: vec![],
            priorities: vec!["Refactor utils.rs".to_string()],
        };
        assert_eq!(analysis.overall_score, 3.5);
        assert!(analysis.hotspots.is_empty());
    }

    #[test]
    fn test_tdg_hotspot_creation() {
        let hotspot = TdgHotspot {
            location: "main.rs:42".to_string(),
            score: 4.5,
        };
        assert_eq!(hotspot.location, "main.rs:42");
        assert_eq!(hotspot.score, 4.5);
    }

    #[test]
    fn test_satd_comment_creation() {
        let comment = SatdComment {
            location: "src/lib.rs:10".to_string(),
            comment: "TODO: fix this".to_string(),
        };
        assert_eq!(comment.location, "src/lib.rs:10");
        assert!(comment.comment.contains("TODO"));
    }

    #[test]
    fn test_satd_analysis_creation() {
        let analysis = SatdAnalysis {
            todos: vec![SatdComment {
                location: "file.rs:1".to_string(),
                comment: "TODO".to_string(),
            }],
            fixmes: vec![],
            hacks: vec![],
            tech_debt: vec![],
            design_debt_count: 1,
            code_debt_count: 2,
            test_debt_count: 3,
            doc_debt_count: 0,
        };
        assert_eq!(analysis.todos.len(), 1);
        assert_eq!(analysis.design_debt_count, 1);
        assert_eq!(analysis.code_debt_count, 2);
    }

    // ============================================================================
    // Error enum tests
    // ============================================================================

    #[test]
    fn test_error_not_implemented() {
        let err = Error::NotImplemented;
        assert!(matches!(err, Error::NotImplemented));
    }

    #[test]
    fn test_error_analysis_failed() {
        let err = Error::AnalysisFailed("test error".to_string());
        if let Error::AnalysisFailed(msg) = err {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected AnalysisFailed variant");
        }
    }

    // ============================================================================
    // Async analysis function tests
    // ============================================================================

    #[tokio::test]
    async fn test_run_entropy_analysis_returns_ok() {
        let temp = create_temp_project();
        let result = run_entropy_analysis(temp.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_provability_analysis_returns_ok() {
        let temp = create_temp_project();
        let result = run_provability_analysis(temp.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_graph_metrics_analysis_returns_ok() {
        let temp = create_temp_project();
        let result = run_graph_metrics_analysis(temp.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_tdg_analysis_returns_ok() {
        let temp = create_temp_project();
        let result = run_tdg_analysis(temp.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_dead_code_analysis_returns_ok() {
        let temp = create_temp_project();
        let result = run_dead_code_analysis(temp.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_satd_analysis_works() {
        let temp = create_temp_project();
        let result = run_satd_analysis(temp.path()).await;
        assert!(result.is_ok());
    }

    // ============================================================================
    // Async builder methods tests
    // ============================================================================

    #[tokio::test]
    async fn test_builder_add_entropy_analysis_async() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_entropy_analysis_async().await;

        let output = builder.build();
        assert!(output.contains("## Entropy Analysis"));
    }

    #[tokio::test]
    async fn test_builder_add_provability_analysis() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_provability_analysis().await;

        let output = builder.build();
        assert!(output.contains("## Provability Analysis"));
    }

    #[tokio::test]
    async fn test_builder_add_graph_metrics() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_graph_metrics().await;

        let output = builder.build();
        assert!(output.contains("## Graph Metrics"));
    }

    #[tokio::test]
    async fn test_builder_add_tdg_analysis_async() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_tdg_analysis_async().await;

        let output = builder.build();
        assert!(output.contains("## Technical Debt Gradient"));
    }

    #[tokio::test]
    async fn test_builder_add_dead_code_analysis() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_dead_code_analysis().await;

        let output = builder.build();
        assert!(output.contains("## Dead Code Analysis"));
    }

    #[tokio::test]
    async fn test_builder_add_satd_analysis() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());
        builder.add_satd_analysis().await;

        let output = builder.build();
        assert!(output.contains("## Self-Admitted Technical Debt"));
    }

    // ============================================================================
    // ProjectContext integration tests
    // ============================================================================

    #[test]
    fn test_builder_add_quality_insights() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());

        // Create a minimal ProjectContext for testing
        let context = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            summary: crate::services::context::ProjectSummary {
                total_files: 10,
                total_functions: 50,
                total_structs: 5,
                total_enums: 3,
                total_traits: 2,
                total_impls: 8,
                dependencies: vec![],
            },
            graph: None,
        };

        builder.add_quality_insights(&context);

        let output = builder.build();
        assert!(output.contains("## Quality Insights"));
        assert!(output.contains("50 functions"));
    }

    #[test]
    fn test_builder_add_recommendations() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());

        let context = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            summary: crate::services::context::ProjectSummary {
                total_files: 5,
                total_functions: 10,
                total_structs: 2,
                total_enums: 1,
                total_traits: 1,
                total_impls: 3,
                dependencies: vec![],
            },
            graph: None,
        };

        builder.add_recommendations(&context);

        let output = builder.build();
        assert!(output.contains("## Recommendations"));
        assert!(output.contains("modularizing"));
    }

    #[test]
    fn test_builder_add_key_components_empty() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());

        let context = ProjectContext {
            project_type: "rust".to_string(),
            files: vec![],
            summary: crate::services::context::ProjectSummary {
                total_files: 0,
                total_functions: 0,
                total_structs: 0,
                total_enums: 0,
                total_traits: 0,
                total_impls: 0,
                dependencies: vec![],
            },
            graph: None,
        };

        builder.add_key_components(&context);

        let output = builder.build();
        assert!(output.contains("## Key Components"));
        assert!(output.contains("No files analyzed"));
    }

    #[test]
    fn test_builder_add_basic_structure_with_context() {
        let temp = create_temp_project();
        let mut builder = UnifiedContextBuilder::new(temp.path());

        let context = ProjectContext {
            project_type: "typescript".to_string(),
            files: vec![],
            summary: crate::services::context::ProjectSummary {
                total_files: 20,
                total_functions: 100,
                total_structs: 10,
                total_enums: 5,
                total_traits: 0,
                total_impls: 15,
                dependencies: vec![],
            },
            graph: None,
        };

        builder.add_basic_structure_with_context(&context);

        let output = builder.build();
        assert!(output.contains("typescript"));
        assert!(output.contains("20"));
        assert!(output.contains("100"));
    }
}
