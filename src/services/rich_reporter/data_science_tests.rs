use super::*;
use crate::services::rich_reporter::types::{Severity, SourceLocation};
use std::path::PathBuf;

fn create_test_finding(id: &str, category: &str, severity: Severity, line: usize) -> Finding {
    Finding {
        id: id.to_string(),
        category: category.to_string(),
        severity,
        location: SourceLocation {
            file: PathBuf::from("test.rs"),
            line,
            column: 1,
            scope: None,
        },
        message: "Test finding".to_string(),
        confidence: 0.9,
        cluster_id: None,
        pagerank: None,
        community: None,
        anomaly_score: None,
        fix_suggestion: None,
    }
}

#[test]
fn test_cluster_findings_empty() {
    let analyzer = DataScienceAnalyzer::default();
    let mut findings: Vec<Finding> = Vec::new();
    let clusters = analyzer.cluster_findings(&mut findings);
    assert!(clusters.is_empty());
}

#[test]
fn test_cluster_findings_single() {
    let analyzer = DataScienceAnalyzer::default();
    let mut findings = vec![create_test_finding("1", "TypeMismatch", Severity::High, 10)];
    let clusters = analyzer.cluster_findings(&mut findings);
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].size, 1);
}

#[test]
fn test_cluster_findings_multiple() {
    let analyzer = DataScienceAnalyzer::new(2, 0.85, 1.0, 0.7);
    let mut findings = vec![
        create_test_finding("1", "TypeMismatch", Severity::High, 10),
        create_test_finding("2", "TypeMismatch", Severity::High, 20),
        create_test_finding("3", "BorrowCheck", Severity::Critical, 30),
        create_test_finding("4", "BorrowCheck", Severity::Critical, 40),
    ];
    let clusters = analyzer.cluster_findings(&mut findings);
    assert!(!clusters.is_empty());
    assert!(findings.iter().all(|f| f.cluster_id.is_some()));
}

#[test]
fn test_detect_communities_no_deps() {
    let analyzer = DataScienceAnalyzer::default();
    let mut findings = vec![create_test_finding("1", "TypeMismatch", Severity::High, 10)];
    let deps: Vec<(String, String)> = Vec::new();
    let communities = analyzer.detect_communities(&mut findings, &deps);
    assert_eq!(communities.len(), 1);
}

#[test]
fn test_analyze_trends_empty() {
    let analyzer = DataScienceAnalyzer::default();
    let metrics: Vec<(String, Vec<(i64, f64)>)> = Vec::new();
    let trends = analyzer.analyze_trends(&metrics);
    assert!(trends.is_empty());
}

#[test]
fn test_analyze_trends_improving() {
    let analyzer = DataScienceAnalyzer::default();
    let data: Vec<(i64, f64)> = (0..10)
        .map(|i| (i as i64, 100.0 - i as f64 * 5.0))
        .collect();
    let metrics = vec![("coverage".to_string(), data)];
    let trends = analyzer.analyze_trends(&metrics);
    assert_eq!(trends.len(), 1);
    // Note: direction depends on interpretation - for costs, decreasing is improving
}

#[test]
fn test_values_to_sparkline() {
    let analyzer = DataScienceAnalyzer::default();
    let values = vec![0.0, 50.0, 100.0];
    let sparkline = analyzer.values_to_sparkline(&values);
    assert_eq!(sparkline.len(), 3);
    assert_eq!(sparkline[0], 0);
    assert_eq!(sparkline[2], 7);
}

#[test]
fn test_forecast_next() {
    let analyzer = DataScienceAnalyzer::default();
    let values = vec![0.0, 10.0, 20.0];
    let forecast = analyzer.forecast_next(&values);
    assert!(forecast.is_some());
    // Linear trend should predict ~30
    assert!((forecast.expect("internal error") - 30.0).abs() < 1.0);
}

#[test]
fn test_detect_anomalies_insufficient_data() {
    let analyzer = DataScienceAnalyzer::default();
    let mut findings = vec![
        create_test_finding("1", "TypeMismatch", Severity::High, 10),
        create_test_finding("2", "TypeMismatch", Severity::High, 20),
    ];
    let anomalies = analyzer.detect_anomalies(&mut findings);
    // Not enough data points (< 5)
    assert!(anomalies.is_empty());
}
