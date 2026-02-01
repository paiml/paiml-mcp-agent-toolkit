//! Coverage boost tests for enhanced_reporting module
//! Tests for ReportFormat, RiskLevel, SectionType, Trend, Severity, EffortLevel, Priority, VisualizationType

use crate::services::enhanced_reporting::{
    AnalysisResults, BigOAnalysis, ComplexityAnalysis, DeadCodeAnalysis, DuplicationAnalysis,
    EffortLevel, ExecutiveSummary, Finding, Location, MetricValue, Priority, ReportConfig,
    ReportFormat, ReportMetadata, ReportSection, RiskLevel, SectionType, Severity, TdgAnalysis,
    Trend, Visualization, VisualizationType,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

// ============ ReportFormat Tests ============

#[test]
fn test_report_format_html() {
    let format = ReportFormat::Html;
    assert_eq!(format, ReportFormat::Html);
}

#[test]
fn test_report_format_markdown() {
    let format = ReportFormat::Markdown;
    assert_eq!(format, ReportFormat::Markdown);
}

#[test]
fn test_report_format_json() {
    let format = ReportFormat::Json;
    assert_eq!(format, ReportFormat::Json);
}

#[test]
fn test_report_format_pdf() {
    let format = ReportFormat::Pdf;
    assert_eq!(format, ReportFormat::Pdf);
}

#[test]
fn test_report_format_dashboard() {
    let format = ReportFormat::Dashboard;
    assert_eq!(format, ReportFormat::Dashboard);
}

#[test]
fn test_report_format_equality() {
    assert_eq!(ReportFormat::Json, ReportFormat::Json);
    assert_ne!(ReportFormat::Json, ReportFormat::Html);
}

#[test]
fn test_report_format_clone() {
    let format = ReportFormat::Markdown;
    let cloned = format.clone();
    assert_eq!(format, cloned);
}

#[test]
fn test_report_format_debug() {
    let format = ReportFormat::Html;
    let debug = format!("{:?}", format);
    assert!(debug.contains("Html"));
}

// ============ RiskLevel Tests ============

#[test]
fn test_risk_level_low() {
    let level = RiskLevel::Low;
    let _ = format!("{:?}", level);
}

#[test]
fn test_risk_level_medium() {
    let level = RiskLevel::Medium;
    let _ = format!("{:?}", level);
}

#[test]
fn test_risk_level_high() {
    let level = RiskLevel::High;
    let _ = format!("{:?}", level);
}

#[test]
fn test_risk_level_critical() {
    let level = RiskLevel::Critical;
    let _ = format!("{:?}", level);
}

// ============ SectionType Tests ============

#[test]
fn test_section_type_complexity() {
    let t = SectionType::Complexity;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_dead_code() {
    let t = SectionType::DeadCode;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_duplication() {
    let t = SectionType::Duplication;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_technical_debt() {
    let t = SectionType::TechnicalDebt;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_security() {
    let t = SectionType::Security;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_performance() {
    let t = SectionType::Performance;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_big_o_analysis() {
    let t = SectionType::BigOAnalysis;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_dependencies() {
    let t = SectionType::Dependencies;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_test_coverage() {
    let t = SectionType::TestCoverage;
    let _ = format!("{:?}", t);
}

#[test]
fn test_section_type_code_smells() {
    let t = SectionType::CodeSmells;
    let _ = format!("{:?}", t);
}

// ============ Trend Tests ============

#[test]
fn test_trend_improving() {
    let trend = Trend::Improving;
    let _ = format!("{:?}", trend);
}

#[test]
fn test_trend_stable() {
    let trend = Trend::Stable;
    let _ = format!("{:?}", trend);
}

#[test]
fn test_trend_degrading() {
    let trend = Trend::Degrading;
    let _ = format!("{:?}", trend);
}

#[test]
fn test_trend_unknown() {
    let trend = Trend::Unknown;
    let _ = format!("{:?}", trend);
}

// ============ Severity Tests ============

#[test]
fn test_severity_info() {
    let sev = Severity::Info;
    let _ = format!("{:?}", sev);
}

#[test]
fn test_severity_low() {
    let sev = Severity::Low;
    let _ = format!("{:?}", sev);
}

#[test]
fn test_severity_medium() {
    let sev = Severity::Medium;
    let _ = format!("{:?}", sev);
}

#[test]
fn test_severity_high() {
    let sev = Severity::High;
    let _ = format!("{:?}", sev);
}

#[test]
fn test_severity_critical() {
    let sev = Severity::Critical;
    let _ = format!("{:?}", sev);
}

// ============ EffortLevel Tests ============

#[test]
fn test_effort_level_trivial() {
    let eff = EffortLevel::Trivial;
    let _ = format!("{:?}", eff);
}

#[test]
fn test_effort_level_easy() {
    let eff = EffortLevel::Easy;
    let _ = format!("{:?}", eff);
}

#[test]
fn test_effort_level_medium() {
    let eff = EffortLevel::Medium;
    let _ = format!("{:?}", eff);
}

#[test]
fn test_effort_level_hard() {
    let eff = EffortLevel::Hard;
    let _ = format!("{:?}", eff);
}

#[test]
fn test_effort_level_very_hard() {
    let eff = EffortLevel::VeryHard;
    let _ = format!("{:?}", eff);
}

// ============ Priority Tests ============

#[test]
fn test_priority_low() {
    let pri = Priority::Low;
    let _ = format!("{:?}", pri);
}

#[test]
fn test_priority_medium() {
    let pri = Priority::Medium;
    let _ = format!("{:?}", pri);
}

#[test]
fn test_priority_high() {
    let pri = Priority::High;
    let _ = format!("{:?}", pri);
}

#[test]
fn test_priority_critical() {
    let pri = Priority::Critical;
    let _ = format!("{:?}", pri);
}

// ============ VisualizationType Tests ============

#[test]
fn test_visualization_type_line_chart() {
    let viz = VisualizationType::LineChart;
    let _ = format!("{:?}", viz);
}

#[test]
fn test_visualization_type_bar_chart() {
    let viz = VisualizationType::BarChart;
    let _ = format!("{:?}", viz);
}

#[test]
fn test_visualization_type_pie_chart() {
    let viz = VisualizationType::PieChart;
    let _ = format!("{:?}", viz);
}

#[test]
fn test_visualization_type_heat_map() {
    let viz = VisualizationType::HeatMap;
    let _ = format!("{:?}", viz);
}

#[test]
fn test_visualization_type_tree_map() {
    let viz = VisualizationType::TreeMap;
    let _ = format!("{:?}", viz);
}

#[test]
fn test_visualization_type_network_graph() {
    let viz = VisualizationType::NetworkGraph;
    let _ = format!("{:?}", viz);
}

#[test]
fn test_visualization_type_table() {
    let viz = VisualizationType::Table;
    let _ = format!("{:?}", viz);
}

// ============ Struct Creation Tests ============

#[test]
fn test_report_config_creation() {
    let config = ReportConfig {
        project_path: PathBuf::from("/test/project"),
        output_format: ReportFormat::Json,
        include_visualizations: true,
        include_executive_summary: true,
        include_recommendations: true,
        confidence_threshold: 80,
        output_path: Some(PathBuf::from("/output/report.json")),
    };
    assert_eq!(config.confidence_threshold, 80);
    assert!(config.include_visualizations);
}

#[test]
fn test_report_config_clone() {
    let config = ReportConfig {
        project_path: PathBuf::from("/test"),
        output_format: ReportFormat::Html,
        include_visualizations: false,
        include_executive_summary: false,
        include_recommendations: false,
        confidence_threshold: 70,
        output_path: None,
    };
    let cloned = config.clone();
    assert_eq!(config.confidence_threshold, cloned.confidence_threshold);
}

#[test]
fn test_metric_value_creation() {
    let metric = MetricValue {
        value: 42.5,
        unit: "ms".to_string(),
        trend: Trend::Improving,
        threshold: Some(100.0),
    };
    assert_eq!(metric.value, 42.5);
    assert_eq!(metric.unit, "ms");
    assert_eq!(metric.threshold, Some(100.0));
}

#[test]
fn test_metric_value_no_threshold() {
    let metric = MetricValue {
        value: 10.0,
        unit: "count".to_string(),
        trend: Trend::Unknown,
        threshold: None,
    };
    assert!(metric.threshold.is_none());
}

#[test]
fn test_location_creation() {
    let loc = Location {
        file: "src/main.rs".to_string(),
        line: Some(42),
        column: Some(10),
    };
    assert_eq!(loc.file, "src/main.rs");
    assert_eq!(loc.line, Some(42));
    assert_eq!(loc.column, Some(10));
}

#[test]
fn test_location_no_position() {
    let loc = Location {
        file: "src/lib.rs".to_string(),
        line: None,
        column: None,
    };
    assert!(loc.line.is_none());
    assert!(loc.column.is_none());
}

#[test]
fn test_finding_creation() {
    let finding = Finding {
        severity: Severity::High,
        category: "Complexity".to_string(),
        description: "High complexity detected".to_string(),
        location: Some(Location {
            file: "test.rs".to_string(),
            line: Some(10),
            column: None,
        }),
        impact: "Maintenance difficulty".to_string(),
        effort: EffortLevel::Medium,
    };
    assert_eq!(finding.category, "Complexity");
    assert!(finding.location.is_some());
}

#[test]
fn test_finding_no_location() {
    let finding = Finding {
        severity: Severity::Info,
        category: "General".to_string(),
        description: "Info message".to_string(),
        location: None,
        impact: "None".to_string(),
        effort: EffortLevel::Trivial,
    };
    assert!(finding.location.is_none());
}

#[test]
fn test_report_metadata_creation() {
    let metadata = ReportMetadata {
        project_name: "test-project".to_string(),
        project_path: "/path/to/project".to_string(),
        report_date: "2025-01-01T00:00:00Z".to_string(),
        tool_version: "1.0.0".to_string(),
        analysis_duration: 5.5,
        analyzed_files: 100,
        total_lines: 10000,
    };
    assert_eq!(metadata.project_name, "test-project");
    assert_eq!(metadata.analyzed_files, 100);
}

#[test]
fn test_executive_summary_creation() {
    let summary = ExecutiveSummary {
        overall_health_score: 85.5,
        critical_issues: 2,
        high_priority_issues: 5,
        key_findings: vec!["Finding 1".to_string(), "Finding 2".to_string()],
        risk_assessment: RiskLevel::Medium,
    };
    assert_eq!(summary.overall_health_score, 85.5);
    assert_eq!(summary.key_findings.len(), 2);
}

#[test]
fn test_report_section_creation() {
    let section = ReportSection {
        title: "Complexity Analysis".to_string(),
        section_type: SectionType::Complexity,
        content: serde_json::json!({"test": "data"}),
        metrics: HashMap::new(),
        findings: vec![],
    };
    assert_eq!(section.title, "Complexity Analysis");
    assert!(section.findings.is_empty());
}

#[test]
fn test_report_section_with_metrics() {
    let mut metrics = HashMap::new();
    metrics.insert(
        "cyclomatic".to_string(),
        MetricValue {
            value: 15.0,
            unit: "CC".to_string(),
            trend: Trend::Stable,
            threshold: Some(20.0),
        },
    );
    let section = ReportSection {
        title: "Metrics Section".to_string(),
        section_type: SectionType::Performance,
        content: serde_json::json!({}),
        metrics,
        findings: vec![],
    };
    assert_eq!(section.metrics.len(), 1);
}

#[test]
fn test_visualization_creation() {
    let viz = Visualization {
        title: "Performance Chart".to_string(),
        viz_type: VisualizationType::LineChart,
        data: serde_json::json!({"values": [1, 2, 3]}),
        config: HashMap::new(),
    };
    assert_eq!(viz.title, "Performance Chart");
}

#[test]
fn test_visualization_with_config() {
    let mut config = HashMap::new();
    config.insert("color".to_string(), "blue".to_string());
    let viz = Visualization {
        title: "Chart".to_string(),
        viz_type: VisualizationType::BarChart,
        data: serde_json::json!({}),
        config,
    };
    assert_eq!(viz.config.len(), 1);
}

// ============ Analysis Result Types Tests ============

#[test]
fn test_complexity_analysis_creation() {
    let analysis = ComplexityAnalysis {
        total_cyclomatic: 100,
        total_cognitive: 80,
        functions: 50,
        max_cyclomatic: 25,
        high_complexity_functions: 5,
        distribution: vec![10, 20, 15, 3, 2],
    };
    assert_eq!(analysis.total_cyclomatic, 100);
    assert_eq!(analysis.functions, 50);
}

#[test]
fn test_dead_code_analysis_creation() {
    let analysis = DeadCodeAnalysis {
        dead_lines: 500,
        dead_functions: 20,
        dead_code_percentage: 5.5,
    };
    assert_eq!(analysis.dead_lines, 500);
    assert_eq!(analysis.dead_code_percentage, 5.5);
}

#[test]
fn test_duplication_analysis_creation() {
    let analysis = DuplicationAnalysis {
        duplicated_lines: 200,
        duplicate_blocks: 15,
        duplication_percentage: 2.5,
    };
    assert_eq!(analysis.duplicated_lines, 200);
    assert_eq!(analysis.duplicate_blocks, 15);
}

#[test]
fn test_tdg_analysis_creation() {
    let analysis = TdgAnalysis {
        average_tdg: 2.5,
        max_tdg: 4.5,
        high_tdg_files: 3,
    };
    assert_eq!(analysis.average_tdg, 2.5);
    assert_eq!(analysis.high_tdg_files, 3);
}

#[test]
fn test_big_o_analysis_creation() {
    let mut distribution = HashMap::new();
    distribution.insert("O(n)".to_string(), 20);
    distribution.insert("O(n^2)".to_string(), 5);

    let analysis = BigOAnalysis {
        analyzed_functions: 50,
        high_complexity_count: 5,
        complexity_distribution: distribution,
    };
    assert_eq!(analysis.analyzed_functions, 50);
    assert_eq!(analysis.complexity_distribution.len(), 2);
}

#[test]
fn test_analysis_results_creation() {
    let results = AnalysisResults {
        total_duration: Duration::from_secs(5),
        analyzed_files: 100,
        total_lines: 10000,
        complexity_analysis: None,
        dead_code_analysis: None,
        duplication_analysis: None,
        tdg_analysis: None,
        big_o_analysis: None,
    };
    assert_eq!(results.analyzed_files, 100);
    assert_eq!(results.total_lines, 10000);
}

#[test]
fn test_analysis_results_with_all_analyses() {
    let results = AnalysisResults {
        total_duration: Duration::from_secs(10),
        analyzed_files: 50,
        total_lines: 5000,
        complexity_analysis: Some(ComplexityAnalysis {
            total_cyclomatic: 50,
            total_cognitive: 40,
            functions: 25,
            max_cyclomatic: 15,
            high_complexity_functions: 2,
            distribution: vec![5, 10, 8, 2],
        }),
        dead_code_analysis: Some(DeadCodeAnalysis {
            dead_lines: 100,
            dead_functions: 5,
            dead_code_percentage: 2.0,
        }),
        duplication_analysis: Some(DuplicationAnalysis {
            duplicated_lines: 50,
            duplicate_blocks: 3,
            duplication_percentage: 1.0,
        }),
        tdg_analysis: Some(TdgAnalysis {
            average_tdg: 1.5,
            max_tdg: 3.0,
            high_tdg_files: 1,
        }),
        big_o_analysis: Some(BigOAnalysis {
            analyzed_functions: 25,
            high_complexity_count: 2,
            complexity_distribution: HashMap::new(),
        }),
    };
    assert!(results.complexity_analysis.is_some());
    assert!(results.dead_code_analysis.is_some());
    assert!(results.duplication_analysis.is_some());
    assert!(results.tdg_analysis.is_some());
    assert!(results.big_o_analysis.is_some());
}

// ============ Serialization Tests ============

#[test]
fn test_complexity_analysis_serialize() {
    let analysis = ComplexityAnalysis {
        total_cyclomatic: 100,
        total_cognitive: 80,
        functions: 50,
        max_cyclomatic: 25,
        high_complexity_functions: 5,
        distribution: vec![10, 20, 15, 3, 2],
    };
    let json = serde_json::to_string(&analysis).unwrap();
    assert!(json.contains("total_cyclomatic"));
}

#[test]
fn test_dead_code_analysis_serialize() {
    let analysis = DeadCodeAnalysis {
        dead_lines: 500,
        dead_functions: 20,
        dead_code_percentage: 5.5,
    };
    let json = serde_json::to_string(&analysis).unwrap();
    assert!(json.contains("dead_lines"));
}

#[test]
fn test_metric_value_serialize() {
    let metric = MetricValue {
        value: 42.5,
        unit: "ms".to_string(),
        trend: Trend::Improving,
        threshold: Some(100.0),
    };
    let json = serde_json::to_string(&metric).unwrap();
    assert!(json.contains("42.5"));
}

#[test]
fn test_location_serialize() {
    let loc = Location {
        file: "test.rs".to_string(),
        line: Some(10),
        column: None,
    };
    let json = serde_json::to_string(&loc).unwrap();
    assert!(json.contains("test.rs"));
}

#[test]
fn test_finding_serialize() {
    let finding = Finding {
        severity: Severity::High,
        category: "Test".to_string(),
        description: "Test finding".to_string(),
        location: None,
        impact: "None".to_string(),
        effort: EffortLevel::Easy,
    };
    let json = serde_json::to_string(&finding).unwrap();
    assert!(json.contains("severity"));
}

// ============ Deserialization Tests ============

#[test]
fn test_complexity_analysis_deserialize() {
    let json = r#"{"total_cyclomatic":100,"total_cognitive":80,"functions":50,"max_cyclomatic":25,"high_complexity_functions":5,"distribution":[10,20]}"#;
    let analysis: ComplexityAnalysis = serde_json::from_str(json).unwrap();
    assert_eq!(analysis.total_cyclomatic, 100);
}

#[test]
fn test_tdg_analysis_deserialize() {
    let json = r#"{"average_tdg":2.5,"max_tdg":4.0,"high_tdg_files":3}"#;
    let analysis: TdgAnalysis = serde_json::from_str(json).unwrap();
    assert_eq!(analysis.average_tdg, 2.5);
}

#[test]
fn test_risk_level_serialization_roundtrip() {
    let level = RiskLevel::High;
    let json = serde_json::to_string(&level).unwrap();
    let deserialized: RiskLevel = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", deserialized);
}

#[test]
fn test_trend_serialization_roundtrip() {
    let trend = Trend::Improving;
    let json = serde_json::to_string(&trend).unwrap();
    let deserialized: Trend = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", deserialized);
}

#[test]
fn test_severity_serialization_roundtrip() {
    let sev = Severity::Critical;
    let json = serde_json::to_string(&sev).unwrap();
    let deserialized: Severity = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", deserialized);
}

#[test]
fn test_effort_level_serialization_roundtrip() {
    let eff = EffortLevel::Hard;
    let json = serde_json::to_string(&eff).unwrap();
    let deserialized: EffortLevel = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", deserialized);
}

#[test]
fn test_priority_serialization_roundtrip() {
    let pri = Priority::High;
    let json = serde_json::to_string(&pri).unwrap();
    let deserialized: Priority = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", deserialized);
}

#[test]
fn test_visualization_type_serialization_roundtrip() {
    let viz = VisualizationType::PieChart;
    let json = serde_json::to_string(&viz).unwrap();
    let deserialized: VisualizationType = serde_json::from_str(&json).unwrap();
    let _ = format!("{:?}", deserialized);
}
