#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/enhanced_reporting.rs
//! Targets: EnhancedReportingService, generate_report, build_metadata,
//! generate_executive_summary, calculate_health_score, extract_key_findings,
//! assess_overall_risk, build_sections, generate_recommendations,
//! create_visualizations, format_as_json, format_as_markdown, format_as_html,
//! format_as_pdf, format_as_dashboard, all struct constructors

use crate::services::enhanced_reporting::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

fn make_analysis_results() -> AnalysisResults {
    AnalysisResults {
        total_duration: Duration::from_secs(5),
        analyzed_files: 100,
        total_lines: 50000,
        complexity_analysis: Some(ComplexityAnalysis {
            total_cyclomatic: 500,
            total_cognitive: 300,
            functions: 200,
            max_cyclomatic: 25,
            high_complexity_functions: 5,
            distribution: vec![100, 50, 30, 15, 5],
        }),
        dead_code_analysis: Some(DeadCodeAnalysis {
            dead_lines: 500,
            dead_functions: 15,
            dead_code_percentage: 1.0,
        }),
        duplication_analysis: Some(DuplicationAnalysis {
            duplicated_lines: 200,
            duplicate_blocks: 25,
            duplication_percentage: 0.4,
        }),
        tdg_analysis: Some(TdgAnalysis {
            average_tdg: 2.5,
            max_tdg: 5.0,
            high_tdg_files: 3,
        }),
        big_o_analysis: Some(BigOAnalysis {
            analyzed_functions: 200,
            high_complexity_count: 8,
            complexity_distribution: {
                let mut m = HashMap::new();
                m.insert("O(1)".to_string(), 100);
                m.insert("O(n)".to_string(), 80);
                m.insert("O(n^2)".to_string(), 15);
                m.insert("O(n^3)".to_string(), 5);
                m
            },
        }),
    }
}

fn make_minimal_analysis_results() -> AnalysisResults {
    AnalysisResults {
        total_duration: Duration::from_millis(100),
        analyzed_files: 1,
        total_lines: 100,
        complexity_analysis: None,
        dead_code_analysis: None,
        duplication_analysis: None,
        tdg_analysis: None,
        big_o_analysis: None,
    }
}

fn make_report_config() -> ReportConfig {
    ReportConfig {
        project_path: PathBuf::from("/test/project"),
        output_format: ReportFormat::Json,
        include_visualizations: true,
        include_executive_summary: true,
        include_recommendations: true,
        confidence_threshold: 80,
        output_path: None,
    }
}

// ==================== Service construction ====================

#[test]
fn test_enhanced_reporting_service_new() {
    let svc = EnhancedReportingService::new();
    assert!(svc.is_ok());
}

#[test]
fn test_enhanced_reporting_service_default() {
    let svc = EnhancedReportingService::default();
    let _ = svc;
}

// ==================== generate_report ====================

#[tokio::test]
async fn test_generate_report_full() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    assert!(!report.metadata.project_name.is_empty());
    assert!(report.executive_summary.overall_health_score <= 100.0);
    assert!(!report.sections.is_empty());
    assert!(!report.visualizations.is_empty());
}

#[tokio::test]
async fn test_generate_report_minimal() {
    let svc = EnhancedReportingService::new().unwrap();
    let mut config = make_report_config();
    config.include_visualizations = false;
    let results = make_minimal_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    assert!(report.sections.is_empty());
    assert!(report.visualizations.is_empty());
    assert!(report.recommendations.is_empty());
}

#[tokio::test]
async fn test_generate_report_without_visualizations() {
    let svc = EnhancedReportingService::new().unwrap();
    let mut config = make_report_config();
    config.include_visualizations = false;
    let results = make_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    assert!(report.visualizations.is_empty());
}

// ==================== format_report ====================

#[tokio::test]
async fn test_format_report_json() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    let json = svc.format_report(&report, ReportFormat::Json).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object());
}

#[tokio::test]
async fn test_format_report_markdown() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    let md = svc.format_report(&report, ReportFormat::Markdown).await.unwrap();
    assert!(md.contains("# "));
    assert!(md.contains("## Executive Summary"));
    assert!(md.contains("## Metadata"));
    assert!(md.contains("## Recommendations"));
}

#[tokio::test]
async fn test_format_report_html() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    let html = svc.format_report(&report, ReportFormat::Html).await.unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html>"));
    assert!(html.contains("</html>"));
    assert!(html.contains("Executive Summary"));
}

#[tokio::test]
async fn test_format_report_pdf() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_minimal_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    let pdf = svc.format_report(&report, ReportFormat::Pdf).await.unwrap();
    assert!(!pdf.is_empty());
}

#[tokio::test]
async fn test_format_report_dashboard() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_minimal_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    let dash = svc.format_report(&report, ReportFormat::Dashboard).await.unwrap();
    assert!(dash.contains("Dashboard"));
}

// ==================== Markdown format deep checks ====================

#[tokio::test]
async fn test_markdown_has_metrics_table() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    let md = svc.format_report(&report, ReportFormat::Markdown).await.unwrap();
    assert!(md.contains("| Metric |"));
}

#[tokio::test]
async fn test_markdown_has_key_findings() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    let md = svc.format_report(&report, ReportFormat::Markdown).await.unwrap();
    assert!(md.contains("Key Findings"));
}

#[tokio::test]
async fn test_markdown_recommendation_priorities() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = make_analysis_results();
    let report = svc.generate_report(config, results).await.unwrap();

    let md = svc.format_report(&report, ReportFormat::Markdown).await.unwrap();
    // Should have priority markers
    assert!(md.contains("HIGH") || md.contains("MEDIUM") || md.contains("LOW") || md.contains("CRITICAL"));
}

// ==================== Analysis data types ====================

#[test]
fn test_complexity_analysis_serde() {
    let ca = ComplexityAnalysis {
        total_cyclomatic: 100,
        total_cognitive: 80,
        functions: 50,
        max_cyclomatic: 30,
        high_complexity_functions: 3,
        distribution: vec![20, 15, 10, 3, 2],
    };
    let json = serde_json::to_string(&ca).unwrap();
    let back: ComplexityAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_cyclomatic, 100);
    assert_eq!(back.distribution.len(), 5);
}

#[test]
fn test_dead_code_analysis_serde() {
    let dca = DeadCodeAnalysis {
        dead_lines: 200,
        dead_functions: 10,
        dead_code_percentage: 2.5,
    };
    let json = serde_json::to_string(&dca).unwrap();
    let back: DeadCodeAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.dead_lines, 200);
}

#[test]
fn test_duplication_analysis_serde() {
    let da = DuplicationAnalysis {
        duplicated_lines: 150,
        duplicate_blocks: 8,
        duplication_percentage: 1.5,
    };
    let json = serde_json::to_string(&da).unwrap();
    let back: DuplicationAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.duplicate_blocks, 8);
}

#[test]
fn test_tdg_analysis_serde() {
    let ta = TdgAnalysis {
        average_tdg: 2.1,
        max_tdg: 4.5,
        high_tdg_files: 2,
    };
    let json = serde_json::to_string(&ta).unwrap();
    let back: TdgAnalysis = serde_json::from_str(&json).unwrap();
    assert!((back.average_tdg - 2.1).abs() < f64::EPSILON);
}

#[test]
fn test_big_o_analysis_serde() {
    let boa = BigOAnalysis {
        analyzed_functions: 100,
        high_complexity_count: 5,
        complexity_distribution: HashMap::new(),
    };
    let json = serde_json::to_string(&boa).unwrap();
    let back: BigOAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.analyzed_functions, 100);
}

// ==================== Enum serde tests ====================

#[test]
fn test_report_format_variants() {
    assert_eq!(ReportFormat::Html, ReportFormat::Html);
    assert_ne!(ReportFormat::Json, ReportFormat::Markdown);
    assert_ne!(ReportFormat::Pdf, ReportFormat::Dashboard);
}

#[test]
fn test_risk_level_serde() {
    for level in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::Critical] {
        let json = serde_json::to_string(&level).unwrap();
        let back: RiskLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", level), format!("{:?}", back));
    }
}

#[test]
fn test_severity_serde() {
    for sev in [Severity::Info, Severity::Low, Severity::Medium, Severity::High, Severity::Critical] {
        let json = serde_json::to_string(&sev).unwrap();
        let back: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", sev), format!("{:?}", back));
    }
}

#[test]
fn test_effort_level_serde() {
    for eff in [EffortLevel::Trivial, EffortLevel::Easy, EffortLevel::Medium, EffortLevel::Hard, EffortLevel::VeryHard] {
        let json = serde_json::to_string(&eff).unwrap();
        let back: EffortLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", eff), format!("{:?}", back));
    }
}

#[test]
fn test_priority_serde() {
    for pri in [Priority::Low, Priority::Medium, Priority::High, Priority::Critical] {
        let json = serde_json::to_string(&pri).unwrap();
        let back: Priority = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", pri), format!("{:?}", back));
    }
}

#[test]
fn test_trend_serde() {
    for trend in [Trend::Improving, Trend::Stable, Trend::Degrading, Trend::Unknown] {
        let json = serde_json::to_string(&trend).unwrap();
        let back: Trend = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", trend), format!("{:?}", back));
    }
}

#[test]
fn test_section_type_serde() {
    for st in [
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
    ] {
        let json = serde_json::to_string(&st).unwrap();
        let back: SectionType = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", st), format!("{:?}", back));
    }
}

#[test]
fn test_visualization_type_serde() {
    for vt in [
        VisualizationType::LineChart,
        VisualizationType::BarChart,
        VisualizationType::PieChart,
        VisualizationType::HeatMap,
        VisualizationType::TreeMap,
        VisualizationType::NetworkGraph,
        VisualizationType::Table,
    ] {
        let json = serde_json::to_string(&vt).unwrap();
        let back: VisualizationType = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", vt), format!("{:?}", back));
    }
}

// ==================== Struct construction ====================

#[test]
fn test_metric_value_construction() {
    let mv = MetricValue {
        value: 42.0,
        unit: "CC".to_string(),
        trend: Trend::Improving,
        threshold: Some(10.0),
    };
    let json = serde_json::to_string(&mv).unwrap();
    let back: MetricValue = serde_json::from_str(&json).unwrap();
    assert!((back.value - 42.0).abs() < f64::EPSILON);
}

#[test]
fn test_finding_construction() {
    let f = Finding {
        severity: Severity::High,
        category: "test".to_string(),
        description: "A test finding".to_string(),
        location: Some(Location {
            file: "src/main.rs".to_string(),
            line: Some(42),
            column: Some(10),
        }),
        impact: "High".to_string(),
        effort: EffortLevel::Medium,
    };
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("src/main.rs"));
}

#[test]
fn test_finding_without_location() {
    let f = Finding {
        severity: Severity::Low,
        category: "misc".to_string(),
        description: "No location".to_string(),
        location: None,
        impact: "Low".to_string(),
        effort: EffortLevel::Trivial,
    };
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("null") || json.contains("location"));
}

#[test]
fn test_recommendation_construction() {
    let rec = Recommendation {
        priority: Priority::Critical,
        category: "security".to_string(),
        title: "Fix vulnerability".to_string(),
        description: "Critical security issue found".to_string(),
        expected_impact: "Prevents data breach".to_string(),
        effort: EffortLevel::Hard,
        related_findings: vec!["CVE-1234".to_string()],
    };
    let json = serde_json::to_string(&rec).unwrap();
    assert!(json.contains("CVE-1234"));
}

#[test]
fn test_visualization_construction() {
    let viz = Visualization {
        title: "Test Chart".to_string(),
        viz_type: VisualizationType::BarChart,
        data: serde_json::json!({"labels": ["a", "b"], "values": [1, 2]}),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&viz).unwrap();
    assert!(json.contains("Test Chart"));
}

#[test]
fn test_report_section_construction() {
    let section = ReportSection {
        title: "Test Section".to_string(),
        section_type: SectionType::Complexity,
        content: serde_json::json!({}),
        metrics: HashMap::new(),
        findings: vec![],
    };
    let json = serde_json::to_string(&section).unwrap();
    assert!(json.contains("Test Section"));
}

#[test]
fn test_report_metadata_construction() {
    let meta = ReportMetadata {
        project_name: "test".to_string(),
        project_path: "/test".to_string(),
        report_date: "2025-01-01".to_string(),
        tool_version: "1.0.0".to_string(),
        analysis_duration: 5.0,
        analyzed_files: 100,
        total_lines: 50000,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let back: ReportMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(back.project_name, "test");
}

#[test]
fn test_executive_summary_construction() {
    let es = ExecutiveSummary {
        overall_health_score: 85.0,
        critical_issues: 2,
        high_priority_issues: 5,
        key_findings: vec!["Finding 1".to_string()],
        risk_assessment: RiskLevel::Medium,
    };
    let json = serde_json::to_string(&es).unwrap();
    assert!(json.contains("85"));
}

#[test]
fn test_unified_analysis_report_construction() {
    let report = UnifiedAnalysisReport {
        metadata: ReportMetadata {
            project_name: "test".to_string(),
            project_path: "/test".to_string(),
            report_date: "2025-01-01".to_string(),
            tool_version: "1.0.0".to_string(),
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
}

// ==================== Health score edge cases ====================

#[tokio::test]
async fn test_health_score_perfect() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = AnalysisResults {
        total_duration: Duration::from_secs(1),
        analyzed_files: 10,
        total_lines: 1000,
        complexity_analysis: Some(ComplexityAnalysis {
            total_cyclomatic: 50,
            total_cognitive: 30,
            functions: 50,
            max_cyclomatic: 5,
            high_complexity_functions: 0,
            distribution: vec![50, 0, 0, 0, 0],
        }),
        dead_code_analysis: Some(DeadCodeAnalysis {
            dead_lines: 0,
            dead_functions: 0,
            dead_code_percentage: 0.0,
        }),
        duplication_analysis: Some(DuplicationAnalysis {
            duplicated_lines: 0,
            duplicate_blocks: 0,
            duplication_percentage: 0.0,
        }),
        tdg_analysis: Some(TdgAnalysis {
            average_tdg: 1.0,
            max_tdg: 2.0,
            high_tdg_files: 0,
        }),
        big_o_analysis: None,
    };
    let report = svc.generate_report(config, results).await.unwrap();
    // Perfect code should have high health score
    assert!(report.executive_summary.overall_health_score >= 90.0);
    assert!(matches!(report.executive_summary.risk_assessment, RiskLevel::Low));
}

#[tokio::test]
async fn test_health_score_terrible() {
    let svc = EnhancedReportingService::new().unwrap();
    let config = make_report_config();
    let results = AnalysisResults {
        total_duration: Duration::from_secs(1),
        analyzed_files: 10,
        total_lines: 1000,
        complexity_analysis: Some(ComplexityAnalysis {
            total_cyclomatic: 3000,
            total_cognitive: 2000,
            functions: 100,
            max_cyclomatic: 50,
            high_complexity_functions: 30,
            distribution: vec![5, 10, 20, 30, 35],
        }),
        dead_code_analysis: Some(DeadCodeAnalysis {
            dead_lines: 300,
            dead_functions: 50,
            dead_code_percentage: 30.0,
        }),
        duplication_analysis: Some(DuplicationAnalysis {
            duplicated_lines: 300,
            duplicate_blocks: 50,
            duplication_percentage: 30.0,
        }),
        tdg_analysis: Some(TdgAnalysis {
            average_tdg: 8.0,
            max_tdg: 10.0,
            high_tdg_files: 20,
        }),
        big_o_analysis: None,
    };
    let report = svc.generate_report(config, results).await.unwrap();
    assert!(report.executive_summary.overall_health_score < 50.0);
}

// ==================== ReportConfig ====================

#[test]
fn test_report_config_construction() {
    let config = ReportConfig {
        project_path: PathBuf::from("/test"),
        output_format: ReportFormat::Markdown,
        include_visualizations: false,
        include_executive_summary: true,
        include_recommendations: false,
        confidence_threshold: 90,
        output_path: Some(PathBuf::from("/output/report.md")),
    };
    assert!(!config.include_visualizations);
    assert_eq!(config.confidence_threshold, 90);
}
