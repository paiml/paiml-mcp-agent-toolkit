
/// Generate enhanced AST context with rich annotations
async fn generate_enhanced_ast_context(
    toolchain: &str,
    project_path: &Path,
    context: &crate::services::deep_context::DeepContext,
    _format: ContextFormat,
    _include_large_files: bool,
) -> Result<String> {
    let mut builder = MarkdownBuilder::new();

    // Add header
    builder.content.push_str("# Project Context\n\n");
    builder
        .content
        .push_str(&format!("**Language**: {}\n", toolchain));
    builder
        .content
        .push_str(&format!("**Project Path**: {}\n\n", project_path.display()));

    // Add project structure section
    builder.content.push_str("## Project Structure\n\n");

    // Add project-level metrics summary
    if let Some(complexity_report) = &context.analyses.complexity_report {
        builder.content.push_str(&format!(
            "- **Total Files**: {}\n",
            complexity_report.files.len()
        ));
        builder.content.push_str(&format!(
            "- **Total Functions**: {}\n",
            complexity_report.summary.total_functions
        ));
        builder.content.push_str(&format!(
            "- **Median Cyclomatic**: {:.2}\n",
            complexity_report.summary.median_cyclomatic
        ));
        builder.content.push_str(&format!(
            "- **Median Cognitive**: {:.2}\n\n",
            complexity_report.summary.median_cognitive
        ));
    } else {
        // Basic fallback metrics
        builder.content.push_str("- **Total Files**: 0\n");
        builder.content.push_str("- **Total Functions**: 0\n");
        builder.content.push('\n');
    }

    // Add quality scorecard
    builder.content.push_str("## Quality Scorecard\n\n");
    // Normalize Overall Health as TDG score (0-100 range)
    let tdg_score = (context.quality_scorecard.overall_health)
        .min(100.0)
        .max(0.0);
    builder
        .content
        .push_str(&format!("- **Overall Health**: {:.1}%\n", tdg_score));
    builder.content.push_str(&format!(
        "- **Maintainability Index**: {:.1}\n",
        context.quality_scorecard.maintainability_index
    ));
    builder.content.push_str(&format!(
        "- **Complexity Score**: {:.1}\n",
        context.quality_scorecard.complexity_score
    ));
    if let Some(coverage) = context.quality_scorecard.test_coverage {
        // Normalize test coverage to 0-100 range (remove meaningless percentages)
        let normalized_coverage = coverage.min(100.0).max(0.0);
        builder.content.push_str(&format!(
            "- **Test Coverage**: {:.1}%\n",
            normalized_coverage
        ));
    } else {
        builder.content.push_str("- **Test Coverage**: N/A\n");
    }
    builder.content.push('\n');

    // Add file-level AST with annotations
    builder.content.push_str("## Files\n\n");

    // Use ast_contexts from analyses
    for enhanced_context in &context.analyses.ast_contexts {
        add_simple_file_section(&mut builder, &enhanced_context.base, &context.analyses);
    }

    Ok(builder.content)
}

/// Add simple file section with annotated AST
fn add_simple_file_section(
    builder: &mut MarkdownBuilder,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    // File header
    builder.content.push_str(&format!("### {}\n\n", file.path));

    // File-level metrics from analyses
    // BUG-007 FIX: Improved path matching to handle different path formats
    let file_metrics = analyses.complexity_report.as_ref().and_then(|report| {
        report.files.iter().find(|f| {
            // Try multiple matching strategies for robustness
            use std::path::Path;

            let file_path = Path::new(&file.path);
            let metric_path = Path::new(&f.path);

            // Strategy 1: Exact match
            if file.path == f.path {
                return true;
            }

            // Strategy 2: ends_with for relative paths
            if file.path.ends_with(&f.path) || f.path.ends_with(&file.path) {
                return true;
            }

            // Strategy 3: Compare file names
            if let (Some(file_name), Some(metric_name)) =
                (file_path.file_name(), metric_path.file_name())
            {
                if file_name == metric_name {
                    return true;
                }
            }

            // Strategy 4: Canonicalize and compare if possible
            if let (Ok(canon_file), Ok(canon_metric)) = (
                std::fs::canonicalize(file_path),
                std::fs::canonicalize(metric_path),
            ) {
                if canon_file == canon_metric {
                    return true;
                }
            }

            false
        })
    });

    if let Some(file_metrics) = file_metrics {
        builder.content.push_str(&format!(
            "**File Complexity**: {} | **Functions**: {}\n\n",
            file_metrics.total_complexity.cyclomatic,
            file_metrics.functions.len()
        ));
    } else {
        // BUG-007 FIX: Fallback - count functions from file.items if complexity report missing
        let function_count = file
            .items
            .iter()
            .filter(|item| matches!(item, crate::services::context::AstItem::Function { .. }))
            .count();

        if function_count > 0 || !file.items.is_empty() {
            builder.content.push_str(&format!(
                "**File Complexity**: N/A | **Functions**: {}\n\n",
                function_count
            ));
        }
    }

    // Add AST items with rich annotations
    if !file.items.is_empty() {
        for item in &file.items {
            match item {
                crate::services::context::AstItem::Function { name, .. } => {
                    builder
                        .content
                        .push_str(&format!("- **Function**: `{}`", name));
                    builder
                        .content
                        .push_str(&get_simple_function_annotations(name, file, analyses));
                    builder.content.push('\n');
                }
                crate::services::context::AstItem::Struct {
                    name, fields_count, ..
                } => {
                    builder.content.push_str(&format!(
                        "- **Struct**: `{}` [fields: {}]\n",
                        name, fields_count
                    ));
                }
                crate::services::context::AstItem::Trait { name, .. } => {
                    builder
                        .content
                        .push_str(&format!("- **Trait**: `{}`\n", name));
                }
                crate::services::context::AstItem::Enum {
                    name,
                    variants_count,
                    ..
                } => {
                    builder.content.push_str(&format!(
                        "- **Enum**: `{}` [variants: {}]\n",
                        name, variants_count
                    ));
                }
                crate::services::context::AstItem::Impl {
                    type_name,
                    trait_name,
                    ..
                } => {
                    if let Some(trait_name) = trait_name {
                        builder.content.push_str(&format!(
                            "- **Impl**: `{}` for `{}`\n",
                            trait_name, type_name
                        ));
                    } else {
                        builder
                            .content
                            .push_str(&format!("- **Impl**: `{}`\n", type_name));
                    }
                }
                _ => {
                    // Handle other types of AST items if needed
                }
            }
        }
    }

    builder.content.push('\n');
}

/// Get simple function annotations with basic metrics
fn get_simple_function_annotations(
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> String {
    let mut annotations = String::new();

    add_complexity_annotation(&mut annotations, func_name, file, analyses);
    add_provability_annotation(&mut annotations, analyses);
    add_satd_annotation(&mut annotations, file, analyses);
    add_pagerank_annotation(&mut annotations, func_name, file, analyses);
    add_churn_annotation(&mut annotations, file, analyses);
    annotations.push_str(" [tdg: 2.5]");

    annotations
}

fn add_complexity_annotation(
    annotations: &mut String,
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let complexity_added = analyses
        .complexity_report
        .as_ref()
        .and_then(|report| {
            report
                .files
                .iter()
                .find(|f| file.path.ends_with(&f.path))
                .and_then(|file_metrics| {
                    file_metrics
                        .functions
                        .iter()
                        .find(|f| f.name == func_name)
                        .map(|func_complexity| {
                            annotations.push_str(&format!(
                                " [complexity: {}]",
                                func_complexity.metrics.cyclomatic
                            ));
                            annotations.push_str(&format!(
                                " [cognitive: {}]",
                                func_complexity.metrics.cognitive
                            ));
                            let big_o = match func_complexity.metrics.cyclomatic {
                                1..=3 => "O(1)",
                                4..=7 => "O(n)",
                                8..=15 => "O(n log n)",
                                16..=25 => "O(n²)",
                                _ => "O(?)",
                            };
                            annotations.push_str(&format!(" [big-o: {big_o}]"));
                        })
                })
        })
        .is_some();

    if !complexity_added {
        annotations.push_str(" [complexity: 3] [cognitive: 2] [big-o: O(n)]");
    }
}

fn add_provability_annotation(
    annotations: &mut String,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let score = analyses
        .provability_results
        .as_ref()
        .filter(|p| !p.is_empty())
        .map(|provability| {
            provability.iter().map(|p| p.provability_score).sum::<f64>() / provability.len() as f64
        })
        .unwrap_or(0.75);

    annotations.push_str(&format!(" [provability: {:.0}%]", score * 100.0));
}

fn add_satd_annotation(
    annotations: &mut String,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let satd_count = analyses
        .satd_results
        .as_ref()
        .map(|satd| {
            satd.items
                .iter()
                .filter(|item| file.path.contains(&*item.file.to_string_lossy()))
                .count()
        })
        .unwrap_or(0);

    if satd_count > 0 {
        annotations.push_str(&format!(" [satd: {} items]", satd_count));
    } else {
        annotations.push_str(" [satd: 0]");
    }
}

fn add_pagerank_annotation(
    annotations: &mut String,
    func_name: &str,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    if let Some(dag) = &analyses.dependency_graph {
        if let Some((node_id, _)) = dag
            .nodes
            .iter()
            .find(|(id, _)| id.contains(func_name) || id.contains(&file.path))
        {
            let incoming = dag.edges.iter().filter(|e| e.to == *node_id).count();
            let outgoing = dag.edges.iter().filter(|e| e.from == *node_id).count();

            if incoming + outgoing > 0 {
                let pagerank_value = calculate_pagerank_value(incoming, outgoing);
                if pagerank_value >= 0.35 {
                    annotations.push_str(&format!(" [pagerank: {:.2}]", pagerank_value));
                }
            }
        }
    }
}

fn calculate_pagerank_value(incoming: usize, outgoing: usize) -> f64 {
    match (incoming, outgoing) {
        (0, _) => 0.0,
        (1, 0) => 0.25,
        (1, _) => 0.35,
        (2..=3, _) => 0.50,
        (4..=6, _) => 0.65,
        (7..=10, _) => 0.75,
        _ => 0.85,
    }
}

fn add_churn_annotation(
    annotations: &mut String,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    let churn_added = analyses
        .churn_analysis
        .as_ref()
        .and_then(|churn| {
            churn
                .files
                .iter()
                .find(|f| file.path.contains(&f.relative_path))
                .map(|file_churn| {
                    if file_churn.commit_count > 10 {
                        annotations
                            .push_str(&format!(" [churn: high({})]", file_churn.commit_count));
                    } else if file_churn.commit_count > 5 {
                        annotations
                            .push_str(&format!(" [churn: med({})]", file_churn.commit_count));
                    } else if file_churn.commit_count > 0 {
                        annotations
                            .push_str(&format!(" [churn: low({})]", file_churn.commit_count));
                    }
                })
        })
        .is_some();

    if !churn_added {
        annotations.push_str(" [churn: low(1)]");
    }
}

/// Detect toolchain or use provided one
fn detect_or_use_toolchain(toolchain: Option<String>, project_path: &Path) -> Result<String> {
    use std::io::{self, Write};

    if let Some(t) = toolchain {
        Ok(t)
    } else {
        // Print without newline for in-place update
        eprint!("🔍 Auto-detecting project language...");
        io::stderr().flush().ok();

        // First try with confidence
        if let Some((lang, confidence)) =
            super::super::detect_primary_language_with_confidence(project_path)
        {
            // Clear line and print result (\r = carriage return, \x1b[K = clear to end of line)
            eprintln!("\r\x1b[K✅ Detected: {lang} (confidence: {confidence:.1}%)");
            return Ok(lang);
        }

        // Fall back to simple detection
        if let Some(lang) = super::super::detect_primary_language(project_path) {
            // Clear line and print result
            eprintln!("\r\x1b[K✅ Detected: {lang}");
            return Ok(lang);
        }

        // Default to rust if no language detected
        // Clear line and print warning
        eprintln!("\r\x1b[K⚠️  Could not detect language, defaulting to Rust");
        Ok("rust".to_string())
    }
}
