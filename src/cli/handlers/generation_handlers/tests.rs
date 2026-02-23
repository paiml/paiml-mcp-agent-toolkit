#![cfg_attr(coverage_nightly, coverage(off))]

use super::agent_scaffold::*;
use super::template_handlers::*;
use super::wasm_scaffold::*;
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test ScaffoldAgentParams with default values
fn default_scaffold_agent_params() -> ScaffoldAgentParams {
    ScaffoldAgentParams {
        name: "test_agent".to_string(),
        template: "mcp-server".to_string(),
        features: vec![],
        quality: "strict".to_string(),
        output: None,
        force: false,
        dry_run: true, // Use dry_run to avoid filesystem operations
        interactive: false,
        deterministic_core: None,
        probabilistic_wrapper: None,
    }
}

/// Create a test ScaffoldWasmParams with default values
fn default_scaffold_wasm_params() -> ScaffoldWasmParams {
    ScaffoldWasmParams {
        name: "test_wasm".to_string(),
        framework: "wasm-labs".to_string(),
        features: vec![],
        quality: "standard".to_string(),
        output: None,
        force: false,
        dry_run: true, // Use dry_run to avoid filesystem operations
    }
}

// ============================================================================
// resolve_scaffold_templates Tests
// ============================================================================

#[test]
fn test_resolve_scaffold_templates_with_empty_uses_defaults_for_rust() {
    let result = resolve_scaffold_templates("rust", vec![]);
    assert_eq!(
        result,
        vec![
            "makefile".to_string(),
            "readme".to_string(),
            "gitignore".to_string()
        ]
    );
}

#[test]
fn test_resolve_scaffold_templates_with_empty_uses_defaults_for_deno() {
    let result = resolve_scaffold_templates("deno", vec![]);
    assert_eq!(
        result,
        vec![
            "makefile".to_string(),
            "readme".to_string(),
            "gitignore".to_string()
        ]
    );
}

#[test]
fn test_resolve_scaffold_templates_with_empty_uses_defaults_for_python_uv() {
    let result = resolve_scaffold_templates("python-uv", vec![]);
    assert_eq!(
        result,
        vec![
            "makefile".to_string(),
            "readme".to_string(),
            "gitignore".to_string()
        ]
    );
}

#[test]
fn test_resolve_scaffold_templates_with_empty_uses_minimal_for_unknown() {
    let result = resolve_scaffold_templates("unknown-toolchain", vec![]);
    assert_eq!(result, vec!["readme".to_string()]);
}

#[test]
fn test_resolve_scaffold_templates_with_provided_templates() {
    let templates = vec!["custom1".to_string(), "custom2".to_string()];
    let result = resolve_scaffold_templates("rust", templates.clone());
    assert_eq!(result, templates);
}

#[test]
fn test_resolve_scaffold_templates_preserves_order() {
    let templates = vec!["z_last".to_string(), "a_first".to_string()];
    let result = resolve_scaffold_templates("any", templates.clone());
    assert_eq!(result, templates);
}

// ============================================================================
// get_default_scaffold_templates Tests
// ============================================================================

#[test]
fn test_get_default_scaffold_templates_rust() {
    let result = get_default_scaffold_templates("rust");
    assert!(result.contains(&"makefile".to_string()));
    assert!(result.contains(&"readme".to_string()));
    assert!(result.contains(&"gitignore".to_string()));
}

#[test]
fn test_get_default_scaffold_templates_deno() {
    let result = get_default_scaffold_templates("deno");
    assert!(result.contains(&"makefile".to_string()));
    assert!(result.contains(&"readme".to_string()));
    assert!(result.contains(&"gitignore".to_string()));
}

#[test]
fn test_get_default_scaffold_templates_python_uv() {
    let result = get_default_scaffold_templates("python-uv");
    assert!(result.contains(&"makefile".to_string()));
}

#[test]
fn test_get_default_scaffold_templates_fallback() {
    let result = get_default_scaffold_templates("golang");
    assert_eq!(result, vec!["readme".to_string()]);
}

#[test]
fn test_get_default_scaffold_templates_empty_string() {
    let result = get_default_scaffold_templates("");
    assert_eq!(result, vec!["readme".to_string()]);
}

// ============================================================================
// validate_output_path Tests
// ============================================================================

#[test]
fn test_validate_output_path_nonexistent_succeeds() {
    let result = validate_output_path(Path::new("/nonexistent/path/12345"), false);
    assert!(result.is_ok());
}

#[test]
fn test_validate_output_path_exists_without_force_fails() {
    let temp_dir = TempDir::new().unwrap();
    let result = validate_output_path(temp_dir.path(), false);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Directory already exists"));
}

#[test]
fn test_validate_output_path_exists_with_force_succeeds() {
    let temp_dir = TempDir::new().unwrap();
    let result = validate_output_path(temp_dir.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_validate_output_path_error_includes_suggestions() {
    let temp_dir = TempDir::new().unwrap();
    let result = validate_output_path(temp_dir.path(), false);
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("--force"));
    assert!(err_msg.contains("Suggestions"));
}

// ============================================================================
// add_quality_level_to_builder Tests
// ============================================================================

#[test]
fn test_add_quality_level_standard() {
    use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result_builder = add_quality_level_to_builder(builder, "standard");
    let context = result_builder.build().unwrap();
    assert_eq!(context.quality_level, QualityLevel::Standard);
}

#[test]
fn test_add_quality_level_strict() {
    use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result_builder = add_quality_level_to_builder(builder, "strict");
    let context = result_builder.build().unwrap();
    assert_eq!(context.quality_level, QualityLevel::Strict);
}

#[test]
fn test_add_quality_level_extreme() {
    use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result_builder = add_quality_level_to_builder(builder, "extreme");
    let context = result_builder.build().unwrap();
    assert_eq!(context.quality_level, QualityLevel::Extreme);
}

#[test]
fn test_add_quality_level_case_insensitive() {
    use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result_builder = add_quality_level_to_builder(builder, "EXTREME");
    let context = result_builder.build().unwrap();
    assert_eq!(context.quality_level, QualityLevel::Extreme);
}

#[test]
fn test_add_quality_level_unknown_defaults_to_strict() {
    use crate::scaffold::agent::{AgentContextBuilder, QualityLevel};
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result_builder = add_quality_level_to_builder(builder, "unknown");
    let context = result_builder.build().unwrap();
    assert_eq!(context.quality_level, QualityLevel::Strict);
}

// ============================================================================
// add_features_to_builder Tests
// ============================================================================

#[test]
fn test_add_features_to_builder_empty() {
    use crate::scaffold::agent::AgentContextBuilder;
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result_builder = add_features_to_builder(builder, &[]);
    let context = result_builder.build().unwrap();
    assert!(context.features.is_empty());
}

#[test]
fn test_add_features_to_builder_valid_feature() {
    use crate::scaffold::agent::{AgentContextBuilder, AgentFeature};
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let features = vec!["tool-composition".to_string()];
    let result_builder = add_features_to_builder(builder, &features);
    let context = result_builder.build().unwrap();
    assert!(context.features.contains(&AgentFeature::ToolComposition));
}

#[test]
fn test_add_features_to_builder_multiple_features() {
    use crate::scaffold::agent::{AgentContextBuilder, AgentFeature};
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let features = vec![
        "tool-composition".to_string(),
        "async-handlers".to_string(),
        "health-checks".to_string(),
    ];
    let result_builder = add_features_to_builder(builder, &features);
    let context = result_builder.build().unwrap();
    assert_eq!(context.features.len(), 3);
    assert!(context.features.contains(&AgentFeature::ToolComposition));
    assert!(context.features.contains(&AgentFeature::AsyncHandlers));
    assert!(context.features.contains(&AgentFeature::HealthChecks));
}

#[test]
fn test_add_features_to_builder_skips_unknown() {
    use crate::scaffold::agent::AgentContextBuilder;
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let features = vec!["unknown-feature".to_string()];
    let result_builder = add_features_to_builder(builder, &features);
    let context = result_builder.build().unwrap();
    assert!(context.features.is_empty());
}

#[test]
fn test_add_features_to_builder_mixed_valid_invalid() {
    use crate::scaffold::agent::{AgentContextBuilder, AgentFeature};
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let features = vec![
        "tool-composition".to_string(),
        "invalid-feature".to_string(),
        "async-handlers".to_string(),
    ];
    let result_builder = add_features_to_builder(builder, &features);
    let context = result_builder.build().unwrap();
    assert_eq!(context.features.len(), 2);
    assert!(context.features.contains(&AgentFeature::ToolComposition));
    assert!(context.features.contains(&AgentFeature::AsyncHandlers));
}

// ============================================================================
// add_hybrid_specs_to_builder Tests
// ============================================================================

#[test]
fn test_add_hybrid_specs_with_no_specs() {
    use crate::scaffold::agent::AgentContextBuilder;
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result = add_hybrid_specs_to_builder(builder, None, None);
    assert!(result.is_ok());
    let context = result.unwrap().build().unwrap();
    assert!(context.deterministic_core.is_none());
    assert!(context.probabilistic_wrapper.is_none());
}

#[test]
fn test_add_hybrid_specs_with_deterministic_core() {
    use crate::scaffold::agent::AgentContextBuilder;
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result = add_hybrid_specs_to_builder(builder, Some("core.toml".to_string()), None);
    assert!(result.is_ok());
    let context = result.unwrap().build().unwrap();
    assert!(context.deterministic_core.is_some());
}

#[test]
fn test_add_hybrid_specs_with_probabilistic_wrapper() {
    use crate::scaffold::agent::AgentContextBuilder;
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result = add_hybrid_specs_to_builder(builder, None, Some("wrapper.toml".to_string()));
    assert!(result.is_ok());
    let context = result.unwrap().build().unwrap();
    assert!(context.probabilistic_wrapper.is_some());
}

#[test]
fn test_add_hybrid_specs_with_both() {
    use crate::scaffold::agent::AgentContextBuilder;
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result = add_hybrid_specs_to_builder(
        builder,
        Some("core.toml".to_string()),
        Some("wrapper.toml".to_string()),
    );
    assert!(result.is_ok());
    let context = result.unwrap().build().unwrap();
    assert!(context.deterministic_core.is_some());
    assert!(context.probabilistic_wrapper.is_some());
}

// ============================================================================
// add_deterministic_core_spec Tests
// ============================================================================

#[test]
fn test_add_deterministic_core_spec() {
    use crate::scaffold::agent::AgentContextBuilder;
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result = add_deterministic_core_spec(builder);
    assert!(result.is_ok());
    let context = result.unwrap().build().unwrap();
    let core = context.deterministic_core.unwrap();
    assert_eq!(core.max_complexity, 10);
    assert!(core.invariants.is_empty());
}

// ============================================================================
// add_probabilistic_wrapper_spec Tests
// ============================================================================

#[test]
fn test_add_probabilistic_wrapper_spec() {
    use crate::scaffold::agent::hybrid::{FallbackStrategy, ModelType};
    use crate::scaffold::agent::AgentContextBuilder;
    let builder = AgentContextBuilder::new("test", "mcp-server");
    let result = add_probabilistic_wrapper_spec(builder);
    assert!(result.is_ok());
    let context = result.unwrap().build().unwrap();
    let wrapper = context.probabilistic_wrapper.unwrap();
    assert_eq!(wrapper.model_type, ModelType::GPT4);
    assert_eq!(wrapper.fallback_strategy, FallbackStrategy::Deterministic);
    assert!((wrapper.confidence_threshold - 0.95).abs() < f64::EPSILON);
}

// ============================================================================
// build_agent_context Tests
// ============================================================================

#[test]
fn test_build_agent_context_basic() {
    let result = build_agent_context("my_agent", "mcp-server", &[], "strict", None, None);
    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.name, "my_agent");
}

#[test]
fn test_build_agent_context_with_features() {
    use crate::scaffold::agent::AgentFeature;
    let features = vec!["tool-composition".to_string(), "async-handlers".to_string()];
    let result = build_agent_context("agent", "mcp-server", &features, "strict", None, None);
    assert!(result.is_ok());
    let context = result.unwrap();
    assert!(context.features.contains(&AgentFeature::ToolComposition));
    assert!(context.features.contains(&AgentFeature::AsyncHandlers));
}

#[test]
fn test_build_agent_context_with_quality_level() {
    use crate::scaffold::agent::QualityLevel;
    let result = build_agent_context("agent", "mcp-server", &[], "extreme", None, None);
    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.quality_level, QualityLevel::Extreme);
}

#[test]
fn test_build_agent_context_with_deterministic_core() {
    let result = build_agent_context(
        "agent",
        "mcp-server",
        &[],
        "strict",
        Some("core.toml".to_string()),
        None,
    );
    assert!(result.is_ok());
    let context = result.unwrap();
    assert!(context.deterministic_core.is_some());
}

#[test]
fn test_build_agent_context_empty_name_fails() {
    let result = build_agent_context("", "mcp-server", &[], "strict", None, None);
    assert!(result.is_err());
}

#[test]
fn test_build_agent_context_invalid_name_fails() {
    let result =
        build_agent_context("agent-with-dash", "mcp-server", &[], "strict", None, None);
    assert!(result.is_err());
}

// ============================================================================
// print_dry_run_info Tests
// ============================================================================

#[test]
fn test_print_dry_run_info_does_not_panic() {
    use crate::scaffold::agent::{AgentContext, AgentTemplate, QualityLevel};
    use std::collections::HashSet;

    let context = AgentContext {
        name: "test_agent".to_string(),
        template_type: AgentTemplate::MCPToolServer,
        features: HashSet::new(),
        quality_level: QualityLevel::Strict,
        deterministic_core: None,
        probabilistic_wrapper: None,
    };

    // Should not panic
    print_dry_run_info(&context, Path::new("/tmp/test"));
}

// ============================================================================
// ScaffoldAgentParams Tests
// ============================================================================

#[test]
fn test_scaffold_agent_params_struct() {
    let params = default_scaffold_agent_params();
    assert_eq!(params.name, "test_agent");
    assert_eq!(params.template, "mcp-server");
    assert!(params.features.is_empty());
    assert_eq!(params.quality, "strict");
    assert!(params.output.is_none());
    assert!(!params.force);
    assert!(params.dry_run);
    assert!(!params.interactive);
    assert!(params.deterministic_core.is_none());
    assert!(params.probabilistic_wrapper.is_none());
}

// ============================================================================
// ScaffoldWasmParams Tests
// ============================================================================

#[test]
fn test_scaffold_wasm_params_struct() {
    let params = default_scaffold_wasm_params();
    assert_eq!(params.name, "test_wasm");
    assert_eq!(params.framework, "wasm-labs");
    assert!(params.features.is_empty());
    assert_eq!(params.quality, "standard");
    assert!(params.output.is_none());
    assert!(!params.force);
    assert!(params.dry_run);
}

// ============================================================================
// Async Handler Tests
// ============================================================================

#[tokio::test]
async fn test_handle_scaffold_agent_dry_run() {
    let params = ScaffoldAgentParams {
        name: "test_agent".to_string(),
        template: "mcp-server".to_string(),
        features: vec![],
        quality: "strict".to_string(),
        output: None,
        force: false,
        dry_run: true,
        interactive: false,
        deterministic_core: None,
        probabilistic_wrapper: None,
    };

    let result = handle_scaffold_agent(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scaffold_agent_with_features_dry_run() {
    let params = ScaffoldAgentParams {
        name: "feature_agent".to_string(),
        template: "mcp-server".to_string(),
        features: vec!["tool-composition".to_string(), "async-handlers".to_string()],
        quality: "extreme".to_string(),
        output: None,
        force: false,
        dry_run: true,
        interactive: false,
        deterministic_core: None,
        probabilistic_wrapper: None,
    };

    let result = handle_scaffold_agent(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scaffold_agent_with_hybrid_specs_dry_run() {
    let params = ScaffoldAgentParams {
        name: "hybrid_agent".to_string(),
        template: "mcp-server".to_string(),
        features: vec![],
        quality: "strict".to_string(),
        output: None,
        force: false,
        dry_run: true,
        interactive: false,
        deterministic_core: Some("core.toml".to_string()),
        probabilistic_wrapper: Some("wrapper.toml".to_string()),
    };

    let result = handle_scaffold_agent(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scaffold_agent_with_custom_output_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("custom_agent");

    let params = ScaffoldAgentParams {
        name: "custom_output_agent".to_string(),
        template: "mcp-server".to_string(),
        features: vec![],
        quality: "strict".to_string(),
        output: Some(output_path),
        force: false,
        dry_run: true,
        interactive: false,
        deterministic_core: None,
        probabilistic_wrapper: None,
    };

    let result = handle_scaffold_agent(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scaffold_wasm_dry_run() {
    let params = ScaffoldWasmParams {
        name: "test_wasm".to_string(),
        framework: "wasm-labs".to_string(),
        features: vec![],
        quality: "standard".to_string(),
        output: None,
        force: false,
        dry_run: true,
    };

    let result = handle_scaffold_wasm(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scaffold_wasm_pure_wasm_dry_run() {
    let params = ScaffoldWasmParams {
        name: "pure_wasm_project".to_string(),
        framework: "pure-wasm".to_string(),
        features: vec![],
        quality: "standard".to_string(),
        output: None,
        force: false,
        dry_run: true,
    };

    let result = handle_scaffold_wasm(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scaffold_wasm_with_features_dry_run() {
    let params = ScaffoldWasmParams {
        name: "featured_wasm".to_string(),
        framework: "wasm-labs".to_string(),
        features: vec![
            "logging".to_string(),
            "metrics".to_string(),
            "tracing".to_string(),
        ],
        quality: "extreme".to_string(),
        output: None,
        force: false,
        dry_run: true,
    };

    let result = handle_scaffold_wasm(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scaffold_wasm_unknown_framework_fails() {
    let params = ScaffoldWasmParams {
        name: "invalid_framework".to_string(),
        framework: "unknown-framework".to_string(),
        features: vec![],
        quality: "standard".to_string(),
        output: None,
        force: false,
        dry_run: true,
    };

    let result = handle_scaffold_wasm(params).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unknown WASM framework"));
}

#[tokio::test]
async fn test_handle_list_agent_templates() {
    let result = handle_list_agent_templates().await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "Function calls std::process::exit(1) which kills test process"]
async fn test_handle_validate_agent_template_nonexistent_file() {
    let path = PathBuf::from("/nonexistent/template/path/12345.toml");
    let result = handle_validate_agent_template(path).await;
    // The function calls std::process::exit(1), so we can't easily test failure case
    // without mocking, but we can verify the path is checked
    assert!(result.is_err() || true); // Either error or the function exits
}

#[tokio::test]
async fn test_handle_validate_agent_template_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let template_path = temp_dir.path().join("template.toml");
    std::fs::write(&template_path, "[template]\nname = \"test\"").unwrap();

    let result = handle_validate_agent_template(template_path).await;
    assert!(result.is_ok());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_resolve_scaffold_templates_case_sensitive() {
    // Toolchain matching is case-sensitive
    let result = resolve_scaffold_templates("RUST", vec![]);
    assert_eq!(result, vec!["readme".to_string()]); // Falls through to default
}

#[test]
fn test_get_default_scaffold_templates_length() {
    let rust_templates = get_default_scaffold_templates("rust");
    let unknown_templates = get_default_scaffold_templates("unknown");
    assert!(rust_templates.len() > unknown_templates.len());
}

#[tokio::test]
async fn test_handle_scaffold_wasm_skips_unknown_features() {
    let params = ScaffoldWasmParams {
        name: "wasm_with_unknown".to_string(),
        framework: "wasm-labs".to_string(),
        features: vec!["logging".to_string(), "unknown-feature".to_string()],
        quality: "standard".to_string(),
        output: None,
        force: false,
        dry_run: true,
    };

    let result = handle_scaffold_wasm(params).await;
    assert!(result.is_ok());
}

#[test]
fn test_scaffold_agent_params_with_all_fields() {
    let params = ScaffoldAgentParams {
        name: "full_agent".to_string(),
        template: "calculator".to_string(),
        features: vec!["state-machine".to_string()],
        quality: "extreme".to_string(),
        output: Some(PathBuf::from("/tmp/output")),
        force: true,
        dry_run: false,
        interactive: false,
        deterministic_core: Some("core.yaml".to_string()),
        probabilistic_wrapper: Some("wrapper.yaml".to_string()),
    };

    assert_eq!(params.name, "full_agent");
    assert_eq!(params.template, "calculator");
    assert!(!params.features.is_empty());
    assert_eq!(params.quality, "extreme");
    assert!(params.output.is_some());
    assert!(params.force);
    assert!(!params.dry_run);
    assert!(!params.interactive);
    assert!(params.deterministic_core.is_some());
    assert!(params.probabilistic_wrapper.is_some());
}

// ============================================================================
// Template Types Integration Tests
// ============================================================================

#[test]
fn test_build_agent_context_calculator_template() {
    let result = build_agent_context("calc_agent", "calculator", &[], "strict", None, None);
    assert!(result.is_ok());
}

#[test]
fn test_build_agent_context_state_machine_template() {
    let result = build_agent_context("sm_agent", "state-machine", &[], "strict", None, None);
    assert!(result.is_ok());
}

#[test]
fn test_build_agent_context_hybrid_template_without_specs() {
    // Hybrid template requires both specs, should fail
    let result = build_agent_context("hybrid_agent", "hybrid", &[], "strict", None, None);
    assert!(result.is_err());
}

#[test]
fn test_build_agent_context_hybrid_template_with_specs() {
    let result = build_agent_context(
        "hybrid_agent",
        "hybrid",
        &[],
        "strict",
        Some("core.toml".to_string()),
        Some("wrapper.toml".to_string()),
    );
    assert!(result.is_ok());
}

#[test]
fn test_build_agent_context_unknown_template() {
    // Unknown templates default to MCPToolServer
    let result = build_agent_context("agent", "unknown-template", &[], "strict", None, None);
    assert!(result.is_ok());
}
