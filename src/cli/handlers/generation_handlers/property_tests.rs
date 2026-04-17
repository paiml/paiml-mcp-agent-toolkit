#![cfg_attr(coverage_nightly, coverage(off))]

use super::agent_scaffold::*;
use super::template_handlers::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_resolve_scaffold_templates_never_returns_empty_for_known_toolchains(
        toolchain in prop_oneof![Just("rust"), Just("deno"), Just("python-uv")]
    ) {
        let result = resolve_scaffold_templates(toolchain, vec![]);
        prop_assert!(!result.is_empty());
    }

    #[test]
    fn test_resolve_scaffold_templates_always_returns_at_least_readme_for_unknown(
        toolchain in "[a-z]{5,10}"
    ) {
        // Filter out known toolchains
        if toolchain != "rust" && toolchain != "deno" && !toolchain.starts_with("python") {
            let result = resolve_scaffold_templates(&toolchain, vec![]);
            prop_assert!(result.contains(&"readme".to_string()));
        }
    }

    #[test]
    fn test_resolve_scaffold_templates_returns_provided_when_not_empty(
        templates in prop::collection::vec("[a-z]+", 1..5)
    ) {
        let result = resolve_scaffold_templates("any", templates.clone());
        prop_assert_eq!(result, templates);
    }

    #[test]
    fn test_get_default_scaffold_templates_never_panics(
        toolchain in ".*"
    ) {
        // Should never panic regardless of input
        let _ = get_default_scaffold_templates(&toolchain);
        prop_assert!(true);
    }

    #[test]
    fn test_validate_output_path_with_force_always_succeeds_for_nonexistent(
        path in "/nonexistent/[a-z]{10}/[a-z]{10}"
    ) {
        let result = validate_output_path(std::path::Path::new(&path), true);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_quality_level_mapping_is_deterministic(
        quality in prop_oneof![
            Just("standard"),
            Just("strict"),
            Just("extreme"),
            Just("STANDARD"),
            Just("STRICT"),
            Just("EXTREME")
        ]
    ) {
        use crate::scaffold::agent::AgentContextBuilder;

        let builder1 = AgentContextBuilder::new("test", "mcp-server");
        let builder2 = AgentContextBuilder::new("test", "mcp-server");

        let result1 = add_quality_level_to_builder(builder1, quality);
        let result2 = add_quality_level_to_builder(builder2, quality);

        let ctx1 = result1.build().unwrap();
        let ctx2 = result2.build().unwrap();

        prop_assert_eq!(ctx1.quality_level, ctx2.quality_level);
    }
}

// Additional property tests for edge cases

proptest! {
    #[test]
    fn test_add_features_never_panics(
        features in prop::collection::vec(".*", 0..10)
    ) {
        use crate::scaffold::agent::AgentContextBuilder;
        let builder = AgentContextBuilder::new("test", "mcp-server");
        let _ = add_features_to_builder(builder, &features);
        prop_assert!(true);
    }

    #[test]
    fn test_build_agent_context_with_valid_name(
        name in "[a-z][a-z0-9_]{2,20}"
    ) {
        let result = build_agent_context(&name, "mcp-server", &[], "strict", None, None);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_build_agent_context_fails_with_invalid_chars(
        invalid_char in "[!@#$%^&*()\\-+=\\[\\]{};':\",./<>?\\\\|`~]"
    ) {
        let name = format!("agent{}", invalid_char);
        let result = build_agent_context(&name, "mcp-server", &[], "strict", None, None);
        prop_assert!(result.is_err());
    }
}
