//! Coverage boost tests for services/enhanced_reporting.rs
//! Tests type instantiation, serialization, and Default implementations

use crate::services::enhanced_reporting::{
    AnalysisResults, BigOAnalysis, ComplexityAnalysis, DeadCodeAnalysis, DuplicationAnalysis,
    EffortLevel, EnhancedReportingService, ExecutiveSummary, Finding, Location, MetricValue,
    Priority, Recommendation, ReportConfig, ReportFormat, ReportMetadata, ReportSection,
    RiskLevel, SectionType, Severity, TdgAnalysis, Trend, UnifiedAnalysisReport, Visualization,
    VisualizationType,
};
use std::collections::HashMap;
use std::path::PathBuf;

// --- ReportFormat tests ---

#[test]
fn test_report_format_variants() {
    let formats = vec![
        ReportFormat::Html,
        ReportFormat::Markdown,
        ReportFormat::Json,
        ReportFormat::Pdf,
        ReportFormat::Dashboard,
    ];
    for f in &formats {
        let _ = format!("{:?}", f);
    }
    assert_eq!(ReportFormat::Html, ReportFormat::Html);
    assert_ne!(ReportFormat::Html, ReportFormat::Json);
}

// --- RiskLevel tests ---

#[test]
fn test_risk_level_variants_debug_serde() {
    let levels = vec![
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ];
    for level in &levels {
        let json = serde_json::to_string(level).unwrap();
        let _: RiskLevel = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", level);
    }
}

// --- Severity tests ---

#[test]
fn test_severity_variants_serde() {
    let severities = vec![
        Severity::Info,
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Critical,
    ];
    for sev in &severities {
        let json = serde_json::to_string(sev).unwrap();
        let _: Severity = serde_json::from_str(&json).unwrap();
    }
}

// --- EffortLevel tests ---

#[test]
fn test_effort_level_variants_serde() {
    let efforts = vec![
        EffortLevel::Trivial,
        EffortLevel::Easy,
        EffortLevel::Medium,
        EffortLevel::Hard,
        EffortLevel::VeryHard,
    ];
    for effort in &efforts {
        let json = serde_json::to_string(effort).unwrap();
        let _: EffortLevel = serde_json::from_str(&json).unwrap();
    }
}

// --- Priority tests ---

#[test]
fn test_priority_variants_serde() {
    let priorities = vec![
        Priority::Low,
        Priority::Medium,
        Priority::High,
        Priority::Critical,
    ];
    for p in &priorities {
        let json = serde_json::to_string(p).unwrap();
        let _: Priority = serde_json::from_str(&json).unwrap();
    }
}

// --- Trend tests ---

#[test]
fn test_trend_variants_serde() {
    let trends = vec![
        Trend::Improving,
        Trend::Stable,
        Trend::Degrading,
        Trend::Unknown,
    ];
    for t in &trends {
        let json = serde_json::to_string(t).unwrap();
        let _: Trend = serde_json::from_str(&json).unwrap();
    }
}

// --- SectionType tests ---

#[test]
fn test_section_type_variants_serde() {
    let types = vec![
        SectionType::Complexity,
        SectionType::DeadCode,
        SectionType::Duplication,
        SectionType::TechnicalDebt,
        SectionType::Security,
        SectionType::Performance,
        SectionType::BigOAnalysis,
        SectionType::Dependencies,
        SectionType::TestCoverage,
        SectionType::CodeSmells,
    ];
    for st in &types {
        let json = serde_json::to_string(st).unwrap();
        let _: SectionType = serde_json::from_str(&json).unwrap();
        let _ = format!("{:?}", st);
    }
}

// --- VisualizationType tests ---

#[test]
fn test_visualization_type_variants_serde() {
    let types = vec![
        VisualizationType::LineChart,
        VisualizationType::BarChart,
        VisualizationType::PieChart,
        VisualizationType::HeatMap,
        VisualizationType::TreeMap,
        VisualizationType::NetworkGraph,
        VisualizationType::Table,
    ];
    for vt in &types {
        let json = serde_json::to_string(vt).unwrap();
        let _: VisualizationType = serde_json::from_str(&json).unwrap();
    }
}

// --- ReportMetadata tests ---

#[test]
fn test_report_metadata_construction_serde() {
    let meta = ReportMetadata {
        project_name: "test-project".to_string(),
        project_path: "/tmp/test".to_string(),
        report_date: "2025-01-01".to_string(),
        tool_version: "2.0.0".to_string(),
        analysis_duration: 5.5,
        analyzed_files: 100,
        total_lines: 5000,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let back: ReportMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(back.project_name, "test-project");
    assert_eq!(back.analyzed_files, 100);
}

// --- ExecutiveSummary tests ---

#[test]
fn test_executive_summary_serde() {
    let summary = ExecutiveSummary {
        overall_health_score: 85.0,
        critical_issues: 2,
        high_priority_issues: 5,
        key_findings: vec!["High complexity in module X".to_string()],
        risk_assessment: RiskLevel::Medium,
    };
    let json = serde_json::to_string(&summary).unwrap();
    let back: ExecutiveSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.critical_issues, 2);
    assert_eq!(back.overall_health_score, 85.0);
}

// --- MetricValue tests ---

#[test]
fn test_metric_value_serde() {
    let mv = MetricValue {
        value: 42.0,
        unit: "percent".to_string(),
        trend: Trend::Improving,
        threshold: Some(50.0),
    };
    let json = serde_json::to_string(&mv).unwrap();
    let back: MetricValue = serde_json::from_str(&json).unwrap();
    assert_eq!(back.value, 42.0);
    assert_eq!(back.unit, "percent");
}

#[test]
fn test_metric_value_no_threshold() {
    let mv = MetricValue {
        value: 10.0,
        unit: "count".to_string(),
        trend: Trend::Stable,
        threshold: None,
    };
    let json = serde_json::to_string(&mv).unwrap();
    assert!(json.contains("null") || !json.contains("threshold"));
}

// --- Location tests ---

#[test]
fn test_location_serde() {
    let loc = Location {
        file: "src/main.rs".to_string(),
        line: Some(42),
        column: Some(5),
    };
    let json = serde_json::to_string(&loc).unwrap();
    let back: Location = serde_json::from_str(&json).unwrap();
    assert_eq!(back.file, "src/main.rs");
    assert_eq!(back.line, Some(42));
}

#[test]
fn test_location_no_line_column() {
    let loc = Location {
        file: "src/lib.rs".to_string(),
        line: None,
        column: None,
    };
    let json = serde_json::to_string(&loc).unwrap();
    let back: Location = serde_json::from_str(&json).unwrap();
    assert!(back.line.is_none());
}

// --- Finding tests ---

#[test]
fn test_finding_serde() {
    let finding = Finding {
        severity: Severity::High,
        category: "complexity".to_string(),
        description: "Function too complex".to_string(),
        location: Some(Location {
            file: "src/main.rs".to_string(),
            line: Some(10),
            column: None,
        }),
        impact: "Reduces maintainability".to_string(),
        effort: EffortLevel::Medium,
    };
    let json = serde_json::to_string(&finding).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.category, "complexity");
}

#[test]
fn test_finding_no_location() {
    let finding = Finding {
        severity: Severity::Info,
        category: "general".to_string(),
        description: "Overall observation".to_string(),
        location: None,
        impact: "Low".to_string(),
        effort: EffortLevel::Trivial,
    };
    let json = serde_json::to_string(&finding).unwrap();
    let _: Finding = serde_json::from_str(&json).unwrap();
}

// --- Recommendation tests ---

#[test]
fn test_recommendation_serde() {
    let rec = Recommendation {
        priority: Priority::High,
        category: "refactoring".to_string(),
        title: "Extract method".to_string(),
        description: "Large function should be split".to_string(),
        expected_impact: "Reduce complexity by 50%".to_string(),
        effort: EffortLevel::Easy,
        related_findings: vec!["finding-1".to_string()],
    };
    let json = serde_json::to_string(&rec).unwrap();
    let back: Recommendation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, "Extract method");
}

// --- Visualization tests ---

#[test]
fn test_visualization_serde() {
    let viz = Visualization {
        title: "Complexity Heatmap".to_string(),
        viz_type: VisualizationType::HeatMap,
        data: serde_json::json!({"values": [1, 2, 3]}),
        config: {
            let mut m = HashMap::new();
            m.insert("color".to_string(), "red".to_string());
            m
        },
    };
    let json = serde_json::to_string(&viz).unwrap();
    let back: Visualization = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, "Complexity Heatmap");
}

// --- ReportSection tests ---

#[test]
fn test_report_section_serde() {
    let section = ReportSection {
        title: "Complexity Analysis".to_string(),
        section_type: SectionType::Complexity,
        content: serde_json::json!({"avg": 5.0}),
        metrics: {
            let mut m = HashMap::new();
            m.insert(
                "avg_complexity".to_string(),
                MetricValue {
                    value: 5.0,
                    unit: "cyclomatic".to_string(),
                    trend: Trend::Stable,
                    threshold: Some(10.0),
                },
            );
            m
        },
        findings: vec![Finding {
            severity: Severity::Medium,
            category: "complexity".to_string(),
            description: "Moderate complexity".to_string(),
            location: None,
            impact: "Low".to_string(),
            effort: EffortLevel::Easy,
        }],
    };
    let json = serde_json::to_string(&section).unwrap();
    let back: ReportSection = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, "Complexity Analysis");
    assert_eq!(back.findings.len(), 1);
}

// --- UnifiedAnalysisReport tests ---

#[test]
fn test_unified_analysis_report_serde() {
    let report = UnifiedAnalysisReport {
        metadata: ReportMetadata {
            project_name: "test".to_string(),
            project_path: ".".to_string(),
            report_date: "2025-01-01".to_string(),
            tool_version: "2.0".to_string(),
            analysis_duration: 1.0,
            analyzed_files: 10,
            total_lines: 1000,
        },
        executive_summary: ExecutiveSummary {
            overall_health_score: 90.0,
            critical_issues: 0,
            high_priority_issues: 1,
            key_findings: vec![],
            risk_assessment: RiskLevel::Low,
        },
        sections: vec![],
        recommendations: vec![],
        visualizations: vec![],
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: UnifiedAnalysisReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.metadata.project_name, "test");
    assert_eq!(back.executive_summary.overall_health_score, 90.0);
}

// --- ReportConfig tests ---

#[test]
fn test_report_config_construction() {
    let config = ReportConfig {
        project_path: PathBuf::from("."),
        output_format: ReportFormat::Json,
        include_visualizations: true,
        include_executive_summary: true,
        include_recommendations: false,
        confidence_threshold: 80,
        output_path: Some(PathBuf::from("report.json")),
    };
    let _ = format!("{:?}", config);
    assert_eq!(config.confidence_threshold, 80);
    assert!(config.include_visualizations);
}

#[test]
fn test_report_config_clone() {
    let config = ReportConfig {
        project_path: PathBuf::from("."),
        output_format: ReportFormat::Markdown,
        include_visualizations: false,
        include_executive_summary: false,
        include_recommendations: true,
        confidence_threshold: 50,
        output_path: None,
    };
    let cloned = config.clone();
    assert_eq!(cloned.confidence_threshold, 50);
    assert!(cloned.output_path.is_none());
}

// --- Analysis result type tests ---

#[test]
fn test_complexity_analysis_serde() {
    let ca = ComplexityAnalysis {
        total_cyclomatic: 100,
        total_cognitive: 200,
        functions: 50,
        max_cyclomatic: 25,
        high_complexity_functions: 5,
        distribution: vec![10, 20, 15, 5],
    };
    let json = serde_json::to_string(&ca).unwrap();
    let back: ComplexityAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_cyclomatic, 100);
    assert_eq!(back.functions, 50);
}

#[test]
fn test_dead_code_analysis_serde() {
    let dca = DeadCodeAnalysis {
        dead_lines: 100,
        dead_functions: 5,
        dead_code_percentage: 3.5,
    };
    let json = serde_json::to_string(&dca).unwrap();
    let back: DeadCodeAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.dead_lines, 100);
}

#[test]
fn test_duplication_analysis_serde() {
    let da = DuplicationAnalysis {
        duplicated_lines: 200,
        duplicate_blocks: 10,
        duplication_percentage: 5.0,
    };
    let json = serde_json::to_string(&da).unwrap();
    let back: DuplicationAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.duplicate_blocks, 10);
}

#[test]
fn test_tdg_analysis_serde() {
    let tdg = TdgAnalysis {
        average_tdg: 3.5,
        max_tdg: 8.0,
        high_tdg_files: 2,
    };
    let json = serde_json::to_string(&tdg).unwrap();
    let back: TdgAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.high_tdg_files, 2);
}

#[test]
fn test_big_o_analysis_serde() {
    let mut dist = HashMap::new();
    dist.insert("O(n)".to_string(), 10);
    dist.insert("O(n^2)".to_string(), 3);
    let boa = BigOAnalysis {
        analyzed_functions: 50,
        high_complexity_count: 3,
        complexity_distribution: dist,
    };
    let json = serde_json::to_string(&boa).unwrap();
    let back: BigOAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.analyzed_functions, 50);
}

#[test]
fn test_analysis_results_debug() {
    let results = AnalysisResults {
        total_duration: std::time::Duration::from_secs(5),
        analyzed_files: 100,
        total_lines: 5000,
        complexity_analysis: None,
        dead_code_analysis: None,
        duplication_analysis: None,
        tdg_analysis: None,
        big_o_analysis: None,
    };
    let _ = format!("{:?}", results);
    assert_eq!(results.analyzed_files, 100);
}

#[test]
fn test_analysis_results_with_all_analyses() {
    let results = AnalysisResults {
        total_duration: std::time::Duration::from_millis(500),
        analyzed_files: 50,
        total_lines: 2000,
        complexity_analysis: Some(ComplexityAnalysis {
            total_cyclomatic: 50,
            total_cognitive: 80,
            functions: 20,
            max_cyclomatic: 15,
            high_complexity_functions: 2,
            distribution: vec![5, 10, 5],
        }),
        dead_code_analysis: Some(DeadCodeAnalysis {
            dead_lines: 50,
            dead_functions: 2,
            dead_code_percentage: 2.5,
        }),
        duplication_analysis: Some(DuplicationAnalysis {
            duplicated_lines: 30,
            duplicate_blocks: 3,
            duplication_percentage: 1.5,
        }),
        tdg_analysis: Some(TdgAnalysis {
            average_tdg: 4.0,
            max_tdg: 7.0,
            high_tdg_files: 1,
        }),
        big_o_analysis: Some(BigOAnalysis {
            analyzed_functions: 20,
            high_complexity_count: 1,
            complexity_distribution: HashMap::new(),
        }),
    };
    let _ = format!("{:?}", results);
    assert!(results.complexity_analysis.is_some());
    assert!(results.big_o_analysis.is_some());
}

// --- EnhancedReportingService tests ---

#[test]
fn test_enhanced_reporting_service_new() {
    let service = EnhancedReportingService::new();
    assert!(service.is_ok());
}

#[test]
fn test_enhanced_reporting_service_default() {
    let service = EnhancedReportingService::default();
    let _ = format!("{:p}", &service);
}
