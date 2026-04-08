#![cfg_attr(coverage_nightly, coverage(off))]
//! Markdown formatting functions for context output.
//!
//! This module provides functions to format project context and deep context
//! analysis results as human-readable markdown.

use super::types::{AstItem, FileContext, GroupedItems, ProjectContext, ProjectSummary};
use crate::services::deep_context::DeepContext;

#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
/// Format context as markdown.
pub fn format_context_as_markdown(context: &ProjectContext) -> String {
    let mut output = String::new();

    format_header(&mut output, context);
    format_summary(&mut output, &context.summary);
    format_dependencies(&mut output, &context.summary.dependencies);
    format_files(&mut output, &context.files);
    format_footer(&mut output);

    output
}

fn format_header(output: &mut String, context: &ProjectContext) {
    output.push_str(&format!(
        "# Project Context: {} Project\n\n",
        context.project_type
    ));
    output.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
}

fn format_summary(output: &mut String, summary: &ProjectSummary) {
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Files analyzed: {}\n", summary.total_files));
    output.push_str(&format!("- Functions: {}\n", summary.total_functions));
    output.push_str(&format!("- Structs: {}\n", summary.total_structs));
    output.push_str(&format!("- Enums: {}\n", summary.total_enums));
    output.push_str(&format!("- Traits: {}\n", summary.total_traits));
    output.push_str(&format!("- Implementations: {}\n", summary.total_impls));
}

fn format_dependencies(output: &mut String, dependencies: &[String]) {
    if !dependencies.is_empty() {
        output.push_str("\n## Dependencies\n\n");
        for dep in dependencies {
            output.push_str(&format!("- {dep}\n"));
        }
    }
}

fn format_files(output: &mut String, files: &[FileContext]) {
    output.push_str("\n## Files\n\n");

    for file in files {
        output.push_str(&format!("### {}\n\n", file.path));

        let grouped_items = group_items_by_type(&file.items);
        format_item_groups(output, &grouped_items);
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn group_items_by_type(items: &[AstItem]) -> GroupedItems<'_> {
    let mut grouped = GroupedItems::new();

    for item in items {
        match item {
            AstItem::Function { .. } => grouped.functions.push(item),
            AstItem::Struct { .. } => grouped.structs.push(item),
            AstItem::Enum { .. } => grouped.enums.push(item),
            AstItem::Trait { .. } => grouped.traits.push(item),
            AstItem::Impl { .. } => grouped.impls.push(item),
            AstItem::Module { .. } => grouped.modules.push(item),
            _ => {}
        }
    }

    grouped
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_item_groups(output: &mut String, groups: &GroupedItems) {
    format_item_group(output, "Modules", &groups.modules, format_module_item);
    format_item_group(output, "Structs", &groups.structs, format_struct_item);
    format_item_group(output, "Enums", &groups.enums, format_enum_item);
    format_item_group(output, "Traits", &groups.traits, format_trait_item);
    format_item_group(output, "Functions", &groups.functions, format_function_item);
    format_item_group(output, "Implementations", &groups.impls, format_impl_item);
}

fn format_item_group<F>(output: &mut String, title: &str, items: &[&AstItem], formatter: F)
where
    F: Fn(&AstItem) -> String,
{
    if !items.is_empty() {
        output.push_str(&format!("**{title}:**\n"));
        for item in items {
            output.push_str(&format!("{}\n", formatter(item)));
        }
        output.push('\n');
    }
}

fn format_module_item(item: &AstItem) -> String {
    if let AstItem::Module {
        name,
        visibility,
        line,
    } = item
    {
        format!("- `{visibility} mod {name}` (line {line})")
    } else {
        String::new()
    }
}

fn format_struct_item(item: &AstItem) -> String {
    if let AstItem::Struct {
        name,
        visibility,
        fields_count,
        derives,
        line,
    } = item
    {
        let mut result = format!("- `{visibility} struct {name}` ({fields_count} fields)");
        if !derives.is_empty() {
            result.push_str(&format!(" [derives: {}]", derives.join(", ")));
        }
        result.push_str(&format!(" (line {line})"));
        result
    } else {
        String::new()
    }
}

fn format_enum_item(item: &AstItem) -> String {
    if let AstItem::Enum {
        name,
        visibility,
        variants_count,
        line,
    } = item
    {
        format!("- `{visibility} enum {name}` ({variants_count} variants) (line {line})")
    } else {
        String::new()
    }
}

fn format_trait_item(item: &AstItem) -> String {
    if let AstItem::Trait {
        name,
        visibility,
        line,
    } = item
    {
        format!("- `{visibility} trait {name}` (line {line})")
    } else {
        String::new()
    }
}

fn format_function_item(item: &AstItem) -> String {
    if let AstItem::Function {
        name,
        visibility,
        is_async,
        line,
    } = item
    {
        format!(
            "- `{} {}fn {}` (line {})",
            visibility,
            if *is_async { "async " } else { "" },
            name,
            line
        )
    } else {
        String::new()
    }
}

fn format_impl_item(item: &AstItem) -> String {
    if let AstItem::Impl {
        type_name,
        trait_name,
        line,
    } = item
    {
        match trait_name {
            Some(trait_name) => {
                format!("- `impl {trait_name} for {type_name}` (line {line})")
            }
            None => format!("- `impl {type_name}` (line {line})"),
        }
    } else {
        String::new()
    }
}

fn format_footer(output: &mut String) {
    output.push_str("---\n");
    output.push_str("Generated by paiml-mcp-agent-toolkit\n");
}

/// Format a comprehensive `DeepContext` as markdown with quality metrics
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_deep_context_as_markdown(context: &DeepContext) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "# Deep Project Context\n\nGenerated: {}\nTool Version: {}\n\n",
        context
            .metadata
            .generated_at
            .format("%Y-%m-%d %H:%M:%S UTC"),
        context.metadata.tool_version
    ));

    // Quality Scorecard
    format_quality_scorecard(&mut output, &context.quality_scorecard);

    // Project Summary
    format_project_summary(&mut output, context);

    // Analysis Results
    format_analysis_results(&mut output, &context.analyses);

    // AST Summary
    format_ast_summary(&mut output, &context.analyses.ast_contexts);

    output
}

fn format_quality_scorecard(
    output: &mut String,
    scorecard: &crate::services::deep_context::QualityScorecard,
) {
    output.push_str("## Quality Scorecard\n\n");
    output.push_str(&format!(
        "- **Overall Health**: {:.1}%\n",
        scorecard.overall_health
    ));
    output.push_str(&format!(
        "- **Complexity Score**: {:.1}%\n",
        scorecard.complexity_score
    ));
    output.push_str(&format!(
        "- **Maintainability Index**: {:.1}%\n",
        scorecard.maintainability_index
    ));
    output.push_str(&format!(
        "- **Modularity Score**: {:.1}%\n",
        scorecard.modularity_score
    ));
    if let Some(coverage) = scorecard.test_coverage {
        output.push_str(&format!("- **Test Coverage**: {coverage:.1}%\n"));
    }
    output.push_str(&format!(
        "- **Refactoring Estimate**: {:.1} hours\n\n",
        scorecard.technical_debt_hours
    ));
}

fn format_project_summary(output: &mut String, context: &DeepContext) {
    output.push_str("## Project Summary\n\n");
    output.push_str(&format!(
        "- **Total Files**: {}\n",
        context.file_tree.total_files
    ));
    output.push_str(&format!(
        "- **Total Size**: {} bytes\n",
        context.file_tree.total_size_bytes
    ));
    output.push_str(&format!(
        "- **AST Contexts**: {}\n",
        context.analyses.ast_contexts.len()
    ));

    // Count various AST items
    let (functions, structs, enums, traits, impls) =
        count_ast_items(&context.analyses.ast_contexts);
    output.push_str(&format!("- **Functions**: {functions}\n"));
    output.push_str(&format!("- **Structs**: {structs}\n"));
    output.push_str(&format!("- **Enums**: {enums}\n"));
    output.push_str(&format!("- **Traits**: {traits}\n"));
    output.push_str(&format!("- **Implementations**: {impls}\n\n"));
}

fn format_analysis_results(
    output: &mut String,
    analyses: &crate::services::deep_context::AnalysisResults,
) {
    output.push_str("## Analysis Results\n\n");

    // Complexity Analysis - Combined formatting to reduce complexity
    if let Some(ref complexity) = analyses.complexity_report {
        output.push_str(&format!(
            "### Complexity Metrics\n\n\
            - **Total Files Analyzed**: {}\n\
            - **Median Cyclomatic Complexity**: {:.1}\n\
            - **Max Cyclomatic Complexity**: {}\n\
            - **Median Cognitive Complexity**: {:.1}\n\
            - **Max Cognitive Complexity**: {}\n\
            - **Refactoring Hours**: {:.1}\n\n",
            complexity.files.len(),
            complexity.summary.median_cyclomatic,
            complexity.summary.max_cyclomatic,
            complexity.summary.median_cognitive,
            complexity.summary.max_cognitive,
            complexity.summary.technical_debt_hours
        ));
    }

    // Churn Analysis - Combined formatting to reduce complexity
    if let Some(ref churn) = analyses.churn_analysis {
        let hotspots = if churn.summary.hotspot_files.is_empty() {
            String::new()
        } else {
            let mut hotspots_str = "- **Top Hotspots**:\n".to_string();
            for (i, hotspot) in churn.summary.hotspot_files.iter().take(5).enumerate() {
                hotspots_str.push_str(&format!("  {}. {}\n", i + 1, hotspot.display()));
            }
            hotspots_str
        };

        output.push_str(&format!(
            "### Code Churn Analysis\n\n\
            - **Analysis Period**: {} days\n\
            - **Total Files Changed**: {}\n\
            - **Total Commits**: {}\n\
            - **Hotspot Files**: {}\n{}\n",
            churn.period_days,
            churn.summary.total_files_changed,
            churn.summary.total_commits,
            churn.summary.hotspot_files.len(),
            hotspots
        ));
    }

    // Dependency Graph - Combined formatting to reduce complexity
    if let Some(ref dag) = analyses.dependency_graph {
        output.push_str(&format!(
            "### Dependency Graph Statistics\n\n\
            - **Total Nodes**: {}\n\
            - **Total Edges**: {}\n\
            - **Graph Analysis**: Dependency relationships analyzed\n\n",
            dag.nodes.len(),
            dag.edges.len()
        ));
    }

    // Dead Code Analysis - Combined formatting to reduce complexity
    if let Some(ref dead_code) = analyses.dead_code_results {
        output.push_str(&format!(
            "### Dead Code Analysis\n\n\
            - **Total Files Analyzed**: {}\n\
            - **Dead Functions Found**: {}\n\
            - **Dead Classes Found**: {}\n\
            - **Dead Lines**: {}\n\n",
            dead_code.summary.total_files_analyzed,
            dead_code.summary.dead_functions,
            dead_code.summary.dead_classes,
            dead_code.summary.total_dead_lines
        ));
    }

    // SATD Analysis - Combined formatting to reduce complexity
    if let Some(ref satd) = analyses.satd_results {
        output.push_str(&format!(
            "### Self-Admitted Debt Analysis\n\n\
            - **Total SATD Items**: {}\n\
            - **Categories**: Various debt types detected\n\n",
            satd.items.len()
        ));
    }
}

fn format_ast_summary(
    output: &mut String,
    ast_contexts: &[crate::services::deep_context::EnhancedFileContext],
) {
    if ast_contexts.is_empty() {
        return;
    }

    output.push_str("## AST Analysis\n\n");

    for enhanced_context in ast_contexts.iter().take(20) {
        // Limit to top 20 files
        let file_context = &enhanced_context.base;
        output.push_str(&format!("### {}\n\n", file_context.path));

        let grouped_items = group_items_by_type(&file_context.items);
        format_item_groups(output, &grouped_items);
    }
}

fn count_ast_items(
    ast_contexts: &[crate::services::deep_context::EnhancedFileContext],
) -> (usize, usize, usize, usize, usize) {
    let mut functions = 0;
    let mut structs = 0;
    let mut enums = 0;
    let mut traits = 0;
    let mut impls = 0;

    for enhanced_context in ast_contexts {
        for item in &enhanced_context.base.items {
            match item {
                AstItem::Function { .. } => functions += 1,
                AstItem::Struct { .. } => structs += 1,
                AstItem::Enum { .. } => enums += 1,
                AstItem::Trait { .. } => traits += 1,
                AstItem::Impl { .. } => impls += 1,
                _ => {}
            }
        }
    }

    (functions, structs, enums, traits, impls)
}
