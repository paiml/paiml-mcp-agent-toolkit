// XML export implementations for TdgExporter
// Included by export.rs - shares parent module scope

impl TdgExporter {
    fn score_to_xml(score: &TdgScore, _options: &ExportOptions) -> Result<String> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<tdg_analysis>\n");
        xml.push_str(&format!(
            "  <total_score>{:.2}</total_score>\n",
            score.total
        ));
        xml.push_str(&format!("  <grade>{}</grade>\n", score.grade));
        xml.push_str(&format!("  <language>{:?}</language>\n", score.language));
        xml.push_str(&format!(
            "  <confidence>{:.2}</confidence>\n",
            score.confidence
        ));
        xml.push_str("  <metrics>\n");
        xml.push_str(&format!(
            "    <structural_complexity>{:.2}</structural_complexity>\n",
            score.structural_complexity
        ));
        xml.push_str(&format!(
            "    <semantic_complexity>{:.2}</semantic_complexity>\n",
            score.semantic_complexity
        ));
        xml.push_str(&format!(
            "    <duplication_ratio>{:.2}</duplication_ratio>\n",
            score.duplication_ratio
        ));
        xml.push_str(&format!(
            "    <coupling_score>{:.2}</coupling_score>\n",
            score.coupling_score
        ));
        xml.push_str(&format!(
            "    <doc_coverage>{:.2}</doc_coverage>\n",
            score.doc_coverage
        ));
        xml.push_str(&format!(
            "    <consistency_score>{:.2}</consistency_score>\n",
            score.consistency_score
        ));
        xml.push_str("  </metrics>\n");

        if let Some(path) = &score.file_path {
            xml.push_str(&format!("  <file_path>{}</file_path>\n", path.display()));
        }

        xml.push_str("</tdg_analysis>\n");
        Ok(xml)
    }
}
