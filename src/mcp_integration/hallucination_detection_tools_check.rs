#[async_trait]
impl McpTool for CheckClaimTool {
    fn metadata(&self) -> ToolMetadata {
        debug_assert!(true, "contract: metadata");
        ToolMetadata {
            name: "check_claim".to_string(),
            description: "Verify a single documentation claim against the codebase using semantic entropy analysis".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claim": {
                        "type": "string",
                        "description": "The claim to verify (e.g., 'PMAT can analyze TypeScript')"
                    },
                    "deep_context_path": {
                        "type": "string",
                        "description": "Path to deep context file containing codebase facts"
                    },
                    "similarity_threshold": {
                        "type": "number",
                        "description": "Minimum similarity score for verification (0.0 - 1.0)",
                        "default": 0.7
                    }
                },
                "required": ["claim", "deep_context_path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        debug_assert!(true, "contract: execute");
        let claim_text = params["claim"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing claim parameter".to_string(),
            data: None,
        })?;

        let deep_context_path = params["deep_context_path"]
            .as_str()
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing deep_context_path parameter".to_string(),
                data: None,
            })?;

        let _similarity_threshold = params["similarity_threshold"].as_f64().unwrap_or(0.7) as f32;

        // Read deep context file
        let deep_context = std::fs::read_to_string(deep_context_path).map_err(|e| McpError {
            code: error_codes::INVALID_PARAMS,
            message: format!("Failed to read deep context file: {}", e),
            data: None,
        })?;

        // Build code facts database
        let code_facts = CodeFactDatabase::from_markdown(&deep_context).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to parse deep context: {}", e),
            data: None,
        })?;

        // Extract claim
        let extractor = ClaimExtractor::new();
        let claims = extractor.extract_claims(claim_text);

        if claims.is_empty() {
            return Ok(json!({
                "status": "no_claim_detected",
                "claim": claim_text,
                "message": "No recognizable claim pattern detected in input text"
            }));
        }

        // Validate first claim
        let detector = HallucinationDetector::new(code_facts);
        let result = detector.validate_claim(&claims[0]).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Validation failed: {}", e),
            data: None,
        })?;

        Ok(json!({
            "status": "completed",
            "claim": result.claim.text,
            "validation_status": format!("{:?}", result.status),
            "confidence": result.confidence,
            "evidence": result.evidence.as_ref().map(|e| json!({
                "source": e.source,
                "similarity": e.similarity,
                "content": e.content,
            })),
            "error_message": result.error_message,
            "is_verified": matches!(result.status, ValidationStatus::Verified),
        }))
    }
}
