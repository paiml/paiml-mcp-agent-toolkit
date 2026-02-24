// Serialization, clone, and debug tests for defect report types
// Included from defect_report.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod serialization_tests {
    use super::*;

    // Test helper: Create a sample defect
    fn create_test_defect(severity: Severity, category: DefectCategory) -> Defect {
        Defect {
            id: "TEST-001".to_string(),
            severity,
            category,
            file_path: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: Some(10),
            column_start: Some(5),
            column_end: Some(20),
            message: "Test defect message".to_string(),
            rule_id: "TEST001".to_string(),
            fix_suggestion: Some("Fix suggestion".to_string()),
            metrics: HashMap::new(),
        }
    }

    // Test helper: Create a sample defect report
    fn create_test_report() -> DefectReport {
        let metadata = ReportMetadata {
            tool: "pmat".to_string(),
            version: "1.0.0".to_string(),
            generated_at: Utc::now(),
            project_root: PathBuf::from("/test/project"),
            total_files_analyzed: 10,
            analysis_duration_ms: 1000,
        };

        let defect = create_test_defect(Severity::High, DefectCategory::Complexity);

        let mut by_severity = BTreeMap::new();
        by_severity.insert("High".to_string(), 1);

        let mut by_category = BTreeMap::new();
        by_category.insert("Complexity".to_string(), 1);

        let summary = DefectSummary {
            total_defects: 1,
            by_severity,
            by_category,
            hotspot_files: vec![FileHotspot {
                path: PathBuf::from("test.rs"),
                defect_count: 1,
                severity_score: 5.0,
            }],
        };

        let mut file_index = BTreeMap::new();
        file_index.insert(PathBuf::from("test.rs"), vec!["TEST-001".to_string()]);

        DefectReport {
            metadata,
            defects: vec![defect],
            summary,
            file_index,
        }
    }

    // === DefectReport Tests ===

    #[test]
    fn test_defect_report_creation() {
        let report = create_test_report();

        assert_eq!(report.metadata.tool, "pmat");
        assert_eq!(report.defects.len(), 1);
        assert_eq!(report.summary.total_defects, 1);
        assert_eq!(report.file_index.len(), 1);
    }

    #[test]
    fn test_defect_report_file_index() {
        let report = create_test_report();

        let defect_ids = report.file_index.get(&PathBuf::from("test.rs"));
        assert!(defect_ids.is_some());
        assert_eq!(defect_ids.unwrap(), &vec!["TEST-001".to_string()]);
    }

    #[test]
    fn test_defect_report_multiple_files() {
        let mut file_index = BTreeMap::new();
        file_index.insert(
            PathBuf::from("a.rs"),
            vec!["A-001".to_string(), "A-002".to_string()],
        );
        file_index.insert(PathBuf::from("b.rs"), vec!["B-001".to_string()]);
        file_index.insert(
            PathBuf::from("c.rs"),
            vec![
                "C-001".to_string(),
                "C-002".to_string(),
                "C-003".to_string(),
            ],
        );

        assert_eq!(file_index.len(), 3);
        assert_eq!(file_index.get(&PathBuf::from("a.rs")).unwrap().len(), 2);
        assert_eq!(file_index.get(&PathBuf::from("c.rs")).unwrap().len(), 3);
    }

    // === Serialization Tests ===

    #[test]
    fn test_severity_serialization() {
        let json = serde_json::to_string(&Severity::Critical).unwrap();
        assert_eq!(json, "\"critical\"");

        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Severity::Critical);
    }

    #[test]
    fn test_defect_category_serialization() {
        let json = serde_json::to_string(&DefectCategory::TechnicalDebt).unwrap();
        assert_eq!(json, "\"technical_debt\"");

        let deserialized: DefectCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, DefectCategory::TechnicalDebt);
    }

    #[test]
    fn test_defect_serialization_roundtrip() {
        let defect = create_test_defect(Severity::High, DefectCategory::Performance);
        let json = serde_json::to_string(&defect).unwrap();
        let deserialized: Defect = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, defect.id);
        assert_eq!(deserialized.severity, defect.severity);
        assert_eq!(deserialized.category, defect.category);
        assert_eq!(deserialized.message, defect.message);
    }

    #[test]
    fn test_defect_report_serialization_roundtrip() {
        let report = create_test_report();
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: DefectReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.metadata.tool, report.metadata.tool);
        assert_eq!(deserialized.defects.len(), report.defects.len());
        assert_eq!(
            deserialized.summary.total_defects,
            report.summary.total_defects
        );
    }

    #[test]
    fn test_file_hotspot_serialization_roundtrip() {
        let hotspot = FileHotspot {
            path: PathBuf::from("test/path.rs"),
            defect_count: 42,
            severity_score: 123.456,
        };

        let json = serde_json::to_string(&hotspot).unwrap();
        let deserialized: FileHotspot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.path, hotspot.path);
        assert_eq!(deserialized.defect_count, hotspot.defect_count);
        assert!((deserialized.severity_score - hotspot.severity_score).abs() < f64::EPSILON);
    }

    // === Clone and Debug Tests ===

    #[test]
    fn test_severity_clone() {
        let severity = Severity::High;
        let cloned = severity;
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_defect_clone() {
        let defect = create_test_defect(Severity::Medium, DefectCategory::Duplication);
        let cloned = defect.clone();
        assert_eq!(cloned.id, defect.id);
        assert_eq!(cloned.severity, defect.severity);
    }

    #[test]
    fn test_defect_report_clone() {
        let report = create_test_report();
        let cloned = report.clone();
        assert_eq!(cloned.metadata.tool, report.metadata.tool);
        assert_eq!(cloned.defects.len(), report.defects.len());
    }

    #[test]
    fn test_severity_debug() {
        let severity = Severity::Critical;
        let debug = format!("{:?}", severity);
        assert!(debug.contains("Critical"));
    }

    #[test]
    fn test_defect_category_debug() {
        let category = DefectCategory::Architecture;
        let debug = format!("{:?}", category);
        assert!(debug.contains("Architecture"));
    }

    #[test]
    fn test_file_ranking_config_debug() {
        let config = FileRankingConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("use_severity"));
        assert!(debug.contains("use_count"));
    }

    #[test]
    fn test_ranked_file_debug() {
        let ranked = RankedFile {
            rank: 1,
            score: 50.0,
            path: PathBuf::from("test.rs"),
            defects: vec![],
        };
        let debug = format!("{:?}", ranked);
        assert!(debug.contains("rank: 1"));
        assert!(debug.contains("score: 50.0"));
    }
}
