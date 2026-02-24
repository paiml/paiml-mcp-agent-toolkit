// Unit tests for defect report core types
// Included from defect_report.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
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

    // === Severity Tests ===

    #[test]
    fn test_defect_id_generation() {
        assert_eq!(Defect::generate_id("CPLX", 0), "CPLX-001");
        assert_eq!(Defect::generate_id("SATD", 99), "SATD-100");
    }

    #[test]
    fn test_defect_id_generation_various_prefixes() {
        assert_eq!(Defect::generate_id("A", 0), "A-001");
        assert_eq!(Defect::generate_id("DEAD", 5), "DEAD-006");
        assert_eq!(Defect::generate_id("PERF", 999), "PERF-1000");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Low, Severity::Low);
        assert_eq!(Severity::Medium, Severity::Medium);
        assert_eq!(Severity::High, Severity::High);
        assert_eq!(Severity::Critical, Severity::Critical);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Low), "Low");
        assert_eq!(format!("{}", Severity::Medium), "Medium");
        assert_eq!(format!("{}", Severity::High), "High");
        assert_eq!(format!("{}", Severity::Critical), "Critical");
    }

    #[test]
    fn test_severity_weight() {
        let defect = Defect {
            id: "TEST-001".to_string(),
            severity: Severity::Critical,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from("test.rs"),
            line_start: 1,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Test".to_string(),
            rule_id: "test".to_string(),
            fix_suggestion: None,
            metrics: HashMap::new(),
        };

        assert_eq!(defect.severity_weight(), 10.0);
    }

    #[test]
    fn test_severity_weight_all_levels() {
        let mut defect = create_test_defect(Severity::Low, DefectCategory::Complexity);
        assert_eq!(defect.severity_weight(), 1.0);

        defect.severity = Severity::Medium;
        assert_eq!(defect.severity_weight(), 3.0);

        defect.severity = Severity::High;
        assert_eq!(defect.severity_weight(), 5.0);

        defect.severity = Severity::Critical;
        assert_eq!(defect.severity_weight(), 10.0);
    }

    // === DefectCategory Tests ===

    #[test]
    fn test_defect_category_all() {
        let categories = DefectCategory::all();
        assert_eq!(categories.len(), 7);
        assert!(categories.contains(&DefectCategory::Complexity));
        assert!(categories.contains(&DefectCategory::TechnicalDebt));
        assert!(categories.contains(&DefectCategory::DeadCode));
        assert!(categories.contains(&DefectCategory::Duplication));
        assert!(categories.contains(&DefectCategory::Performance));
        assert!(categories.contains(&DefectCategory::Architecture));
        assert!(categories.contains(&DefectCategory::TestCoverage));
    }

    #[test]
    fn test_defect_category_display() {
        assert_eq!(format!("{}", DefectCategory::Complexity), "Complexity");
        assert_eq!(
            format!("{}", DefectCategory::TechnicalDebt),
            "Technical Debt"
        );
        assert_eq!(format!("{}", DefectCategory::DeadCode), "Dead Code");
        assert_eq!(format!("{}", DefectCategory::Duplication), "Duplication");
        assert_eq!(format!("{}", DefectCategory::Performance), "Performance");
        assert_eq!(format!("{}", DefectCategory::Architecture), "Architecture");
        assert_eq!(format!("{}", DefectCategory::TestCoverage), "Test Coverage");
    }

    #[test]
    fn test_defect_category_ordering() {
        assert!(DefectCategory::TechnicalDebt > DefectCategory::Complexity);
        assert!(DefectCategory::DeadCode > DefectCategory::TechnicalDebt);
        assert!(DefectCategory::TestCoverage > DefectCategory::Architecture);
    }

    #[test]
    fn test_defect_category_equality() {
        assert_eq!(DefectCategory::Complexity, DefectCategory::Complexity);
        assert_ne!(DefectCategory::Complexity, DefectCategory::Performance);
    }

    #[test]
    fn test_defect_category_hash() {
        let mut map: HashMap<DefectCategory, i32> = HashMap::new();
        map.insert(DefectCategory::Complexity, 1);
        map.insert(DefectCategory::Performance, 2);
        assert_eq!(map.get(&DefectCategory::Complexity), Some(&1));
        assert_eq!(map.get(&DefectCategory::Performance), Some(&2));
    }

    // === Defect Tests ===

    #[test]
    fn test_defect_creation() {
        let defect = create_test_defect(Severity::High, DefectCategory::Complexity);
        assert_eq!(defect.id, "TEST-001");
        assert_eq!(defect.severity, Severity::High);
        assert_eq!(defect.category, DefectCategory::Complexity);
        assert_eq!(defect.file_path, PathBuf::from("test.rs"));
        assert_eq!(defect.line_start, 1);
        assert_eq!(defect.line_end, Some(10));
        assert_eq!(defect.column_start, Some(5));
        assert_eq!(defect.column_end, Some(20));
        assert_eq!(defect.message, "Test defect message");
        assert_eq!(defect.rule_id, "TEST001");
        assert_eq!(defect.fix_suggestion, Some("Fix suggestion".to_string()));
    }

    #[test]
    fn test_defect_with_metrics() {
        let mut metrics = HashMap::new();
        metrics.insert("complexity".to_string(), 25.0);
        metrics.insert("lines".to_string(), 100.0);

        let defect = Defect {
            id: "CPLX-001".to_string(),
            severity: Severity::High,
            category: DefectCategory::Complexity,
            file_path: PathBuf::from("complex.rs"),
            line_start: 10,
            line_end: Some(50),
            column_start: None,
            column_end: None,
            message: "High complexity".to_string(),
            rule_id: "CPLX001".to_string(),
            fix_suggestion: Some("Extract function".to_string()),
            metrics,
        };

        assert_eq!(defect.metrics.get("complexity"), Some(&25.0));
        assert_eq!(defect.metrics.get("lines"), Some(&100.0));
    }

    #[test]
    fn test_defect_minimal() {
        let defect = Defect {
            id: "MIN-001".to_string(),
            severity: Severity::Low,
            category: DefectCategory::DeadCode,
            file_path: PathBuf::from("unused.rs"),
            line_start: 1,
            line_end: None,
            column_start: None,
            column_end: None,
            message: "Unused function".to_string(),
            rule_id: "DEAD001".to_string(),
            fix_suggestion: None,
            metrics: HashMap::new(),
        };

        assert_eq!(defect.line_end, None);
        assert_eq!(defect.column_start, None);
        assert_eq!(defect.fix_suggestion, None);
        assert!(defect.metrics.is_empty());
    }

    // === ReportMetadata Tests ===

    #[test]
    fn test_report_metadata_creation() {
        let now = Utc::now();
        let metadata = ReportMetadata {
            tool: "pmat".to_string(),
            version: "2.0.0".to_string(),
            generated_at: now,
            project_root: PathBuf::from("/home/user/project"),
            total_files_analyzed: 100,
            analysis_duration_ms: 5000,
        };

        assert_eq!(metadata.tool, "pmat");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.generated_at, now);
        assert_eq!(metadata.project_root, PathBuf::from("/home/user/project"));
        assert_eq!(metadata.total_files_analyzed, 100);
        assert_eq!(metadata.analysis_duration_ms, 5000);
    }

    // === DefectSummary Tests ===

    #[test]
    fn test_defect_summary_creation() {
        let mut by_severity = BTreeMap::new();
        by_severity.insert("Low".to_string(), 5);
        by_severity.insert("Medium".to_string(), 3);
        by_severity.insert("High".to_string(), 2);

        let mut by_category = BTreeMap::new();
        by_category.insert("Complexity".to_string(), 4);
        by_category.insert("DeadCode".to_string(), 6);

        let summary = DefectSummary {
            total_defects: 10,
            by_severity,
            by_category,
            hotspot_files: vec![],
        };

        assert_eq!(summary.total_defects, 10);
        assert_eq!(summary.by_severity.get("Low"), Some(&5));
        assert_eq!(summary.by_category.get("Complexity"), Some(&4));
    }

    #[test]
    fn test_defect_summary_with_hotspots() {
        let hotspots = vec![
            FileHotspot {
                path: PathBuf::from("src/main.rs"),
                defect_count: 10,
                severity_score: 50.0,
            },
            FileHotspot {
                path: PathBuf::from("src/lib.rs"),
                defect_count: 5,
                severity_score: 25.0,
            },
        ];

        let summary = DefectSummary {
            total_defects: 15,
            by_severity: BTreeMap::new(),
            by_category: BTreeMap::new(),
            hotspot_files: hotspots,
        };

        assert_eq!(summary.hotspot_files.len(), 2);
        assert_eq!(summary.hotspot_files[0].defect_count, 10);
        assert_eq!(summary.hotspot_files[1].severity_score, 25.0);
    }

    // === FileHotspot Tests ===

    #[test]
    fn test_file_hotspot_creation() {
        let hotspot = FileHotspot {
            path: PathBuf::from("src/complex.rs"),
            defect_count: 25,
            severity_score: 75.5,
        };

        assert_eq!(hotspot.path, PathBuf::from("src/complex.rs"));
        assert_eq!(hotspot.defect_count, 25);
        assert!((hotspot.severity_score - 75.5).abs() < f64::EPSILON);
    }

    // === FileRankingConfig Tests ===

    #[test]
    fn test_file_ranking_config_default() {
        let config = FileRankingConfig::default();

        assert!(config.use_severity);
        assert!(config.use_count);
        assert_eq!(config.category_weights.len(), 7);
        assert_eq!(
            config.category_weights.get(&DefectCategory::Complexity),
            Some(&1.5)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::Performance),
            Some(&2.0)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::Architecture),
            Some(&1.8)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::TechnicalDebt),
            Some(&1.2)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::DeadCode),
            Some(&1.0)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::Duplication),
            Some(&1.3)
        );
        assert_eq!(
            config.category_weights.get(&DefectCategory::TestCoverage),
            Some(&0.8)
        );
    }

    #[test]
    fn test_file_ranking_config_custom() {
        let mut category_weights = HashMap::new();
        category_weights.insert(DefectCategory::Complexity, 3.0);

        let config = FileRankingConfig {
            use_severity: false,
            use_count: true,
            category_weights,
        };

        assert!(!config.use_severity);
        assert!(config.use_count);
        assert_eq!(config.category_weights.len(), 1);
        assert_eq!(
            config.category_weights.get(&DefectCategory::Complexity),
            Some(&3.0)
        );
    }

    // === RankedFile Tests ===

    #[test]
    fn test_ranked_file_creation() {
        let defects = vec![
            create_test_defect(Severity::High, DefectCategory::Complexity),
            create_test_defect(Severity::Medium, DefectCategory::DeadCode),
        ];

        let ranked = RankedFile {
            rank: 1,
            score: 100.5,
            path: PathBuf::from("src/main.rs"),
            defects,
        };

        assert_eq!(ranked.rank, 1);
        assert!((ranked.score - 100.5).abs() < f64::EPSILON);
        assert_eq!(ranked.path, PathBuf::from("src/main.rs"));
        assert_eq!(ranked.defects.len(), 2);
    }

    #[test]
    fn test_ranked_file_empty_defects() {
        let ranked = RankedFile {
            rank: 5,
            score: 0.0,
            path: PathBuf::from("src/empty.rs"),
            defects: vec![],
        };

        assert_eq!(ranked.rank, 5);
        assert_eq!(ranked.score, 0.0);
        assert!(ranked.defects.is_empty());
    }
}
