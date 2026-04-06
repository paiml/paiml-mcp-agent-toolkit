#[async_trait]
impl McpTool for ValidateDocumentationTool {
    fn metadata(&self) -> ToolMetadata {
        debug_assert!(true, "contract: metadata");
        ToolMetadata {
            name: "validate_documentation".to_string(),
            description: "Validate documentation claims against codebase to detect hallucinations, broken references, and 404 errors using semantic entropy analysis".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "documentation_path": {
                        "type": "string",
                        "description": "Path to documentation file to validate (README.md, CLAUDE.md, etc.)"
                    },
                    "deep_context_path": {
                        "type": "string",
                        "description": "Path to deep context file containing codebase facts"
                    },
                    "similarity_threshold": {
                        "type": "number",
                        "description": "Minimum similarity score for verification (0.0 - 1.0)",
                        "default": 0.7
                    },
                    "fail_on_error": {
                        "type": "boolean",
                        "description": "Return error status if any claims fail validation",
                        "default": false
                    }
                },
                "required": ["documentation_path", "deep_context_path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        debug_assert!(true, "contract: execute");
        // Extract parameters
        let doc_path = params["documentation_path"]
            .as_str()
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing documentation_path parameter".to_string(),
                data: None,
            })?;

        let deep_context_path = params["deep_context_path"]
            .as_str()
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing deep_context_path parameter".to_string(),
                data: None,
            })?;

        let similarity_threshold = params["similarity_threshold"].as_f64().unwrap_or(0.7) as f32;

        let fail_on_error = params["fail_on_error"].as_bool().unwrap_or(false);

        // Read documentation file
        let doc_content = std::fs::read_to_string(doc_path).map_err(|e| McpError {
            code: error_codes::INVALID_PARAMS,
            message: format!("Failed to read documentation file: {}", e),
            data: None,
        })?;

        // Read deep context file
        let deep_context = std::fs::read_to_string(deep_context_path).map_err(|e| McpError {
            code: error_codes::INVALID_PARAMS,
            message: format!("Failed to read deep context file: {}", e),
            data: None,
        })?;

        // Build code facts database from deep context
        let code_facts = CodeFactDatabase::from_markdown(&deep_context).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Failed to parse deep context: {}", e),
            data: None,
        })?;

        // Extract claims from documentation
        let extractor = ClaimExtractor::new();
        let mut claims = extractor.extract_claims(&doc_content);

        // Set source file for all claims
        let doc_path_buf = PathBuf::from(doc_path);
        for claim in &mut claims {
            claim.source_file = doc_path_buf.clone();
        }

        // Validate all claims
        let detector = HallucinationDetector::new(code_facts);
        let mut results = Vec::new();

        for claim in &claims {
            let result = detector.validate_claim(claim).map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Validation failed: {}", e),
                data: None,
            })?;

            results.push(json!({
                "claim": result.claim.text,
                "line": result.claim.line_number,
                "status": format!("{:?}", result.status),
                "confidence": result.confidence,
                "evidence": result.evidence.as_ref().map(|e| e.source.clone()),
                "error": result.error_message,
            }));
        }

        // Calculate summary statistics
        let total = results.len();
        let verified = results
            .iter()
            .filter(|r| r["status"].as_str() == Some("Verified"))
            .count();
        let unverified = results
            .iter()
            .filter(|r| r["status"].as_str() == Some("Unverified"))
            .count();
        let contradictions = results
            .iter()
            .filter(|r| r["status"].as_str() == Some("Contradiction"))
            .count();
        let not_found = results
            .iter()
            .filter(|r| r["status"].as_str() == Some("NotFound"))
            .count();

        let summary = json!({
            "total_claims": total,
            "verified": verified,
            "unverified": unverified,
            "contradictions": contradictions,
            "not_found": not_found,
            "pass_rate": if total > 0 { verified as f64 / total as f64 } else { 0.0 },
        });

        // Return error if fail_on_error is true and there are failures
        if fail_on_error && (contradictions > 0 || not_found > 0) {
            return Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!(
                    "Documentation validation failed: {} contradictions, {} not found",
                    contradictions, not_found
                ),
                data: Some(json!({
                    "summary": summary,
                    "results": results,
                })),
            });
        }

        Ok(json!({
            "status": "completed",
            "summary": summary,
            "results": results,
            "documentation_path": doc_path,
            "deep_context_path": deep_context_path,
            "similarity_threshold": similarity_threshold,
        }))
    }
}
