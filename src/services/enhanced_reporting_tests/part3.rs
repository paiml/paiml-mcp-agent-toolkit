        let service = EnhancedReportingService::default();
        let config = ReportConfig {
            project_path: PathBuf::from("/test/project"),
            output_format: ReportFormat::Json,
            include_visualizations: false, // No visualizations
            include_executive_summary: true,
            include_recommendations: true,
            confidence_threshold: 80,
            output_path: None,
        };
        let results = create_full_analysis_results();
        let report = service.generate_report(config, results).await;
        assert!(report.is_ok());
        let report = report.unwrap();
        assert!(report.visualizations.is_empty());
    }

    // Report Formatting Tests (Async)

    #[tokio::test]
    async fn test_format_report_json() {
        let service = EnhancedReportingService::default();
        let config = create_test_report_config();
        let results = create_minimal_analysis_results();
        let report = service.generate_report(config, results).await.unwrap();
        let formatted = service.format_report(&report, ReportFormat::Json).await;
        assert!(formatted.is_ok());
        let json_str = formatted.unwrap();
        // Verify it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        assert!(parsed.is_ok());
    }

    #[tokio::test]
    async fn test_format_report_markdown() {
        let service = EnhancedReportingService::default();
        let config = create_test_report_config();
        let results = create_full_analysis_results();
        let report = service.generate_report(config, results).await.unwrap();
        let formatted = service.format_report(&report, ReportFormat::Markdown).await;
        assert!(formatted.is_ok());
        let md = formatted.unwrap();
        assert!(md.contains("# "));
        assert!(md.contains("## Metadata"));
        assert!(md.contains("## Executive Summary"));
    }

    #[tokio::test]
    async fn test_format_report_html() {
        let service = EnhancedReportingService::default();
        let config = create_test_report_config();
        let results = create_minimal_analysis_results();
        let report = service.generate_report(config, results).await.unwrap();
        let formatted = service.format_report(&report, ReportFormat::Html).await;
        assert!(formatted.is_ok());
        let html = formatted.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html>"));
        assert!(html.contains("Analysis Report"));
    }

    #[tokio::test]
    async fn test_format_report_pdf() {
        let service = EnhancedReportingService::default();
        let config = create_test_report_config();
        let results = create_minimal_analysis_results();
        let report = service.generate_report(config, results).await.unwrap();
        let formatted = service.format_report(&report, ReportFormat::Pdf).await;
        assert!(formatted.is_ok());
        let pdf = formatted.unwrap();
        assert!(pdf.contains("PDF Report Generated"));
    }

    #[tokio::test]
    async fn test_format_report_dashboard() {
        let service = EnhancedReportingService::default();
        let config = create_test_report_config();
        let results = create_minimal_analysis_results();
        let report = service.generate_report(config, results).await.unwrap();
        let formatted = service
            .format_report(&report, ReportFormat::Dashboard)
            .await;
        assert!(formatted.is_ok());
        let dashboard = formatted.unwrap();
        assert!(dashboard.contains("Analysis Dashboard"));
    }

    // Metadata Building Tests

    #[test]
    fn test_build_metadata() {
        let service = EnhancedReportingService::default();
        let config = ReportConfig {
            project_path: PathBuf::from("/test/my-project"),
            output_format: ReportFormat::Json,
            include_visualizations: false,
            include_executive_summary: true,
            include_recommendations: true,
            confidence_threshold: 80,
            output_path: Some(PathBuf::from("/output/report.json")),
        };
        let results = create_minimal_analysis_results();
        let metadata = service.build_metadata(&config, &results);
        assert!(metadata.is_ok());
        let metadata = metadata.unwrap();
        assert_eq!(metadata.project_name, "my-project");
        assert_eq!(metadata.project_path, "/test/my-project");
        assert_eq!(metadata.analyzed_files, 10);
        assert_eq!(metadata.total_lines, 1000);
        assert!(!metadata.tool_version.is_empty());
    }

    #[test]
    fn test_build_metadata_root_path() {
        let service = EnhancedReportingService::default();
        let config = ReportConfig {
            project_path: PathBuf::from("/"),
            output_format: ReportFormat::Json,
            include_visualizations: false,
            include_executive_summary: true,
            include_recommendations: true,
            confidence_threshold: 80,
            output_path: None,
        };
        let results = create_minimal_analysis_results();
        let metadata = service.build_metadata(&config, &results);
        assert!(metadata.is_ok());
    }

    // Executive Summary Tests

    #[test]
    fn test_generate_executive_summary() {
        let service = EnhancedReportingService::default();
        let results = create_full_analysis_results();
        let summary = service.generate_executive_summary(&results);
        assert!(summary.is_ok());
        let summary = summary.unwrap();
        assert!(summary.overall_health_score >= 0.0);
        assert!(summary.overall_health_score <= 100.0);
    }

    #[test]
    fn test_count_issues_by_severity() {
        let service = EnhancedReportingService::default();
        let results = create_minimal_analysis_results();
        // This is a simplified implementation that returns 0
        let count = service.count_issues_by_severity(&results, Severity::Critical);
        assert_eq!(count, 0);
    }

    // ReportFormat Tests

    #[test]
    fn test_report_format_equality() {
        assert_eq!(ReportFormat::Html, ReportFormat::Html);
        assert_eq!(ReportFormat::Markdown, ReportFormat::Markdown);
        assert_eq!(ReportFormat::Json, ReportFormat::Json);
        assert_eq!(ReportFormat::Pdf, ReportFormat::Pdf);
        assert_eq!(ReportFormat::Dashboard, ReportFormat::Dashboard);
        assert_ne!(ReportFormat::Html, ReportFormat::Markdown);
    }

    #[test]
    fn test_report_format_debug() {
        let format = ReportFormat::Html;
        let debug_str = format!("{:?}", format);
        assert_eq!(debug_str, "Html");
    }

    #[test]
    fn test_report_format_clone() {
        let format = ReportFormat::Markdown;
        let cloned = format.clone();
        assert_eq!(format, cloned);
    }

    // Enum Debug/Serialize Tests

    #[test]
    fn test_risk_level_debug() {
        assert_eq!(format!("{:?}", RiskLevel::Low), "Low");
        assert_eq!(format!("{:?}", RiskLevel::Medium), "Medium");
        assert_eq!(format!("{:?}", RiskLevel::High), "High");
        assert_eq!(format!("{:?}", RiskLevel::Critical), "Critical");
    }

    #[test]
    fn test_trend_debug() {
        assert_eq!(format!("{:?}", Trend::Improving), "Improving");
        assert_eq!(format!("{:?}", Trend::Stable), "Stable");
        assert_eq!(format!("{:?}", Trend::Degrading), "Degrading");
        assert_eq!(format!("{:?}", Trend::Unknown), "Unknown");
    }

    #[test]
    fn test_severity_debug() {
        assert_eq!(format!("{:?}", Severity::Info), "Info");
        assert_eq!(format!("{:?}", Severity::Low), "Low");
        assert_eq!(format!("{:?}", Severity::Medium), "Medium");
        assert_eq!(format!("{:?}", Severity::High), "High");
        assert_eq!(format!("{:?}", Severity::Critical), "Critical");
    }

    #[test]
    fn test_effort_level_debug() {
        assert_eq!(format!("{:?}", EffortLevel::Trivial), "Trivial");
        assert_eq!(format!("{:?}", EffortLevel::Easy), "Easy");
        assert_eq!(format!("{:?}", EffortLevel::Medium), "Medium");
        assert_eq!(format!("{:?}", EffortLevel::Hard), "Hard");
        assert_eq!(format!("{:?}", EffortLevel::VeryHard), "VeryHard");
    }

    #[test]
    fn test_priority_debug() {
        assert_eq!(format!("{:?}", Priority::Low), "Low");
        assert_eq!(format!("{:?}", Priority::Medium), "Medium");
        assert_eq!(format!("{:?}", Priority::High), "High");
        assert_eq!(format!("{:?}", Priority::Critical), "Critical");
    }

    #[test]
    fn test_section_type_debug() {
        assert_eq!(format!("{:?}", SectionType::Complexity), "Complexity");
        assert_eq!(format!("{:?}", SectionType::DeadCode), "DeadCode");
        assert_eq!(format!("{:?}", SectionType::Duplication), "Duplication");
        assert_eq!(format!("{:?}", SectionType::TechnicalDebt), "TechnicalDebt");
        assert_eq!(format!("{:?}", SectionType::Security), "Security");
        assert_eq!(format!("{:?}", SectionType::Performance), "Performance");
        assert_eq!(format!("{:?}", SectionType::BigOAnalysis), "BigOAnalysis");
        assert_eq!(format!("{:?}", SectionType::Dependencies), "Dependencies");
        assert_eq!(format!("{:?}", SectionType::TestCoverage), "TestCoverage");
        assert_eq!(format!("{:?}", SectionType::CodeSmells), "CodeSmells");
    }

    #[test]
    fn test_visualization_type_debug() {
        assert_eq!(format!("{:?}", VisualizationType::LineChart), "LineChart");
        assert_eq!(format!("{:?}", VisualizationType::BarChart), "BarChart");
        assert_eq!(format!("{:?}", VisualizationType::PieChart), "PieChart");
        assert_eq!(format!("{:?}", VisualizationType::HeatMap), "HeatMap");
        assert_eq!(format!("{:?}", VisualizationType::TreeMap), "TreeMap");
        assert_eq!(
            format!("{:?}", VisualizationType::NetworkGraph),
            "NetworkGraph"
        );
        assert_eq!(format!("{:?}", VisualizationType::Table), "Table");
    }

    // Serialization Tests

    #[test]
    fn test_complexity_analysis_serialize() {
        let analysis = ComplexityAnalysis {
            total_cyclomatic: 100,
            total_cognitive: 150,
            functions: 20,
            max_cyclomatic: 15,
            high_complexity_functions: 2,
            distribution: vec![10, 5, 3, 1, 1],
        };
        let json = serde_json::to_string(&analysis);
        assert!(json.is_ok());
        let json = json.unwrap();
        assert!(json.contains("\"total_cyclomatic\":100"));
    }

    #[test]
    fn test_dead_code_analysis_serialize() {
        let analysis = DeadCodeAnalysis {
            dead_lines: 50,
            dead_functions: 5,
            dead_code_percentage: 2.5,
        };
        let json = serde_json::to_string(&analysis);
        assert!(json.is_ok());
    }

    #[test]
    fn test_duplication_analysis_serialize() {
        let analysis = DuplicationAnalysis {
            duplicated_lines: 100,
            duplicate_blocks: 10,
            duplication_percentage: 5.0,
        };
        let json = serde_json::to_string(&analysis);
        assert!(json.is_ok());
    }

    #[test]
    fn test_tdg_analysis_serialize() {
        let analysis = TdgAnalysis {
            average_tdg: 2.5,
            max_tdg: 5.0,
            high_tdg_files: 3,
        };
        let json = serde_json::to_string(&analysis);
        assert!(json.is_ok());
    }

    #[test]
    fn test_big_o_analysis_serialize() {
        let analysis = BigOAnalysis {
            analyzed_functions: 50,
            high_complexity_count: 3,
            complexity_distribution: HashMap::new(),
        };
        let json = serde_json::to_string(&analysis);
        assert!(json.is_ok());
    }

    #[test]
    fn test_unified_analysis_report_serialize() {
        let report = UnifiedAnalysisReport {
            metadata: ReportMetadata {
                project_name: "test".to_string(),
                project_path: "/test".to_string(),
                report_date: "2024-01-01".to_string(),
                tool_version: "1.0.0".to_string(),
                analysis_duration: 10.0,
                analyzed_files: 10,
                total_lines: 1000,
            },
            executive_summary: ExecutiveSummary {
                overall_health_score: 85.0,
                critical_issues: 0,
                high_priority_issues: 2,
                key_findings: vec!["Finding 1".to_string()],
                risk_assessment: RiskLevel::Low,
            },
            sections: Vec::new(),
            recommendations: Vec::new(),
            visualizations: Vec::new(),
        };
        let json = serde_json::to_string(&report);
        assert!(json.is_ok());
    }

    // ReportConfig Tests

    #[test]
    fn test_report_config_clone() {
        let config = create_test_report_config();
        let cloned = config.clone();
        assert_eq!(config.project_path, cloned.project_path);
        assert_eq!(config.output_format, cloned.output_format);
        assert_eq!(config.include_visualizations, cloned.include_visualizations);
    }

    #[test]
    fn test_report_config_debug() {
        let config = create_test_report_config();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ReportConfig"));
    }

    // Location Tests

    #[test]
    fn test_location_serialize() {
        let location = Location {
            file: "test.rs".to_string(),
            line: Some(42),
            column: Some(10),
        };
        let json = serde_json::to_string(&location);
        assert!(json.is_ok());
        let json = json.unwrap();
        assert!(json.contains("\"file\":\"test.rs\""));
        assert!(json.contains("\"line\":42"));
    }

    #[test]
    fn test_location_without_line_column() {
        let location = Location {
            file: "test.rs".to_string(),
            line: None,
            column: None,
        };
        let json = serde_json::to_string(&location);
        assert!(json.is_ok());
    }

    // Finding Tests

    #[test]
    fn test_finding_serialize() {
        let finding = Finding {
            severity: Severity::High,
            category: "Complexity".to_string(),
            description: "High complexity detected".to_string(),
            location: Some(Location {
                file: "test.rs".to_string(),
                line: Some(100),
                column: None,
            }),
            impact: "Maintenance burden".to_string(),
            effort: EffortLevel::Medium,
        };
        let json = serde_json::to_string(&finding);
        assert!(json.is_ok());
    }

    // Recommendation Tests

    #[test]
    fn test_recommendation_serialize() {
        let rec = Recommendation {
            priority: Priority::High,
            category: "Refactoring".to_string(),
            title: "Reduce complexity".to_string(),
            description: "Break down complex functions".to_string(),
            expected_impact: "Better maintainability".to_string(),
            effort: EffortLevel::Medium,
            related_findings: vec!["finding1".to_string()],
        };
        let json = serde_json::to_string(&rec);
        assert!(json.is_ok());
    }

    // Visualization Tests

    #[test]
    fn test_visualization_serialize() {
        let viz = Visualization {
            title: "Test Chart".to_string(),
            viz_type: VisualizationType::BarChart,
            data: serde_json::json!({"labels": ["a", "b"], "values": [1, 2]}),
            config: {
                let mut map = HashMap::new();
                map.insert("key".to_string(), "value".to_string());
                map
            },
        };
        let json = serde_json::to_string(&viz);
        assert!(json.is_ok());
    }

    // MetricValue Tests

    #[test]
    fn test_metric_value_serialize() {
        let metric = MetricValue {
            value: 42.5,
            unit: "lines".to_string(),
            trend: Trend::Improving,
            threshold: Some(50.0),
        };
        let json = serde_json::to_string(&metric);
        assert!(json.is_ok());
