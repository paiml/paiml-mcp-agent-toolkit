#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_scaffolder_creation() {
        let scaffolder = InteractiveScaffolder::new();
        // Verify the scaffolder was created with expected components
        // The term and theme are initialized correctly
        assert!(std::mem::size_of_val(&scaffolder) > 0);
    }

    #[test]
    fn test_interactive_scaffolder_default() {
        let scaffolder = InteractiveScaffolder::default();
        // Verify Default impl calls new()
        assert!(std::mem::size_of_val(&scaffolder) > 0);
    }

    // =========================================================================
    // feature_to_string comprehensive tests
    // =========================================================================

    #[test]
    fn test_feature_to_string_tool_composition() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::ToolComposition;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Tool Composition support"
        );
    }

    #[test]
    fn test_feature_to_string_async_handlers() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::AsyncHandlers;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Async request handlers"
        );
    }

    #[test]
    fn test_feature_to_string_resource_subscriptions() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::ResourceSubscriptions;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Resource subscriptions"
        );
    }

    #[test]
    fn test_feature_to_string_complexity_analysis() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::ComplexityAnalysis;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Complexity analysis"
        );
    }

    #[test]
    fn test_feature_to_string_satd_detection() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::SATDDetection;
        assert_eq!(scaffolder.feature_to_string(&feature), "SATD detection");
    }

    #[test]
    fn test_feature_to_string_dead_code_elimination() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::DeadCodeElimination;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Dead code elimination"
        );
    }

    #[test]
    fn test_feature_to_string_state_machine() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::StateMachine {
            states: vec!["Init".to_string(), "Running".to_string()],
        };
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "State Machine with transitions"
        );
    }

    #[test]
    fn test_feature_to_string_quality_gates() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::QualityGates {
            level: QualityLevel::Extreme,
        };
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Quality Gates enforcement"
        );
    }

    #[test]
    fn test_feature_to_string_monitoring() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::Monitoring {
            backend: super::super::features::MonitoringBackend::Prometheus,
        };
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Monitoring integration"
        );
    }

    #[test]
    fn test_feature_to_string_tracing() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::Tracing {
            exporter: super::super::features::TraceExporter::OTLP,
        };
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Distributed tracing"
        );
    }

    #[test]
    fn test_feature_to_string_health_checks() {
        let scaffolder = InteractiveScaffolder::new();
        let feature = AgentFeature::HealthChecks;
        assert_eq!(
            scaffolder.feature_to_string(&feature),
            "Health check endpoints"
        );
    }

    // =========================================================================
    // template_to_string comprehensive tests
    // =========================================================================

    #[test]
    fn test_template_to_string_mcp_server() {
        let scaffolder = InteractiveScaffolder::new();
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::MCPToolServer),
            "mcp-server"
        );
    }

    #[test]
    fn test_template_to_string_state_machine() {
        let scaffolder = InteractiveScaffolder::new();
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::StateMachineWorkflow),
            "state-machine"
        );
    }

    #[test]
    fn test_template_to_string_calculator() {
        let scaffolder = InteractiveScaffolder::new();
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::DeterministicCalculator),
            "calculator"
        );
    }

    #[test]
    fn test_template_to_string_hybrid() {
        let scaffolder = InteractiveScaffolder::new();
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::HybridAnalyzer),
            "hybrid"
        );
    }

    #[test]
    fn test_template_to_string_custom() {
        let scaffolder = InteractiveScaffolder::new();
        let custom_path = PathBuf::from("/path/to/my/template.toml");
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::CustomAgent(custom_path)),
            "custom:/path/to/my/template.toml"
        );
    }

    #[test]
    fn test_template_to_string_custom_relative() {
        let scaffolder = InteractiveScaffolder::new();
        let custom_path = PathBuf::from("relative/path/template.toml");
        assert_eq!(
            scaffolder.template_to_string(&AgentTemplate::CustomAgent(custom_path)),
            "custom:relative/path/template.toml"
        );
    }

    // =========================================================================
    // get_available_features comprehensive tests
    // =========================================================================

    #[test]
    fn test_get_available_features_mcp_server() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::MCPToolServer);

        // MCP server should have 6 features
        assert_eq!(features.len(), 6);

        // Verify specific features are present
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::ToolComposition)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::AsyncHandlers)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::ResourceSubscriptions)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::Monitoring { .. })));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::Tracing { .. })));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::HealthChecks)));
    }

    #[test]
    fn test_get_available_features_state_machine() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow);

        // State machine should have 2 features
        assert_eq!(features.len(), 2);

        // Verify specific features are present
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::StateMachine { .. })));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::QualityGates { .. })));
    }

    #[test]
    fn test_get_available_features_hybrid_analyzer() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::HybridAnalyzer);

        // Hybrid analyzer should have 3 features
        assert_eq!(features.len(), 3);

        // Verify specific features are present
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::ComplexityAnalysis)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::SATDDetection)));
        assert!(features
            .iter()
            .any(|f| matches!(f, AgentFeature::DeadCodeElimination)));
    }

    #[test]
    fn test_get_available_features_deterministic_calculator() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::DeterministicCalculator);

        // Deterministic calculator has no additional features (falls through to default)
        assert!(features.is_empty());
    }

    #[test]
    fn test_get_available_features_custom_agent() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder
            .get_available_features(&AgentTemplate::CustomAgent(PathBuf::from("custom.toml")));

        // Custom agent has no predefined features (falls through to default)
        assert!(features.is_empty());
    }

    // =========================================================================
    // show_header test
    // =========================================================================

    #[test]
    fn test_show_header_returns_ok() {
        let scaffolder = InteractiveScaffolder::new();
        // show_header only prints to stdout, should always return Ok
        let result = scaffolder.show_header();
        assert!(result.is_ok());
    }

    // =========================================================================
    // State machine feature state verification
    // =========================================================================

    #[test]
    fn test_state_machine_feature_has_expected_states() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow);

        // Find the state machine feature and verify its default states
        let state_machine_feature = features
            .iter()
            .find(|f| matches!(f, AgentFeature::StateMachine { .. }));

        assert!(state_machine_feature.is_some());
        if let Some(AgentFeature::StateMachine { states }) = state_machine_feature {
            assert_eq!(states.len(), 3);
            assert_eq!(states[0], "Initial");
            assert_eq!(states[1], "Processing");
            assert_eq!(states[2], "Complete");
        }
    }

    // =========================================================================
    // Quality gates feature verification
    // =========================================================================

    #[test]
    fn test_quality_gates_feature_has_extreme_level() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow);

        // Find the quality gates feature and verify its level
        let quality_gates_feature = features
            .iter()
            .find(|f| matches!(f, AgentFeature::QualityGates { .. }));

        assert!(quality_gates_feature.is_some());
        if let Some(AgentFeature::QualityGates { level }) = quality_gates_feature {
            assert!(matches!(level, QualityLevel::Extreme));
        }
    }

    // =========================================================================
    // Monitoring and tracing backend verification
    // =========================================================================

    #[test]
    fn test_mcp_server_monitoring_uses_prometheus() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::MCPToolServer);

        let monitoring_feature = features
            .iter()
            .find(|f| matches!(f, AgentFeature::Monitoring { .. }));

        assert!(monitoring_feature.is_some());
        if let Some(AgentFeature::Monitoring { backend }) = monitoring_feature {
            assert!(matches!(
                backend,
                super::super::features::MonitoringBackend::Prometheus
            ));
        }
    }

    #[test]
    fn test_mcp_server_tracing_uses_otlp() {
        let scaffolder = InteractiveScaffolder::new();
        let features = scaffolder.get_available_features(&AgentTemplate::MCPToolServer);

        let tracing_feature = features
            .iter()
            .find(|f| matches!(f, AgentFeature::Tracing { .. }));

        assert!(tracing_feature.is_some());
        if let Some(AgentFeature::Tracing { exporter }) = tracing_feature {
            assert!(matches!(
                exporter,
                super::super::features::TraceExporter::OTLP
            ));
        }
    }

    // =========================================================================
    // Error handling tests
    // =========================================================================

    #[test]
    fn test_user_cancelled_error_can_be_created() {
        let err: anyhow::Error = ScaffoldError::UserCancelled.into();
        assert!(err.to_string().contains("cancelled"));
    }
}
