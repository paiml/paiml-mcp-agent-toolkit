#![cfg_attr(coverage_nightly, coverage(off))]
//! OpenAPI coverage tests for HTTP server

// Coverage-instrumented tests (NOT coverage(off)) for generate_openapi_spec
#[cfg(test)]
mod openapi_coverage_tests {
    use super::super::openapi::generate_openapi_spec;
    use super::super::types::AppError;
    use serde_json::json;

    #[test]
    fn test_openapi_spec_version_3() {
        let spec = generate_openapi_spec();
        assert_eq!(spec["openapi"], "3.0.0");
    }

    #[test]
    fn test_openapi_spec_info_section() {
        let spec = generate_openapi_spec();
        assert_eq!(spec["info"]["title"], "PMAT API");
        assert!(spec["info"]["version"].is_string());
        assert!(spec["info"]["description"]
            .as_str()
            .unwrap()
            .contains("uniform contracts"));
    }

    #[test]
    fn test_openapi_spec_server_url() {
        let spec = generate_openapi_spec();
        assert!(spec["servers"].is_array());
        assert_eq!(spec["servers"][0]["url"], "http://localhost:8080");
        assert_eq!(spec["servers"][0]["description"], "Local server");
    }

    #[test]
    fn test_openapi_spec_all_api_paths() {
        let spec = generate_openapi_spec();
        let paths = spec["paths"].as_object().unwrap();

        let expected_paths = [
            "/api/analyze/complexity",
            "/api/analyze/satd",
            "/api/analyze/dead-code",
            "/api/analyze/tdg",
            "/api/analyze/lint-hotspot",
            "/api/quality-gate",
            "/api/refactor/auto",
        ];

        for path in &expected_paths {
            assert!(paths.contains_key(*path), "Missing path: {}", path);
            assert!(
                paths[*path]["post"].is_object(),
                "Path {} should have POST",
                path
            );
        }
    }

    #[test]
    fn test_openapi_spec_components_schemas() {
        let spec = generate_openapi_spec();
        let schemas = spec["components"]["schemas"].as_object().unwrap();

        let expected_schemas = [
            "BaseAnalysisContract",
            "AnalyzeComplexityContract",
            "AnalyzeSatdContract",
            "AnalyzeDeadCodeContract",
            "AnalyzeTdgContract",
            "AnalyzeLintHotspotContract",
            "QualityGateContract",
            "RefactorAutoContract",
        ];

        for schema in &expected_schemas {
            assert!(schemas.contains_key(*schema), "Missing schema: {}", schema);
        }
    }

    #[test]
    fn test_openapi_spec_base_contract_required_fields() {
        let spec = generate_openapi_spec();
        let base = &spec["components"]["schemas"]["BaseAnalysisContract"];
        assert_eq!(base["type"], "object");

        let required = base["required"].as_array().unwrap();
        assert!(required.contains(&json!("path")));
    }

    #[test]
    fn test_openapi_spec_base_contract_properties() {
        let spec = generate_openapi_spec();
        let props = &spec["components"]["schemas"]["BaseAnalysisContract"]["properties"];

        assert!(props["path"].is_object());
        assert!(props["format"].is_object());
        assert!(props["output"].is_object());
        assert!(props["top_files"].is_object());
        assert!(props["include_tests"].is_object());
        assert!(props["timeout"].is_object());
    }

    #[test]
    fn test_openapi_spec_format_enum_values() {
        let spec = generate_openapi_spec();
        let format_enum =
            &spec["components"]["schemas"]["BaseAnalysisContract"]["properties"]["format"]["enum"];
        let formats: Vec<&str> = format_enum
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        assert!(formats.contains(&"table"));
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"yaml"));
        assert!(formats.contains(&"markdown"));
        assert!(formats.contains(&"csv"));
        assert!(formats.contains(&"summary"));
    }

    #[test]
    fn test_openapi_spec_refactor_contract() {
        let spec = generate_openapi_spec();
        let refactor = &spec["components"]["schemas"]["RefactorAutoContract"];
        assert_eq!(refactor["type"], "object");

        let required = refactor["required"].as_array().unwrap();
        assert!(required.contains(&json!("file")));

        let props = &refactor["properties"];
        assert!(props["file"].is_object());
        assert!(props["dry_run"].is_object());
        assert!(props["target_complexity"].is_object());
    }

    #[test]
    fn test_openapi_spec_all_post_have_summaries() {
        let spec = generate_openapi_spec();
        let paths = spec["paths"].as_object().unwrap();
        for (path, methods) in paths {
            if let Some(post) = methods.get("post") {
                assert!(
                    post.get("summary").is_some(),
                    "Path {} POST missing summary",
                    path
                );
                let summary = post["summary"].as_str().unwrap();
                assert!(!summary.is_empty(), "Path {} POST has empty summary", path);
            }
        }
    }

    #[test]
    fn test_app_error_from_path_not_found() {
        let err = anyhow::anyhow!("Path not found: /foo/bar");
        let app_err: AppError = AppError::from(err);
        match app_err {
            AppError::BadRequest(msg) => assert!(msg.contains("Path not found")),
            _ => panic!("Expected BadRequest for 'Path not found'"),
        }
    }

    #[test]
    fn test_app_error_from_invalid_timeout() {
        let err = anyhow::anyhow!("Invalid timeout value");
        let app_err: AppError = AppError::from(err);
        match app_err {
            AppError::BadRequest(msg) => assert!(msg.contains("Invalid timeout")),
            _ => panic!("Expected BadRequest for 'Invalid timeout'"),
        }
    }

    #[test]
    fn test_app_error_from_generic_error() {
        let err = anyhow::anyhow!("something went wrong");
        let app_err: AppError = AppError::from(err);
        match app_err {
            AppError::Internal(_) => {}
            _ => panic!("Expected Internal for generic error"),
        }
    }
}
