//! MCP tools for hallucination detection and documentation validation
//!
//! Exposes Sprint 37's hallucination detection system via MCP to enable
//! AI agents to validate documentation claims against the actual codebase.
//!
//! Based on peer-reviewed research:
//! - Semantic Entropy (Farquhar et al., Nature 2024)
//! - MIND framework (IJCAI 2025)
//! - Unified Detection Framework (Complex & Intelligent Systems 2025)

use super::*;
use crate::agents::registry::AgentRegistry;
use crate::services::hallucination_detector::{
    ClaimExtractor, CodeFactDatabase, HallucinationDetector, ValidationStatus,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Validate documentation tool - checks documentation claims against codebase
pub struct ValidateDocumentationTool {
    _registry: Arc<AgentRegistry>,
}

impl ValidateDocumentationTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for ValidateDocumentationTool {
    fn metadata(&self) -> ToolMetadata {
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

        let similarity_threshold = params["similarity_threshold"]
            .as_f64()
            .unwrap_or(0.7) as f32;

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

/// Check single claim tool - validates a single claim against codebase
pub struct CheckClaimTool {
    _registry: Arc<AgentRegistry>,
}

impl CheckClaimTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for CheckClaimTool {
    fn metadata(&self) -> ToolMetadata {
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

        let _similarity_threshold = params["similarity_threshold"]
            .as_f64()
            .unwrap_or(0.7) as f32;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

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
        assert_eq!(schema["required"], json!(["documentation_path", "deep_context_path"]));
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
        assert!(result.is_ok(), "Should succeed with valid input: {:?}", result);

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
        assert!(result.is_ok(), "Should succeed with valid claim: {:?}", result);

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
}
