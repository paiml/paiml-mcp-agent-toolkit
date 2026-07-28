//! Tests for SimpleMcpHandler

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::super::SimpleMcpHandler;
    use serde_json::json;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// Helper function to create a temporary test directory with a Rust file
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            r#"
fn main() {
    println!("Hello, world!");
}

// TODO: Fix this later
fn unused_function() {
    let x = 1;
    if x > 0 {
        println!("positive");
    }
}
"#,
        )
        .expect("Failed to write test file");
        temp_dir
    }

    // ============================================
    // SimpleMcpHandler Construction Tests
    // ============================================

    #[test]
    fn test_simple_mcp_handler_new() {
        let handler = SimpleMcpHandler::new();
        assert!(handler.is_ok(), "SimpleMcpHandler::new() should succeed");
    }

    // ============================================
    // Tool Definitions Tests
    // ============================================

    #[test]
    fn test_get_tool_definitions_returns_all_tools() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let definitions = handler.get_tool_definitions();

        // Verify it returns a JSON object with tools array
        assert!(definitions.is_object(), "Should return JSON object");
        let tools = definitions.get("tools").expect("Should have tools array");
        assert!(tools.is_array(), "tools should be an array");

        let tools_array = tools.as_array().expect("Should be array");
        assert_eq!(tools_array.len(), 8, "Should have 8 tools defined");

        // Verify all expected tool names are present
        let tool_names: Vec<&str> = tools_array
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();

        assert!(tool_names.contains(&"analyze_complexity"));
        assert!(tool_names.contains(&"analyze_satd"));
        assert!(tool_names.contains(&"analyze_dead_code"));
        assert!(tool_names.contains(&"analyze_tdg"));
        assert!(tool_names.contains(&"analyze_lint_hotspot"));
        assert!(tool_names.contains(&"analyze_entropy"));
        assert!(tool_names.contains(&"quality_gate"));
        assert!(tool_names.contains(&"refactor_auto"));
    }

    #[test]
    fn test_tool_definitions_have_required_fields() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let definitions = handler.get_tool_definitions();
        let tools = definitions
            .get("tools")
            .and_then(|t| t.as_array())
            .expect("Should have tools array");

        for tool in tools {
            assert!(tool.get("name").is_some(), "Each tool should have a name");
            assert!(
                tool.get("description").is_some(),
                "Each tool should have a description"
            );
            assert!(
                tool.get("parameters").is_some(),
                "Each tool should have parameters"
            );

            // Verify parameters have proper JSON Schema structure
            let params = tool.get("parameters").expect("Should have parameters");
            assert_eq!(
                params.get("type").and_then(|t| t.as_str()),
                Some("object"),
                "Parameters should be object type"
            );
            assert!(
                params.get("properties").is_some(),
                "Parameters should have properties"
            );
            assert!(
                params.get("required").is_some(),
                "Parameters should have required array"
            );
        }
    }

    // ============================================
    // Schema Tests
    // ============================================

    #[test]
    fn test_complexity_schema_structure() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let schema = handler.get_complexity_schema();

        let properties = schema
            .get("properties")
            .expect("Should have properties")
            .as_object()
            .expect("Properties should be object");

        // Verify all expected properties exist
        assert!(properties.contains_key("path"));
        assert!(properties.contains_key("format"));
        assert!(properties.contains_key("output"));
        assert!(properties.contains_key("top_files"));
        assert!(properties.contains_key("include_tests"));
        assert!(properties.contains_key("timeout"));
        assert!(properties.contains_key("max_cyclomatic"));
        assert!(properties.contains_key("max_cognitive"));
        assert!(properties.contains_key("max_halstead"));

        // Verify format enum values
        let format_enum = properties
            .get("format")
            .and_then(|f| f.get("enum"))
            .and_then(|e| e.as_array())
            .expect("Format should have enum array");
        assert!(format_enum.len() == 6, "Format should have 6 options");
    }

    #[test]
    fn test_satd_schema_structure() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let schema = handler.get_satd_schema();

        let properties = schema
            .get("properties")
            .expect("Should have properties")
            .as_object()
            .expect("Properties should be object");

        assert!(properties.contains_key("path"));
        assert!(properties.contains_key("severity"));
        assert!(properties.contains_key("critical_only"));
        assert!(properties.contains_key("strict"));
        assert!(properties.contains_key("fail_on_violation"));

        // Verify severity enum
        let severity_enum = properties
            .get("severity")
            .and_then(|s| s.get("enum"))
            .and_then(|e| e.as_array())
            .expect("Severity should have enum array");
        assert_eq!(severity_enum.len(), 4, "Severity should have 4 levels");
    }

    #[test]
    fn test_dead_code_schema_structure() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let schema = handler.get_dead_code_schema();

        let properties = schema
            .get("properties")
            .expect("Should have properties")
            .as_object()
            .expect("Properties should be object");

        assert!(properties.contains_key("path"));
        assert!(properties.contains_key("include_unreachable"));
        assert!(properties.contains_key("min_dead_lines"));
        assert!(properties.contains_key("max_percentage"));
        assert!(properties.contains_key("fail_on_violation"));
    }

    #[test]
    fn test_tdg_schema_structure() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let schema = handler.get_tdg_schema();

        let properties = schema
            .get("properties")
            .expect("Should have properties")
            .as_object()
            .expect("Properties should be object");

        assert!(properties.contains_key("path"));
        assert!(properties.contains_key("threshold"));
        assert!(properties.contains_key("include_components"));
        assert!(properties.contains_key("critical_only"));
    }

    #[test]
    fn test_lint_hotspot_schema_structure() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let schema = handler.get_lint_hotspot_schema();

        let properties = schema
            .get("properties")
            .expect("Should have properties")
            .as_object()
            .expect("Properties should be object");

        assert!(properties.contains_key("path"));
        assert!(properties.contains_key("file"));
        assert!(properties.contains_key("max_density"));
        assert!(properties.contains_key("min_confidence"));
        assert!(properties.contains_key("enforce"));
        assert!(properties.contains_key("dry_run"));
    }

    #[test]
    fn test_quality_gate_schema_structure() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let schema = handler.get_quality_gate_schema();

        let properties = schema
            .get("properties")
            .expect("Should have properties")
            .as_object()
            .expect("Properties should be object");

        assert!(properties.contains_key("path"));
        assert!(properties.contains_key("profile"));
        assert!(properties.contains_key("file"));
        assert!(properties.contains_key("fail_on_violation"));
        assert!(properties.contains_key("verbose"));

        // Verify profile enum
        let profile_enum = properties
            .get("profile")
            .and_then(|p| p.get("enum"))
            .and_then(|e| e.as_array())
            .expect("Profile should have enum array");
        assert_eq!(profile_enum.len(), 4, "Profile should have 4 options");
    }

    #[test]
    fn test_entropy_schema_structure() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let schema = handler.get_entropy_schema();

        let properties = schema
            .get("properties")
            .expect("Should have properties")
            .as_object()
            .expect("Properties should be object");

        assert!(properties.contains_key("path"));
        assert!(properties.contains_key("min_severity"));
        assert!(properties.contains_key("top_violations"));
        assert!(properties.contains_key("file"));

        // Verify min_severity enum
        let severity_enum = properties
            .get("min_severity")
            .and_then(|s| s.get("enum"))
            .and_then(|e| e.as_array())
            .expect("min_severity should have enum array");
        assert_eq!(severity_enum.len(), 3, "min_severity should have 3 levels");
    }

    #[test]
    fn test_refactor_schema_structure() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");
        let schema = handler.get_refactor_schema();

        let properties = schema
            .get("properties")
            .expect("Should have properties")
            .as_object()
            .expect("Properties should be object");

        assert!(properties.contains_key("file"));
        assert!(properties.contains_key("format"));
        assert!(properties.contains_key("output"));
        assert!(properties.contains_key("target_complexity"));
        assert!(properties.contains_key("dry_run"));
        assert!(properties.contains_key("timeout"));

        // Verify required field is file, not path
        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .expect("Should have required array");
        assert!(
            required.iter().any(|v| v.as_str() == Some("file")),
            "file should be required"
        );
    }

    // ============================================
    // Tool Call Handler Tests - Success Cases
    // ============================================

    #[tokio::test]
    async fn test_handle_tool_call_analyze_complexity() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "format": "json",
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_complexity", params).await;
        assert!(result.is_ok(), "analyze_complexity should succeed");

        let value = result.unwrap();
        assert!(value.get("results").is_some(), "Should have results");
        assert!(value.get("summary").is_some(), "Should have summary");
        assert!(value.get("metadata").is_some(), "Should have metadata");
    }

    #[tokio::test]
    async fn test_handle_tool_call_analyze_satd() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "strict": true,
            "critical_only": false,
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_satd", params).await;
        assert!(result.is_ok(), "analyze_satd should succeed");

        let value = result.unwrap();
        assert!(value.get("results").is_some(), "Should have results");
    }

    #[tokio::test]
    async fn test_handle_tool_call_analyze_dead_code() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "include_unreachable": true,
            "min_dead_lines": 1,
            "max_percentage": 50.0,
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_dead_code", params).await;
        assert!(result.is_ok(), "analyze_dead_code should succeed");

        let value = result.unwrap();
        assert!(value.get("results").is_some(), "Should have results");
    }

    #[tokio::test]
    async fn test_handle_tool_call_analyze_tdg() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "threshold": 1.5,
            "include_components": true,
            "critical_only": false,
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_tdg", params).await;
        assert!(result.is_ok(), "analyze_tdg should succeed");

        let value = result.unwrap();
        assert!(value.get("results").is_some(), "Should have results");
    }

    #[tokio::test]
    async fn test_handle_tool_call_analyze_lint_hotspot() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "max_density": 5.0,
            "min_confidence": 0.8,
            "enforce": false,
            "dry_run": true,
            "timeout": 30
        });

        let result = handler
            .handle_tool_call("analyze_lint_hotspot", params)
            .await;
        assert!(result.is_ok(), "analyze_lint_hotspot should succeed");

        let value = result.unwrap();
        assert!(value.get("results").is_some(), "Should have results");
    }

    #[tokio::test]
    async fn test_handle_tool_call_analyze_entropy() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "min_severity": "medium",
            "top_violations": 10,
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_entropy", params).await;
        assert!(result.is_ok(), "analyze_entropy should succeed");

        let value = result.unwrap();
        assert!(
            value.get("violations").is_some(),
            "Should have violations field"
        );
    }

    #[tokio::test]
    async fn test_handle_tool_call_quality_gate_standard() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "profile": "standard",
            "fail_on_violation": false,
            "verbose": true,
            "timeout": 30
        });

        let result = handler.handle_tool_call("quality_gate", params).await;
        assert!(result.is_ok(), "quality_gate should succeed");

        let value = result.unwrap();
        assert!(value.get("passed").is_some(), "Should have passed field");
        assert!(
            value.get("violations").is_some(),
            "Should have violations field"
        );
    }

    #[tokio::test]
    async fn test_handle_tool_call_refactor_auto() {
        let temp_dir = create_test_project();
        let test_file = temp_dir.path().join("test.rs");
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "file": test_file.to_str().unwrap(),
            "target_complexity": 10,
            "dry_run": true,
            "timeout": 30
        });

        let result = handler.handle_tool_call("refactor_auto", params).await;
        assert!(result.is_ok(), "refactor_auto should succeed");

        let value = result.unwrap();
        assert!(value.get("plan").is_some(), "Should have plan field");
        assert!(value.get("dry_run").is_some(), "Should have dry_run field");
    }

    // ============================================
    // Tool Call Handler Tests - Error Cases
    // ============================================

    #[tokio::test]
    async fn test_handle_tool_call_unknown_tool() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": "/tmp"
        });

        let result = handler.handle_tool_call("unknown_tool", params).await;
        assert!(result.is_err(), "Unknown tool should return error");

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Unknown tool"),
            "Error should mention unknown tool"
        );
    }

    #[tokio::test]
    async fn test_handle_tool_call_invalid_params() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        // Missing required 'file' parameter for refactor_auto (no default)
        let params = json!({
            "target_complexity": 10,
            "dry_run": true
        });

        let result = handler.handle_tool_call("refactor_auto", params).await;
        assert!(
            result.is_err(),
            "Missing required 'file' param should return error"
        );
    }

    #[tokio::test]
    async fn test_handle_tool_call_invalid_path() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": "/nonexistent/path/that/does/not/exist",
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_complexity", params).await;
        assert!(result.is_err(), "Invalid path should return error");
    }

    // ============================================
    // Backward Compatibility Tests
    // ============================================

    #[tokio::test]
    async fn test_backward_compatibility_project_path() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        // Using deprecated 'project_path' instead of 'path'
        let params = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_complexity", params).await;
        assert!(
            result.is_ok(),
            "Should accept deprecated project_path parameter"
        );
    }

    #[tokio::test]
    async fn test_backward_compatibility_format_mapping() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        // Using alternative format names
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "format": "human",  // Should map to "summary"
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_complexity", params).await;
        assert!(result.is_ok(), "Should accept alternative format names");
    }

    // ============================================
    // Quality Gate Profile Tests
    // ============================================

    #[tokio::test]
    async fn test_quality_gate_strict_profile() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "profile": "strict",
            "fail_on_violation": false,
            "timeout": 30
        });

        let result = handler.handle_tool_call("quality_gate", params).await;
        assert!(
            result.is_ok(),
            "quality_gate with strict profile should work"
        );
    }

    #[tokio::test]
    async fn test_quality_gate_extreme_profile() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "profile": "extreme",
            "fail_on_violation": false,
            "timeout": 30
        });

        let result = handler.handle_tool_call("quality_gate", params).await;
        assert!(
            result.is_ok(),
            "quality_gate with extreme profile should work"
        );
    }

    #[tokio::test]
    async fn test_quality_gate_toyota_profile() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "profile": "toyota",
            "fail_on_violation": false,
            "timeout": 30
        });

        let result = handler.handle_tool_call("quality_gate", params).await;
        assert!(
            result.is_ok(),
            "quality_gate with toyota profile should work"
        );
    }

    // ============================================
    // Output Format Tests
    // ============================================

    #[tokio::test]
    async fn test_complexity_all_output_formats() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let formats = ["table", "json", "yaml", "markdown", "csv", "summary"];

        for format in formats {
            let params = json!({
                "path": temp_dir.path().to_str().unwrap(),
                "format": format,
                "timeout": 30
            });

            let result = handler.handle_tool_call("analyze_complexity", params).await;
            assert!(result.is_ok(), "Format '{}' should work", format);
        }
    }

    // ============================================
    // Edge Cases and Boundary Tests
    // ============================================

    #[tokio::test]
    async fn test_complexity_with_all_optional_params() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "format": "json",
            "top_files": 5,
            "include_tests": true,
            "timeout": 60,
            "max_cyclomatic": 20,
            "max_cognitive": 15,
            "max_halstead": 25.0
        });

        let result = handler.handle_tool_call("analyze_complexity", params).await;
        assert!(result.is_ok(), "Should handle all optional params");
    }

    #[tokio::test]
    async fn test_dead_code_with_boundary_percentage() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        // Test with 0% boundary
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "max_percentage": 0.0,
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_dead_code", params).await;
        assert!(result.is_ok(), "Should handle 0% max_percentage");

        // Test with 100% boundary
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "max_percentage": 100.0,
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_dead_code", params).await;
        assert!(result.is_ok(), "Should handle 100% max_percentage");
    }

    #[tokio::test]
    async fn test_lint_hotspot_confidence_boundaries() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        // Test with 0.0 confidence
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "min_confidence": 0.0,
            "timeout": 30
        });

        let result = handler
            .handle_tool_call("analyze_lint_hotspot", params)
            .await;
        assert!(result.is_ok(), "Should handle 0.0 min_confidence");

        // Test with 1.0 confidence
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "min_confidence": 1.0,
            "timeout": 30
        });

        let result = handler
            .handle_tool_call("analyze_lint_hotspot", params)
            .await;
        assert!(result.is_ok(), "Should handle 1.0 min_confidence");
    }

    #[tokio::test]
    async fn test_entropy_severity_levels() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        for severity in ["low", "medium", "high"] {
            let params = json!({
                "path": temp_dir.path().to_str().unwrap(),
                "min_severity": severity,
                "timeout": 30
            });

            let result = handler.handle_tool_call("analyze_entropy", params).await;
            assert!(result.is_ok(), "Severity '{}' should work", severity);
        }
    }

    #[tokio::test]
    async fn test_tdg_threshold_boundaries() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        // Test with 0.0 threshold
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "threshold": 0.0,
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_tdg", params).await;
        assert!(result.is_ok(), "Should handle 0.0 threshold");

        // Test with high threshold
        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "threshold": 10.0,
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_tdg", params).await;
        assert!(result.is_ok(), "Should handle high threshold");
    }

    // ============================================
    // Response Structure Validation Tests
    // ============================================

    #[tokio::test]
    async fn test_complexity_response_structure() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "timeout": 30
        });

        let result = handler
            .handle_tool_call("analyze_complexity", params)
            .await
            .expect("Should succeed");

        // Verify response structure
        assert!(result.get("results").and_then(|r| r.as_array()).is_some());
        assert!(result.get("summary").and_then(|s| s.as_str()).is_some());

        let metadata = result.get("metadata").expect("Should have metadata");
        assert!(metadata.get("path").is_some());
        assert!(metadata.get("format").is_some());
        assert!(metadata.get("include_tests").is_some());
        assert!(metadata.get("timeout").is_some());
        assert!(metadata.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_quality_gate_response_structure() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "profile": "standard",
            "fail_on_violation": false,
            "timeout": 30
        });

        let result = handler
            .handle_tool_call("quality_gate", params)
            .await
            .expect("Should succeed");

        // Verify response structure
        assert!(result.get("passed").and_then(|p| p.as_bool()).is_some());
        assert!(result
            .get("violations")
            .and_then(|v| v.as_array())
            .is_some());
        assert!(result.get("profile").is_some());
        assert!(result.get("summary").and_then(|s| s.as_str()).is_some());
        assert!(result.get("metadata").is_some());
    }

    #[tokio::test]
    async fn test_refactor_response_structure() {
        let temp_dir = create_test_project();
        let test_file = temp_dir.path().join("test.rs");
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "file": test_file.to_str().unwrap(),
            "target_complexity": 10,
            "dry_run": true,
            "timeout": 30
        });

        let result = handler
            .handle_tool_call("refactor_auto", params)
            .await
            .expect("Should succeed");

        // Verify response structure
        let plan = result.get("plan").expect("Should have plan");
        assert!(plan.get("file").is_some());
        assert!(plan.get("current_complexity").is_some());
        assert!(plan.get("target_complexity").is_some());
        assert!(plan.get("operations").is_some());
        assert!(plan.get("estimated_reduction").is_some());
        assert!(plan.get("applied").is_some());

        assert!(result.get("dry_run").and_then(|d| d.as_bool()).is_some());
        assert!(result.get("summary").and_then(|s| s.as_str()).is_some());
    }

    // ============================================
    // Entropy Analysis with File Path Tests
    // ============================================

    #[tokio::test]
    async fn test_entropy_with_specific_file() {
        let temp_dir = create_test_project();
        let test_file = temp_dir.path().join("test.rs");
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "file": test_file.to_str().unwrap(),
            "timeout": 30
        });

        let result = handler.handle_tool_call("analyze_entropy", params).await;
        assert!(result.is_ok(), "Should analyze specific file");
    }

    // ============================================
    // SATD Severity Filter Tests
    // ============================================

    #[tokio::test]
    async fn test_satd_severity_levels() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        for severity in ["low", "medium", "high", "critical"] {
            let params = json!({
                "path": temp_dir.path().to_str().unwrap(),
                "severity": severity,
                "timeout": 30
            });

            let result = handler.handle_tool_call("analyze_satd", params).await;
            assert!(result.is_ok(), "SATD severity '{}' should work", severity);
        }
    }

    // ============================================
    // Quality Gate File-Specific Analysis Tests
    // ============================================

    #[tokio::test]
    async fn test_quality_gate_with_specific_file() {
        let temp_dir = create_test_project();
        let test_file = temp_dir.path().join("test.rs");
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "file": test_file.to_str().unwrap(),
            "profile": "standard",
            "fail_on_violation": false,
            "timeout": 30
        });

        let result = handler.handle_tool_call("quality_gate", params).await;
        assert!(
            result.is_ok(),
            "Quality gate with specific file should work"
        );
    }

    // ============================================
    // Lint Hotspot File-Specific Analysis Tests
    // ============================================

    #[tokio::test]
    async fn test_lint_hotspot_with_specific_file() {
        let temp_dir = create_test_project();
        let test_file = temp_dir.path().join("test.rs");
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "file": test_file.to_str().unwrap(),
            "timeout": 30
        });

        let result = handler
            .handle_tool_call("analyze_lint_hotspot", params)
            .await;
        assert!(
            result.is_ok(),
            "Lint hotspot with specific file should work"
        );
    }

    // ============================================
    // Concurrent Access Tests
    // ============================================

    #[tokio::test]
    async fn test_concurrent_tool_calls() {
        let temp_dir = create_test_project();
        let handler = Arc::new(SimpleMcpHandler::new().expect("Failed to create handler"));

        let path = temp_dir.path().to_str().unwrap().to_string();

        // Spawn multiple concurrent tool calls
        let mut handles = vec![];

        for tool in ["analyze_complexity", "analyze_satd", "analyze_dead_code"] {
            let handler_clone = Arc::clone(&handler);
            let path_clone = path.clone();
            let tool_name = tool.to_string();

            let handle = tokio::spawn(async move {
                let params = json!({
                    "path": path_clone,
                    "timeout": 30
                });
                handler_clone.handle_tool_call(&tool_name, params).await
            });

            handles.push(handle);
        }

        // Wait for all calls to complete
        for handle in handles {
            let result = handle.await.expect("Task should complete");
            assert!(result.is_ok(), "Concurrent call should succeed");
        }
    }

    // ============================================
    // Empty/Minimal Project Tests
    // ============================================

    #[tokio::test]
    async fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        let params = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "timeout": 30
        });

        // Should handle empty directories gracefully
        let result = handler.handle_tool_call("analyze_complexity", params).await;
        // This may succeed with empty results or fail gracefully
        // We just verify it doesn't panic
        let _ = result;
    }

    // ============================================
    // Default Parameter Tests
    // ============================================

    #[tokio::test]
    async fn test_minimal_params_use_defaults() {
        let temp_dir = create_test_project();
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        // Only provide required 'path' parameter
        let params = json!({
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = handler.handle_tool_call("analyze_complexity", params).await;
        assert!(result.is_ok(), "Should work with only required params");
    }

    // ============================================
    // Schema JSON Structure Tests
    // ============================================

    #[test]
    fn test_all_schemas_are_valid_json_schema() {
        let handler = SimpleMcpHandler::new().expect("Failed to create handler");

        // All schemas should have proper JSON Schema structure
        let schemas = [
            handler.get_complexity_schema(),
            handler.get_satd_schema(),
            handler.get_dead_code_schema(),
            handler.get_tdg_schema(),
            handler.get_lint_hotspot_schema(),
            handler.get_quality_gate_schema(),
            handler.get_entropy_schema(),
            handler.get_refactor_schema(),
        ];

        for schema in schemas {
            assert_eq!(schema.get("type").and_then(|t| t.as_str()), Some("object"));
            assert!(schema.get("properties").is_some());
            assert!(schema.get("required").is_some());
        }
    }
}
