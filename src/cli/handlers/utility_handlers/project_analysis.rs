
/// Create analyzer and perform project analysis
#[allow(dead_code)]
async fn analyze_project(
    project_path: &Path,
    include_large_files: bool,
    skip_expensive_metrics: bool,
) -> Result<crate::services::deep_context::DeepContext> {
    use crate::services::deep_context::{
        AnalysisType, CacheStrategy, DagType as DeepDagType, DeepContextAnalyzer, DeepContextConfig,
    };
    use crate::services::file_classifier::FileClassifierConfig;

    let config = DeepContextConfig {
        include_analyses: if skip_expensive_metrics {
            vec![
                // Only include AST analysis when skipping expensive metrics
                AnalysisType::Ast,
            ]
        } else {
            vec![
                // Note: AST analysis can be expensive on large codebases
                // Consider using --exclude patterns or specific paths
                AnalysisType::Ast,
                AnalysisType::Complexity,
                AnalysisType::Satd,
                // Skip expensive analyses for context generation
                // AnalysisType::DeadCode,
                // AnalysisType::Provability,
                // AnalysisType::Churn,
            ]
        },
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
        file_classifier_config: if include_large_files {
            Some(FileClassifierConfig {
                skip_vendor: true,
                max_line_length: 10_000,
                max_file_size: 10_485_760, // 10MB when including large files
            })
        } else {
            None
        },
    };

    let analyzer = DeepContextAnalyzer::new(config);
    analyzer.analyze_project(&project_path.to_path_buf()).await
}

/// Build project context from deep context analysis
/// Build project context from working SimpleDeepContext (unified approach)
#[allow(dead_code)]
fn build_project_context_from_simple(
    detected_toolchain: String,
    analysis_report: &crate::services::simple_deep_context::SimpleAnalysisReport,
) -> Result<crate::services::context::ProjectContext> {
    use crate::services::context::{ProjectContext, ProjectSummary};

    let project_context = ProjectContext {
        project_type: detected_toolchain,
        files: vec![], // Simple context doesn't use file-based approach
        graph: None,
        summary: ProjectSummary {
            total_files: analysis_report.file_count,
            total_functions: analysis_report.complexity_metrics.total_functions, // This is the working count!
            total_structs: 0, // Simple context focuses on functions
            total_enums: 0,
            total_traits: 0,
            total_impls: 0,
            dependencies: vec![],
        },
    };

    Ok(project_context)
}

/// Legacy function - kept for compatibility but will be removed in unification
#[allow(dead_code)]
fn build_project_context(
    detected_toolchain: String,
    deep_context: &crate::services::deep_context::DeepContext,
) -> Result<crate::services::context::ProjectContext> {
    use crate::services::context::{ProjectContext, ProjectSummary};

    let mut project_context = ProjectContext {
        project_type: detected_toolchain,
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

    // Convert deep context AST contexts to FileContext with metadata
    project_context.files = deep_context
        .analyses
        .ast_contexts
        .iter()
        .map(|enhanced_ctx| process_file_context(enhanced_ctx, &deep_context.analyses))
        .collect();

    // Update summary statistics (use complexity report if available, like deep-context does)
    if let Some(complexity_report) = &deep_context.analyses.complexity_report {
        project_context.summary.total_functions = complexity_report
            .files
            .iter()
            .map(|f| f.functions.len())
            .sum();
        // Update other stats from complexity report
        project_context.summary.total_files = complexity_report.files.len();
    } else {
        // Fallback to AST-based counting if no complexity report
        update_project_summary(&mut project_context);
    }

    Ok(project_context)
}

/// Process individual file context with enrichment
#[allow(dead_code)]
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

/// Simplified context output formatting (unified approach)
#[allow(dead_code)]
fn format_context_output_simple(
    project_context: &crate::services::context::ProjectContext,
    detected_toolchain: &str,
    project_path: &Path,
    format: ContextFormat,
    _graph_annotations: Option<&Vec<crate::graph::ContextAnnotation>>,
) -> Result<String> {
    let output = match format {
        ContextFormat::Markdown => simple_markdown_format(project_context, detected_toolchain),
        ContextFormat::LlmOptimized => {
            simple_llm_format(project_context, detected_toolchain, project_path)
        }
        ContextFormat::Json => simple_json_format(project_context, detected_toolchain)?,
        ContextFormat::Sarif => simple_sarif_format(project_context, detected_toolchain)?,
    };

    Ok(output)
}

fn simple_markdown_format(ctx: &crate::services::context::ProjectContext, lang: &str) -> String {
    let mut md = String::new();
    md.push_str("# Project Context\n\n## Project Structure\n\n");
    md.push_str(&format!("- **Language**: {}\n", lang));
    md.push_str(&format!("- **Total Files**: {}\n", ctx.summary.total_files));
    md.push_str(&format!(
        "- **Total Functions**: {}\n",
        ctx.summary.total_functions
    ));
    md.push_str(&format!(
        "- **Total Structs**: {}\n",
        ctx.summary.total_structs
    ));
    md.push_str(&format!("- **Total Enums**: {}\n", ctx.summary.total_enums));
    md.push_str(&format!(
        "- **Total Traits**: {}\n\n",
        ctx.summary.total_traits
    ));

    if !ctx.files.is_empty() {
        md.push_str("## Key Components\n\n");
        for file in &ctx.files {
            md.push_str(&format!("### File: {}\n", file.path));
            let functions: Vec<&str> = file
                .items
                .iter()
                .filter_map(|item| match item {
                    crate::services::context::AstItem::Function { name, .. } => Some(name.as_str()),
                    _ => None,
                })
                .collect();

            if !functions.is_empty() {
                md.push_str("**Functions:**\n");
                for func in functions {
                    md.push_str(&format!("- `{}`\n", func));
                }
            }
            md.push('\n');
        }
    }
    md
}

fn simple_llm_format(
    ctx: &crate::services::context::ProjectContext,
    lang: &str,
    path: &Path,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Project: {} ({})\n\nSummary:\n- Files: {}\n- Functions: {}\n- Types: {} structs, {} enums, {} traits\n\n",
        path.file_name().unwrap_or_default().to_string_lossy(),
        lang,
        ctx.summary.total_files,
        ctx.summary.total_functions,
        ctx.summary.total_structs,
        ctx.summary.total_enums,
        ctx.summary.total_traits
    ));

    if ctx.summary.total_functions > 0 {
        out.push_str(&format!(
            "Key Components:\n\nAnalysis detected {} functions across {} files:\n\n(Individual function names require --format deep for detailed AST analysis)\n\n",
            ctx.summary.total_functions, ctx.summary.total_files
        ));
    }

    if ctx.summary.total_functions > 20 {
        out.push_str(&format!(
            "Quality Insights:\n- Large codebase with {} functions across {} files\n",
            ctx.summary.total_functions, ctx.summary.total_files
        ));
        if ctx.summary.total_files > 0 {
            let avg = ctx.summary.total_functions as f64 / ctx.summary.total_files as f64;
            out.push_str(&format!("- Average {:.1} functions per file\n", avg));
            if avg > 10.0 {
                out.push_str("- Consider splitting large files for better maintainability\n");
            }
        }
        out.push('\n');
    }

    out.push_str("Recommendations:\n");
    if ctx.summary.total_functions == 0 {
        out.push_str("- No functions detected - ensure language is properly supported\n");
    } else if ctx.summary.total_functions > 50 {
        out.push_str("- Consider modularizing the codebase for better organization\n");
    }
    if ctx.files.is_empty() {
        out.push_str("- Enable detailed AST analysis for function-level insights\n");
    }
    out
}

fn simple_json_format(
    ctx: &crate::services::context::ProjectContext,
    lang: &str,
) -> Result<String> {
    let mut json = serde_json::json!({
        "project_type": lang,
        "summary": {
            "total_files": ctx.summary.total_files,
            "total_functions": ctx.summary.total_functions,
            "total_structs": ctx.summary.total_structs,
            "total_enums": ctx.summary.total_enums,
            "total_traits": ctx.summary.total_traits,
        }
    });

    if !ctx.files.is_empty() {
        json["files"] = serde_json::json!(ctx.files.iter().map(|file| {
            let funcs: Vec<_> = file.items.iter().filter_map(|item| match item {
                crate::services::context::AstItem::Function { name, .. } => Some(name.clone()),
                _ => None,
            }).collect();
            serde_json::json!({"path": file.path, "functions": funcs, "function_count": funcs.len()})
        }).collect::<Vec<_>>());
    }

    serde_json::to_string_pretty(&json).map_err(Into::into)
}

fn simple_sarif_format(
    ctx: &crate::services::context::ProjectContext,
    lang: &str,
) -> Result<String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-context",
                    "version": "2.98.0",
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit"
                }
            },
            "results": [],
            "properties": {
                "total_functions": ctx.summary.total_functions,
                "total_files": ctx.summary.total_files,
                "language": lang
            }
        }]
    })).map_err(Into::into)
}

/// Update project summary statistics
#[allow(dead_code)]
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
#[allow(dead_code)]
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
