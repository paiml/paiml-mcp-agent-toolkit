#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_quality::enforcement::EnforcerConfig;
    use crate::unified_quality::foundation::MonitorConfig;

    #[test]
    fn test_github_config_default() {
        let thresholds = QualityThresholds::default();
        assert_eq!(thresholds.max_complexity_increase, 50);
        assert_eq!(thresholds.max_satd_increase, 5);
        assert_eq!(thresholds.min_coverage, 0.8);
        assert!(thresholds.block_on_violation);
    }

    #[test]
    fn test_workflow_triggers_default() {
        let triggers = WorkflowTriggers::default();
        assert!(triggers.on_pull_request);
        assert!(triggers.on_push_main);
        assert!(triggers.on_schedule.is_some());
        assert_eq!(triggers.branches.len(), 2);
    }

    #[test]
    fn test_comment_template_default() {
        let template = CommentTemplate::default();
        assert!(template.header.contains("Code Quality Report"));
        assert!(template.success_template.contains("Quality checks passed"));
        assert!(template.failure_template.contains("Quality checks failed"));
    }

    #[test]
    fn test_workflow_yaml_generation() {
        let monitor = QualityMonitor::new(MonitorConfig::default()).unwrap();
        let enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let config = GitHubConfig {
            repository: "owner/repo".to_string(),
            token: "token".to_string(),
            quality_thresholds: QualityThresholds::default(),
            triggers: WorkflowTriggers::default(),
            comments: CommentConfig::default(),
        };

        let integration = GitHubActionsIntegration::new(monitor, enforcer, config);
        let yaml = integration.generate_workflow_yaml();

        assert!(yaml.contains("name: Quality Gate"));
        assert!(yaml.contains("pull_request:"));
        assert!(yaml.contains("pmat unified-quality analyze"));
    }

    #[test]
    fn test_violation_severity_ordering() {
        let severities = [
            ViolationSeverity::Info,
            ViolationSeverity::Warning,
            ViolationSeverity::Error,
            ViolationSeverity::Critical,
        ];

        // Just test that all variants exist and can be created
        assert_eq!(severities.len(), 4);
    }

    #[test]
    fn test_team_extraction() {
        let monitor = QualityMonitor::new(MonitorConfig::default()).unwrap();
        let enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let config = GitHubConfig {
            repository: "my-org/my-repo".to_string(),
            token: "token".to_string(),
            quality_thresholds: QualityThresholds::default(),
            triggers: WorkflowTriggers::default(),
            comments: CommentConfig::default(),
        };

        let integration = GitHubActionsIntegration::new(monitor, enforcer, config);
        let team = integration.extract_team_from_repository();
        assert_eq!(team, "my-org");
    }

    #[test]
    fn test_workflow_status_variants() {
        let statuses = [
            WorkflowStatus::Success,
            WorkflowStatus::Warning,
            WorkflowStatus::Failure,
            WorkflowStatus::Error("test".to_string()),
        ];

        assert_eq!(statuses.len(), 4);

        // Test Debug formatting
        let _ = format!("{:?}", WorkflowStatus::Success);
        let _ = format!("{:?}", WorkflowStatus::Error("test".to_string()));
    }

    #[test]
    fn test_comment_config_default() {
        let config = CommentConfig::default();
        assert!(config.post_summary);
        assert!(!config.post_details);
        assert!(config.update_existing);
    }

    #[test]
    fn test_quality_thresholds_clone() {
        let thresholds = QualityThresholds::default();
        let cloned = thresholds.clone();
        assert_eq!(
            cloned.max_complexity_increase,
            thresholds.max_complexity_increase
        );
    }

    #[test]
    fn test_quality_thresholds_serialization() {
        let thresholds = QualityThresholds {
            max_complexity_increase: 100,
            max_satd_increase: 10,
            min_coverage: 0.9,
            block_on_violation: false,
        };
        let json = serde_json::to_string(&thresholds).unwrap();
        let deserialized: QualityThresholds = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_complexity_increase, 100);
        assert!(!deserialized.block_on_violation);
    }

    #[test]
    fn test_workflow_triggers_clone() {
        let triggers = WorkflowTriggers::default();
        let cloned = triggers.clone();
        assert_eq!(cloned.on_pull_request, triggers.on_pull_request);
    }

    #[test]
    fn test_workflow_triggers_custom() {
        let triggers = WorkflowTriggers {
            on_pull_request: false,
            on_push_main: false,
            on_schedule: None,
            branches: vec!["develop".to_string()],
        };
        assert!(!triggers.on_pull_request);
        assert!(triggers.on_schedule.is_none());
        assert_eq!(triggers.branches.len(), 1);
    }

    #[test]
    fn test_comment_template_clone() {
        let template = CommentTemplate::default();
        let cloned = template.clone();
        assert_eq!(cloned.header, template.header);
    }

    #[test]
    fn test_quality_analysis_creation() {
        let analysis = QualityAnalysis {
            files_analyzed: 10,
            total_complexity: 100,
            complexity_change: 5,
            satd_count: 3,
            satd_change: 1,
            coverage: 0.85,
            coverage_change: 0.02,
            violations: vec![],
        };
        assert_eq!(analysis.files_analyzed, 10);
        assert_eq!(analysis.total_complexity, 100);
    }

    #[test]
    fn test_quality_analysis_with_violations() {
        let violation = QualityViolation {
            file: "src/main.rs".to_string(),
            violation_type: "complexity".to_string(),
            severity: ViolationSeverity::Error,
            message: "High complexity".to_string(),
            line: Some(42),
        };
        let analysis = QualityAnalysis {
            files_analyzed: 1,
            total_complexity: 50,
            complexity_change: 10,
            satd_count: 2,
            satd_change: 2,
            coverage: 0.75,
            coverage_change: -0.05,
            violations: vec![violation],
        };
        assert_eq!(analysis.violations.len(), 1);
        assert_eq!(analysis.violations[0].file, "src/main.rs");
    }

    #[test]
    fn test_quality_violation_serialization() {
        let violation = QualityViolation {
            file: "test.rs".to_string(),
            violation_type: "satd".to_string(),
            severity: ViolationSeverity::Warning,
            message: "TODO found".to_string(),
            line: None,
        };
        let json = serde_json::to_string(&violation).unwrap();
        let deserialized: QualityViolation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.file, "test.rs");
        assert!(deserialized.line.is_none());
    }

    #[test]
    fn test_violation_severity_serialization() {
        let severities = vec![
            ViolationSeverity::Info,
            ViolationSeverity::Warning,
            ViolationSeverity::Error,
            ViolationSeverity::Critical,
        ];
        for severity in severities {
            let json = serde_json::to_string(&severity).unwrap();
            let deserialized: ViolationSeverity = serde_json::from_str(&json).unwrap();
            let _ = format!("{:?}", deserialized);
        }
    }

    #[test]
    fn test_workflow_result_creation() {
        let result = WorkflowResult {
            status: WorkflowStatus::Success,
            analysis: QualityAnalysis {
                files_analyzed: 5,
                total_complexity: 25,
                complexity_change: 0,
                satd_count: 0,
                satd_change: 0,
                coverage: 0.9,
                coverage_change: 0.0,
                violations: vec![],
            },
            decision: Decision::Approved,
            comment: Some("All good!".to_string()),
            outputs: HashMap::new(),
        };
        assert!(matches!(result.status, WorkflowStatus::Success));
        assert!(result.comment.is_some());
    }

    #[test]
    fn test_workflow_result_with_outputs() {
        let mut outputs = HashMap::new();
        outputs.insert("status".to_string(), "Success".to_string());
        outputs.insert("complexity".to_string(), "100".to_string());

        let result = WorkflowResult {
            status: WorkflowStatus::Warning,
            analysis: QualityAnalysis {
                files_analyzed: 3,
                total_complexity: 100,
                complexity_change: 50,
                satd_count: 5,
                satd_change: 3,
                coverage: 0.7,
                coverage_change: -0.1,
                violations: vec![],
            },
            decision: Decision::Warning("Budget low".to_string()),
            comment: None,
            outputs,
        };
        assert_eq!(result.outputs.get("status"), Some(&"Success".to_string()));
    }

    #[test]
    fn test_github_config_serialization() {
        let config = GitHubConfig {
            repository: "test/repo".to_string(),
            token: "secret".to_string(),
            quality_thresholds: QualityThresholds::default(),
            triggers: WorkflowTriggers::default(),
            comments: CommentConfig::default(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GitHubConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.repository, "test/repo");
    }

    #[test]
    fn test_comment_config_serialization() {
        let config = CommentConfig {
            post_summary: false,
            post_details: true,
            update_existing: false,
            template: CommentTemplate::default(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CommentConfig = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.post_summary);
        assert!(deserialized.post_details);
    }

    #[test]
    fn test_quality_analysis_clone() {
        let analysis = QualityAnalysis {
            files_analyzed: 2,
            total_complexity: 20,
            complexity_change: 5,
            satd_count: 1,
            satd_change: 0,
            coverage: 0.95,
            coverage_change: 0.05,
            violations: vec![],
        };
        let cloned = analysis.clone();
        assert_eq!(cloned.files_analyzed, analysis.files_analyzed);
    }

    #[test]
    fn test_workflow_result_serialization() {
        let result = WorkflowResult {
            status: WorkflowStatus::Failure,
            analysis: QualityAnalysis {
                files_analyzed: 1,
                total_complexity: 200,
                complexity_change: 100,
                satd_count: 10,
                satd_change: 5,
                coverage: 0.5,
                coverage_change: -0.3,
                violations: vec![],
            },
            decision: Decision::Blocked {
                suggestion: "Reduce complexity".to_string(),
                refactor_targets: vec![],
            },
            comment: None,
            outputs: HashMap::new(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Failure"));
    }

    #[test]
    fn test_team_extraction_no_slash() {
        let monitor = QualityMonitor::new(MonitorConfig::default()).unwrap();
        let enforcer = ErrorBudgetEnforcer::new(EnforcerConfig::default());
        let config = GitHubConfig {
            repository: "single-name".to_string(),
            token: "token".to_string(),
            quality_thresholds: QualityThresholds::default(),
            triggers: WorkflowTriggers::default(),
            comments: CommentConfig::default(),
        };

        let integration = GitHubActionsIntegration::new(monitor, enforcer, config);
        let team = integration.extract_team_from_repository();
        assert_eq!(team, "single-name");
    }
}
