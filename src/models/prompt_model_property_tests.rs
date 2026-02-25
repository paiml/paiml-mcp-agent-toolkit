proptest! {
    #[test]
    fn test_render_idempotent(var_value in "[a-zA-Z0-9_-]+") {
        let yaml = r#"
name: test
description: Test
category: test
priority: high
prompt: "Test ${VAR}"
"#;
        let prompt = WorkflowPrompt::from_yaml(yaml).expect("internal error");
        let mut vars = HashMap::new();
        vars.insert("VAR".to_string(), var_value);

        let rendered1 = prompt.render(&vars);
        let rendered2 = prompt.render(&vars);
        prop_assert_eq!(rendered1, rendered2);
    }

    #[test]
    fn test_to_yaml_roundtrip(name in "[a-z-]+", desc in ".*") {
        let prompt = WorkflowPrompt {
            name: name.clone(),
            description: desc.clone(),
            category: "test".to_string(),
            priority: "high".to_string(),
            prompt: "test".to_string(),
            methodology: None,
            constraints: None,
            heuristics: None,
            toyota_way_principles: None,
            quality_gates: None,
            validation_tools: None,
            testing_approaches: None,
            coverage_target: None,
            mutation_score_target: None,
            zero_tolerance: None,
            validation: None,
            mutation_targets: None,
            improvement_goals: None,
            refactoring_targets: None,
            optimization_targets: None,
            tools: None,
            vulnerability_tolerance: None,
            security_tools: None,
        };

        let yaml = prompt.to_yaml().expect("internal error");
        let parsed = WorkflowPrompt::from_yaml(&yaml).expect("internal error");
        prop_assert_eq!(parsed.name, name);
        prop_assert_eq!(parsed.description, desc);
    }
}
