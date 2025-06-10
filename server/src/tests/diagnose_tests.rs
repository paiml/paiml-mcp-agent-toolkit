#[cfg(test)]
mod tests {
    use crate::cli::diagnose::{
        DiagnoseArgs, DiagnosticFormat, FeatureResult, FeatureStatus,
        DiagnosticSummary, BuildInfo, CompactErrorContext,
    };

    #[test]
    fn test_diagnostic_format_enum() {
        // Test all format variants
        let pretty = DiagnosticFormat::Pretty;
        let json = DiagnosticFormat::Json;
        let compact = DiagnosticFormat::Compact;
        
        // Verify Debug trait
        assert!(format!("{:?}", pretty).contains("Pretty"));
        assert!(format!("{:?}", json).contains("Json"));
        assert!(format!("{:?}", compact).contains("Compact"));
    }

    #[test]
    fn test_diagnose_args_defaults() {
        // Test default values
        let args = DiagnoseArgs {
            format: DiagnosticFormat::Pretty,
            only: vec![],
            skip: vec![],
            timeout: 60,
        };
        
        assert!(matches!(args.format, DiagnosticFormat::Pretty));
        assert!(args.only.is_empty());
        assert!(args.skip.is_empty());
        assert_eq!(args.timeout, 60);
    }

    #[test]
    fn test_feature_result_variants() {
        let success = FeatureResult {
            status: FeatureStatus::Ok,
            duration_us: 1000,
            error: None,
            metrics: None,
        };
        
        let degraded = FeatureResult {
            status: FeatureStatus::Degraded("Slow performance".to_string()),
            duration_us: 5000,
            error: None,
            metrics: Some(serde_json::json!({"latency": "high"})),
        };
        
        let failed = FeatureResult {
            status: FeatureStatus::Failed,
            duration_us: 100,
            error: Some("Connection failed".to_string()),
            metrics: None,
        };
        
        let skipped = FeatureResult {
            status: FeatureStatus::Skipped("Feature disabled".to_string()),
            duration_us: 0,
            error: None,
            metrics: None,
        };
        
        assert!(matches!(success.status, FeatureStatus::Ok));
        assert_eq!(success.duration_us, 1000);
        assert!(matches!(degraded.status, FeatureStatus::Degraded(_)));
        assert!(failed.error.is_some());
        assert!(matches!(skipped.status, FeatureStatus::Skipped(_)));
    }

    #[test]
    fn test_diagnostic_summary() {
        let summary = DiagnosticSummary {
            total: 10,
            passed: 7,
            failed: 1,
            warnings: 2,
            skipped: 0,
            health_percentage: 70.0,
            recommendations: vec![
                "Consider enabling caching".to_string(),
                "Update dependencies".to_string(),
            ],
        };
        
        assert_eq!(summary.total, 10);
        assert_eq!(summary.passed, 7);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.warnings, 2);
        assert_eq!(summary.health_percentage, 70.0);
        assert_eq!(summary.recommendations.len(), 2);
    }

    #[test]
    fn test_build_info_serialization() {
        let build_info = BuildInfo {
            rust_version: "1.70.0".to_string(),
            cargo_version: "1.70.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            profile: "release".to_string(),
            features: vec!["ast".to_string(), "git".to_string()],
            build_date: "2024-01-01".to_string(),
            git_commit: "abcdef12".to_string(),
        };
        
        // Test serialization
        let json = serde_json::to_string(&build_info).unwrap();
        assert!(json.contains("1.70.0"));
        assert!(json.contains("x86_64-unknown-linux-gnu"));
        assert!(json.contains("release"));
        assert!(json.contains("abcdef12"));
    }

    #[test]
    fn test_compact_error_context() {
        use std::collections::HashMap;
        
        let mut error_patterns = HashMap::new();
        error_patterns.insert("FileNotFound".to_string(), 3);
        error_patterns.insert("ParseError".to_string(), 2);
        
        let context = CompactErrorContext {
            failed_features: vec!["ast_parser".to_string(), "cache".to_string()],
            error_patterns,
            suggested_fixes: vec![
                crate::cli::diagnose::SuggestedFix {
                    description: "Check file permissions".to_string(),
                    command: Some("chmod +r files/".to_string()),
                    documentation: None,
                },
            ],
            environment_issues: vec!["Missing RUST_LOG variable".to_string()],
        };
        
        assert_eq!(context.failed_features.len(), 2);
        assert_eq!(context.error_patterns.len(), 2);
        assert_eq!(context.suggested_fixes.len(), 1);
        assert_eq!(context.environment_issues.len(), 1);
    }

    #[test]
    fn test_feature_status_serialization() {
        // Test that FeatureStatus serializes correctly
        let ok = FeatureStatus::Ok;
        let degraded = FeatureStatus::Degraded("Slow".to_string());
        let failed = FeatureStatus::Failed;
        let skipped = FeatureStatus::Skipped("Disabled".to_string());
        
        // Verify Debug trait works
        assert!(format!("{:?}", ok).contains("Ok"));
        assert!(format!("{:?}", degraded).contains("Degraded"));
        assert!(format!("{:?}", failed).contains("Failed"));
        assert!(format!("{:?}", skipped).contains("Skipped"));
    }

    #[test]
    fn test_diagnose_args_with_filters() {
        let args = DiagnoseArgs {
            format: DiagnosticFormat::Json,
            only: vec!["ast".to_string(), "cache".to_string()],
            skip: vec!["slow_test".to_string()],
            timeout: 30,
        };
        
        assert!(matches!(args.format, DiagnosticFormat::Json));
        assert_eq!(args.only.len(), 2);
        assert!(args.only.contains(&"ast".to_string()));
        assert_eq!(args.skip.len(), 1);
        assert_eq!(args.timeout, 30);
    }
}
