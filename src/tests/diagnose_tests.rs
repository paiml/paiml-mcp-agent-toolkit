#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::cli::diagnose::{
        BuildInfo, CompactErrorContext, DiagnoseArgs, DiagnosticFormat, DiagnosticSummary,
        EnvironmentSnapshot, FeatureResult, FeatureStatus, SuggestedFix,
    };
    use std::collections::BTreeMap;

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
            degraded: 1,
            skipped: 1,
            all_passed: false,
            success_rate: 70.0,
        };

        assert_eq!(summary.total, 10);
        assert_eq!(summary.passed, 7);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.degraded, 1);
        assert!(!summary.all_passed);
        assert!((summary.success_rate - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_build_info_serialization() {
        let build_info = BuildInfo {
            rust_version: "1.70.0".to_string(),
            build_date: "2024-01-01".to_string(),
            git_commit: Some("abcdef12".to_string()),
            features: vec!["ast".to_string(), "git".to_string()],
        };

        // Test serialization
        let json = serde_json::to_string(&build_info).unwrap();
        assert!(json.contains("1.70.0"));
        assert!(json.contains("2024-01-01"));
        assert!(json.contains("abcdef12"));
    }

    #[test]
    fn test_build_info_current() {
        let info = BuildInfo::current();
        // Current should always return valid data
        assert!(!info.rust_version.is_empty());
        assert!(!info.build_date.is_empty());
    }

    #[test]
    fn test_compact_error_context() {
        let mut error_patterns: BTreeMap<String, Vec<String>> = BTreeMap::new();
        error_patterns.insert("FileNotFound".to_string(), vec!["file.rs".to_string()]);
        error_patterns.insert("ParseError".to_string(), vec!["lib.rs".to_string()]);

        let context = CompactErrorContext {
            failed_features: vec!["ast_parser".to_string(), "cache".to_string()],
            error_patterns,
            suggested_fixes: vec![SuggestedFix {
                feature: "ast_parser".to_string(),
                error_pattern: "FileNotFound".to_string(),
                fix_command: Some("chmod +r files/".to_string()),
                documentation_link: None,
            }],
            environment: EnvironmentSnapshot {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                cpu_count: 8,
                memory_mb: 16384,
                cwd: "/home/user/project".to_string(),
            },
        };

        assert_eq!(context.failed_features.len(), 2);
        assert_eq!(context.error_patterns.len(), 2);
        assert_eq!(context.suggested_fixes.len(), 1);
        assert_eq!(context.environment.os, "linux");
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

    #[test]
    fn test_suggested_fix_structure() {
        let fix = SuggestedFix {
            feature: "cache".to_string(),
            error_pattern: "CacheMiss".to_string(),
            fix_command: Some("pmat cache --warm".to_string()),
            documentation_link: Some("https://docs.pmat.io/cache".to_string()),
        };

        assert_eq!(fix.feature, "cache");
        assert!(fix.fix_command.is_some());
        assert!(fix.documentation_link.is_some());
    }

    #[test]
    fn test_environment_snapshot_structure() {
        let env = EnvironmentSnapshot {
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            cpu_count: 10,
            memory_mb: 32768,
            cwd: "/Users/developer/project".to_string(),
        };

        assert_eq!(env.os, "macos");
        assert_eq!(env.arch, "aarch64");
        assert_eq!(env.cpu_count, 10);
        assert_eq!(env.memory_mb, 32768);
        assert_eq!(env.cwd, "/Users/developer/project");
    }

    #[test]
    fn test_diagnostic_summary_all_passed() {
        let summary = DiagnosticSummary {
            total: 5,
            passed: 5,
            failed: 0,
            degraded: 0,
            skipped: 0,
            all_passed: true,
            success_rate: 100.0,
        };

        assert!(summary.all_passed);
        assert!((summary.success_rate - 100.0).abs() < f64::EPSILON);
    }
}
