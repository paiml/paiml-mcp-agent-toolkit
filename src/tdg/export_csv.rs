// CSV export implementations for TdgExporter
// Included by export.rs - shares parent module scope

impl TdgExporter {
    fn score_to_csv(score: &TdgScore, _options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: score_to_csv");
        let mut csv = String::new();
        csv.push_str("metric,value\n");
        csv.push_str(&format!("total_score,{:.2}\n", score.total));
        csv.push_str(&format!("grade,{}\n", score.grade));
        csv.push_str(&format!(
            "structural_complexity,{:.2}\n",
            score.structural_complexity
        ));
        csv.push_str(&format!(
            "semantic_complexity,{:.2}\n",
            score.semantic_complexity
        ));
        csv.push_str(&format!(
            "duplication_ratio,{:.2}\n",
            score.duplication_ratio
        ));
        csv.push_str(&format!("coupling_score,{:.2}\n", score.coupling_score));
        csv.push_str(&format!("doc_coverage,{:.2}\n", score.doc_coverage));
        csv.push_str(&format!(
            "consistency_score,{:.2}\n",
            score.consistency_score
        ));
        csv.push_str(&format!("confidence,{:.2}\n", score.confidence));
        csv.push_str(&format!("language,{:?}\n", score.language));

        if let Some(path) = &score.file_path {
            csv.push_str(&format!("file_path,{}\n", path.display()));
        }

        Ok(csv)
    }

    fn project_to_csv(project: &ProjectScore, options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: project_to_csv");
        let mut csv = String::new();
        csv.push_str("file_path,total_score,grade,structural,semantic,duplication,coupling,documentation,consistency,language\n");

        for score in &project.files {
            let path = score
                .file_path
                .as_ref().map_or_else(|| "unknown".to_string(), |p| p.display().to_string());

            csv.push_str(&format!(
                "{},{:.2},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:?}\n",
                path,
                score.total,
                score.grade,
                score.structural_complexity,
                score.semantic_complexity,
                score.duplication_ratio,
                score.coupling_score,
                score.doc_coverage,
                score.consistency_score,
                score.language,
            ));
        }

        if options.include_metadata {
            csv.push_str("\nSummary\n");
            csv.push_str(&format!("total_files,{}\n", project.total_files));
            csv.push_str(&format!("average_score,{:.2}\n", project.average_score));
            csv.push_str(&format!("average_grade,{}\n", project.average_grade));
        }

        Ok(csv)
    }

    fn comparison_to_csv(comparison: &Comparison, _options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: comparison_to_csv");
        let mut csv = String::new();
        csv.push_str("metric,source1,source2,delta\n");

        csv.push_str(&format!(
            "total_score,{:.2},{:.2},{:.2}\n",
            comparison.source1.total, comparison.source2.total, comparison.delta
        ));

        csv.push_str(&format!(
            "grade,{},{},\n",
            comparison.source1.grade, comparison.source2.grade
        ));

        csv.push_str(&format!(
            "structural_complexity,{:.2},{:.2},{:.2}\n",
            comparison.source1.structural_complexity,
            comparison.source2.structural_complexity,
            comparison.source2.structural_complexity - comparison.source1.structural_complexity
        ));

        csv.push_str(&format!(
            "semantic_complexity,{:.2},{:.2},{:.2}\n",
            comparison.source1.semantic_complexity,
            comparison.source2.semantic_complexity,
            comparison.source2.semantic_complexity - comparison.source1.semantic_complexity
        ));

        csv.push_str(&format!(
            "improvement_percentage,,,{:.2}%\n",
            comparison.improvement_percentage
        ));
        csv.push_str(&format!("winner,{}\n", comparison.winner));

        Ok(csv)
    }
}
