            analyzed_files: 50,
            total_lines: 10000,
            complexity_analysis: Some(ComplexityAnalysis {
                total_cyclomatic: 2000,
                total_cognitive: 3000,
                functions: 50,
                max_cyclomatic: 30,
                high_complexity_functions: 15,
                distribution: vec![10, 15, 10, 10, 5],
            }),
            dead_code_analysis: Some(DeadCodeAnalysis {
                dead_lines: 1500,
                dead_functions: 20,
                dead_code_percentage: 15.0,
            }),
            duplication_analysis: None,
            tdg_analysis: None,
            big_o_analysis: None,
        };
        let risk = service.assess_overall_risk(&results);
        assert!(matches!(risk, RiskLevel::Medium | RiskLevel::High));
    }

    #[test]
    fn test_risk_assessment_critical() {
        let service = EnhancedReportingService::default();
        let results = create_unhealthy_analysis_results();
        let risk = service.assess_overall_risk(&results);
        assert!(matches!(risk, RiskLevel::Critical | RiskLevel::High));
    }

    // Key Findings Extraction Tests

    #[test]
    fn test_extract_key_findings_empty() {
        let service = EnhancedReportingService::default();
        let results = create_minimal_analysis_results();
        let findings = service.extract_key_findings(&results);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_extract_key_findings_high_complexity() {
        let service = EnhancedReportingService::default();
        let results = AnalysisResults {
            total_duration: Duration::from_secs(10),
            analyzed_files: 50,
            total_lines: 10000,
            complexity_analysis: Some(ComplexityAnalysis {
                total_cyclomatic: 1000,
                total_cognitive: 1500,
                functions: 50,
                max_cyclomatic: 25, // > 20, triggers finding
                high_complexity_functions: 10,
                distribution: vec![20, 15, 10, 3, 2],
            }),
            dead_code_analysis: None,
            duplication_analysis: None,
            tdg_analysis: None,
            big_o_analysis: None,
        };
        let findings = service.extract_key_findings(&results);
        assert!(!findings.is_empty());
        assert!(findings[0].contains("high complexity"));
    }

    #[test]
    fn test_extract_key_findings_dead_functions() {
        let service = EnhancedReportingService::default();
        let results = AnalysisResults {
            total_duration: Duration::from_secs(10),
            analyzed_files: 50,
            total_lines: 10000,
            complexity_analysis: None,
            dead_code_analysis: Some(DeadCodeAnalysis {
                dead_lines: 500,
                dead_functions: 15, // > 10, triggers finding
                dead_code_percentage: 5.0,
            }),
            duplication_analysis: None,
            tdg_analysis: None,
            big_o_analysis: None,
        };
        let findings = service.extract_key_findings(&results);
        assert!(!findings.is_empty());
        assert!(findings[0].contains("unused functions"));
    }

    #[test]
    fn test_extract_key_findings_duplicate_blocks() {
        let service = EnhancedReportingService::default();
        let results = AnalysisResults {
            total_duration: Duration::from_secs(10),
            analyzed_files: 50,
            total_lines: 10000,
            complexity_analysis: None,
            dead_code_analysis: None,
            duplication_analysis: Some(DuplicationAnalysis {
                duplicated_lines: 1000,
                duplicate_blocks: 25, // > 20, triggers finding
                duplication_percentage: 10.0,
            }),
            tdg_analysis: None,
            big_o_analysis: None,
        };
        let findings = service.extract_key_findings(&results);
        assert!(!findings.is_empty());
        assert!(findings[0].contains("duplicate code blocks"));
    }

    #[test]
    fn test_extract_key_findings_all_issues() {
        let service = EnhancedReportingService::default();
        let results = create_full_analysis_results();
        let findings = service.extract_key_findings(&results);
        // Should find complexity and duplication issues
        assert!(findings.len() >= 2);
    }

    // Section Building Tests

    #[test]
    fn test_build_complexity_section() {
        let service = EnhancedReportingService::default();
        let complexity = ComplexityAnalysis {
            total_cyclomatic: 500,
            total_cognitive: 800,
            functions: 50,
            max_cyclomatic: 25,
            high_complexity_functions: 10,
            distribution: vec![20, 15, 10, 3, 2],
        };
        let section = service.build_complexity_section(&complexity);
        assert!(section.is_ok());
        let section = section.unwrap();
        assert_eq!(section.title, "Code Complexity Analysis");
        assert!(matches!(section.section_type, SectionType::Complexity));
        assert!(section.metrics.contains_key("total_cyclomatic"));
        assert!(section.metrics.contains_key("average_cyclomatic"));
        assert!(!section.findings.is_empty());
    }

    #[test]
    fn test_build_complexity_section_low_complexity() {
        let service = EnhancedReportingService::default();
        let complexity = ComplexityAnalysis {
            total_cyclomatic: 100,
            total_cognitive: 150,
            functions: 50,
            max_cyclomatic: 15, // <= 20, should be Medium severity
            high_complexity_functions: 0,
            distribution: vec![40, 10, 0, 0, 0],
        };
        let section = service.build_complexity_section(&complexity).unwrap();
        assert!(matches!(section.findings[0].severity, Severity::Medium));
    }

    #[test]
    fn test_build_dead_code_section() {
        let service = EnhancedReportingService::default();
        let dead_code = DeadCodeAnalysis {
            dead_lines: 200,
            dead_functions: 10,
            dead_code_percentage: 2.0,
        };
        let section = service.build_dead_code_section(&dead_code);
        assert!(section.is_ok());
        let section = section.unwrap();
        assert_eq!(section.title, "Dead Code Analysis");
        assert!(matches!(section.section_type, SectionType::DeadCode));
        assert!(section.metrics.contains_key("dead_code_ratio"));
    }

    #[test]
    fn test_build_duplication_section() {
        let service = EnhancedReportingService::default();
        let duplication = DuplicationAnalysis {
            duplicated_lines: 500,
            duplicate_blocks: 20,
            duplication_percentage: 5.0,
        };
        let section = service.build_duplication_section(&duplication);
        assert!(section.is_ok());
        let section = section.unwrap();
        assert_eq!(section.title, "Code Duplication Analysis");
        assert!(matches!(section.section_type, SectionType::Duplication));
        assert!(section.metrics.contains_key("duplication_ratio"));
    }

    #[test]
    fn test_build_tdg_section() {
        let service = EnhancedReportingService::default();
        let tdg = TdgAnalysis {
            average_tdg: 3.5,
            max_tdg: 6.0,
            high_tdg_files: 8,
        };
        let section = service.build_tdg_section(&tdg);
        assert!(section.is_ok());
        let section = section.unwrap();
        assert_eq!(section.title, "Code Quality Gradient");
        assert!(matches!(section.section_type, SectionType::TechnicalDebt));
        assert!(section.metrics.contains_key("average_tdg"));
    }

    #[test]
    fn test_build_big_o_section() {
        let service = EnhancedReportingService::default();
        let big_o = BigOAnalysis {
            analyzed_functions: 80,
            high_complexity_count: 5,
            complexity_distribution: {
                let mut map = HashMap::new();
                map.insert("O(1)".to_string(), 30);
                map.insert("O(n)".to_string(), 35);
                map
            },
        };
        let section = service.build_big_o_section(&big_o);
        assert!(section.is_ok());
        let section = section.unwrap();
        assert_eq!(section.title, "Algorithmic Complexity Analysis");
        assert!(matches!(section.section_type, SectionType::BigOAnalysis));
        assert!(section.metrics.contains_key("high_complexity_functions"));
    }

    #[test]
    fn test_build_sections_empty() {
        let service = EnhancedReportingService::default();
        let config = create_test_report_config();
        let results = create_minimal_analysis_results();
        let sections = service.build_sections(&results, &config);
        assert!(sections.is_ok());
        assert!(sections.unwrap().is_empty());
    }

    #[test]
    fn test_build_sections_full() {
        let service = EnhancedReportingService::default();
        let config = create_test_report_config();
        let results = create_full_analysis_results();
        let sections = service.build_sections(&results, &config);
        assert!(sections.is_ok());
        let sections = sections.unwrap();
        // Should have 5 sections: complexity, dead_code, duplication, tdg, big_o
        assert_eq!(sections.len(), 5);
    }

    // Recommendation Generation Tests

    #[test]
    fn test_generate_recommendations_empty() {
        let service = EnhancedReportingService::default();
        let results = create_minimal_analysis_results();
        let sections = Vec::new();
        let recommendations = service.generate_recommendations(&results, &sections);
        assert!(recommendations.is_ok());
        assert!(recommendations.unwrap().is_empty());
    }

    #[test]
    fn test_generate_recommendations_high_complexity() {
        let service = EnhancedReportingService::default();
        let results = AnalysisResults {
            total_duration: Duration::from_secs(10),
            analyzed_files: 50,
            total_lines: 10000,
            complexity_analysis: Some(ComplexityAnalysis {
                total_cyclomatic: 1000,
                total_cognitive: 1500,
                functions: 50,
                max_cyclomatic: 25, // > 20, triggers recommendation
                high_complexity_functions: 10,
                distribution: vec![20, 15, 10, 3, 2],
            }),
            dead_code_analysis: None,
            duplication_analysis: None,
            tdg_analysis: None,
            big_o_analysis: None,
        };
        let sections = Vec::new();
        let recommendations = service
            .generate_recommendations(&results, &sections)
            .unwrap();
        assert!(!recommendations.is_empty());
        assert!(matches!(recommendations[0].priority, Priority::High));
        assert_eq!(recommendations[0].category, "Complexity");
    }

    #[test]
    fn test_generate_recommendations_dead_code() {
        let service = EnhancedReportingService::default();
        let results = AnalysisResults {
            total_duration: Duration::from_secs(10),
            analyzed_files: 50,
            total_lines: 10000,
            complexity_analysis: None,
            dead_code_analysis: Some(DeadCodeAnalysis {
                dead_lines: 500,
                dead_functions: 15, // > 10, triggers recommendation
                dead_code_percentage: 5.0,
            }),
            duplication_analysis: None,
            tdg_analysis: None,
            big_o_analysis: None,
        };
        let sections = Vec::new();
        let recommendations = service
            .generate_recommendations(&results, &sections)
            .unwrap();
        assert!(!recommendations.is_empty());
        assert!(matches!(recommendations[0].priority, Priority::Medium));
        assert_eq!(recommendations[0].category, "Dead Code");
    }

    #[test]
    fn test_generate_recommendations_all_issues() {
        let service = EnhancedReportingService::default();
        let results = AnalysisResults {
            total_duration: Duration::from_secs(10),
            analyzed_files: 50,
            total_lines: 10000,
            complexity_analysis: Some(ComplexityAnalysis {
                total_cyclomatic: 1000,
                total_cognitive: 1500,
                functions: 50,
                max_cyclomatic: 30,
                high_complexity_functions: 15,
                distribution: vec![10, 10, 15, 10, 5],
            }),
            dead_code_analysis: Some(DeadCodeAnalysis {
                dead_lines: 500,
                dead_functions: 20,
                dead_code_percentage: 5.0,
            }),
            duplication_analysis: None,
            tdg_analysis: None,
            big_o_analysis: None,
        };
        let sections = Vec::new();
        let recommendations = service
            .generate_recommendations(&results, &sections)
            .unwrap();
        assert_eq!(recommendations.len(), 2);
    }

    // Visualization Tests

    #[test]
    fn test_create_complexity_distribution_chart() {
        let service = EnhancedReportingService::default();
        let complexity = ComplexityAnalysis {
            total_cyclomatic: 500,
            total_cognitive: 800,
            functions: 50,
            max_cyclomatic: 25,
            high_complexity_functions: 10,
            distribution: vec![20, 15, 10, 3, 2],
        };
        let viz = service.create_complexity_distribution_chart(&complexity);
        assert!(viz.is_ok());
        let viz = viz.unwrap();
        assert_eq!(viz.title, "Complexity Distribution");
        assert!(matches!(viz.viz_type, VisualizationType::BarChart));
    }

    #[test]
    fn test_create_health_score_gauge() {
        let service = EnhancedReportingService::default();
        let viz = service.create_health_score_gauge(85.5);
        assert!(viz.is_ok());
        let viz = viz.unwrap();
        assert_eq!(viz.title, "Overall Health Score");
        assert!(viz.data.get("value").is_some());
    }

    #[test]
    fn test_create_issue_distribution_chart() {
        let service = EnhancedReportingService::default();
        let sections = vec![ReportSection {
            title: "Test Section".to_string(),
            section_type: SectionType::Complexity,
            content: serde_json::json!({}),
            metrics: HashMap::new(),
            findings: vec![Finding {
                severity: Severity::High,
                category: "Test".to_string(),
                description: "Test finding".to_string(),
                location: None,
                impact: "Test impact".to_string(),
                effort: EffortLevel::Medium,
            }],
        }];
        let viz = service.create_issue_distribution_chart(&sections);
        assert!(viz.is_ok());
        let viz = viz.unwrap();
        assert_eq!(viz.title, "Issue Distribution by Category");
        assert!(matches!(viz.viz_type, VisualizationType::PieChart));
    }

    #[test]
    fn test_create_visualizations() {
        let service = EnhancedReportingService::default();
        let results = create_full_analysis_results();
        let config = create_test_report_config();
        let sections = service.build_sections(&results, &config).unwrap();
        let visualizations = service.create_visualizations(&results, &sections);
        assert!(visualizations.is_ok());
        let visualizations = visualizations.unwrap();
        // Should have at least: complexity distribution, health gauge, issue distribution
        assert!(visualizations.len() >= 3);
    }

    // Report Generation Tests (Async)

    #[tokio::test]
    async fn test_generate_report_minimal() {
        let service = EnhancedReportingService::default();
        let config = ReportConfig {
            project_path: PathBuf::from("/test/project"),
            output_format: ReportFormat::Json,
            include_visualizations: false,
            include_executive_summary: true,
            include_recommendations: true,
            confidence_threshold: 80,
            output_path: None,
        };
        let results = create_minimal_analysis_results();
        let report = service.generate_report(config, results).await;
        assert!(report.is_ok());
        let report = report.unwrap();
        assert_eq!(report.metadata.project_name, "project");
        assert!(report.sections.is_empty());
        assert!(report.visualizations.is_empty());
    }

    #[tokio::test]
    async fn test_generate_report_full() {
        let service = EnhancedReportingService::default();
        let config = create_test_report_config();
        let results = create_full_analysis_results();
        let report = service.generate_report(config, results).await;
        assert!(report.is_ok());
        let report = report.unwrap();
        assert!(!report.sections.is_empty());
        assert!(!report.visualizations.is_empty());
    }

    #[tokio::test]
    async fn test_generate_report_without_visualizations() {
