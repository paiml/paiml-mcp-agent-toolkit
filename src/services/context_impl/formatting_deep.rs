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
    use crate::services::deep_context::QualityScorecard as QS;
    output.push_str(&format!(
        "- **Overall Health**: {}\n",
        QS::render(scorecard.overall_health, "%")
    ));
    output.push_str(&format!(
        "- **Complexity Score**: {}\n",
        QS::render(scorecard.complexity_score, "%")
    ));
    output.push_str(&format!(
        "- **Maintainability Index**: {}\n",
        QS::render(scorecard.maintainability_index, "%")
    ));
    output.push_str(&format!(
        "- **Modularity Score**: {}\n",
        QS::render(scorecard.modularity_score, "%")
    ));
    // Only rendered when a coverage artifact was actually found.
    if let Some(coverage) = scorecard.test_coverage {
        output.push_str(&format!("- **Test Coverage**: {coverage:.1}%\n"));
    }
    output.push_str(&format!(
        "- **Refactoring Estimate**: {}\n\n",
        QS::render(scorecard.technical_debt_hours, " hours")
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
