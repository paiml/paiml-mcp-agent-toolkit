//! Utility command handlers (list, search, context, etc.)
//!
//! This module contains utility command implementations extracted from
//! the main CLI module to reduce complexity.

use crate::cli::{ContextFormat, OutputFormat};
use crate::models::template::TemplateResource;
use crate::services::context::AstItem;
use crate::services::template_service::{list_templates, search_templates};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Handle template listing command
pub async fn handle_list(
    server: Arc<StatelessTemplateServer>,
    toolchain: Option<String>,
    category: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let templates =
        list_templates(server.as_ref(), toolchain.as_deref(), category.as_deref()).await?;

    match format {
        OutputFormat::Table => super::super::analysis_utilities::print_table(&templates),
        OutputFormat::Json => {
            let templates_deref: Vec<&TemplateResource> =
                templates.iter().map(std::convert::AsRef::as_ref).collect();
            println!("{}", serde_json::to_string_pretty(&templates_deref)?);
        }
        OutputFormat::Yaml => {
            let templates_deref: Vec<&TemplateResource> =
                templates.iter().map(std::convert::AsRef::as_ref).collect();
            println!("{}", serde_yaml::to_string(&templates_deref)?);
        }
    }
    Ok(())
}

// Helper structures for markdown formatting
struct MarkdownBuilder {
    content: String,
}

#[allow(dead_code)]
impl MarkdownBuilder {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn add_header(&mut self, level: usize, text: &str) {
        for _ in 0..level {
            self.content.push('#');
        }
        self.content.push(' ');
        self.content.push_str(text);
        self.content.push_str("\n\n");
    }

    fn add_metric(&mut self, label: &str, value: impl std::fmt::Display) {
        self.content.push_str(&format!("- **{label}**: {value}\n"));
    }

    fn add_percentage_metric(&mut self, label: &str, value: f64) {
        self.content
            .push_str(&format!("- **{label}**: {value:.1}%\n"));
    }

    fn add_newline(&mut self) {
        self.content.push('\n');
    }

    fn build(self) -> String {
        self.content
    }
}

/// Handle template search command
pub async fn handle_search(
    server: Arc<StatelessTemplateServer>,
    query: String,
    toolchain: Option<String>,
    limit: usize,
) -> Result<()> {
    let results = search_templates(server.clone(), &query, toolchain.as_deref()).await?;

    for (i, result) in results.iter().take(limit).enumerate() {
        println!(
            "{:2}. {} (score: {:.2})",
            i + 1,
            result.template.uri,
            result.relevance
        );
        if !result.matches.is_empty() {
            println!("    Matches: {}", result.matches.join(", "));
        }
    }
    Ok(())
}

/// Handle context generation command
#[allow(clippy::too_many_arguments)]
pub async fn handle_context(
    toolchain: Option<String>,
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: ContextFormat,
    include_large_files: bool,
    skip_expensive_metrics: bool,
    language: Option<String>,
    languages: Option<Vec<String>>,
) -> Result<()> {
    use crate::services::deep_context::{
        AnalysisType, CacheStrategy, DagType, DeepContextAnalyzer, DeepContextConfig,
    };
    use crate::services::language_override::{get_effective_languages, LanguageOverride};

    // BUG-012: Apply language override if specified
    let override_opts = LanguageOverride {
        language,
        languages,
    };
    let effective_languages = get_effective_languages(&override_opts, &project_path)?;

    // Use the first effective language as the toolchain (for now - single language support)
    // TODO: Full multi-language support in future sprint
    let toolchain = if !effective_languages.is_empty() {
        effective_languages[0].clone()
    } else {
        detect_or_use_toolchain(toolchain, &project_path)?
    };

    // Configure deep context analysis - RESTORE FULL ANALYSIS CAPABILITY
    let config = DeepContextConfig {
        include_analyses: if skip_expensive_metrics {
            vec![
                AnalysisType::Ast,
                AnalysisType::Complexity,
                AnalysisType::DeadCode,
                AnalysisType::Satd,
            ]
        } else {
            // FULL analysis with ALL annotations - this is what users expect!
            vec![
                AnalysisType::Ast,
                AnalysisType::Complexity,
                AnalysisType::Churn,
                AnalysisType::TechnicalDebtGradient,
                AnalysisType::DeadCode,
                AnalysisType::Satd,
                AnalysisType::Provability,
                AnalysisType::BigO,
            ]
        },
        period_days: 30, // Restore full period for proper churn analysis
        dag_type: DagType::CallGraph,
        complexity_thresholds: Some(crate::services::deep_context::ComplexityThresholds {
            max_cyclomatic: 20,
            max_cognitive: 25,
        }),
        max_depth: Some(5),       // Smart bounds - limit depth for performance
        include_patterns: vec![], // Remove overly restrictive patterns - let file classifier handle it
        exclude_patterns: vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/build/**".to_string(),
            "**/dist/**".to_string(),
            "**/.git/**".to_string(),
            "**/fuzz/**".to_string(),
        ],
        cache_strategy: CacheStrategy::Normal,
        parallel: num_cpus::get(), // Use all available CPUs like cargo test does
        file_classifier_config: None,
    };

    // Run the deep context analysis
    let analyzer = DeepContextAnalyzer::new(config);
    let context = analyzer.analyze_project(&project_path).await?;

    // Generate enhanced annotated AST output
    let output_content = generate_enhanced_ast_context(
        &toolchain,
        &project_path,
        &context,
        format,
        include_large_files,
    )
    .await?;

    // Write output
    write_context_output(output, &output_content).await?;

    Ok(())
}

include!("context_generation.rs");
include!("project_analysis.rs");
include!("context_output.rs");

pub use super::utility_serve_handlers::handle_serve;

/// Handle diagnose command
pub async fn handle_diagnose(args: crate::cli::diagnose::DiagnoseArgs) -> Result<()> {
    crate::cli::diagnose::handle_diagnose(args).await
}

/// Generate graph context analysis for workspace
/// COMPLEXITY: 6 (following pmat quality standards)
#[allow(dead_code)]
async fn generate_graph_context_analysis(
    project_path: &Path,
) -> Result<Vec<crate::graph::ContextAnnotation>> {
    use crate::graph::{DependencyGraphBuilder, GraphContextAnnotator};

    let builder = DependencyGraphBuilder::from_workspace(project_path)?;
    let graph = builder.build()?;

    let annotator = GraphContextAnnotator::new();
    let annotations = annotator.annotate_context(&graph);

    Ok(annotations)
}

/// Format context output with graph analysis integration
/// COMPLEXITY: 8 (enhanced version of existing formatter)
#[allow(dead_code)]
fn format_context_output_with_graph(
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
    project_path: &Path,
    format: ContextFormat,
    graph_annotations: Option<&Vec<crate::graph::ContextAnnotation>>,
) -> Result<String> {
    // First, get the standard output
    let mut output = format_context_output(
        project_context,
        deep_context,
        detected_toolchain,
        project_path,
        format.clone(),
    )?;

    // Add graph analysis section if available
    if let Some(annotations) = graph_annotations {
        let graph_section = generate_graph_section(annotations, format);
        output.push_str(&graph_section);
    }

    Ok(output)
}

/// Generate graph analysis section for output
/// COMPLEXITY: 7
#[allow(dead_code)]
fn generate_graph_section(
    annotations: &[crate::graph::ContextAnnotation],
    format: ContextFormat,
) -> String {
    let mut content = String::new();

    match format {
        ContextFormat::Markdown => {
            content.push_str("\n\n## 📊 Graph Analysis\n\n");
            content.push_str("### 🎯 File Importance Rankings (PageRank)\n\n");

            for (i, annotation) in annotations.iter().take(10).enumerate() {
                content.push_str(&format!(
                    "{}. **{}** (Score: {:.3})\n   - Community: {}\n   - Complexity: {}\n\n",
                    i + 1,
                    annotation.file_path,
                    annotation.importance_score,
                    annotation.community_id,
                    annotation.complexity_rank
                ));
            }

            if !annotations.is_empty() {
                content.push_str("### 🏘️ Community Clusters\n\n");
                let annotator = crate::graph::GraphContextAnnotator::new();
                let clusters = annotator.get_community_clusters(annotations);

                for (community_id, files) in clusters {
                    content.push_str(&format!(
                        "**Community {}**: {} files\n",
                        community_id,
                        files.len()
                    ));
                    for file in files.iter().take(5) {
                        content.push_str(&format!("  - {}\n", file));
                    }
                    if files.len() > 5 {
                        content.push_str(&format!("  - ... and {} more files\n", files.len() - 5));
                    }
                    content.push('\n');
                }
            }
        }
        ContextFormat::Json => {
            // For JSON format, the graph data would be integrated into the main JSON structure
            // For now, we'll add a simple summary
            content.push_str(&format!(
                "\n\"graph_analysis\": {{\"file_count\": {}, \"community_count\": {}}}\n",
                annotations.len(),
                annotations
                    .iter()
                    .map(|a| a.community_id)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
            ));
        }
        ContextFormat::Sarif | ContextFormat::LlmOptimized => {
            // For other formats, add minimal graph info
            content.push_str(&format!(
                "Graph analysis: {} files analyzed",
                annotations.len()
            ));
        }
    }

    content
}

// Tests extracted to utility_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "utility_handlers_tests.rs"]
mod tests;
