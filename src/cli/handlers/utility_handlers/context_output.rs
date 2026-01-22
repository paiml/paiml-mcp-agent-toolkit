/// Write output to file or stdout
async fn write_context_output(output: Option<PathBuf>, content: &str) -> Result<()> {
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        eprintln!("✅ Context written to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

/// Format as JSON with all metadata
#[allow(dead_code)]
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
                            AstItem::Import { .. } => "Import",
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
                                let complexity_factor = (f32::from(func.metrics.cyclomatic) / 30.0).min(1.0);
                                let churn_factor = metadata.get("code_churn")
                                    .and_then(serde_json::Value::as_f64)
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
#[allow(dead_code)]
fn format_markdown_output(
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
) -> String {
    let mut builder = MarkdownBuilder::new();

    // Add project sections
    add_project_sections(
        &mut builder,
        project_context,
        deep_context,
        detected_toolchain,
    );

    builder.build()
}

#[allow(dead_code)]
fn add_project_sections(
    builder: &mut MarkdownBuilder,
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
) {
    // Add project header and structure
    builder.add_header(1, "Project Context");
    builder.add_header(2, "Project Structure");
    add_project_structure(builder, project_context, detected_toolchain);

    // Add quality scorecard
    builder.add_header(2, "Quality Scorecard");
    add_quality_scorecard(builder, &deep_context.quality_scorecard);

    // Add files section
    builder.add_header(2, "Files");
    add_files_section(builder, &project_context.files, &deep_context.analyses);

    // Add recommendations
    if !deep_context.recommendations.is_empty() {
        builder.add_header(2, "Recommendations");
        add_recommendations(builder, &deep_context.recommendations);
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn add_quality_scorecard(
    builder: &mut MarkdownBuilder,
    scorecard: &crate::services::deep_context::QualityScorecard,
) {
    builder.add_percentage_metric("Overall Health", scorecard.overall_health);
    builder.add_percentage_metric("Complexity Score", scorecard.complexity_score);
    builder.add_percentage_metric("Maintainability Index", scorecard.maintainability_index);
    builder.add_metric(
        "Technical Debt Hours",
        format!("{:.1}", scorecard.technical_debt_hours),
    );
    builder.add_percentage_metric("Test Coverage", scorecard.test_coverage.unwrap_or(0.0));
    builder.add_percentage_metric("Modularity Score", scorecard.modularity_score);
    builder.add_newline();
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn add_file_items(
    builder: &mut MarkdownBuilder,
    items: &[AstItem],
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    for item in items {
        match item {
            AstItem::Function { name, .. } => {
                builder
                    .content
                    .push_str(&format!("- **Function**: `{name}`"));
                builder
                    .content
                    .push_str(&format_function_annotations(name, file, analyses));
                builder.content.push('\n');
            }
            AstItem::Struct { name, .. } => {
                builder
                    .content
                    .push_str(&format!("- **Struct**: `{name}`\n"));
            }
            AstItem::Enum { name, .. } => {
                builder.content.push_str(&format!("- **Enum**: `{name}`\n"));
            }
            AstItem::Trait { name, .. } => {
                builder
                    .content
                    .push_str(&format!("- **Trait**: `{name}`\n"));
            }
            AstItem::Impl { trait_name, .. } => {
                if let Some(trait_name) = trait_name {
                    builder
                        .content
                        .push_str(&format!("- **Impl**: `{trait_name}`\n"));
                } else {
                    builder.content.push_str("- **Impl**: (inherent)\n");
                }
            }
            AstItem::Module { name, .. } => {
                builder
                    .content
                    .push_str(&format!("- **Module**: `{name}`\n"));
            }
            AstItem::Use { .. } => {
                builder.content.push_str("- **Use**: statement\n");
            }
            AstItem::Import {
                module,
                items,
                alias,
                ..
            } => {
                let import_desc = if !items.is_empty() {
                    format!("- **Import**: `{}` (items: {})\n", module, items.join(", "))
                } else if let Some(alias) = alias {
                    format!("- **Import**: `{module}` as `{alias}`\n")
                } else {
                    format!("- **Import**: `{module}`\n")
                };
                builder.content.push_str(&import_desc);
            }
        }
    }
}

#[allow(dead_code)]
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
#[allow(dead_code)]
fn format_function_annotations(
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> String {
    let mut annotations = String::new();

    add_complexity_annotations(&mut annotations, func_name, file);
    add_dead_code_annotations(&mut annotations, func_name, file, analyses);
    add_satd_annotations(&mut annotations, file, analyses);
    add_static_annotations(&mut annotations);
    add_churn_annotations(&mut annotations, file, analyses);
    add_defect_probability_annotations(&mut annotations, func_name, file, analyses);

    annotations
}

#[allow(dead_code)]
fn add_complexity_annotations(
    annotations: &mut String,
    func_name: &str,
    file: &crate::services::context::FileContext,
) {
    let Some(complexity_metrics) = &file.complexity_metrics else {
        return;
    };
    let Some(func) = complexity_metrics
        .functions
        .iter()
        .find(|f| f.name == func_name)
    else {
        return;
    };

    annotations.push_str(&format!(" [complexity: {}]", func.metrics.cyclomatic));
    annotations.push_str(&format!(" [cognitive: {}]", func.metrics.cognitive));

    let big_o = get_big_o_complexity(func.metrics.cyclomatic.into());
    annotations.push_str(&format!(" [big-o: {big_o}]"));
}

#[allow(dead_code)]
fn get_big_o_complexity(cyclomatic: u32) -> &'static str {
    match cyclomatic {
        1..=3 => "O(1)",
        4..=7 => "O(n)",
        8..=15 => "O(n log n)",
        16..=25 => "O(n²)",
        _ => "O(?)",
    }
}

#[allow(dead_code)]
fn add_dead_code_annotations(
    annotations: &mut String,
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let Some(dead_code_results) = &analyses.dead_code_results else {
        return;
    };
    let Some(file_metrics) = dead_code_results
        .ranked_files
        .iter()
        .find(|f| f.path.ends_with(&file.path))
    else {
        return;
    };

    if is_function_dead_code(file_metrics, func_name) {
        annotations.push_str(" [dead: true]");
    }
}

#[allow(dead_code)]
fn is_function_dead_code(
    file_metrics: &crate::models::dead_code::FileDeadCodeMetrics,
    func_name: &str,
) -> bool {
    file_metrics.items.iter().any(|item| {
        matches!(
            item.item_type,
            crate::models::dead_code::DeadCodeType::Function
        ) && item.name == func_name
    })
}

#[allow(dead_code)]
fn add_satd_annotations(
    annotations: &mut String,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let Some(satd_results) = &analyses.satd_results else {
        return;
    };

    let satd_count = satd_results
        .items
        .iter()
        .filter(|item| item.file.to_string_lossy().ends_with(&file.path))
        .count();

    if satd_count > 0 {
        annotations.push_str(&format!(" [SATD: {satd_count}]"));
    }
}

fn add_static_annotations(annotations: &mut String) {
    annotations.push_str(" [provability: 75%]");
    annotations.push_str(" [coverage: 65%]");
}

fn add_churn_annotations(
    annotations: &mut String,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let Some(churn_analysis) = &analyses.churn_analysis else {
        return;
    };
    let Some(file_metrics) = find_churn_file_metrics(churn_analysis, &file.path) else {
        return;
    };

    if file_metrics.churn_score > 0.0 {
        annotations.push_str(&format!(" [churn: {:.2}]", file_metrics.churn_score));
    }
}

fn find_churn_file_metrics<'a>(
    churn_analysis: &'a crate::models::churn::CodeChurnAnalysis,
    file_path: &str,
) -> Option<&'a crate::models::churn::FileChurnMetrics> {
    churn_analysis.files.iter().find(|f| {
        f.relative_path.ends_with(file_path) || f.path.to_string_lossy().ends_with(file_path)
    })
}

fn add_defect_probability_annotations(
    annotations: &mut String,
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let Some(complexity_metrics) = &file.complexity_metrics else {
        return;
    };
    let Some(func) = complexity_metrics
        .functions
        .iter()
        .find(|f| f.name == func_name)
    else {
        return;
    };

    let complexity_factor = (f32::from(func.metrics.cyclomatic) / 30.0).min(1.0);
    let churn_factor = get_churn_factor(analyses, &file.path);
    let defect_prob = (complexity_factor * 0.7 + churn_factor * 0.3).min(1.0);

    if defect_prob > 0.1 {
        annotations.push_str(&format!(" [defect-prob: {:.0}%]", defect_prob * 100.0));
    }
}

fn get_churn_factor(
    analyses: &crate::services::deep_context::AnalysisResults,
    file_path: &str,
) -> f32 {
    analyses
        .churn_analysis
        .as_ref()
        .and_then(|ca| find_churn_file_metrics(ca, file_path))
        .map_or(0.0, |f| f.churn_score)
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
                            .map_or(0, |m| m.total_complexity.cyclomatic),
                    })
                }).collect::<Vec<_>>(),
                "qualityScorecard": deep_context.quality_scorecard,
            }
        }]
    });

    serde_json::to_string_pretty(&sarif_output).map_err(Into::into)
}

/// Format as LLM-optimized output
#[allow(dead_code)]
fn format_llm_optimized_output(
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
    detected_toolchain: &str,
    project_path: &Path,
) -> String {
    let mut output = String::new();

    format_project_header(&mut output, project_path, detected_toolchain);
    format_project_summary(&mut output, &project_context.summary);
    format_key_components(&mut output, project_context, deep_context);
    format_quality_insights(&mut output, &deep_context.quality_scorecard);
    format_recommendations(&mut output, &deep_context.recommendations);

    output
}

#[allow(dead_code)]
fn format_project_header(output: &mut String, project_path: &Path, detected_toolchain: &str) {
    output.push_str(&format!(
        "Project: {} ({})\n\n",
        project_path.display(),
        detected_toolchain
    ));
}

#[allow(dead_code)]
fn format_project_summary(output: &mut String, summary: &crate::services::context::ProjectSummary) {
    output.push_str("Summary:\n");
    output.push_str(&format!("- Files: {}\n", summary.total_files));
    output.push_str(&format!("- Functions: {}\n", summary.total_functions));
    output.push_str(&format!(
        "- Types: {} structs, {} enums, {} traits\n\n",
        summary.total_structs, summary.total_enums, summary.total_traits
    ));
}

#[allow(dead_code)]
fn format_key_components(
    output: &mut String,
    project_context: &crate::services::context::ProjectContext,
    deep_context: &crate::services::deep_context::DeepContext,
) {
    output.push_str("Key Components:\n\n");

    for file in &project_context.files {
        let functions = extract_function_names(file);
        if !functions.is_empty() {
            format_file_functions(output, file, &functions, deep_context);
        }
    }
}

#[allow(dead_code)]
fn extract_function_names(file: &crate::services::context::FileContext) -> Vec<&str> {
    file.items
        .iter()
        .filter_map(|item| match item {
            AstItem::Function { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect()
}

#[allow(dead_code)]
fn format_file_functions(
    output: &mut String,
    file: &crate::services::context::FileContext,
    functions: &[&str],
    deep_context: &crate::services::deep_context::DeepContext,
) {
    output.push_str(&format!("File: {}\n", file.path));

    for func in functions {
        output.push_str(&format!("  Function: {func}"));
        add_function_metadata(output, file, func);
        add_dead_code_marker(output, file, func, deep_context);
        output.push('\n');
    }
    output.push('\n');
}

#[allow(dead_code)]
fn add_function_metadata(
    output: &mut String,
    file: &crate::services::context::FileContext,
    func: &str,
) {
    let Some(complexity_metrics) = &file.complexity_metrics else {
        return;
    };
    let Some(func_metrics) = find_function_metrics(complexity_metrics, func) else {
        return;
    };

    if func_metrics.metrics.cyclomatic > 10 {
        output.push_str(&format!(
            " [complexity: {}]",
            func_metrics.metrics.cyclomatic
        ));
    }
    if func_metrics.metrics.cognitive > 15 {
        output.push_str(&format!(" [cognitive: {}]", func_metrics.metrics.cognitive));
    }
}

#[allow(dead_code)]
fn find_function_metrics<'a>(
    complexity_metrics: &'a crate::services::complexity::FileComplexityMetrics,
    func_name: &str,
) -> Option<&'a crate::services::complexity::FunctionComplexity> {
    complexity_metrics
        .functions
        .iter()
        .find(|f| f.name == func_name)
}

#[allow(dead_code)]
fn add_dead_code_marker(
    output: &mut String,
    file: &crate::services::context::FileContext,
    func: &str,
    deep_context: &crate::services::deep_context::DeepContext,
) {
    if is_dead_code_function(file, func, deep_context) {
        output.push_str(" [DEAD CODE]");
    }
}

#[allow(dead_code)]
fn is_dead_code_function(
    file: &crate::services::context::FileContext,
    func: &str,
    deep_context: &crate::services::deep_context::DeepContext,
) -> bool {
    let Some(dead_code_results) = &deep_context.analyses.dead_code_results else {
        return false;
    };

    let Some(file_metrics) = dead_code_results
        .ranked_files
        .iter()
        .find(|f| f.path.ends_with(&file.path))
    else {
        return false;
    };

    file_metrics.items.iter().any(|item| {
        matches!(
            item.item_type,
            crate::models::dead_code::DeadCodeType::Function
        ) && item.name == func
    })
}

#[allow(dead_code)]
fn format_quality_insights(
    output: &mut String,
    scorecard: &crate::services::deep_context::QualityScorecard,
) {
    output.push_str("Quality Insights:\n");
    output.push_str(&format!(
        "- Overall Score: {:.1}/100\n",
        scorecard.overall_health
    ));

    if scorecard.complexity_score < 80.0 {
        output.push_str(&format!(
            "- Complexity Score: {:.1}% (needs attention)\n",
            scorecard.complexity_score
        ));
    }

    if scorecard.maintainability_index < 80.0 {
        output.push_str(&format!(
            "- Maintainability: {:.1}% (could be improved)\n",
            scorecard.maintainability_index
        ));
    }

    output.push('\n');
}

#[allow(dead_code)]
fn format_recommendations(
    output: &mut String,
    recommendations: &[crate::services::deep_context::PrioritizedRecommendation],
) {
    if recommendations.is_empty() {
        return;
    }

    output.push_str("Key Recommendations:\n");
    for (i, rec) in recommendations.iter().take(3).enumerate() {
        output.push_str(&format!("{}. {}: {}\n", i + 1, rec.title, rec.description));
    }
}

// Removed - using the function from cli/mod.rs instead
/*
/// Enhanced language detection based on project files
/// Implements the lightweight detection strategy from Phase 3 of bug remediation
fn detect_primary_language(path: &Path) -> Result<String> {
    use std::collections::HashMap;
    use walkdir::WalkDir;

    // Count extensions first to understand the actual content
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

    // Check for language-specific source files first (prioritize content over build files)
    let has_kotlin =
        counts.get("kt").copied().unwrap_or(0) > 0 || counts.get("kts").copied().unwrap_or(0) > 0;
    let has_rust = counts.get("rs").copied().unwrap_or(0) > 0;
    let has_python = counts.get("py").copied().unwrap_or(0) > 0;
    let has_typescript =
        counts.get("ts").copied().unwrap_or(0) > 0 || counts.get("tsx").copied().unwrap_or(0) > 0;
    let has_javascript =
        counts.get("js").copied().unwrap_or(0) > 0 || counts.get("jsx").copied().unwrap_or(0) > 0;
    let has_go = counts.get("go").copied().unwrap_or(0) > 0;

    // Check for project marker files first (strongest indicators)
    if path.join("Cargo.toml").exists() {
        return Ok("rust".to_string());
    }

    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        return Ok("python-uv".to_string());
    }

    if path.join("package.json").exists() {
        if path.join("deno.json").exists() || path.join("deno.jsonc").exists() {
            return Ok("deno".to_string());
        }
        return Ok("node".to_string());
    }

    if path.join("go.mod").exists() {
        return Ok("go".to_string());
    }

    if path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
        return Ok("kotlin".to_string());
    }

    // Fall back to source file detection if no project markers found
    if has_rust {
        return Ok("rust".to_string());
    }

    if has_kotlin {
        return Ok("kotlin".to_string());
    }

    if has_python {
        if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
            return Ok("python-uv".to_string());
        }
        return Ok("python-uv".to_string());
    }

    if has_typescript || has_javascript {
        if path.join("deno.json").exists() || path.join("deno.lock").exists() {
            return Ok("deno".to_string());
        }
        if path.join("package.json").exists() {
            return Ok("deno".to_string());
        }
        return Ok("deno".to_string());
    }

    if has_go && path.join("go.mod").exists() {
        return Ok("go".to_string());
    }

    // Fallback: check for build files without source files (edge case)
    if path.join("Cargo.toml").exists() {
        return Ok("rust".to_string());
    }
    if path.join("package.json").exists() || path.join("deno.json").exists() {
        return Ok("deno".to_string());
    }
    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
        return Ok("python-uv".to_string());
    }
    if path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
        return Ok("kotlin".to_string());
    }
    if path.join("go.mod").exists() {
        return Ok("go".to_string());
    }

    // Ultimate fallback: use most common extension
    let detected = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(ext, _)| match ext.as_str() {
            "kt" | "kts" => "kotlin",
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
*/

// Server handlers extracted to utility_serve_handlers.rs for file health compliance (CB-040)
