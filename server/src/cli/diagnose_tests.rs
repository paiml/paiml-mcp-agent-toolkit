//! Tests for diagnose module

use super::*;

#[test]
fn test_diagnostic_format_variants() {
    let formats = vec![
        DiagnosticFormat::Pretty,
        DiagnosticFormat::Json,
        DiagnosticFormat::Compact,
    ];

    for format in formats {
        match format {
            DiagnosticFormat::Pretty => assert_eq!(format!("{:?}", format), "Pretty"),
            DiagnosticFormat::Json => assert_eq!(format!("{:?}", format), "Json"),
            DiagnosticFormat::Compact => assert_eq!(format!("{:?}", format), "Compact"),
        }
    }
}

#[test]
fn test_build_info_creation() {
    let info = BuildInfo::current();

    // Just verify it creates successfully
    assert!(!info.rust_version.is_empty());
    assert!(!info.features.is_empty());
}

#[test]
fn test_feature_status_variants() {
    // Test FeatureStatus enum
    let ok = FeatureStatus::Ok;
    let degraded = FeatureStatus::Degraded("test".to_string());
    let failed = FeatureStatus::Failed;
    let skipped = FeatureStatus::Skipped("reason".to_string());

    assert!(matches!(ok, FeatureStatus::Ok));
    assert!(matches!(degraded, FeatureStatus::Degraded(_)));
    assert!(matches!(failed, FeatureStatus::Failed));
    assert!(matches!(skipped, FeatureStatus::Skipped(_)));
}
