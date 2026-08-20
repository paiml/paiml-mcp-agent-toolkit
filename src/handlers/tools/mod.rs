#![cfg_attr(coverage_nightly, coverage(off))]
// Tools handlers - split for file health (CB-040)
include!("core_tools.rs");
include!("extended_tools.rs");

#[cfg(test)]
mod tests {
    //! PMAT-647: cover extended_tools_complexity.rs pure helpers.
    use super::*;
    use serde_json::json;
    use std::path::Path;

    fn mkdir(p: &Path) {
        std::fs::create_dir_all(p).expect("mkdir");
    }
    fn touch(p: &Path) {
        if let Some(parent) = p.parent() {
            mkdir(parent);
        }
        std::fs::write(p, "").expect("touch");
    }

    // --- parse_complexity_args ---

    #[test]
    fn test_parse_complexity_args_valid() {
        let args = parse_complexity_args(json!({
            "project_path": "/tmp",
            "toolchain": "rust",
            "max_cyclomatic": 10,
        }))
        .expect("valid parse");
        assert_eq!(args.project_path.as_deref(), Some("/tmp"));
        assert_eq!(args.toolchain.as_deref(), Some("rust"));
        assert_eq!(args.max_cyclomatic, Some(10));
    }

    #[test]
    fn test_parse_complexity_args_empty_object_all_none() {
        let args = parse_complexity_args(json!({})).expect("empty object is valid");
        assert!(args.project_path.is_none());
        assert!(args.toolchain.is_none());
        assert!(args.max_cyclomatic.is_none());
    }

    #[test]
    fn test_parse_complexity_args_wrong_types_return_err() {
        let result = parse_complexity_args(json!({"max_cyclomatic": "not-a-number"}));
        let err = result.unwrap_err();
        assert!(err.contains("Invalid analyze_complexity arguments"));
    }

    // --- detect_toolchain ---

    #[test]
    fn test_detect_toolchain_explicit_arg_wins() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Create Cargo.toml, but explicit arg should win.
        touch(&tmp.path().join("Cargo.toml"));
        let t = detect_toolchain(&Some("deno".to_string()), tmp.path());
        assert_eq!(t, "deno");
    }

    #[test]
    fn test_detect_toolchain_detects_rust_from_cargo_toml() {
        let tmp = tempfile::TempDir::new().unwrap();
        touch(&tmp.path().join("Cargo.toml"));
        assert_eq!(detect_toolchain(&None, tmp.path()), "rust");
    }

    #[test]
    fn test_detect_toolchain_detects_deno_from_package_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        touch(&tmp.path().join("package.json"));
        assert_eq!(detect_toolchain(&None, tmp.path()), "deno");
    }

    #[test]
    fn test_detect_toolchain_detects_deno_from_deno_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        touch(&tmp.path().join("deno.json"));
        assert_eq!(detect_toolchain(&None, tmp.path()), "deno");
    }

    #[test]
    fn test_detect_toolchain_detects_python_uv_from_pyproject() {
        let tmp = tempfile::TempDir::new().unwrap();
        touch(&tmp.path().join("pyproject.toml"));
        assert_eq!(detect_toolchain(&None, tmp.path()), "python-uv");
    }

    #[test]
    fn test_detect_toolchain_detects_python_uv_from_requirements_txt() {
        let tmp = tempfile::TempDir::new().unwrap();
        touch(&tmp.path().join("requirements.txt"));
        assert_eq!(detect_toolchain(&None, tmp.path()), "python-uv");
    }

    #[test]
    fn test_detect_toolchain_defaults_to_rust() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Empty project, no manifests.
        assert_eq!(detect_toolchain(&None, tmp.path()), "rust");
    }

    // --- build_complexity_thresholds ---

    #[test]
    fn test_build_complexity_thresholds_defaults_when_none() {
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: None,
            top_files: None,
        };
        let t = build_complexity_thresholds(&args);
        // Defaults come from ComplexityThresholds::default(); just verify they're non-zero.
        assert!(t.cyclomatic_error > 0);
        assert!(t.cognitive_error > 0);
    }

    #[test]
    fn test_build_complexity_thresholds_sets_cyclomatic() {
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: Some(20),
            max_cognitive: None,
            include: None,
            top_files: None,
        };
        let t = build_complexity_thresholds(&args);
        assert_eq!(t.cyclomatic_error, 20);
        assert_eq!(t.cyclomatic_warn, 15); // 20 * 3 / 4
    }

    #[test]
    fn test_build_complexity_thresholds_sets_cognitive() {
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: None,
            max_cognitive: Some(12),
            include: None,
            top_files: None,
        };
        let t = build_complexity_thresholds(&args);
        assert_eq!(t.cognitive_error, 12);
        assert_eq!(t.cognitive_warn, 9); // 12 * 3 / 4
    }

    #[test]
    fn test_build_complexity_thresholds_min_warn_is_one() {
        // `max * 3 / 4` floors to 0 for small values; .max(1) clamps to 1.
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: Some(1),
            max_cognitive: Some(1),
            include: None,
            top_files: None,
        };
        let t = build_complexity_thresholds(&args);
        assert_eq!(t.cyclomatic_warn, 1);
        assert_eq!(t.cognitive_warn, 1);
    }

    // --- should_analyze_file ---

    #[test]
    fn test_should_analyze_file_rust_only_rs() {
        assert!(should_analyze_file(Path::new("main.rs"), "rust"));
        assert!(!should_analyze_file(Path::new("main.ts"), "rust"));
        assert!(!should_analyze_file(Path::new("main"), "rust"));
    }

    #[test]
    fn test_should_analyze_file_deno_accepts_ts_tsx_js_jsx() {
        for ext in ["ts", "tsx", "js", "jsx"] {
            let path = format!("file.{ext}");
            assert!(
                should_analyze_file(Path::new(&path), "deno"),
                "failed for {ext}"
            );
        }
        assert!(!should_analyze_file(Path::new("file.py"), "deno"));
    }

    #[test]
    fn test_should_analyze_file_python_uv_only_py() {
        assert!(should_analyze_file(Path::new("x.py"), "python-uv"));
        assert!(!should_analyze_file(Path::new("x.rs"), "python-uv"));
    }

    #[test]
    fn test_should_analyze_file_unknown_toolchain_rejects_all() {
        assert!(!should_analyze_file(Path::new("x.rs"), "unknown-toolchain"));
        assert!(!should_analyze_file(Path::new("x.py"), ""));
    }

    // --- matches_include_filters ---

    #[test]
    fn test_matches_include_filters_none_returns_true() {
        assert!(matches_include_filters(Path::new("src/main.rs"), &None));
    }

    #[test]
    fn test_matches_include_filters_empty_vec_returns_true() {
        assert!(matches_include_filters(
            Path::new("src/main.rs"),
            &Some(vec![])
        ));
    }

    #[test]
    fn test_matches_include_filters_any_pattern_matches() {
        let patterns = Some(vec!["*.rs".to_string(), "lib".to_string()]);
        assert!(matches_include_filters(Path::new("src/main.rs"), &patterns));
    }

    #[test]
    fn test_matches_include_filters_no_pattern_matches_is_false() {
        let patterns = Some(vec!["*.py".to_string(), "*.ts".to_string()]);
        assert!(!matches_include_filters(
            Path::new("src/main.rs"),
            &patterns
        ));
    }

    // --- matches_pattern ---

    #[test]
    fn test_matches_pattern_double_star_matches_trailing_suffix() {
        assert!(matches_pattern("src/a/b/c.rs", "src/**/c.rs"));
    }

    #[test]
    fn test_matches_pattern_double_star_with_three_parts_returns_false() {
        // "**" splits → 3 parts → fallback to false.
        assert!(!matches_pattern("a/b/c", "a/**/b/**"));
    }

    #[test]
    fn test_matches_pattern_extension_wildcard() {
        assert!(matches_pattern("src/main.rs", "*.rs"));
        assert!(!matches_pattern("src/main.ts", "*.rs"));
    }

    #[test]
    fn test_matches_pattern_substring_fallback() {
        assert!(matches_pattern("path/to/my-handler.rs", "my-handler"));
        assert!(!matches_pattern("path/to/other.rs", "my-handler"));
    }

    // --- format_complexity_output ---

    fn empty_report() -> crate::services::complexity::ComplexityReport {
        crate::services::complexity::aggregate_results(Vec::new())
    }

    fn args_with_format(fmt: &str) -> AnalyzeComplexityArgs {
        AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: Some(fmt.to_string()),
            max_cyclomatic: None,
            max_cognitive: None,
            include: None,
            top_files: None,
        }
    }

    #[test]
    fn test_format_complexity_output_json_returns_valid_json() {
        let rpt = empty_report();
        let out = format_complexity_output(&rpt, &args_with_format("json"));
        let _: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    }

    #[test]
    fn test_format_complexity_output_sarif_contains_recognizable_content() {
        let rpt = empty_report();
        let out = format_complexity_output(&rpt, &args_with_format("sarif"));
        // Valid SARIF is JSON with a schema URL, or on fallback an error string.
        assert!(!out.is_empty());
    }

    #[test]
    fn test_format_complexity_output_full_returns_non_empty() {
        let rpt = empty_report();
        let out = format_complexity_output(&rpt, &args_with_format("full"));
        assert!(!out.is_empty());
    }

    #[test]
    fn test_format_complexity_output_default_summary_when_no_format() {
        let rpt = empty_report();
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: None,
            max_cyclomatic: None,
            max_cognitive: None,
            include: None,
            top_files: None,
        };
        let out = format_complexity_output(&rpt, &args);
        assert!(!out.is_empty());
    }

    // --- generate_complexity_content ---

    #[test]
    fn test_generate_complexity_content_top_files_zero_falls_back_to_summary() {
        let rpt = empty_report();
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: Some("summary".into()),
            max_cyclomatic: None,
            max_cognitive: None,
            include: None,
            top_files: Some(0),
        };
        let out = generate_complexity_content(&rpt, &[], &args);
        assert!(!out.is_empty());
    }

    #[test]
    fn test_generate_complexity_content_no_top_files_uses_summary() {
        let rpt = empty_report();
        let args = args_with_format("summary");
        let out = generate_complexity_content(&rpt, &[], &args);
        assert!(!out.is_empty());
    }

    #[test]
    fn test_generate_complexity_content_top_files_nonzero_uses_rankings() {
        let rpt = empty_report();
        let args = AnalyzeComplexityArgs {
            project_path: None,
            toolchain: None,
            format: Some("summary".into()),
            max_cyclomatic: None,
            max_cognitive: None,
            include: None,
            top_files: Some(5),
        };
        let out = generate_complexity_content(&rpt, &[], &args);
        // With empty file_metrics, rankings produce an empty table header.
        assert!(out.contains("Top 0") || out.contains("Top"));
    }

    // --- format_complexity_rankings json path ---

    #[test]
    fn test_format_complexity_rankings_json_output_has_rankings_key() {
        let args = args_with_format("json");
        let rankings: Vec<(String, crate::services::ranking::CompositeComplexityScore)> = vec![];
        let out = format_complexity_rankings(&rankings, &args);
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(v.get("rankings").is_some());
        assert!(v.get("analysis_type").is_some());
    }

    #[test]
    fn test_format_complexity_rankings_table_output_has_header() {
        let args = args_with_format("summary");
        let rankings: Vec<(String, crate::services::ranking::CompositeComplexityScore)> = vec![];
        let out = format_complexity_rankings(&rankings, &args);
        assert!(out.contains("Top 0 Complexity Files"));
        assert!(out.contains("| Rank |"));
    }

    // --- resolve_project_path_complexity error path ---

    #[test]
    fn test_resolve_project_path_complexity_none_is_err() {
        let err = resolve_project_path_complexity(None).unwrap_err();
        // Must fail — the D101 rule rejects None/missing values.
        assert!(!err.is_empty());
    }

    #[test]
    fn test_resolve_project_path_complexity_empty_string_is_err() {
        let err = resolve_project_path_complexity(Some(String::new())).unwrap_err();
        assert!(!err.is_empty());
    }
}

// ============================================================================
// R22-1 / D101 regression tests — cwd-exfiltration parity fix
// ============================================================================
//
// Proves that the live `src/handlers/tools/` dispatcher now rejects missing,
// null, and empty/whitespace `project_path` with JSON-RPC `-32602` across
// every analysis handler that previously fell back to
// `std::env::current_dir()`. Mirrors the R21-5 / D99 test layout in
// `src/agent/mcp_server_tests.rs` (PR #371).

#[cfg(test)]
mod r22_1_d101_cwd_guard_tests {
    use super::*;
    use serde_json::json;

    fn assert_invalid_params(response: &McpResponse, context: &str) {
        assert!(
            response.error.is_some(),
            "{context}: expected an error response, got success: {response:?}"
        );
        let err = response.error.as_ref().unwrap();
        assert_eq!(
            err.code, -32602,
            "{context}: expected JSON-RPC -32602 (Invalid params), got {}: message={}",
            err.code, err.message
        );
        assert!(
            err.message.contains("project_path"),
            "{context}: error message should name the offending field: {}",
            err.message
        );
    }

    // --- handle_analyze_complexity ----------------------------------------

    #[tokio::test]
    async fn analyze_complexity_rejects_missing_project_path() {
        let response = handle_analyze_complexity(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_complexity / missing");
    }

    #[tokio::test]
    async fn analyze_complexity_rejects_null_project_path() {
        let response = handle_analyze_complexity(json!(1), json!({ "project_path": null })).await;
        assert_invalid_params(&response, "analyze_complexity / null");
    }

    #[tokio::test]
    async fn analyze_complexity_rejects_empty_project_path() {
        let response = handle_analyze_complexity(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_complexity / empty");
    }

    #[tokio::test]
    async fn analyze_complexity_rejects_whitespace_project_path() {
        let response = handle_analyze_complexity(json!(1), json!({ "project_path": "   " })).await;
        assert_invalid_params(&response, "analyze_complexity / whitespace");
    }

    // --- handle_analyze_code_churn ----------------------------------------

    #[tokio::test]
    async fn analyze_code_churn_rejects_missing_project_path() {
        let response = handle_analyze_code_churn(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_code_churn / missing");
    }

    #[tokio::test]
    async fn analyze_code_churn_rejects_empty_project_path() {
        let response = handle_analyze_code_churn(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_code_churn / empty");
    }

    // --- handle_analyze_dag -----------------------------------------------

    #[tokio::test]
    async fn analyze_dag_rejects_missing_project_path() {
        let response = handle_analyze_dag(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_dag / missing");
    }

    #[tokio::test]
    async fn analyze_dag_rejects_empty_project_path() {
        let response = handle_analyze_dag(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_dag / empty");
    }

    /// #1020: this handler built the graph with the 400-edge budget already
    /// applied — which deletes every node the surviving edges do not touch — and
    /// then filtered for `Calls` edges that nothing on this path ever created.
    /// `dag_type: "call-graph"` was empty for EVERY project, of any size.
    #[tokio::test]
    async fn analyze_dag_call_graph_is_not_empty_over_rust_sources() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("main.rs"),
            "fn main() { helper_one(); }\nfn helper_one() { helper_two(); }\nfn helper_two() {}\n",
        )
        .expect("write");

        let response = handle_analyze_dag(
            json!(1),
            json!({ "project_path": dir.path().display().to_string(), "dag_type": "call-graph" }),
        )
        .await;

        let result = response.result.expect("analyze_dag must succeed");
        assert_eq!(result["graph_type"], "CallGraph");
        assert!(
            result["edges"].as_u64().unwrap_or(0) > 0,
            "call graph over Rust sources came back with {} nodes / {} edges",
            result["nodes"],
            result["edges"]
        );
    }

    // --- handle_generate_context ------------------------------------------

    #[tokio::test]
    async fn generate_context_rejects_missing_project_path() {
        let response = handle_generate_context(json!(1), json!({})).await;
        assert_invalid_params(&response, "generate_context / missing");
    }

    #[tokio::test]
    async fn generate_context_rejects_empty_project_path() {
        let response = handle_generate_context(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "generate_context / empty");
    }

    // --- handle_analyze_system_architecture -------------------------------

    #[tokio::test]
    async fn analyze_system_architecture_rejects_missing_project_path() {
        let response = handle_analyze_system_architecture(json!(1), json!({})).await;
        assert_invalid_params(&response, "analyze_system_architecture / missing");
    }

    #[tokio::test]
    async fn analyze_system_architecture_rejects_empty_project_path() {
        let response =
            handle_analyze_system_architecture(json!(1), json!({ "project_path": "" })).await;
        assert_invalid_params(&response, "analyze_system_architecture / empty");
    }

    // --- require_project_path helper --------------------------------------

    #[test]
    fn require_project_path_accepts_nonempty() {
        let out = require_project_path(Some("/tmp/x".to_string())).unwrap();
        assert_eq!(out, std::path::PathBuf::from("/tmp/x"));
    }

    #[test]
    fn require_project_path_rejects_none() {
        let err = require_project_path(None).unwrap_err();
        assert!(err.contains("project_path"), "{err}");
        assert!(err.contains("D101"), "{err}");
    }

    #[test]
    fn require_project_path_rejects_empty() {
        let err = require_project_path(Some(String::new())).unwrap_err();
        assert!(err.contains("non-empty"), "{err}");
    }

    #[test]
    fn require_project_path_rejects_whitespace() {
        let err = require_project_path(Some("  \n\t ".to_string())).unwrap_err();
        assert!(err.contains("non-empty"), "{err}");
    }
}
