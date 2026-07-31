// Comprehensive markdown formatting methods for DeepContextAnalyzer

use super::{
    AnalysisResults, AnnotatedFileTree, DeepContextAnalyzer, PrioritizedRecommendation,
    QualityScorecard,
};

impl DeepContextAnalyzer {
    pub(super) fn append_project_overview(
        &self,
        output: &mut String,
        overview: &Option<crate::models::project_meta::ProjectOverview>,
    ) -> anyhow::Result<()> {
        if let Some(ref overview) = overview {
            output.push_str("## Project Overview\n\n");
            if !overview.compressed_description.is_empty() {
                output.push_str(&overview.compressed_description);
                output.push_str("\n\n");
            }
            if !overview.key_features.is_empty() {
                output.push_str("**Key Features:**\n");
                for feature in &overview.key_features {
                    output.push_str(&format!("- {feature}\n"));
                }
                output.push('\n');
            }
            if let Some(ref arch) = overview.architecture_summary {
                output.push_str("**Architecture:**\n");
                output.push_str(arch);
                output.push_str("\n\n");
            }
        }
        Ok(())
    }

    pub(super) fn append_build_info(
        &self,
        output: &mut String,
        build_info: &Option<crate::models::project_meta::BuildInfo>,
    ) -> anyhow::Result<()> {
        if let Some(ref build_info) = build_info {
            output.push_str("## Build System\n\n");
            output.push_str(&format!(
                "**Detected Toolchain:** {}\n",
                build_info.toolchain
            ));
            if !build_info.targets.is_empty() {
                output.push_str(&format!(
                    "**Primary Targets:** {}\n",
                    build_info.targets.join(", ")
                ));
            }
            if !build_info.dependencies.is_empty() {
                output.push_str(&format!(
                    "**Key Dependencies:** {}\n",
                    build_info.dependencies.join(", ")
                ));
            }
            if let Some(ref cmd) = build_info.primary_command {
                output.push_str(&format!("**Build Command:** `{cmd}`\n"));
            }
            output.push('\n');
        }
        Ok(())
    }

    pub(super) fn append_quality_scorecard(
        &self,
        output: &mut String,
        scorecard: &QualityScorecard,
    ) -> anyhow::Result<()> {
        output.push_str("## Quality Scorecard\n\n");
        output.push_str(&format!(
            "- Overall Health: {}\n",
            QualityScorecard::render(scorecard.overall_health, "%")
        ));
        output.push_str(&format!(
            "- Maintainability Index: {}\n",
            QualityScorecard::render(scorecard.maintainability_index, "%")
        ));
        output.push_str(&format!(
            "- Refactoring Time: {}\n",
            QualityScorecard::render(scorecard.technical_debt_hours, " hours")
        ));
        output.push_str(&format!(
            "- Complexity Score: {}\n",
            QualityScorecard::render(scorecard.complexity_score, "%")
        ));
        output.push('\n');
        Ok(())
    }

    pub(super) fn append_project_structure(
        &self,
        output: &mut String,
        file_tree: &AnnotatedFileTree,
    ) -> anyhow::Result<()> {
        output.push_str("## Project Structure\n\n");
        output.push_str("```\n");
        output.push_str(&format!(
            "Total Files: {}\nTotal Size: {} bytes\n",
            file_tree.total_files, file_tree.total_size_bytes
        ));
        output.push_str("\n```\n\n");
        Ok(())
    }

    pub(super) fn append_analysis_results(
        &self,
        output: &mut String,
        analyses: &AnalysisResults,
    ) -> anyhow::Result<()> {
        output.push_str("## Analysis Results\n\n");

        if !analyses.ast_contexts.is_empty() {
            output.push_str(&format!(
                "### AST Analysis\n- Files analyzed: {}\n\n",
                analyses.ast_contexts.len()
            ));
        }

        if let Some(ref complexity) = analyses.complexity_report {
            output.push_str(&format!("### Complexity Analysis\n- Total files: {}\n- Total functions: {}\n- Median cyclomatic complexity: {:.1}\n\n",
                complexity.summary.total_files, complexity.summary.total_functions, complexity.summary.median_cyclomatic));
        }

        if let Some(ref churn) = analyses.churn_analysis {
            output.push_str(&format!(
                "### Code Churn\n- Files analyzed: {}\n- Total commits: {}\n\n",
                churn.files.len(),
                churn.summary.total_commits
            ));
        }
        Ok(())
    }

    pub(super) fn append_recommendations(
        &self,
        output: &mut String,
        recommendations: &[PrioritizedRecommendation],
    ) -> anyhow::Result<()> {
        if !recommendations.is_empty() {
            output.push_str("## Recommendations\n\n");
            for (i, rec) in recommendations.iter().enumerate() {
                output.push_str(&format!(
                    "{}. **{}** (Priority: {:?})\n   {}\n   Effort: {:?}\n\n",
                    i + 1,
                    rec.title,
                    rec.priority,
                    rec.description,
                    rec.estimated_effort
                ));
            }
        }
        Ok(())
    }
}
