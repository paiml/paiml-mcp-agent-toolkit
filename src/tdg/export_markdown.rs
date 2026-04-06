// Markdown export implementations for TdgExporter
// Included by export.rs - shares parent module scope

impl TdgExporter {
    fn score_to_markdown(score: &TdgScore, options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: score_to_markdown");
        let mut md = String::new();

        md.push_str("# TDG Analysis Report\n\n");

        if let Some(path) = &score.file_path {
            md.push_str(&format!("**File:** `{}`\n\n", path.display()));
        }

        md.push_str(&format!(
            "## Overall Score: {:.1}/100 ({})\n\n",
            score.total, score.grade
        ));

        md.push_str("### Metrics Breakdown\n\n");
        md.push_str("| Metric | Score | Max |\n");
        md.push_str("|--------|-------|-----|\n");
        md.push_str(&format!(
            "| Structural Complexity | {:.1} | 25.0 |\n",
            score.structural_complexity
        ));
        md.push_str(&format!(
            "| Semantic Complexity | {:.1} | 20.0 |\n",
            score.semantic_complexity
        ));
        md.push_str(&format!(
            "| Duplication Ratio | {:.1} | 20.0 |\n",
            score.duplication_ratio
        ));
        md.push_str(&format!(
            "| Coupling Score | {:.1} | 15.0 |\n",
            score.coupling_score
        ));
        md.push_str(&format!(
            "| Documentation | {:.1} | 10.0 |\n",
            score.doc_coverage
        ));
        md.push_str(&format!(
            "| Consistency | {:.1} | 10.0 |\n",
            score.consistency_score
        ));

        if options.include_recommendations {
            md.push_str("\n### Recommendations\n\n");
            for rec in Self::generate_recommendations(score) {
                md.push_str(&format!("- {rec}\n"));
            }
        }

        Ok(md)
    }

    fn project_to_markdown(project: &ProjectScore, _options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: project_to_markdown");
        let mut md = String::new();

        md.push_str("# Project TDG Analysis\n\n");
        md.push_str(&format!("**Total Files:** {}\n", project.total_files));
        md.push_str(&format!(
            "**Average Score:** {:.1}/100 ({})\n\n",
            project.average_score, project.average_grade
        ));

        md.push_str("## Language Distribution\n\n");
        for (lang, count) in &project.language_distribution {
            md.push_str(&format!("- {lang:?}: {count} files\n"));
        }

        md.push_str("\n## File Scores\n\n");
        md.push_str("| File | Score | Grade |\n");
        md.push_str("|------|-------|-------|\n");

        for score in &project.files {
            let path = score
                .file_path
                .as_ref().map_or_else(|| "unknown".to_string(), |p| p.display().to_string());
            md.push_str(&format!(
                "| {} | {:.1} | {} |\n",
                path, score.total, score.grade
            ));
        }

        Ok(md)
    }

    fn comparison_to_markdown(comparison: &Comparison, _options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: comparison_to_markdown");
        let mut md = String::new();

        md.push_str("# TDG Comparison Report\n\n");
        md.push_str(&format!("**Winner:** {}\n", comparison.winner));
        md.push_str(&format!(
            "**Improvement:** {:.1}% ({:+.1} points)\n\n",
            comparison.improvement_percentage, comparison.delta
        ));

        md.push_str("## Score Comparison\n\n");
        md.push_str("| Metric | Source 1 | Source 2 | Delta |\n");
        md.push_str("|--------|----------|----------|-------|\n");
        md.push_str(&format!(
            "| Total Score | {:.1} | {:.1} | {:+.1} |\n",
            comparison.source1.total, comparison.source2.total, comparison.delta
        ));
        md.push_str(&format!(
            "| Grade | {} | {} | - |\n",
            comparison.source1.grade, comparison.source2.grade
        ));

        if !comparison.improvements.is_empty() {
            md.push_str("\n## Improvements\n\n");
            for improvement in &comparison.improvements {
                md.push_str(&format!("- {improvement}\n"));
            }
        }

        if !comparison.regressions.is_empty() {
            md.push_str("\n## Regressions\n\n");
            for regression in &comparison.regressions {
                md.push_str(&format!("- {regression}\n"));
            }
        }

        Ok(md)
    }

    fn generate_recommendations(score: &TdgScore) -> Vec<String> {
        debug_assert!(true, "contract: generate_recommendations");
        let mut recommendations = Vec::new();

        if score.structural_complexity < 15.0 {
            recommendations.push(
                "Consider refactoring complex functions to improve structural complexity"
                    .to_string(),
            );
        }

        if score.semantic_complexity < 15.0 {
            recommendations
                .push("Simplify nested logic and reduce cognitive complexity".to_string());
        }

        if score.duplication_ratio < 15.0 {
            recommendations
                .push("Extract common code patterns into reusable functions".to_string());
        }

        if score.doc_coverage < 7.0 {
            recommendations.push("Add documentation for public APIs and complex logic".to_string());
        }

        if score.coupling_score < 10.0 {
            recommendations
                .push("Reduce dependencies between modules to improve coupling".to_string());
        }

        recommendations
    }
}
