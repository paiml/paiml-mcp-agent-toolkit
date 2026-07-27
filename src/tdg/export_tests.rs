// Tests for TDG export functionality
// Included by export.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::Language;

    fn create_test_score() -> TdgScore {
        TdgScore {
            structural_complexity: 20.0,
            semantic_complexity: 18.0,
            duplication_ratio: 17.0,
            coupling_score: 12.0,
            doc_coverage: 8.0,
            consistency_score: 8.0,
            entropy_score: 15.0,
            total: 83.0,
            grade: Grade::BPlus,
            confidence: 0.95,
            language: Language::Rust,
            file_path: Some(std::path::PathBuf::from("test.rs")),
            penalties_applied: Vec::new(),
        }
    }

    #[test]
    fn test_json_export() {
        let score = create_test_score();
        let options = ExportOptions::default();

        let result = TdgExporter::export_score(&score, &options);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("\"total\": 83.0"));
        assert!(json.contains("\"grade\": \"B+\""));
    }

    #[test]
    fn test_csv_export() {
        let score = create_test_score();
        let mut options = ExportOptions::default();
        options.format = ExportFormat::Csv;

        let result = TdgExporter::export_score(&score, &options);
        assert!(result.is_ok());

        let csv = result.unwrap();
        assert!(csv.contains("total_score,83.00"));
        assert!(csv.contains("grade,B+"));
    }

    #[test]
    fn test_sarif_export() {
        let score = create_test_score();
        let mut options = ExportOptions::default();
        options.format = ExportFormat::Sarif;

        let result = TdgExporter::export_score(&score, &options);
        assert!(result.is_ok());

        let sarif = result.unwrap();
        assert!(sarif.contains("\"$schema\""));
        assert!(sarif.contains("\"version\": \"2.1.0\""));
    }

    #[test]
    fn test_markdown_export() {
        let score = create_test_score();
        let mut options = ExportOptions::default();
        options.format = ExportFormat::Markdown;

        let result = TdgExporter::export_score(&score, &options);
        assert!(result.is_ok());

        let md = result.unwrap();
        assert!(md.contains("# TDG Analysis Report"));
        assert!(md.contains("## Overall Score: 83.0/100 (B+)"));
    }
}

