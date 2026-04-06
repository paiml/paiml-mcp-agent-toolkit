// Legacy markdown formatting methods for DeepContextAnalyzer

use super::{
    AnnotatedFileTree, AnnotatedNode, DeepContext, DeepContextAnalyzer, EnhancedFileContext,
    NodeType, QualityScorecard,
};
use crate::models::tdg::TDGSeverity;

impl DeepContextAnalyzer {
    /// Legacy format method (kept for backward compatibility)
    pub fn format_as_comprehensive_markdown_legacy(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<String> {
        let mut output = String::with_capacity(1024);

        // Step 1: Format header and metadata
        self.format_legacy_header(&mut output, context)?;

        // Step 2: Format main content sections
        self.format_legacy_main_sections(&mut output, context)?;

        // Step 3: Format analysis sections
        self.format_legacy_analysis_sections(&mut output, context)?;

        Ok(output)
    }

    /// Format header and metadata for legacy markdown
    fn format_legacy_header(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let project_name = context
            .metadata
            .project_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        writeln!(output, "# Deep Context: {project_name}")?;
        writeln!(output, "Generated: {}", context.metadata.generated_at)?;
        writeln!(output, "Version: {}", context.metadata.tool_version)?;
        writeln!(
            output,
            "Analysis Time: {:.2}s",
            context.metadata.analysis_duration.as_secs_f64()
        )?;
        writeln!(
            output,
            "Cache Hit Rate: {:.1}%",
            context.metadata.cache_stats.hit_rate * 100.0
        )?;

        Ok(())
    }

    /// Format main content sections for legacy markdown
    fn format_legacy_main_sections(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        self.write_quality_scorecard_section(output, &context.quality_scorecard)?;
        self.write_project_structure_section(output, &context.file_tree)?;
        self.write_ast_section_if_present(output, &context.analyses.ast_contexts)?;
        Ok(())
    }

    fn write_quality_scorecard_section(
        &self,
        output: &mut String,
        scorecard: &QualityScorecard,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "\n## Quality Scorecard\n")?;
        writeln!(
            output,
            "- **Overall Health**: {} ({:.1}/100)",
            self.overall_health_emoji(scorecard.overall_health),
            scorecard.overall_health
        )?;
        writeln!(
            output,
            "- **Maintainability Index**: {:.1}",
            scorecard.maintainability_index
        )?;
        writeln!(
            output,
            "- **Refactoring Time**: {:.1} hours estimated",
            scorecard.technical_debt_hours
        )?;
        Ok(())
    }

    fn write_project_structure_section(
        &self,
        output: &mut String,
        file_tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "\n## Project Structure\n")?;
        writeln!(output, "```")?;
        self.format_annotated_tree(output, file_tree)?;
        writeln!(output, "```\n")?;
        Ok(())
    }

    fn write_ast_section_if_present(
        &self,
        output: &mut String,
        ast_contexts: &[EnhancedFileContext],
    ) -> anyhow::Result<()> {
        debug_assert!(!ast_contexts.is_empty(), "ast_contexts must not be empty");
        if !ast_contexts.is_empty() {
            self.format_enhanced_ast_section(output, ast_contexts)?;
        }
        Ok(())
    }

    /// Format analysis sections for legacy markdown
    fn format_legacy_analysis_sections(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        // Code quality metrics
        self.format_complexity_hotspots(output, context)?;
        self.format_churn_analysis(output, context)?;
        self.format_technical_debt(output, context)?;
        self.format_dead_code_analysis(output, context)?;

        // Cross-language references
        self.format_cross_references(output, &context.analyses.cross_language_refs)?;

        // Defect probability analysis
        self.format_defect_predictions(output, context)?;

        // Actionable recommendations
        self.format_prioritized_recommendations(output, &context.recommendations)?;

        Ok(())
    }

    pub(crate) fn format_annotated_tree(
        &self,
        output: &mut String,
        tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        self.format_tree_node(output, &tree.root, "", true)?;
        writeln!(
            output,
            "\n\u{1f4ca} Total Files: {}, Total Size: {} bytes",
            tree.total_files, tree.total_size_bytes
        )?;
        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn format_tree_node(
        &self,
        output: &mut String,
        node: &AnnotatedNode,
        prefix: &str,
        is_last: bool,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        let connector = if is_last {
            "\u{2514}\u{2500}\u{2500} "
        } else {
            "\u{251c}\u{2500}\u{2500} "
        };
        let extension = if is_last { "    " } else { "\u{2502}   " };

        let node_display = self.format_node_display(node)?;
        writeln!(output, "{prefix}{connector}{node_display}")?;

        // Process children
        let child_prefix = format!("{prefix}{extension}");
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            self.format_tree_node(output, child, &child_prefix, is_last_child)?;
        }

        Ok(())
    }

    fn format_node_display(&self, node: &AnnotatedNode) -> anyhow::Result<String> {
        let mut display = node.name.clone();

        if matches!(node.node_type, NodeType::Directory) {
            display.push('/');
        }

        let annotations = self.collect_node_annotations(&node.annotations);
        if !annotations.is_empty() {
            display.push_str(&format!(" [{}]", annotations.join(" ")));
        }

        Ok(display)
    }

    pub(crate) fn write_file_metrics(
        &self,
        output: &mut String,
        context: &EnhancedFileContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        // Complexity metrics if available
        if let Some(ref complexity) = context.complexity_metrics {
            writeln!(output, "\n**Complexity Metrics:**")?;
            writeln!(
                output,
                "  - Cyclomatic: {:.1} | Cognitive: {:.1} | Lines: {}",
                complexity.total_complexity.cyclomatic,
                complexity.total_complexity.cognitive,
                complexity.total_complexity.lines
            )?;
        }

        // Churn metrics if available
        if let Some(ref churn) = context.churn_metrics {
            writeln!(output, "\n**Code Churn:**")?;
            writeln!(
                output,
                "  - {} commits by {} authors",
                churn.commits, churn.authors
            )?;
        }

        // TDG Score
        if let Some(ref tdg) = context.defects.tdg_score {
            writeln!(output, "\n**Code Quality Gradient:** {:.2}\n", tdg.value)?;
            writeln!(
                output,
                "**TDG Severity:** {:?}\n",
                TDGSeverity::from(tdg.value)
            )?;
        }

        Ok(())
    }
}
