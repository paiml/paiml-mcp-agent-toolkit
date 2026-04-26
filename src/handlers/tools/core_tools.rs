use crate::models::churn::ChurnOutputFormat;

// Import handlers from extracted module (CB-040)
use crate::handlers::tools_advanced::{
    handle_analyze_dead_code, handle_analyze_deep_context, handle_analyze_lint_hotspot,
    handle_analyze_makefile_lint, handle_analyze_provability, handle_analyze_satd,
    handle_analyze_tdg, handle_quality_driven_development,
};
use crate::models::mcp::{
    GenerateTemplateArgs, ListTemplatesArgs, McpRequest, McpResponse, ScaffoldProjectArgs,
    SearchTemplatesArgs, ToolCallParams, ValidateTemplateArgs,
};
use crate::models::template::{ParameterSpec, TemplateResource};
use crate::services::git_analysis::GitAnalysisService;
use crate::services::template_service;
use crate::TemplateServerTrait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info};

/// R22-1 / D101 — Reject missing, null, or empty/whitespace-only
/// `project_path` arguments across MCP analysis handlers.
///
/// This is the parity fix for R21-5 / D99 (PR #371), which added the same
/// guard to the `src/agent/mcp_server_protocol.rs` handlers. The live
/// dispatcher in `src/handlers/tools/` was still silently defaulting to
/// `std::env::current_dir()`, which lets a remote MCP client exfiltrate
/// information about the server's launch directory by sending `{}` or
/// `{"project_path": null}` / `{"project_path": ""}`.
///
/// The helper takes the already-deserialised `Option<String>` (since the
/// handlers use typed arg structs like `AnalyzeComplexityArgs`) and returns
/// a validated `PathBuf` or an error string that the caller maps to
/// JSON-RPC `-32602` (Invalid params).
///
/// Mirrors the R21-1 / D100 fail-loud pattern already in
/// `handle_analyze_dead_code`.
fn require_project_path(project_path_arg: Option<String>) -> Result<PathBuf, String> {
    let Some(raw) = project_path_arg else {
        return Err(
            "'project_path' is required and must be a non-empty string — \
null/missing is rejected to avoid silently analyzing the server's current \
directory (R22-1 / D101)"
                .to_string(),
        );
    };
    if raw.trim().is_empty() {
        return Err(
            "'project_path' must be a non-empty string — empty/whitespace values \
are rejected to avoid silently analyzing the server's current directory \
(R22-1 / D101)"
                .to_string(),
        );
    }
    Ok(PathBuf::from(raw))
}

// --- Shared project_path glob resolution (R22-2 / D102) ---
include!("core_tools_path_resolve.rs");

// --- Tool call dispatch (routing, classification) ---
include!("core_tools_dispatch.rs");

// --- Template tool handlers (generate, list, validate, scaffold, search, server info) ---
include!("core_tools_template_handlers.rs");

// --- Code churn analysis handlers and formatters ---
include!("core_tools_churn.rs");

#[cfg(test)]
mod template_handlers_tests {
    //! Wave 38 PR1 — pure-helper coverage for core_tools_template_handlers.rs.
    //! Async generic handlers (handle_generate_template, handle_list_templates,
    //! handle_validate_template, handle_scaffold_project, handle_search_templates)
    //! are NOT exercised here — they require a real TemplateServerTrait + tokio
    //! runtime + fixture templates.
    use super::*;
    use crate::models::template::{
        ParameterSpec, ParameterType, TemplateCategory, TemplateResource, Toolchain,
    };

    fn make_param_spec(
        name: &str,
        required: bool,
        validation_pattern: Option<&str>,
    ) -> ParameterSpec {
        ParameterSpec {
            name: name.to_string(),
            param_type: ParameterType::String,
            required,
            default_value: None,
            validation_pattern: validation_pattern.map(String::from),
            description: String::new(),
        }
    }

    fn make_template_resource(uri: &str, params: Vec<ParameterSpec>) -> TemplateResource {
        TemplateResource {
            uri: uri.to_string(),
            name: "test".to_string(),
            description: "test desc".to_string(),
            toolchain: Toolchain::RustCli {
                cargo_features: vec![],
            },
            category: TemplateCategory::Makefile,
            parameters: params,
            s3_object_key: "k".to_string(),
            content_hash: "h".to_string(),
            semantic_version: semver::Version::parse("1.0.0").unwrap(),
            dependency_graph: vec![],
        }
    }

    // ── parse_validate_template_args ────────────────────────────────────────

    #[test]
    fn test_parse_validate_template_args_minimal_ok() {
        // PIN: ValidateTemplateArgs requires BOTH `resource_uri` AND `parameters`
        // (parameters can be an empty Map). Missing `parameters` rejects.
        let v = serde_json::json!({
            "resource_uri": "template://makefile/rust/cli",
            "parameters": {},
        });
        let args = parse_validate_template_args(v).unwrap();
        assert_eq!(args.resource_uri, "template://makefile/rust/cli");
    }

    #[test]
    fn test_parse_validate_template_args_missing_parameters_field_rejected() {
        // PIN: bare resource_uri without parameters is rejected — handle_validate_template
        // surfaces this as "Missing required field: parameters" via a special error path.
        let v = serde_json::json!({"resource_uri": "template://makefile/rust/cli"});
        assert!(parse_validate_template_args(v).is_err());
    }

    #[test]
    fn test_parse_validate_template_args_empty_object_rejected() {
        let v = serde_json::json!({});
        assert!(parse_validate_template_args(v).is_err());
    }

    // ── find_missing_required_parameters ────────────────────────────────────

    #[test]
    fn test_find_missing_required_parameters_all_present_returns_empty() {
        let mut params = serde_json::Map::new();
        params.insert("name".to_string(), serde_json::json!("foo"));
        let specs = vec![make_param_spec("name", true, None)];
        assert!(find_missing_required_parameters(&params, &specs).is_empty());
    }

    #[test]
    fn test_find_missing_required_parameters_required_absent_returned() {
        let params = serde_json::Map::new();
        let specs = vec![make_param_spec("name", true, None)];
        let missing = find_missing_required_parameters(&params, &specs);
        assert_eq!(missing, vec!["name".to_string()]);
    }

    #[test]
    fn test_find_missing_required_parameters_optional_absent_not_returned() {
        // PIN: only required=true triggers the check; optional missing fields are silently OK.
        let params = serde_json::Map::new();
        let specs = vec![make_param_spec("name", false, None)];
        assert!(find_missing_required_parameters(&params, &specs).is_empty());
    }

    #[test]
    fn test_find_missing_required_parameters_multiple_missing() {
        let params = serde_json::Map::new();
        let specs = vec![
            make_param_spec("a", true, None),
            make_param_spec("b", true, None),
            make_param_spec("c", false, None),
        ];
        let missing = find_missing_required_parameters(&params, &specs);
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"a".to_string()));
        assert!(missing.contains(&"b".to_string()));
    }

    // ── validate_single_parameter ───────────────────────────────────────────

    #[test]
    fn test_validate_single_parameter_no_pattern_returns_none() {
        let spec = make_param_spec("name", true, None);
        assert!(validate_single_parameter("name", &serde_json::json!("anything"), &spec).is_none());
    }

    #[test]
    fn test_validate_single_parameter_pattern_match_returns_none() {
        let spec = make_param_spec("name", true, Some(r"^[a-z]+$"));
        assert!(validate_single_parameter("name", &serde_json::json!("abc"), &spec).is_none());
    }

    #[test]
    fn test_validate_single_parameter_pattern_no_match_returns_err() {
        let spec = make_param_spec("name", true, Some(r"^[a-z]+$"));
        let err = validate_single_parameter("name", &serde_json::json!("ABC"), &spec).unwrap();
        assert!(err.contains("does not match pattern"));
        assert!(err.contains("'name'"));
    }

    #[test]
    fn test_validate_single_parameter_invalid_pattern_silently_passes() {
        // PIN: an invalid regex (e.g. unmatched paren) silently returns None — no error
        // surfaced to caller. Fragile but pinned to prevent silent regression.
        let spec = make_param_spec("name", true, Some(r"[invalid("));
        assert!(validate_single_parameter("name", &serde_json::json!("anything"), &spec).is_none());
    }

    #[test]
    fn test_validate_single_parameter_non_string_value_silently_passes() {
        // PIN: only str values are checked against the regex; other JSON types
        // (numbers, bools, arrays) silently pass even if a pattern is set.
        let spec = make_param_spec("count", true, Some(r"^\d+$"));
        assert!(validate_single_parameter("count", &serde_json::json!(42), &spec).is_none());
        assert!(validate_single_parameter("flag", &serde_json::json!(true), &spec).is_none());
    }

    // ── validate_parameter_values ───────────────────────────────────────────

    #[test]
    fn test_validate_parameter_values_unknown_param_returns_err() {
        let mut params = serde_json::Map::new();
        params.insert("unknown_x".to_string(), serde_json::json!("v"));
        let specs = vec![make_param_spec("known", true, None)];
        let errs = validate_parameter_values(&params, &specs);
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("Unknown parameter: unknown_x"));
    }

    #[test]
    fn test_validate_parameter_values_known_param_with_pattern_collected() {
        let mut params = serde_json::Map::new();
        params.insert("name".to_string(), serde_json::json!("ABC"));
        let specs = vec![make_param_spec("name", true, Some(r"^[a-z]+$"))];
        let errs = validate_parameter_values(&params, &specs);
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("does not match pattern"));
    }

    #[test]
    fn test_validate_parameter_values_all_valid_returns_empty() {
        let mut params = serde_json::Map::new();
        params.insert("a".to_string(), serde_json::json!("abc"));
        params.insert("b".to_string(), serde_json::json!("xyz"));
        let specs = vec![
            make_param_spec("a", true, None),
            make_param_spec("b", true, None),
        ];
        assert!(validate_parameter_values(&params, &specs).is_empty());
    }

    // ── validate_template_parameters (composes find_missing + validate_values) ──

    #[test]
    fn test_validate_template_parameters_clean_path() {
        let mut params = serde_json::Map::new();
        params.insert("name".to_string(), serde_json::json!("ok"));
        let resource =
            make_template_resource("template://x", vec![make_param_spec("name", true, None)]);
        let r = validate_template_parameters(&params, &resource);
        assert!(r.missing_required.is_empty());
        assert!(r.validation_errors.is_empty());
    }

    #[test]
    fn test_validate_template_parameters_finds_both_missing_and_invalid() {
        let mut params = serde_json::Map::new();
        params.insert("a".to_string(), serde_json::json!("BAD"));
        let resource = make_template_resource(
            "template://x",
            vec![
                make_param_spec("a", true, Some(r"^[a-z]+$")),
                make_param_spec("b", true, None),
            ],
        );
        let r = validate_template_parameters(&params, &resource);
        assert_eq!(r.missing_required, vec!["b".to_string()]);
        assert_eq!(r.validation_errors.len(), 1);
        assert!(r.validation_errors[0].contains("does not match pattern"));
    }

    // ── create_validation_response ──────────────────────────────────────────

    #[test]
    fn test_create_validation_response_valid_path() {
        let result = ValidationResult {
            missing_required: vec![],
            validation_errors: vec![],
        };
        let resp = create_validation_response(serde_json::json!(1), result, "template://x");
        let v = serde_json::to_value(&resp).unwrap();
        let payload = v.get("result").unwrap();
        assert_eq!(payload["valid"], true);
        assert_eq!(payload["template_uri"], "template://x");
    }

    #[test]
    fn test_create_validation_response_invalid_path_counts_both_failure_types() {
        let result = ValidationResult {
            missing_required: vec!["x".to_string(), "y".to_string()],
            validation_errors: vec!["bad pattern for z".to_string()],
        };
        let resp = create_validation_response(serde_json::json!(1), result, "template://x");
        let v = serde_json::to_value(&resp).unwrap();
        let payload = v.get("result").unwrap();
        assert_eq!(payload["valid"], false);
        assert!(payload["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("3 errors"));
    }

    // ── get_template_variant ────────────────────────────────────────────────

    #[test]
    fn test_get_template_variant_makefile_rust() {
        assert_eq!(get_template_variant("makefile", "rust"), Some("cli"));
    }

    #[test]
    fn test_get_template_variant_readme_deno() {
        assert_eq!(get_template_variant("readme", "deno"), Some("cli"));
    }

    #[test]
    fn test_get_template_variant_gitignore_python() {
        assert_eq!(get_template_variant("gitignore", "python-uv"), Some("cli"));
    }

    #[test]
    fn test_get_template_variant_unknown_template_returns_none() {
        assert!(get_template_variant("unknown_type", "rust").is_none());
    }

    #[test]
    fn test_get_template_variant_unsupported_toolchain_returns_none() {
        // PIN: makefile + java is NOT supported despite makefile being valid.
        // The match is on (template_type, toolchain) jointly.
        assert!(get_template_variant("makefile", "java").is_none());
    }
}
