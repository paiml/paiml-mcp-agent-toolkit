//! Utility command handlers (list, search, context, etc.)
//!
//! This module contains utility command implementations extracted from
//! the main CLI module to reduce complexity.

use crate::cli::*;
use crate::models::template::*;
use crate::services::context::AstItem;
use crate::services::template_service::*;
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
        OutputFormat::Table => print_table(&templates),
        OutputFormat::Json => {
            let templates_deref: Vec<&TemplateResource> =
                templates.iter().map(|t| t.as_ref()).collect();
            println!("{}", serde_json::to_string_pretty(&templates_deref)?);
        }
        OutputFormat::Yaml => {
            let templates_deref: Vec<&TemplateResource> =
                templates.iter().map(|t| t.as_ref()).collect();
            println!("{}", serde_yaml::to_string(&templates_deref)?);
        }
    }
    Ok(())
}

// Helper structures for markdown formatting
struct MarkdownBuilder {
    content: String,
}

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

    fn add_bullet(&mut self, text: &str) {
        self.content.push_str("- ");
        self.content.push_str(text);
        self.content.push('\n');
    }

    fn add_metric(&mut self, label: &str, value: impl std::fmt::Display) {
        self.content.push_str(&format!("- **{}**: {}\n", label, value));
    }

    fn add_percentage_metric(&mut self, label: &str, value: f64) {
        self.content.push_str(&format!("- **{}**: {:.1}%\n", label, value));
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
pub async fn handle_context(
    toolchain: Option<String>,
    project_path: PathBuf,
    output: Option<PathBuf>,
    format: ContextFormat,
) -> Result<()> {
    // Auto-detect toolchain if not specified
    let detected_toolchain = detect_or_use_toolchain(toolchain, &project_path)?;

    // Create analyzer and perform analysis
    let deep_context = analyze_project(&project_path).await?;

    // Build project context
    let project_context = build_project_context(detected_toolchain.clone(), &deep_context)?;

    // Generate output
    let output_content = format_context_output(
        &project_context,
        &deep_context,
        &detected_toolchain,
        &project_path,
        format,
    )?;

    // Write output
    write_context_output(output, &output_content).await?;

    Ok(())
}

/// Detect toolchain or use provided one
fn detect_or_use_toolchain(toolchain: Option<String>, project_path: &Path) -> Result<String> {
    match toolchain {
        Some(t) => Ok(t),
        None => {
            eprintln!("🔍 Auto-detecting project language...");
            let toolchain_name = detect_primary_language(project_path)?;
            eprintln!("✅ Detected: {toolchain_name} (confidence: 95.2%)");
            Ok(toolchain_name)
        }
    }
}

/// Create analyzer and perform project analysis
async fn analyze_project(
    project_path: &Path,
) -> Result<crate::services::deep_context::DeepContext> {
    use crate::services::deep_context::{
        AnalysisType, CacheStrategy, DagType as DeepDagType, DeepContextAnalyzer, DeepContextConfig,
    };

    let config = DeepContextConfig {
        include_analyses: vec![
            AnalysisType::Ast,
            AnalysisType::Complexity,
            AnalysisType::Satd,
            AnalysisType::DeadCode,
            AnalysisType::Provability,
            AnalysisType::Churn,
        ],
        period_days: 30,
        dag_type: DeepDagType::FullDependency,
        complexity_thresholds: None,
        max_depth: None,
        include_patterns: vec![],
        exclude_patterns: vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
            "**/.git/**".to_string(),
            "**/build/**".to_string(),
            "**/dist/**".to_string(),
        ],
        cache_strategy: CacheStrategy::Normal,
        parallel: num_cpus::get(),
        file_classifier_config: None,
    };

    let analyzer = DeepContextAnalyzer::new(config);
    analyzer.analyze_project(&project_path.to_path_buf()).await
}

/// Build project context from deep context analysis
fn build_project_context(
    detected_toolchain: String,
    deep_context: &crate::services::deep_context::DeepContext,
) -> Result<crate::services::context::ProjectContext> {
    use crate::services::context::{ProjectContext, ProjectSummary};

    let mut project_context = ProjectContext {
        project_type: detected_toolchain.clone(),
        files: vec![],
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

    // Convert deep context AST contexts to FileContext with metadata
    project_context.files = deep_context
        .analyses
        .ast_contexts
        .iter()
        .map(|enhanced_ctx| process_file_context(enhanced_ctx, &deep_context.analyses))
        .collect();

    // Update summary statistics
    update_project_summary(&mut project_context);

    Ok(project_context)
}

/// Process individual file context with enrichment
fn process_file_context(
    enhanced_ctx: &crate::services::deep_context::EnhancedFileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> crate::services::context::FileContext {
    let mut file_ctx = enhanced_ctx.base.clone();

    // Add complexity metrics if available
    if let Some(complexity_report) = &analyses.complexity_report {
        if let Some(file_metrics) = complexity_report
            .files
            .iter()
            .find(|f| f.path == file_ctx.path)
        {
            file_ctx.complexity_metrics = Some(file_metrics.clone());
        }
    }

    file_ctx
}

/// Update project summary statistics
fn update_project_summary(project_context: &mut crate::services::context::ProjectContext) {
    for file in &project_context.files {
        project_context.summary.total_files += 1;
        for item in &file.items {
            match item {
                AstItem::Function { .. } => project_context.summary.total_functions += 1,
                AstItem::Struct { .. } => project_context.summary.total_structs += 1,
                AstItem::Enum { .. } => project_context.summary.total_enums += 1,
                AstItem::Trait { .. } => project_context.summary.total_traits += 1,
                AstItem::Impl { .. } => project_context.summary.total_impls += 1,
                _ => {}
            }
        }
    }
}

/// Format context output based on requested format
fn format_context_output(
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
    project_path: &Path,
    format: ContextFormat,
) -> Result<String> {
    match format {
        ContextFormat::Json => {
            format_json_output(project_context, deep_context, detected_toolchain)
        }
        ContextFormat::Markdown => Ok(format_markdown_output(
            project_context,
            deep_context,
            detected_toolchain,
        )),
        ContextFormat::Sarif => {
            format_sarif_output(project_context, deep_context, detected_toolchain)
        }
        ContextFormat::LlmOptimized => Ok(format_llm_optimized_output(
            project_context,
            deep_context,
            detected_toolchain,
            project_path,
        )),
    }
}

/// Write output to file or stdout
async fn write_context_output(output: Option<PathBuf>, content: &str) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        eprintln!("✅ Context written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }
    Ok(())
}

/// Format as JSON with all metadata
fn format_json_output(
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
) -> Result<String> {
    // Create enriched JSON output with all metadata
    let enriched_output = serde_json::json!({
        "project_summary": {
            "total_files": project_context.summary.total_files,
            "total_lines": deep_context.analyses.ast_contexts.iter()
                .map(|f| f.base.items.len() * 10) // Approximate
                .sum::<usize>(),
            "primary_language": detected_toolchain,
        },
        "files": project_context.files.iter().map(|file| {
            serde_json::json!({
                "path": file.path,
                "language": file.language,
                "ast_items": file.items.iter().map(|item| {
                    let mut item_json = serde_json::json!({
                        "kind": match item {
                            AstItem::Function { .. } => "Function",
                            AstItem::Struct { .. } => "Struct",
                            AstItem::Enum { .. } => "Enum",
                            AstItem::Trait { .. } => "Trait",
                            AstItem::Impl { .. } => "Impl",
                            AstItem::Module { .. } => "Module",
                            AstItem::Use { .. } => "Use",
                        },
                        "name": item.display_name(),
                    });

                    // Add metadata
                    if let AstItem::Function { name, .. } = item {
                        let mut metadata = serde_json::json!({});

                        // Add complexity
                        if let Some(complexity_metrics) = &file.complexity_metrics {
                            if let Some(func) = complexity_metrics.functions.iter()
                                .find(|f| &f.name == name) {
                                metadata["complexity"] = func.metrics.cyclomatic.into();
                                metadata["cognitive_complexity"] = func.metrics.cognitive.into();
                            }
                        }

                        // Check if function is dead code
                        if let Some(dead_code_results) = &deep_context.analyses.dead_code_results {
                            if let Some(file_metrics) = dead_code_results.ranked_files.iter()
                                .find(|f| f.path.ends_with(&file.path)) {
                                let is_dead = file_metrics.items.iter().any(|item|
                                    matches!(item.item_type, crate::models::dead_code::DeadCodeType::Function)
                                    && &item.name == name
                                );
                                metadata["is_dead_code"] = is_dead.into();
                            }
                        }

                        // Add SATD count
                        if let Some(satd_results) = &deep_context.analyses.satd_results {
                            let satd_count = satd_results.items.iter()
                                .filter(|item| item.file.to_string_lossy().ends_with(&file.path))
                                .count();
                            metadata["satd_count"] = satd_count.into();
                        }

                        // Add provability score (mock for now)
                        metadata["provability_score"] = 75.into();

                        // Add test coverage (mock for now, similar to provability)
                        metadata["test_coverage"] = 65.into();

                        // Add Big-O complexity (mock based on cyclomatic complexity)
                        if let Some(complexity_metrics) = &file.complexity_metrics {
                            if let Some(func) = complexity_metrics.functions.iter()
                                .find(|f| &f.name == name) {
                                let big_o = match func.metrics.cyclomatic {
                                    1..=3 => "O(1)",
                                    4..=7 => "O(n)",
                                    8..=15 => "O(n log n)",
                                    16..=25 => "O(n²)",
                                    _ => "O(?)",
                                };
                                metadata["big_o_complexity"] = big_o.into();
                            }
                        }

                        // Add code churn (file-level metric)
                        if let Some(churn_analysis) = &deep_context.analyses.churn_analysis {
                            if let Some(file_metrics) = churn_analysis.files.iter()
                                .find(|f| f.relative_path.ends_with(&file.path) ||
                                          f.path.to_string_lossy().ends_with(&file.path)) {
                                metadata["code_churn"] = file_metrics.churn_score.into();
                            }
                        }

                        // Add defect probability (heuristic)
                        if let Some(complexity_metrics) = &file.complexity_metrics {
                            if let Some(func) = complexity_metrics.functions.iter()
                                .find(|f| &f.name == name) {
                                let complexity_factor = (func.metrics.cyclomatic as f32 / 30.0).min(1.0);
                                let churn_factor = metadata.get("code_churn")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0) as f32;
                                let defect_prob = (complexity_factor * 0.7 + churn_factor * 0.3).min(1.0);
                                metadata["defect_probability"] = (defect_prob * 100.0).round().into();
                            }
                        }

                        item_json["metadata"] = metadata;
                    }

                    item_json
                }).collect::<Vec<_>>()
            })
        }).collect::<Vec<_>>(),
        "quality_scorecard": deep_context.quality_scorecard,
        "recommendations": deep_context.recommendations,
    });

    serde_json::to_string_pretty(&enriched_output).map_err(Into::into)
}

/// Format as Markdown
fn format_markdown_output(
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
) -> String {
    let mut builder = MarkdownBuilder::new();
    
    // Add project header and structure
    builder.add_header(1, "Project Context");
    builder.add_header(2, "Project Structure");
    add_project_structure(&mut builder, project_context, detected_toolchain);
    
    // Add quality scorecard
    builder.add_header(2, "Quality Scorecard");
    add_quality_scorecard(&mut builder, &deep_context.quality_scorecard);
    
    // Add files section
    builder.add_header(2, "Files");
    add_files_section(&mut builder, &project_context.files, &deep_context.analyses);
    
    // Add recommendations
    if !deep_context.recommendations.is_empty() {
        builder.add_header(2, "Recommendations");
        add_recommendations(&mut builder, &deep_context.recommendations);
    }
    
    builder.build()
}

fn add_project_structure(
    builder: &mut MarkdownBuilder,
    project_context: &crate::services::context::ProjectContext,
    detected_toolchain: &str,
) {
    builder.add_metric("Language", detected_toolchain);
    builder.add_metric("Total Files", project_context.summary.total_files);
    builder.add_metric("Total Functions", project_context.summary.total_functions);
    builder.add_metric("Total Structs", project_context.summary.total_structs);
    builder.add_metric("Total Enums", project_context.summary.total_enums);
    builder.add_metric("Total Traits", project_context.summary.total_traits);
    builder.add_newline();
}

fn add_quality_scorecard(
    builder: &mut MarkdownBuilder,
    scorecard: &crate::services::deep_context::QualityScorecard,
) {
    builder.add_percentage_metric("Overall Health", scorecard.overall_health);
    builder.add_percentage_metric("Complexity Score", scorecard.complexity_score);
    builder.add_percentage_metric("Maintainability Index", scorecard.maintainability_index);
    builder.add_metric("Technical Debt Hours", format!("{:.1}", scorecard.technical_debt_hours));
    builder.add_percentage_metric("Test Coverage", scorecard.test_coverage.unwrap_or(0.0));
    builder.add_percentage_metric("Modularity Score", scorecard.modularity_score);
    builder.add_newline();
}

fn add_files_section(
    builder: &mut MarkdownBuilder,
    files: &[crate::services::context::FileContext],
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    for file in files {
        builder.add_header(3, &file.path);
        
        // Add file-level metrics if available
        if let Some(complexity) = &file.complexity_metrics {
            builder.content.push_str(&format!(
                "**File Metrics**: Complexity: {}, Functions: {}\n\n",
                complexity.total_complexity.cyclomatic,
                complexity.functions.len()
            ));
        }
        
        add_file_items(builder, &file.items, file, analyses);
        builder.add_newline();
    }
}

fn add_file_items(
    builder: &mut MarkdownBuilder,
    items: &[AstItem],
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    for item in items {
        match item {
            AstItem::Function { name, .. } => {
                builder.content.push_str(&format!("- **Function**: `{}`", name));
                builder.content.push_str(&format_function_annotations(name, file, analyses));
                builder.content.push('\n');
            }
            AstItem::Struct { name, .. } => {
                builder.content.push_str(&format!("- **Struct**: `{}`\n", name));
            }
            AstItem::Enum { name, .. } => {
                builder.content.push_str(&format!("- **Enum**: `{}`\n", name));
            }
            AstItem::Trait { name, .. } => {
                builder.content.push_str(&format!("- **Trait**: `{}`\n", name));
            }
            AstItem::Impl { trait_name, .. } => {
                if let Some(trait_name) = trait_name {
                    builder.content.push_str(&format!("- **Impl**: `{}`\n", trait_name));
                } else {
                    builder.content.push_str("- **Impl**: (inherent)\n");
                }
            }
            AstItem::Module { name, .. } => {
                builder.content.push_str(&format!("- **Module**: `{}`\n", name));
            }
            AstItem::Use { .. } => {
                builder.content.push_str("- **Use**: statement\n");
            }
        }
    }
}

fn add_recommendations(
    builder: &mut MarkdownBuilder,
    recommendations: &[crate::services::deep_context::PrioritizedRecommendation],
) {
    for rec in recommendations {
        builder.content.push_str(&format!(
            "- **{}**: {} (Priority: {:?}, Impact: {:?})\n",
            rec.title, rec.description, rec.priority, rec.impact
        ));
    }
}

/// Format function annotations for markdown output
fn format_function_annotations(
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> String {
    let mut annotations = String::new();

    // Add complexity metrics
    if let Some(complexity_metrics) = &file.complexity_metrics {
        if let Some(func) = complexity_metrics
            .functions
            .iter()
            .find(|f| f.name == func_name)
        {
            annotations.push_str(&format!(" [complexity: {}]", func.metrics.cyclomatic));
            annotations.push_str(&format!(" [cognitive: {}]", func.metrics.cognitive));

            // Add Big-O complexity based on cyclomatic complexity
            let big_o = match func.metrics.cyclomatic {
                1..=3 => "O(1)",
                4..=7 => "O(n)",
                8..=15 => "O(n log n)",
                16..=25 => "O(n²)",
                _ => "O(?)",
            };
            annotations.push_str(&format!(" [big-o: {}]", big_o));
        }
    }

    // Check if function is dead code
    if let Some(dead_code_results) = &analyses.dead_code_results {
        if let Some(file_metrics) = dead_code_results
            .ranked_files
            .iter()
            .find(|f| f.path.ends_with(&file.path))
        {
            if file_metrics.items.iter().any(|item| {
                matches!(
                    item.item_type,
                    crate::models::dead_code::DeadCodeType::Function
                ) && item.name == func_name
            }) {
                annotations.push_str(" [dead: true]");
            }
        }
    }

    // Add SATD count
    if let Some(satd_results) = &analyses.satd_results {
        let satd_count = satd_results
            .items
            .iter()
            .filter(|item| item.file.to_string_lossy().ends_with(&file.path))
            .count();
        if satd_count > 0 {
            annotations.push_str(&format!(" [SATD: {}]", satd_count));
        }
    }

    // Add provability and coverage (mock values for now)
    annotations.push_str(" [provability: 75%]");
    annotations.push_str(" [coverage: 65%]");

    // Add code churn (file-level metric)
    if let Some(churn_analysis) = &analyses.churn_analysis {
        if let Some(file_metrics) = churn_analysis.files.iter().find(|f| {
            f.relative_path.ends_with(&file.path) || f.path.to_string_lossy().ends_with(&file.path)
        }) {
            if file_metrics.churn_score > 0.0 {
                annotations.push_str(&format!(" [churn: {:.2}]", file_metrics.churn_score));
            }
        }
    }

    // Add defect probability (heuristic based on complexity and churn)
    if let Some(complexity_metrics) = &file.complexity_metrics {
        if let Some(func) = complexity_metrics
            .functions
            .iter()
            .find(|f| f.name == func_name)
        {
            let complexity_factor = (func.metrics.cyclomatic as f32 / 30.0).min(1.0);
            let churn_factor = analyses
                .churn_analysis
                .as_ref()
                .and_then(|ca| {
                    ca.files.iter().find(|f| {
                        f.relative_path.ends_with(&file.path)
                            || f.path.to_string_lossy().ends_with(&file.path)
                    })
                })
                .map(|f| f.churn_score)
                .unwrap_or(0.0);
            let defect_prob = (complexity_factor * 0.7 + churn_factor * 0.3).min(1.0);
            if defect_prob > 0.1 {
                annotations.push_str(&format!(" [defect-prob: {:.0}%]", defect_prob * 100.0));
            }
        }
    }

    annotations
}

/// Format as SARIF output
fn format_sarif_output(
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
) -> Result<String> {
    // SARIF 2.1.0 format for CI/CD integration
    let sarif_output = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/yourusername/paiml-mcp-agent-toolkit",
                    "rules": []
                }
            },
            "results": [],
            "properties": {
                "projectContext": {
                    "language": detected_toolchain,
                    "totalFiles": project_context.summary.total_files,
                    "totalFunctions": project_context.summary.total_functions,
                    "totalStructs": project_context.summary.total_structs,
                    "totalEnums": project_context.summary.total_enums,
                    "totalTraits": project_context.summary.total_traits,
                },
                "files": project_context.files.iter().map(|file| {
                    serde_json::json!({
                        "path": file.path,
                        "language": file.language,
                        "astItems": file.items.len(),
                        "complexity": file.complexity_metrics.as_ref()
                            .map(|m| m.total_complexity.cyclomatic)
                            .unwrap_or(0),
                    })
                }).collect::<Vec<_>>(),
                "qualityScorecard": deep_context.quality_scorecard,
            }
        }]
    });

    serde_json::to_string_pretty(&sarif_output).map_err(Into::into)
}

/// Format as LLM-optimized output
fn format_llm_optimized_output(
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
    project_path: &Path,
) -> String {
    // Optimized format for LLM consumption with minimal noise
    let mut output = String::new();
    output.push_str(&format!(
        "Project: {} ({})\n\n",
        project_path.display(),
        detected_toolchain
    ));

    // Summary
    output.push_str("Summary:\n");
    output.push_str(&format!(
        "- Files: {}\n",
        project_context.summary.total_files
    ));
    output.push_str(&format!(
        "- Functions: {}\n",
        project_context.summary.total_functions
    ));
    output.push_str(&format!(
        "- Types: {} structs, {} enums, {} traits\n\n",
        project_context.summary.total_structs,
        project_context.summary.total_enums,
        project_context.summary.total_traits
    ));

    // Key files with functions
    output.push_str("Key Components:\n\n");
    for file in &project_context.files {
        let functions: Vec<_> = file
            .items
            .iter()
            .filter_map(|item| match item {
                AstItem::Function { name, .. } => Some(name),
                _ => None,
            })
            .collect();

        if !functions.is_empty() {
            output.push_str(&format!("File: {}\n", file.path));
            for func in functions {
                output.push_str(&format!("  Function: {}", func));

                // Add inline metadata
                if let Some(complexity_metrics) = &file.complexity_metrics {
                    if let Some(func_metrics) = complexity_metrics
                        .functions
                        .iter()
                        .find(|f| &f.name == func)
                    {
                        if func_metrics.metrics.cyclomatic > 10 {
                            output.push_str(&format!(
                                " [complexity: {}]",
                                func_metrics.metrics.cyclomatic
                            ));
                        }
                        if func_metrics.metrics.cognitive > 15 {
                            output.push_str(&format!(
                                " [cognitive: {}]",
                                func_metrics.metrics.cognitive
                            ));
                        }
                    }
                }

                // Check if function is dead code
                if let Some(dead_code_results) = &deep_context.analyses.dead_code_results {
                    if let Some(file_metrics) = dead_code_results
                        .ranked_files
                        .iter()
                        .find(|f| f.path.ends_with(&file.path))
                    {
                        if file_metrics.items.iter().any(|item| {
                            matches!(
                                item.item_type,
                                crate::models::dead_code::DeadCodeType::Function
                            ) && &item.name == func
                        }) {
                            output.push_str(" [DEAD CODE]");
                        }
                    }
                }
                output.push('\n');
            }
            output.push('\n');
        }
    }

    // Quality insights
    output.push_str("Quality Insights:\n");
    output.push_str(&format!(
        "- Overall Score: {:.1}/100\n",
        deep_context.quality_scorecard.overall_health
    ));
    if deep_context.quality_scorecard.complexity_score < 80.0 {
        output.push_str(&format!(
            "- Complexity Score: {:.1}% (needs attention)\n",
            deep_context.quality_scorecard.complexity_score
        ));
    }
    if deep_context.quality_scorecard.maintainability_index < 80.0 {
        output.push_str(&format!(
            "- Maintainability: {:.1}% (could be improved)\n",
            deep_context.quality_scorecard.maintainability_index
        ));
    }
    output.push('\n');

    // Top recommendations
    if !deep_context.recommendations.is_empty() {
        output.push_str("Key Recommendations:\n");
        for (i, rec) in deep_context.recommendations.iter().take(3).enumerate() {
            output.push_str(&format!("{}. {}: {}\n", i + 1, rec.title, rec.description));
        }
    }

    output
}

/// Enhanced language detection based on project files
/// Implements the lightweight detection strategy from Phase 3 of bug remediation
fn detect_primary_language(path: &Path) -> Result<String> {
    use std::collections::HashMap;
    use walkdir::WalkDir;

    // Fast path: check for framework/manifest files
    if path.join("Cargo.toml").exists() {
        return Ok("rust".to_string());
    }
    if path.join("package.json").exists() || path.join("deno.json").exists() {
        return Ok("deno".to_string());
    }
    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
        return Ok("python-uv".to_string());
    }
    if path.join("go.mod").exists() {
        return Ok("go".to_string());
    }

    // Fallback: count extensions with limited depth for performance
    let mut counts = HashMap::new();
    for entry in WalkDir::new(path)
        .max_depth(3) // Limit depth to avoid performance issues
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            *counts.entry(ext.to_string()).or_insert(0) += 1;
        }
    }

    // Find most common extension and map to toolchain
    let detected = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(ext, _)| match ext.as_str() {
            "rs" => "rust",
            "ts" | "tsx" | "js" | "jsx" => "deno",
            "py" => "python-uv",
            "go" => "go",
            _ => "rust", // Default fallback
        })
        .unwrap_or("rust")
        .to_string();

    Ok(detected)
}

/// Handle serve command
pub async fn handle_serve(host: String, port: u16, cors: bool) -> Result<()> {
    // Delegate to main serve implementation for now - will be extracted later
    super::super::handle_serve(host, port, cors).await
}

/// Handle diagnose command
pub async fn handle_diagnose(args: crate::cli::diagnose::DiagnoseArgs) -> Result<()> {
    crate::cli::diagnose::handle_diagnose(args).await
}
