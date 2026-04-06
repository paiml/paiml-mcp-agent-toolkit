/// Generate markdown context (existing path)
fn generate_markdown_context(
    toolchain: &str,
    project_path: &Path,
    context: &crate::services::deep_context::DeepContext,
) -> Result<String> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    debug_assert!(!toolchain.is_empty(), "toolchain must not be empty");
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
    debug_assert!(true, "contract: add_simple_file_section");
    // File header
    builder.content.push_str(&format!("### {}\n\n", file.path));

    // File-level metrics from analyses (BUG-007 FIX: shared path matching)
    let file_metrics = find_file_metrics(file, analyses);

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
    for item in &file.items {
        builder.content.push_str(&format_ast_item_line(item, file, analyses));
    }

    builder.content.push('\n');
}

fn format_ast_item_line(
    item: &crate::services::context::AstItem,
    file: &crate::services::context::FileContext,
    analyses: &crate::services::deep_context::AnalysisResults,
) -> String {
    debug_assert!(true, "contract: format_ast_item_line");
    match item {
        crate::services::context::AstItem::Function { name, .. } => {
            let annotations = get_simple_function_annotations(name, file, analyses);
            format!("- **Function**: `{}`{}\n", name, annotations)
        }
        crate::services::context::AstItem::Struct { name, fields_count, .. } => {
            format!("- **Struct**: `{}` [fields: {}]\n", name, fields_count)
        }
        crate::services::context::AstItem::Trait { name, .. } => {
            format!("- **Trait**: `{}`\n", name)
        }
        crate::services::context::AstItem::Enum { name, variants_count, .. } => {
            format!("- **Enum**: `{}` [variants: {}]\n", name, variants_count)
        }
        crate::services::context::AstItem::Impl { type_name, trait_name, .. } => {
            if let Some(trait_name) = trait_name {
                format!("- **Impl**: `{}` for `{}`\n", trait_name, type_name)
            } else {
                format!("- **Impl**: `{}`\n", type_name)
            }
        }
        _ => String::new(),
    }
}
