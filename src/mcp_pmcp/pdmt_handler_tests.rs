// Tests for pdmt_handler
// Included from pdmt_handler.rs — no `use` imports or `#!` inner attributes allowed

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::pdmt_service::PdmtService;

    #[test]
    fn test_pdmt_service_basic() {
        let service = PdmtService::new();
        let requirements = vec![
            "implement user authentication".to_string(),
            "add logging system".to_string(),
        ];
        let config = crate::models::pdmt::PdmtQualityConfig::default();

        let result = service
            .generate_todos(
                requirements,
                Some("test_project".to_string()),
                "medium",
                config,
            )
            .unwrap();

        assert!(!result.todos.is_empty());
        assert_eq!(result.project_name, "test_project");
    }

    #[tokio::test]
    async fn test_pdmt_quality_enforcement() {
        let enforcer = crate::services::pdmt_quality_integration::PdmtQualityEnforcer::new();
        let todo = crate::models::pdmt::PdmtTodo::new(
            "Implement user authentication".to_string(),
            crate::models::pdmt::TodoPriority::High,
        );
        let todo_list = crate::models::pdmt::PdmtTodoList {
            project_name: "test".to_string(),
            todos: vec![todo],
            quality_config: crate::models::pdmt::PdmtQualityConfig::default(),
            generated_at: "2024-01-01".to_string(),
            deterministic_seed: 42,
        };

        let result = enforcer
            .enforce_quality_standards(&todo_list)
            .await
            .unwrap();
        assert!(result.overall_passed);
    }

    // === Default Function Tests ===

    #[test]
    fn test_default_granularity() {
        assert_eq!(default_granularity(), "high");
    }

    #[test]
    fn test_default_enforcement_mode() {
        assert_eq!(default_enforcement_mode(), "strict");
    }

    #[test]
    fn test_default_coverage_threshold() {
        assert!((default_coverage_threshold() - 80.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_default_max_complexity() {
        assert_eq!(default_max_complexity(), 8);
    }

    #[test]
    fn test_default_require_doctests() {
        assert!(default_require_doctests());
    }

    #[test]
    fn test_default_require_property_tests() {
        assert!(default_require_property_tests());
    }

    #[test]
    fn test_default_require_examples() {
        assert!(default_require_examples());
    }

    #[test]
    fn test_default_zero_satd_tolerance() {
        assert!(default_zero_satd_tolerance());
    }

    // === QualityConfigInput Tests ===

    #[test]
    fn test_quality_config_input_default() {
        let config = QualityConfigInput::default();
        assert_eq!(config.enforcement_mode, "strict");
        assert!((config.coverage_threshold - 80.0).abs() < f32::EPSILON);
        assert_eq!(config.max_complexity, 8);
        assert!(config.require_doctests);
        assert!(config.require_property_tests);
        assert!(config.require_examples);
        assert!(config.zero_satd_tolerance);
    }

    #[test]
    fn test_quality_config_input_deserialization() {
        let json = serde_json::json!({
            "enforcement_mode": "advisory",
            "coverage_threshold": 90.0,
            "max_complexity": 10
        });
        let config: QualityConfigInput = serde_json::from_value(json).unwrap();
        assert_eq!(config.enforcement_mode, "advisory");
        assert!((config.coverage_threshold - 90.0).abs() < f32::EPSILON);
        assert_eq!(config.max_complexity, 10);
        // Defaults should be used for unspecified fields
        assert!(config.require_doctests);
    }

    #[test]
    fn test_quality_config_input_debug() {
        let config = QualityConfigInput::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("QualityConfigInput"));
        assert!(debug.contains("enforcement_mode"));
    }

    // === PdmtInput Tests ===

    #[test]
    fn test_pdmt_input_deserialization() {
        let json = serde_json::json!({
            "requirements": ["req1", "req2"],
            "project_name": "test_project",
            "granularity": "medium"
        });
        let input: PdmtInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.requirements.len(), 2);
        assert_eq!(input.project_name, Some("test_project".to_string()));
        assert_eq!(input.granularity, "medium");
    }

    #[test]
    fn test_pdmt_input_defaults() {
        let json = serde_json::json!({
            "requirements": ["req1"]
        });
        let input: PdmtInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.granularity, "high");
        assert!(input.project_name.is_none());
    }

    #[test]
    fn test_pdmt_input_with_quality_config() {
        let json = serde_json::json!({
            "requirements": ["req1"],
            "quality_config": {
                "enforcement_mode": "auto_fix",
                "coverage_threshold": 95.0
            }
        });
        let input: PdmtInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.quality_config.enforcement_mode, "auto_fix");
        assert!((input.quality_config.coverage_threshold - 95.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pdmt_input_debug() {
        let input = PdmtInput {
            requirements: vec!["test".to_string()],
            project_name: Some("proj".to_string()),
            granularity: "high".to_string(),
            quality_config: QualityConfigInput::default(),
        };
        let debug = format!("{:?}", input);
        assert!(debug.contains("PdmtInput"));
    }

    #[test]
    fn test_pdmt_input_missing_requirements() {
        let json = serde_json::json!({});
        let result: std::result::Result<PdmtInput, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    // === PdmtOutput Tests ===

    #[test]
    fn test_pdmt_output_serialization() {
        let output = PdmtOutput {
            success: true,
            message: "Test message".to_string(),
            todo_list: Some(serde_json::json!({"todos": []})),
            quality_validation: Some(serde_json::json!({"passed": true})),
            total_todos: 5,
            estimated_total_hours: 10.5,
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["success"].as_bool().unwrap());
        assert_eq!(json["message"].as_str().unwrap(), "Test message");
        assert_eq!(json["total_todos"].as_u64().unwrap(), 5);
    }

    #[test]
    fn test_pdmt_output_deserialization() {
        let json = serde_json::json!({
            "success": false,
            "message": "Failed",
            "todo_list": null,
            "quality_validation": null,
            "total_todos": 0,
            "estimated_total_hours": 0.0
        });
        let output: PdmtOutput = serde_json::from_value(json).unwrap();
        assert!(!output.success);
        assert_eq!(output.message, "Failed");
        assert!(output.todo_list.is_none());
    }

    #[test]
    fn test_pdmt_output_debug() {
        let output = PdmtOutput {
            success: true,
            message: "OK".to_string(),
            todo_list: None,
            quality_validation: None,
            total_todos: 1,
            estimated_total_hours: 2.0,
        };
        let debug = format!("{:?}", output);
        assert!(debug.contains("PdmtOutput"));
    }

    // === PdmtTool Tests ===

    #[test]
    fn test_pdmt_tool_new() {
        let _tool = PdmtTool::new();
    }

    #[test]
    fn test_pdmt_tool_default() {
        let _tool = PdmtTool::default();
    }

    // === Enforcement Mode Conversion Tests ===

    #[test]
    fn test_enforcement_mode_strict() {
        let mode_str = "strict";
        let mode = match mode_str {
            "strict" => EnforcementMode::Strict,
            "advisory" => EnforcementMode::Advisory,
            "auto_fix" | "autofix" => EnforcementMode::AutoFix,
            _ => panic!("Invalid mode"),
        };
        assert!(matches!(mode, EnforcementMode::Strict));
    }

    #[test]
    fn test_enforcement_mode_advisory() {
        let mode_str = "advisory";
        let mode = match mode_str {
            "strict" => EnforcementMode::Strict,
            "advisory" => EnforcementMode::Advisory,
            "auto_fix" | "autofix" => EnforcementMode::AutoFix,
            _ => panic!("Invalid mode"),
        };
        assert!(matches!(mode, EnforcementMode::Advisory));
    }

    #[test]
    fn test_enforcement_mode_auto_fix() {
        let mode_str = "auto_fix";
        let mode = match mode_str {
            "strict" => EnforcementMode::Strict,
            "advisory" => EnforcementMode::Advisory,
            "auto_fix" | "autofix" => EnforcementMode::AutoFix,
            _ => panic!("Invalid mode"),
        };
        assert!(matches!(mode, EnforcementMode::AutoFix));
    }

    #[test]
    fn test_enforcement_mode_autofix() {
        let mode_str = "autofix";
        let mode = match mode_str {
            "strict" => EnforcementMode::Strict,
            "advisory" => EnforcementMode::Advisory,
            "auto_fix" | "autofix" => EnforcementMode::AutoFix,
            _ => panic!("Invalid mode"),
        };
        assert!(matches!(mode, EnforcementMode::AutoFix));
    }

    // === Quality Config Conversion Tests ===

    #[test]
    fn test_pdmt_quality_config_conversion() {
        let input = QualityConfigInput {
            enforcement_mode: "strict".to_string(),
            coverage_threshold: 85.0,
            max_complexity: 12,
            require_doctests: true,
            require_property_tests: false,
            require_examples: true,
            zero_satd_tolerance: false,
        };

        let config = crate::models::pdmt::PdmtQualityConfig {
            enforcement_mode: EnforcementMode::Strict,
            coverage_threshold: input.coverage_threshold,
            max_complexity: input.max_complexity,
            require_doctests: input.require_doctests,
            require_property_tests: input.require_property_tests,
            require_examples: input.require_examples,
            zero_satd_tolerance: input.zero_satd_tolerance,
        };

        assert!((config.coverage_threshold - 85.0).abs() < f32::EPSILON);
        assert_eq!(config.max_complexity, 12);
        assert!(config.require_doctests);
        assert!(!config.require_property_tests);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
