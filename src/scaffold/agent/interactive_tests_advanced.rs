#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn template_to_string_never_empty(_idx in 0usize..5) {
            let scaffolder = InteractiveScaffolder::new();
            let templates = vec![
                AgentTemplate::MCPToolServer,
                AgentTemplate::StateMachineWorkflow,
                AgentTemplate::DeterministicCalculator,
                AgentTemplate::HybridAnalyzer,
                AgentTemplate::CustomAgent(PathBuf::from("test.toml")),
            ];
            let template = &templates[_idx];
            let result = scaffolder.template_to_string(template);
            prop_assert!(!result.is_empty(), "template_to_string should never return empty string");
        }

        #[test]
        fn feature_to_string_never_empty(_idx in 0usize..11) {
            let scaffolder = InteractiveScaffolder::new();
            let features = vec![
                AgentFeature::StateMachine { states: vec!["A".to_string()] },
                AgentFeature::QualityGates { level: QualityLevel::Standard },
                AgentFeature::ToolComposition,
                AgentFeature::AsyncHandlers,
                AgentFeature::ResourceSubscriptions,
                AgentFeature::ComplexityAnalysis,
                AgentFeature::SATDDetection,
                AgentFeature::DeadCodeElimination,
                AgentFeature::Monitoring { backend: super::super::features::MonitoringBackend::Prometheus },
                AgentFeature::Tracing { exporter: super::super::features::TraceExporter::OTLP },
                AgentFeature::HealthChecks,
            ];
            let feature = &features[_idx];
            let result = scaffolder.feature_to_string(feature);
            prop_assert!(!result.is_empty(), "feature_to_string should never return empty string");
        }

        #[test]
        fn get_available_features_returns_valid_vec(_idx in 0usize..5) {
            let scaffolder = InteractiveScaffolder::new();
            let templates = vec![
                AgentTemplate::MCPToolServer,
                AgentTemplate::StateMachineWorkflow,
                AgentTemplate::DeterministicCalculator,
                AgentTemplate::HybridAnalyzer,
                AgentTemplate::CustomAgent(PathBuf::from("test.toml")),
            ];
            let template = &templates[_idx];
            let features = scaffolder.get_available_features(template);
            // Should not panic and return a valid Vec
            prop_assert!(features.len() >= 0);
        }

        #[test]
        fn custom_template_path_preserved(path in "[a-zA-Z0-9_/]+\\.toml") {
            let scaffolder = InteractiveScaffolder::new();
            let template = AgentTemplate::CustomAgent(PathBuf::from(&path));
            let result = scaffolder.template_to_string(&template);
            prop_assert!(result.contains(&path), "Custom template path should be preserved in string");
            prop_assert!(result.starts_with("custom:"), "Custom template string should start with 'custom:'");
        }

        #[test]
        fn state_machine_states_preserved(states in prop::collection::vec("[a-zA-Z]+", 1..10)) {
            let scaffolder = InteractiveScaffolder::new();
            let feature = AgentFeature::StateMachine { states: states.clone() };
            let result = scaffolder.feature_to_string(&feature);
            // The display string doesn't show individual states, but the conversion should succeed
            prop_assert!(!result.is_empty());
            prop_assert_eq!(result, "State Machine with transitions");
        }

        #[test]
        fn quality_level_variants_produce_same_output(level in 0u8..3) {
            let scaffolder = InteractiveScaffolder::new();
            let quality_level = match level {
                0 => QualityLevel::Standard,
                1 => QualityLevel::Strict,
                _ => QualityLevel::Extreme,
            };
            let feature = AgentFeature::QualityGates { level: quality_level };
            let result = scaffolder.feature_to_string(&feature);
            prop_assert_eq!(result, "Quality Gates enforcement");
        }

        #[test]
        fn scaffolder_creation_is_deterministic(_seed in 0u64..1000) {
            // Creating multiple scaffolders should succeed consistently
            let scaffolder1 = InteractiveScaffolder::new();
            let scaffolder2 = InteractiveScaffolder::new();

            // Both should produce the same template strings
            prop_assert_eq!(
                scaffolder1.template_to_string(&AgentTemplate::MCPToolServer),
                scaffolder2.template_to_string(&AgentTemplate::MCPToolServer)
            );
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test that all template types have a consistent string representation
    #[test]
    fn test_all_templates_have_unique_strings() {
        let scaffolder = InteractiveScaffolder::new();
        let templates = vec![
            AgentTemplate::MCPToolServer,
            AgentTemplate::StateMachineWorkflow,
            AgentTemplate::DeterministicCalculator,
            AgentTemplate::HybridAnalyzer,
        ];

        let strings: Vec<String> = templates
            .iter()
            .map(|t| scaffolder.template_to_string(t))
            .collect();

        // Check all strings are unique
        let unique: std::collections::HashSet<_> = strings.iter().collect();
        assert_eq!(
            unique.len(),
            strings.len(),
            "All template strings should be unique"
        );
    }

    /// Test that all features have displayable strings
    #[test]
    fn test_all_features_have_displayable_strings() {
        let scaffolder = InteractiveScaffolder::new();

        // Get all features from all templates
        let mut all_features = Vec::new();
        all_features.extend(scaffolder.get_available_features(&AgentTemplate::MCPToolServer));
        all_features
            .extend(scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow));
        all_features.extend(scaffolder.get_available_features(&AgentTemplate::HybridAnalyzer));

        // All features should have non-empty display strings
        for feature in &all_features {
            let display = scaffolder.feature_to_string(feature);
            assert!(
                !display.is_empty(),
                "Feature {:?} should have a non-empty display string",
                feature
            );
        }
    }

    /// Test the relationship between templates and their features
    #[test]
    fn test_template_feature_relationships() {
        let scaffolder = InteractiveScaffolder::new();

        // MCP server has the most features
        let mcp_features = scaffolder.get_available_features(&AgentTemplate::MCPToolServer);
        let sm_features = scaffolder.get_available_features(&AgentTemplate::StateMachineWorkflow);
        let hybrid_features = scaffolder.get_available_features(&AgentTemplate::HybridAnalyzer);
        let calc_features = scaffolder.get_available_features(&AgentTemplate::DeterministicCalculator);

        assert!(
            mcp_features.len() > sm_features.len(),
            "MCP server should have more features than state machine"
        );
        assert!(
            hybrid_features.len() > calc_features.len(),
            "Hybrid analyzer should have more features than calculator"
        );
        assert!(
            calc_features.is_empty(),
            "Deterministic calculator should have no predefined features"
        );
    }

    /// Test that Default and new() produce equivalent scaffolders
    #[test]
    fn test_default_equivalence() {
        let via_new = InteractiveScaffolder::new();
        let via_default = InteractiveScaffolder::default();

        // Both should produce identical template strings
        assert_eq!(
            via_new.template_to_string(&AgentTemplate::MCPToolServer),
            via_default.template_to_string(&AgentTemplate::MCPToolServer)
        );
        assert_eq!(
            via_new.template_to_string(&AgentTemplate::HybridAnalyzer),
            via_default.template_to_string(&AgentTemplate::HybridAnalyzer)
        );
    }

    /// Test that feature string representations are user-friendly
    #[test]
    fn test_feature_strings_are_user_friendly() {
        let scaffolder = InteractiveScaffolder::new();

        // All feature strings should be capitalized and descriptive
        let features_to_check = vec![
            (AgentFeature::ToolComposition, "Tool Composition support"),
            (AgentFeature::AsyncHandlers, "Async request handlers"),
            (AgentFeature::HealthChecks, "Health check endpoints"),
            (AgentFeature::ComplexityAnalysis, "Complexity analysis"),
        ];

        for (feature, expected) in features_to_check {
            let result = scaffolder.feature_to_string(&feature);
            assert_eq!(result, expected);
            // Verify the string starts with a capital letter
            assert!(
                result.chars().next().map(|c| c.is_uppercase()).unwrap_or(false),
                "Feature string '{}' should start with uppercase",
                result
            );
        }
    }

    /// Test monitoring backend variations
    #[test]
    fn test_monitoring_backend_variations() {
        use super::super::features::MonitoringBackend;
        let scaffolder = InteractiveScaffolder::new();

        let backends = vec![
            MonitoringBackend::Prometheus,
            MonitoringBackend::OpenTelemetry,
            MonitoringBackend::Custom("datadog".to_string()),
        ];

        for backend in backends {
            let feature = AgentFeature::Monitoring {
                backend: backend.clone(),
            };
            let result = scaffolder.feature_to_string(&feature);
            // All monitoring features should have the same display string
            assert_eq!(result, "Monitoring integration");
        }
    }

    /// Test tracing exporter variations
    #[test]
    fn test_tracing_exporter_variations() {
        use super::super::features::TraceExporter;
        let scaffolder = InteractiveScaffolder::new();

        let exporters = vec![
            TraceExporter::OTLP,
            TraceExporter::Jaeger,
            TraceExporter::Zipkin,
        ];

        for exporter in exporters {
            let feature = AgentFeature::Tracing {
                exporter: exporter.clone(),
            };
            let result = scaffolder.feature_to_string(&feature);
            // All tracing features should have the same display string
            assert_eq!(result, "Distributed tracing");
        }
    }

    /// Test quality level variations in QualityGates feature
    #[test]
    fn test_quality_level_variations() {
        let scaffolder = InteractiveScaffolder::new();

        let levels = vec![
            QualityLevel::Standard,
            QualityLevel::Strict,
            QualityLevel::Extreme,
        ];

        for level in levels {
            let feature = AgentFeature::QualityGates { level };
            let result = scaffolder.feature_to_string(&feature);
            // All quality gates features should have the same display string
            assert_eq!(result, "Quality Gates enforcement");
        }
    }
}
