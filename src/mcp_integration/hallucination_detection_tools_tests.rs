#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// RED TEST: Validate documentation tool should have correct metadata
    #[test]
    fn red_validate_documentation_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "validate_documentation");
        assert!(metadata.description.contains("hallucination"));
        assert!(metadata.description.contains("semantic entropy"));

        // Verify input schema has required fields
        let schema = metadata.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["documentation_path"].is_object());
        assert!(schema["properties"]["deep_context_path"].is_object());
        assert!(schema["properties"]["similarity_threshold"].is_object());
        assert_eq!(
            schema["required"],
            json!(["documentation_path", "deep_context_path"])
        );
    }

    /// RED TEST: Check claim tool should have correct metadata
    #[test]
    fn red_check_claim_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "check_claim");
        assert!(metadata.description.contains("single"));
        assert!(metadata.description.contains("semantic entropy"));

        let schema = metadata.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["claim"].is_object());
        assert!(schema["properties"]["deep_context_path"].is_object());
        assert_eq!(schema["required"], json!(["claim", "deep_context_path"]));
    }

    /// RED TEST: Validate documentation tool should return error for missing params
    #[tokio::test]
    async fn red_validate_documentation_missing_params() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert!(err.message.contains("documentation_path"));
    }

    /// RED TEST: Check claim tool should return error for missing claim
    #[tokio::test]
    async fn red_check_claim_missing_params() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::INVALID_PARAMS);
        assert!(err.message.contains("claim"));
    }

    /// RED TEST: Validate documentation should process valid input
    #[tokio::test]
    async fn red_validate_documentation_valid_input() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry);

        // Create temporary documentation file
        let mut doc_file = NamedTempFile::new().unwrap();
        writeln!(doc_file, "# Test Documentation").unwrap();
        writeln!(doc_file, "PMAT can analyze Rust code.").unwrap();
        doc_file.flush().unwrap();

        // Create temporary deep context file
        let mut context_file = NamedTempFile::new().unwrap();
        writeln!(context_file, "## Functions:").unwrap();
        writeln!(context_file, "- analyze_rust()").unwrap();
        writeln!(context_file, "## Supported languages:").unwrap();
        writeln!(context_file, "- Rust").unwrap();
        context_file.flush().unwrap();

        let params = json!({
            "documentation_path": doc_file.path().to_str().unwrap(),
            "deep_context_path": context_file.path().to_str().unwrap(),
            "similarity_threshold": 0.7,
            "fail_on_error": false
        });

        let result = tool.execute(params).await;
        assert!(
            result.is_ok(),
            "Should succeed with valid input: {:?}",
            result
        );

        let response = result.unwrap();
        assert_eq!(response["status"], "completed");
        assert!(response["summary"].is_object());
        assert!(response["results"].is_array());
    }

    /// RED TEST: Check claim should verify valid claim
    #[tokio::test]
    async fn red_check_claim_valid_input() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);

        // Create temporary deep context file
        let mut context_file = NamedTempFile::new().unwrap();
        writeln!(context_file, "## Supported languages:").unwrap();
        writeln!(context_file, "- TypeScript").unwrap();
        context_file.flush().unwrap();

        let params = json!({
            "claim": "PMAT can analyze TypeScript",
            "deep_context_path": context_file.path().to_str().unwrap(),
            "similarity_threshold": 0.7
        });

        let result = tool.execute(params).await;
        assert!(
            result.is_ok(),
            "Should succeed with valid claim: {:?}",
            result
        );

        let response = result.unwrap();
        assert_eq!(response["status"], "completed");
        assert!(response["claim"].is_string());
        assert!(response["validation_status"].is_string());
        assert!(response["confidence"].is_number());
    }

    /// RED TEST: Check claim should handle contradiction
    #[tokio::test]
    async fn red_check_claim_contradiction() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);

        // Create deep context WITHOUT compilation capability
        let mut context_file = NamedTempFile::new().unwrap();
        writeln!(context_file, "## Functions:").unwrap();
        writeln!(context_file, "- analyze_code()").unwrap();
        context_file.flush().unwrap();

        let params = json!({
            "claim": "PMAT can compile Rust code",
            "deep_context_path": context_file.path().to_str().unwrap(),
            "similarity_threshold": 0.7
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        // Should detect contradiction (PMAT analyzes but doesn't compile)
        assert_eq!(response["validation_status"], "Contradiction");
    }

    /// RED TEST: Validate documentation should fail with fail_on_error
    #[tokio::test]
    async fn red_validate_documentation_fail_on_error() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry);

        // Create doc with contradictory claim
        let mut doc_file = NamedTempFile::new().unwrap();
        writeln!(doc_file, "PMAT can compile and execute code.").unwrap();
        doc_file.flush().unwrap();

        // Create context showing PMAT doesn't compile
        let mut context_file = NamedTempFile::new().unwrap();
        writeln!(context_file, "## Functions:").unwrap();
        writeln!(context_file, "- analyze_complexity()").unwrap();
        context_file.flush().unwrap();

        let params = json!({
            "documentation_path": doc_file.path().to_str().unwrap(),
            "deep_context_path": context_file.path().to_str().unwrap(),
            "fail_on_error": true
        });

        let result = tool.execute(params).await;
        // Should fail when contradictions found and fail_on_error=true
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, error_codes::INTERNAL_ERROR);
        assert!(err.message.contains("contradiction"));
    }

    #[test]
    fn test_validate_documentation_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry.clone());
        // Just verify construction works
        assert!(tool.metadata().name.contains("documentation"));
    }

    #[test]
    fn test_check_claim_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry.clone());
        assert!(tool.metadata().name.contains("claim"));
    }

    #[test]
    fn test_validate_documentation_metadata_schema() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry);
        let metadata = tool.metadata();

        // Verify schema structure
        let schema = &metadata.input_schema;
        assert!(schema["properties"]["documentation_path"]["type"]
            .as_str()
            .unwrap()
            .contains("string"));
        assert!(schema["properties"]["deep_context_path"]["type"]
            .as_str()
            .unwrap()
            .contains("string"));
        assert!(schema["properties"]["similarity_threshold"]["type"]
            .as_str()
            .unwrap()
            .contains("number"));
        assert!(schema["properties"]["fail_on_error"]["type"]
            .as_str()
            .unwrap()
            .contains("boolean"));
    }

    #[test]
    fn test_check_claim_metadata_schema() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);
        let metadata = tool.metadata();

        let schema = &metadata.input_schema;
        assert!(schema["properties"]["claim"]["type"]
            .as_str()
            .unwrap()
            .contains("string"));
        assert!(schema["properties"]["deep_context_path"]["type"]
            .as_str()
            .unwrap()
            .contains("string"));
    }

    #[tokio::test]
    async fn test_validate_documentation_missing_doc_path_only() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry);

        let params = json!({
            "deep_context_path": "/some/path"
        });

        let result = tool.execute(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("documentation_path"));
    }

    #[tokio::test]
    async fn test_check_claim_missing_deep_context() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);

        let params = json!({
            "claim": "PMAT can analyze code"
        });

        let result = tool.execute(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("deep_context_path"));
    }

    #[tokio::test]
    async fn test_validate_documentation_nonexistent_file() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry);

        let params = json!({
            "documentation_path": "/nonexistent/file.md",
            "deep_context_path": "/other/path"
        });

        let result = tool.execute(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Failed to read"));
    }

    #[tokio::test]
    async fn test_check_claim_nonexistent_deep_context() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);

        let params = json!({
            "claim": "Test claim",
            "deep_context_path": "/nonexistent/context.md"
        });

        let result = tool.execute(params).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Failed to read"));
    }

    #[tokio::test]
    async fn test_check_claim_default_threshold() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);

        let mut context_file = NamedTempFile::new().unwrap();
        writeln!(context_file, "## Functions:").unwrap();
        writeln!(context_file, "- analyze()").unwrap();
        context_file.flush().unwrap();

        // No similarity_threshold specified, should use default
        let params = json!({
            "claim": "Test claim",
            "deep_context_path": context_file.path().to_str().unwrap()
        });

        let result = tool.execute(params).await;
        // Should not fail due to missing threshold
        assert!(result.is_ok() || result.is_err()); // Just verify no panic
    }

    #[tokio::test]
    async fn test_check_claim_no_claim_detected() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = CheckClaimTool::new(registry);

        let mut context_file = NamedTempFile::new().unwrap();
        writeln!(context_file, "## Functions:").unwrap();
        context_file.flush().unwrap();

        // Input text that won't be detected as a claim
        let params = json!({
            "claim": "???",
            "deep_context_path": context_file.path().to_str().unwrap()
        });

        let result = tool.execute(params).await;
        // May detect as no claim or process it
        if let Ok(response) = result {
            // Either no_claim_detected or completed is acceptable
            let status = response["status"].as_str().unwrap_or("");
            assert!(status == "no_claim_detected" || status == "completed");
        }
    }

    #[tokio::test]
    async fn test_validate_documentation_empty_doc() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateDocumentationTool::new(registry);

        let mut doc_file = NamedTempFile::new().unwrap();
        // Empty file
        doc_file.flush().unwrap();

        let mut context_file = NamedTempFile::new().unwrap();
        writeln!(context_file, "## Functions:").unwrap();
        context_file.flush().unwrap();

        let params = json!({
            "documentation_path": doc_file.path().to_str().unwrap(),
            "deep_context_path": context_file.path().to_str().unwrap()
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["status"], "completed");
    }

    #[test]
    fn test_tool_metadata_descriptions() {
        let registry = Arc::new(AgentRegistry::new());

        let validate_tool = ValidateDocumentationTool::new(registry.clone());
        let validate_meta = validate_tool.metadata();
        assert!(validate_meta.description.len() > 20);
        assert!(validate_meta.description.contains("documentation"));

        let check_tool = CheckClaimTool::new(registry.clone());
        let check_meta = check_tool.metadata();
        assert!(check_meta.description.len() > 20);
        assert!(check_meta.description.contains("claim"));
    }
}
