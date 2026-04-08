#![cfg_attr(coverage_nightly, coverage(off))]
use serde_json::Value;

use super::traits::{AnalysisService, TemplateService};
use super::types::*;
use super::AppError;

/// Default implementations for testing
#[derive(Default)]
pub struct DefaultTemplateService;

#[async_trait::async_trait]
impl TemplateService for DefaultTemplateService {
    async fn list_templates(&self, _query: &ListTemplatesQuery) -> Result<TemplateList, AppError> {
        Ok(TemplateList {
            templates: vec![TemplateInfo {
                id: "makefile/rust/cli".to_string(),
                name: "Rust CLI Makefile".to_string(),
                description: "Makefile template for Rust CLI projects".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
            }],
            total: 1,
        })
    }

    async fn get_template(&self, template_id: &str) -> Result<TemplateInfo, AppError> {
        if template_id == "makefile/rust/cli" {
            Ok(TemplateInfo {
                id: template_id.to_string(),
                name: "Rust CLI Makefile".to_string(),
                description: "Makefile template for Rust CLI projects".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
            })
        } else {
            Err(AppError::NotFound(format!(
                "Template not found: {template_id}"
            )))
        }
    }

    async fn generate_template(
        &self,
        params: &GenerateParams,
    ) -> Result<GeneratedTemplate, AppError> {
        Ok(GeneratedTemplate {
            template_id: params.template_uri.clone(),
            content: format!(
                "# Generated Makefile for {}\n\nall:\n\techo 'Building project'\n",
                params
                    .parameters
                    .get("project_name")
                    .unwrap_or(&Value::String("unknown".to_string()))
            ),
            metadata: TemplateMetadata {
                name: "Generated Makefile".to_string(),
                version: "1.0.0".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
            },
        })
    }
}

#[derive(Default)]
/// Default analysis service.
pub struct DefaultAnalysisService;

#[async_trait::async_trait]
impl AnalysisService for DefaultAnalysisService {
    async fn analyze_complexity(
        &self,
        _params: &ComplexityParams,
    ) -> Result<ComplexityAnalysis, AppError> {
        Ok(ComplexityAnalysis {
            summary: ComplexitySummary {
                total_functions: 0,
                average_complexity: 0.0,
                max_complexity: 0,
                files_analyzed: 0,
            },
            files: vec![],
        })
    }

    async fn analyze_churn(&self, _params: &ChurnParams) -> Result<ChurnAnalysis, AppError> {
        Ok(ChurnAnalysis {
            summary: ChurnSummary {
                total_commits: 0,
                files_changed: 0,
                period_days: 0,
            },
            hotspots: vec![],
        })
    }

    async fn analyze_dag(&self, params: &DagParams) -> Result<DagAnalysis, AppError> {
        use crate::cli::DagType;
        use crate::services::dag_builder::DagBuilder;
        use crate::services::mermaid_generator::{MermaidGenerator, MermaidOptions};

        // Parse DAG type from string
        let dag_type = match params.dag_type.as_str() {
            "call-graph" => DagType::CallGraph,
            "import-graph" => DagType::ImportGraph,
            "inheritance" => DagType::Inheritance,
            "full-dependency" => DagType::FullDependency,
            _ => DagType::CallGraph, // Default fallback
        };

        let project_path = std::path::Path::new(&params.project_path);

        // Use context analysis to get project data, then build DAG
        let context = crate::services::context::analyze_project(project_path, "rust")
            .await
            .map_err(|e| AppError::Analysis(format!("Context analysis failed: {e}")))?;

        // Build dependency graph with edge truncation
        let dependency_graph = DagBuilder::build_from_project(&context);

        // Filter by DAG type
        let filtered_graph = match dag_type {
            DagType::CallGraph => crate::services::dag_builder::filter_call_edges(dependency_graph),
            DagType::ImportGraph => {
                crate::services::dag_builder::filter_import_edges(dependency_graph)
            }
            DagType::Inheritance => {
                crate::services::dag_builder::filter_inheritance_edges(dependency_graph)
            }
            DagType::FullDependency => dependency_graph,
        };

        // Generate Mermaid graph
        let options = MermaidOptions {
            show_complexity: params.show_complexity,
            ..Default::default()
        };
        let mermaid_generator = MermaidGenerator::new(options);
        let graph_string = mermaid_generator.generate(&filtered_graph);

        Ok(DagAnalysis {
            graph: graph_string,
            nodes: filtered_graph.nodes.len(),
            edges: filtered_graph.edges.len(),
            cycles: vec![], // TRACKED: Implement cycle detection
        })
    }

    async fn generate_context(&self, _params: &ContextParams) -> Result<ProjectContext, AppError> {
        Ok(ProjectContext {
            project_name: "unknown".to_string(),
            toolchain: "unknown".to_string(),
            structure: ProjectStructure {
                directories: vec![],
                files: vec![],
            },
            metrics: ContextMetrics {
                total_files: 0,
                total_lines: 0,
                complexity_score: 0.0,
            },
        })
    }

    async fn analyze_dead_code(
        &self,
        _params: &DeadCodeParams,
    ) -> Result<DeadCodeAnalysis, AppError> {
        Ok(DeadCodeAnalysis {
            summary: DeadCodeSummary {
                total_files_analyzed: 0,
                files_with_dead_code: 0,
                total_dead_lines: 0,
                dead_percentage: 0.0,
            },
            files: vec![],
        })
    }
}
