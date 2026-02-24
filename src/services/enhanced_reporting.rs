//! Enhanced Reporting System - Phase 6 Day 14-15
//!
//! Provides a unified reporting framework that consolidates multiple analysis
//! outputs into comprehensive, multi-format reports with visualizations.

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

// Types: structs, enums, analysis result types, Default impl
include!("enhanced_reporting_types.rs");

// Core impl: new, generate_report, build_metadata, executive summary, health score
include!("enhanced_reporting_generation.rs");

// Section building: build_sections, build_*_section, generate_recommendations
include!("enhanced_reporting_sections.rs");

// Visualization creation + format_report + format_as_* methods
include!("enhanced_reporting_formatting.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_serde_roundtrip() {
        for level in [
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Critical,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let back: RiskLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(
                std::mem::discriminant(&level),
                std::mem::discriminant(&back)
            );
        }
    }

    #[test]
    fn test_section_type_serde_roundtrip() {
        for st in [
            SectionType::Complexity,
            SectionType::DeadCode,
            SectionType::Duplication,
            SectionType::TechnicalDebt,
            SectionType::BigOAnalysis,
            SectionType::Security,
            SectionType::Performance,
            SectionType::Dependencies,
            SectionType::TestCoverage,
            SectionType::CodeSmells,
        ] {
            let json = serde_json::to_string(&st).unwrap();
            let _back: SectionType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_trend_serde_roundtrip() {
        for t in [Trend::Improving, Trend::Stable, Trend::Degrading] {
            let json = serde_json::to_string(&t).unwrap();
            let back: Trend = serde_json::from_str(&json).unwrap();
            assert_eq!(std::mem::discriminant(&t), std::mem::discriminant(&back));
        }
    }

    #[test]
    fn test_severity_serde_roundtrip() {
        for s in [
            Severity::Low,
            Severity::Medium,
            Severity::High,
            Severity::Critical,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: Severity = serde_json::from_str(&json).unwrap();
            assert_eq!(std::mem::discriminant(&s), std::mem::discriminant(&back));
        }
    }

    #[test]
    fn test_effort_level_serde_roundtrip() {
        for e in [
            EffortLevel::Trivial,
            EffortLevel::Easy,
            EffortLevel::Medium,
            EffortLevel::Hard,
            EffortLevel::VeryHard,
        ] {
            let json = serde_json::to_string(&e).unwrap();
            let back: EffortLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(std::mem::discriminant(&e), std::mem::discriminant(&back));
        }
    }

    #[test]
    fn test_priority_serde_roundtrip() {
        for p in [
            Priority::Low,
            Priority::Medium,
            Priority::High,
            Priority::Critical,
        ] {
            let json = serde_json::to_string(&p).unwrap();
            let back: Priority = serde_json::from_str(&json).unwrap();
            assert_eq!(std::mem::discriminant(&p), std::mem::discriminant(&back));
        }
    }

    #[test]
    fn test_visualization_type_serde_roundtrip() {
        for vt in [
            VisualizationType::BarChart,
            VisualizationType::PieChart,
            VisualizationType::LineChart,
            VisualizationType::HeatMap,
            VisualizationType::TreeMap,
            VisualizationType::NetworkGraph,
            VisualizationType::Table,
        ] {
            let json = serde_json::to_string(&vt).unwrap();
            let _back: VisualizationType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_metric_value_serde() {
        let mv = MetricValue {
            value: 42.5,
            unit: "percent".to_string(),
            trend: Trend::Improving,
            threshold: Some(50.0),
        };
        let json = serde_json::to_string(&mv).unwrap();
        let back: MetricValue = serde_json::from_str(&json).unwrap();
        assert_eq!(back.value, 42.5);
        assert!(matches!(back.trend, Trend::Improving));
        assert_eq!(back.threshold, Some(50.0));
    }

    #[test]
    fn test_finding_serde() {
        let f = Finding {
            severity: Severity::High,
            category: "complexity".to_string(),
            description: "Function too complex".to_string(),
            location: Some(Location {
                file: "src/main.rs".to_string(),
                line: Some(42),
                column: Some(1),
            }),
            impact: "Hard to maintain".to_string(),
            effort: EffortLevel::Medium,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: Finding = serde_json::from_str(&json).unwrap();
        assert_eq!(back.category, "complexity");
        assert!(back.location.is_some());
    }

    #[test]
    fn test_recommendation_serde() {
        let r = Recommendation {
            priority: Priority::High,
            category: "testing".to_string(),
            title: "Increase coverage".to_string(),
            description: "Add unit tests".to_string(),
            effort: EffortLevel::Medium,
            expected_impact: "Better quality".to_string(),
            related_findings: vec!["finding-1".to_string()],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: Recommendation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Increase coverage");
        assert_eq!(back.related_findings.len(), 1);
    }

    #[test]
    fn test_report_format_equality() {
        assert_eq!(ReportFormat::Json, ReportFormat::Json);
        assert_eq!(ReportFormat::Markdown, ReportFormat::Markdown);
        assert_eq!(ReportFormat::Html, ReportFormat::Html);
        assert_ne!(ReportFormat::Json, ReportFormat::Markdown);
    }

    #[test]
    fn test_report_config_construction() {
        let config = ReportConfig {
            project_path: PathBuf::from("/test"),
            output_format: ReportFormat::Json,
            include_visualizations: true,
            include_executive_summary: true,
            include_recommendations: true,
            confidence_threshold: 80,
            output_path: None,
        };
        assert_eq!(config.confidence_threshold, 80);
        assert!(config.include_visualizations);
    }

    #[test]
    fn test_enhanced_reporting_service_default() {
        let service = EnhancedReportingService::default();
        let _ = format!("{:?}", "service created");
        drop(service);
    }

    #[test]
    fn test_executive_summary_serde() {
        let summary = ExecutiveSummary {
            overall_health_score: 85.0,
            critical_issues: 0,
            high_priority_issues: 2,
            key_findings: vec!["Good coverage".to_string()],
            risk_assessment: RiskLevel::Low,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: ExecutiveSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.overall_health_score, 85.0);
        assert!(matches!(back.risk_assessment, RiskLevel::Low));
    }

    #[test]
    fn test_visualization_serde() {
        let viz = Visualization {
            title: "Test Chart".to_string(),
            viz_type: VisualizationType::BarChart,
            data: serde_json::json!({"values": [1, 2, 3]}),
            config: HashMap::new(),
        };
        let json = serde_json::to_string(&viz).unwrap();
        let back: Visualization = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Test Chart");
    }

    #[test]
    fn test_report_section_serde() {
        let section = ReportSection {
            title: "Complexity".to_string(),
            section_type: SectionType::Complexity,
            content: serde_json::json!({}),
            metrics: HashMap::new(),
            findings: vec![],
        };
        let json = serde_json::to_string(&section).unwrap();
        let back: ReportSection = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Complexity");
        assert!(back.findings.is_empty());
    }

    #[test]
    fn test_analysis_results_minimal() {
        let results = AnalysisResults {
            total_duration: std::time::Duration::from_secs(1),
            analyzed_files: 10,
            total_lines: 1000,
            complexity_analysis: None,
            dead_code_analysis: None,
            duplication_analysis: None,
            tdg_analysis: None,
            big_o_analysis: None,
        };
        assert_eq!(results.analyzed_files, 10);
        assert!(results.complexity_analysis.is_none());
    }

    #[test]
    fn test_complexity_analysis_fields() {
        let ca = ComplexityAnalysis {
            total_cyclomatic: 100,
            total_cognitive: 150,
            functions: 20,
            max_cyclomatic: 15,
            high_complexity_functions: 2,
            distribution: vec![10, 5, 3, 1, 1],
        };
        assert_eq!(ca.functions, 20);
        assert_eq!(ca.distribution.len(), 5);
    }

    #[test]
    fn test_location_serde() {
        let loc = Location {
            file: "src/lib.rs".to_string(),
            line: Some(42),
            column: Some(10),
        };
        let json = serde_json::to_string(&loc).unwrap();
        let back: Location = serde_json::from_str(&json).unwrap();
        assert_eq!(back.file, "src/lib.rs");
        assert_eq!(back.line, Some(42));
    }

    #[test]
    fn test_location_none_fields() {
        let loc = Location {
            file: "test.rs".to_string(),
            line: None,
            column: None,
        };
        let json = serde_json::to_string(&loc).unwrap();
        let back: Location = serde_json::from_str(&json).unwrap();
        assert!(back.line.is_none());
        assert!(back.column.is_none());
    }
}
