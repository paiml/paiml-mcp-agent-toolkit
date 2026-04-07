// DeepContextAnalyzer formatting methods - extracted for file health (CB-040)
// Split into semantic submodules for maintainability
//
// The DeepContextAnalyzer struct is defined in the parent module (deep_context/mod.rs)
// and these submodules add formatting impl blocks to it.

mod analysis_sections;
mod ast_formatting;
mod indicators;
mod legacy;
mod markdown;
mod sarif;

use super::{
    AnalysisResults, AnnotatedFileTree, AnnotatedNode, CrossLangReference, DeepContext,
    DeepContextAnalyzer, DefectHotspot, DefectSummary, EnhancedFileContext, NodeAnnotations,
    NodeType, PrioritizedRecommendation, Priority, QualityScorecard,
};

impl DeepContextAnalyzer {
    /// Format as comprehensive markdown output using simple formatting
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn format_as_comprehensive_markdown(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<String> {
        let mut output = String::with_capacity(1024);
        output.push_str("# Deep Context Analysis Report\n\n");

        self.append_project_overview(&mut output, &context.project_overview)?;
        self.append_build_info(&mut output, &context.build_info)?;
        self.append_quality_scorecard(&mut output, &context.quality_scorecard)?;
        self.append_project_structure(&mut output, &context.file_tree)?;
        self.append_analysis_results(&mut output, &context.analyses)?;
        self.append_recommendations(&mut output, &context.recommendations)?;

        Ok(output)
    }

    /// Format as JSON output for machine consumption and API responses
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn format_as_json(&self, context: &DeepContext) -> anyhow::Result<String> {
        serde_json::to_string_pretty(context)
            .map_err(|e| anyhow::anyhow!("Failed to serialize to JSON: {e}"))
    }

    fn overall_health_emoji(&self, health: f64) -> &'static str {
        if health >= 80.0 {
            "\u{2705}"
        } else if health >= 60.0 {
            "\u{26a0}\u{fe0f}"
        } else {
            "\u{274c}"
        }
    }
}
