// Analysis section formatting: complexity hotspots, churn, debt, dead code,
// cross references, defect predictions, and recommendations

use super::{
    CrossLangReference, DeepContext, DeepContextAnalyzer, DefectHotspot, DefectSummary,
    PrioritizedRecommendation, Priority,
};
use crate::models::churn::CodeChurnAnalysis;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

impl DeepContextAnalyzer {
    pub(crate) fn format_complexity_hotspots(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if let Some(ref complexity) = context.analyses.complexity_report {
            writeln!(output, "## Complexity Hotspots\n")?;

            // Find top 10 most complex functions
            let mut all_functions: Vec<_> = complexity
                .files
                .par_iter()
                .flat_map(|f| f.functions.par_iter().map(move |func| (f, func)))
                .collect();
            all_functions.sort_by_key(|(_, func)| std::cmp::Reverse(func.metrics.cyclomatic));

            writeln!(output, "| Function | File | Cyclomatic | Cognitive |")?;
            writeln!(output, "|----------|------|------------|-----------|")?;

            for (file, func) in all_functions.iter().take(10) {
                writeln!(
                    output,
                    "| `{}` | `{}` | {} | {} |",
                    func.name, file.path, func.metrics.cyclomatic, func.metrics.cognitive
                )?;
            }
            writeln!(output)?;
        }

        Ok(())
    }

    pub(crate) fn format_churn_analysis(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref churn) = context.analyses.churn_analysis {
            self.write_churn_header(output)?;
            self.write_churn_summary(output, churn)?;
            self.write_churn_files_table(output, &churn.files)?;
        }
        Ok(())
    }

    fn write_churn_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Code Churn Analysis\n")?;
        Ok(())
    }

    fn write_churn_summary(
        &self,
        output: &mut String,
        churn: &CodeChurnAnalysis,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Summary:**")?;
        writeln!(output, "- Total Commits: {}", churn.summary.total_commits)?;
        writeln!(output, "- Files Changed: {}", churn.files.len())?;
        Ok(())
    }

    fn write_churn_files_table(
        &self,
        output: &mut String,
        files: &[crate::models::churn::FileChurnMetrics],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;

        // Sort files by commit count
        let mut sorted_files = files.to_vec();
        sorted_files.sort_by_key(|f| std::cmp::Reverse(f.commit_count));

        writeln!(output, "\n**Top Changed Files:**")?;
        writeln!(output, "| File | Commits | Authors |")?;
        writeln!(output, "|------|---------|---------|")?;

        for file in sorted_files.iter().take(10) {
            writeln!(
                output,
                "| `{}` | {} | {} |",
                file.relative_path,
                file.commit_count,
                file.unique_authors.len()
            )?;
        }
        writeln!(output)?;
        Ok(())
    }

    pub(crate) fn format_technical_debt(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref satd) = context.analyses.satd_results {
            use std::fmt::Write;
            writeln!(output, "## Code Quality Analysis\n")?;
            self.write_satd_severity_summary(output, satd)?;
            self.write_critical_items(output, satd)?;
            writeln!(output)?;
        }
        Ok(())
    }

    fn write_satd_severity_summary(
        &self,
        output: &mut String,
        satd: &crate::services::satd_detector::SATDAnalysisResult,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        let by_severity = self.group_satd_by_severity(satd);
        writeln!(output, "**SATD Summary:**")?;
        for (severity, count) in by_severity {
            writeln!(output, "- {severity:?}: {count}")?;
        }
        Ok(())
    }

    fn group_satd_by_severity<'a>(
        &self,
        satd: &'a crate::services::satd_detector::SATDAnalysisResult,
    ) -> FxHashMap<&'a crate::services::satd_detector::Severity, i32> {
        let mut by_severity = FxHashMap::default();
        for item in &satd.items {
            *by_severity.entry(&item.severity).or_insert(0) += 1;
        }
        by_severity
    }

    fn write_critical_items(
        &self,
        output: &mut String,
        satd: &crate::services::satd_detector::SATDAnalysisResult,
    ) -> anyhow::Result<()> {
        let critical_items = self.get_critical_satd_items(satd);
        if critical_items.is_empty() {
            return Ok(());
        }

        use std::fmt::Write;
        writeln!(output, "\n**Critical Items:**")?;
        for item in critical_items {
            writeln!(
                output,
                "- `{}:{} {}`: {}",
                item.file.display(),
                item.line,
                item.category,
                item.text.trim()
            )?;
        }
        Ok(())
    }

    fn get_critical_satd_items<'a>(
        &self,
        satd: &'a crate::services::satd_detector::SATDAnalysisResult,
    ) -> Vec<&'a crate::services::satd_detector::TechnicalDebt> {
        satd.items
            .iter()
            .filter(|item| {
                matches!(
                    item.severity,
                    crate::services::satd_detector::Severity::Critical
                )
            })
            .take(5)
            .collect()
    }

    pub(crate) fn format_dead_code_analysis(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        if let Some(ref dead_code) = context.analyses.dead_code_results {
            self.write_dead_code_header(output)?;
            self.write_dead_code_summary(output, &dead_code.summary)?;
            self.write_dead_code_files_table(output, &dead_code.ranked_files)?;
        }
        Ok(())
    }

    fn write_dead_code_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Dead Code Analysis\n")?;
        Ok(())
    }

    fn write_dead_code_summary(
        &self,
        output: &mut String,
        summary: &crate::models::dead_code::DeadCodeSummary,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Summary:**")?;
        writeln!(output, "- Dead Functions: {}", summary.dead_functions)?;
        writeln!(output, "- Total Dead Lines: {}", summary.total_dead_lines)?;
        Ok(())
    }

    fn write_dead_code_files_table(
        &self,
        output: &mut String,
        files: &[crate::models::dead_code::FileDeadCodeMetrics],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !files.is_empty() {
            writeln!(output, "\n**Top Files with Dead Code:**")?;
            writeln!(output, "| File | Dead Lines | Dead Functions |")?;
            writeln!(output, "|------|------------|----------------|")?;

            for file in files.iter().take(10) {
                writeln!(
                    output,
                    "| `{}` | {} | {} |",
                    file.path, file.dead_lines, file.dead_functions
                )?;
            }
            writeln!(output)?;
        }
        Ok(())
    }

    pub(crate) fn format_cross_references(
        &self,
        output: &mut String,
        cross_refs: &[CrossLangReference],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !cross_refs.is_empty() {
            writeln!(output, "## Cross-Language References\n")?;

            writeln!(output, "| Source | Target | Type | Confidence |")?;
            writeln!(output, "|--------|--------|------|------------|")?;

            for cross_ref in cross_refs {
                writeln!(
                    output,
                    "| `{}` | `{}` | {:?} | {:.1}% |",
                    cross_ref.source_file.display(),
                    cross_ref.target_file.display(),
                    cross_ref.reference_type,
                    cross_ref.confidence * 100.0
                )?;
            }
            writeln!(output)?;
        }

        Ok(())
    }

    pub(crate) fn format_defect_predictions(
        &self,
        output: &mut String,
        context: &DeepContext,
    ) -> anyhow::Result<()> {
        self.write_defect_header(output)?;
        self.write_defect_summary(output, &context.defect_summary)?;
        self.write_defect_hotspots_table(output, &context.hotspots)?;
        Ok(())
    }

    fn write_defect_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Defect Probability Analysis\n")?;
        Ok(())
    }

    fn write_defect_summary(
        &self,
        output: &mut String,
        summary: &DefectSummary,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Risk Assessment:**")?;
        writeln!(
            output,
            "- Total Defects Predicted: {}",
            summary.total_defects
        )?;
        writeln!(
            output,
            "- Defect Density: {:.2} defects per 1000 lines",
            summary.defect_density
        )?;
        Ok(())
    }

    fn write_defect_hotspots_table(
        &self,
        output: &mut String,
        hotspots: &[DefectHotspot],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !hotspots.is_empty() {
            writeln!(output, "\n**High-Risk Hotspots:**")?;
            writeln!(output, "| File:Line | Risk Score | Effort (hours) |")?;
            writeln!(output, "|-----------|------------|----------------|")?;

            for hotspot in hotspots.iter().take(10) {
                writeln!(
                    output,
                    "| `{}:{}` | {:.1} | {:.1} |",
                    hotspot.location.file.display(),
                    hotspot.location.line,
                    hotspot.composite_score,
                    hotspot.refactoring_effort.estimated_hours
                )?;
            }
        }
        writeln!(output)?;
        Ok(())
    }

    pub(crate) fn format_prioritized_recommendations(
        &self,
        output: &mut String,
        recommendations: &[PrioritizedRecommendation],
    ) -> anyhow::Result<()> {
        if recommendations.is_empty() {
            return Ok(());
        }

        self.write_recommendations_header(output)?;

        for (i, rec) in recommendations.iter().enumerate() {
            self.write_single_recommendation(output, i, rec)?;
        }

        Ok(())
    }

    fn write_recommendations_header(&self, output: &mut String) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "## Prioritized Recommendations\n")?;
        Ok(())
    }

    fn write_single_recommendation(
        &self,
        output: &mut String,
        index: usize,
        rec: &PrioritizedRecommendation,
    ) -> anyhow::Result<()> {
        let priority_emoji = self.get_priority_emoji(&rec.priority);
        self.write_recommendation_title(output, priority_emoji, index + 1, &rec.title)?;
        self.write_recommendation_details(output, rec)?;
        self.write_recommendation_prerequisites(output, &rec.prerequisites)?;
        Ok(())
    }

    fn get_priority_emoji(&self, priority: &Priority) -> &'static str {
        match priority {
            Priority::Critical => "\u{1f534}",
            Priority::High => "\u{1f7e1}",
            Priority::Medium => "\u{1f535}",
            Priority::Low => "\u{26aa}",
        }
    }

    fn write_recommendation_title(
        &self,
        output: &mut String,
        emoji: &str,
        number: usize,
        title: &str,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "### {emoji} {number} {title}")?;
        Ok(())
    }

    fn write_recommendation_details(
        &self,
        output: &mut String,
        rec: &PrioritizedRecommendation,
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        writeln!(output, "**Description:** {}", rec.description)?;
        writeln!(output, "**Effort:** {:?}", rec.estimated_effort)?;
        writeln!(output, "**Impact:** {:?}", rec.impact)?;
        Ok(())
    }

    fn write_recommendation_prerequisites(
        &self,
        output: &mut String,
        prerequisites: &[String],
    ) -> anyhow::Result<()> {
        use std::fmt::Write;
        if !prerequisites.is_empty() {
            writeln!(output, "**Prerequisites:**")?;
            for prereq in prerequisites {
                writeln!(output, "- {prereq}")?;
            }
        }
        writeln!(output)?;
        Ok(())
    }
}
