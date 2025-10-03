use super::*;
use crate::agents::analyzer_actor::AnalyzerActor;
use crate::agents::messages::{AnalyzeMessage, TransformMessage, ValidateMessage};
use crate::agents::registry::AgentRegistry;
use crate::agents::transformer_actor::TransformerActor;
use crate::agents::validator_actor::ValidatorActor;
use crate::agents::Priority;
use actix::prelude::*;
use serde_json::json;
use std::sync::Arc;

// Analyze tool - invokes analyzer agent
pub struct AnalyzeTool {
    _registry: Arc<AgentRegistry>,
    analyzer: Option<Addr<AnalyzerActor>>,
}

impl AnalyzeTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
            analyzer: None,
        }
    }

    pub fn new_with_actor(registry: Arc<AgentRegistry>, analyzer: Addr<AnalyzerActor>) -> Self {
        Self {
            _registry: registry,
            analyzer: Some(analyzer),
        }
    }
}

#[async_trait]
impl McpTool for AnalyzeTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "analyze".to_string(),
            description: "Analyze code for quality metrics and issues".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Source code to analyze"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "metrics": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Metrics to calculate"
                    }
                },
                "required": ["code", "language"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let code = params["code"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing code parameter".to_string(),
            data: None,
        })?;

        let _language = params["language"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing language parameter".to_string(),
            data: None,
        })?;

        // Get analyzer actor
        let analyzer = self.analyzer.as_ref().ok_or_else(|| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: "Analyzer actor not initialized".to_string(),
            data: None,
        })?;

        // Create message with priority
        let priority = params["priority"]
            .as_str()
            .and_then(|p| match p {
                "critical" => Some(Priority::Critical),
                "high" => Some(Priority::High),
                "low" => Some(Priority::Low),
                _ => Some(Priority::Normal),
            })
            .unwrap_or(Priority::Normal);

        let message = AnalyzeMessage {
            code: code.to_string(),
            priority,
        };

        // Send message to analyzer actor
        let response = analyzer
            .send(message)
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Actor communication failed: {}", e),
                data: None,
            })?
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Analysis failed: {}", e),
                data: None,
            })?;

        // Convert AgentResponse to MCP format
        match response {
            crate::agents::AgentResponse::Analyzed(metrics) => Ok(json!({
                "type": "text",
                "text": format!("Analysis Results:\n\nComplexity: {}\nLines: {}\nFunctions: {}\nClasses: {}\nImports: {}\n",
                    metrics.complexity,
                    metrics.lines_of_code,
                    metrics.functions,
                    metrics.classes,
                    metrics.imports
                )
            })),
            _ => Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Unexpected response type".to_string(),
                data: None,
            }),
        }
    }
}

// Transform tool - invokes transformer agent
pub struct TransformTool {
    _registry: Arc<AgentRegistry>,
    transformer: Option<Addr<TransformerActor>>,
}

impl TransformTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
            transformer: None,
        }
    }

    pub fn new_with_actor(registry: Arc<AgentRegistry>, transformer: Addr<TransformerActor>) -> Self {
        Self {
            _registry: registry,
            transformer: Some(transformer),
        }
    }
}

#[async_trait]
impl McpTool for TransformTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "transform".to_string(),
            description: "Transform code using AST manipulation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Source code to transform"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "transformation": {
                        "type": "string",
                        "description": "Type of transformation",
                        "enum": ["optimize", "minify", "beautify", "refactor"]
                    },
                    "options": {
                        "type": "object",
                        "description": "Transformation options"
                    }
                },
                "required": ["code", "language", "transformation"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let code = params["code"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing code parameter".to_string(),
            data: None,
        })?;

        let _transformation = params["transformation"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing transformation parameter".to_string(),
            data: None,
        })?;

        // Get transformer actor
        let transformer = self.transformer.as_ref().ok_or_else(|| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: "Transformer actor not initialized".to_string(),
            data: None,
        })?;

        // Create message with priority
        let priority = params["priority"]
            .as_str()
            .and_then(|p| match p {
                "critical" => Some(Priority::Critical),
                "high" => Some(Priority::High),
                "low" => Some(Priority::Low),
                _ => Some(Priority::Normal),
            })
            .unwrap_or(Priority::Normal);

        // Extract rules (optional)
        let rules = params["rules"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let message = TransformMessage {
            code: code.to_string(),
            rules,
            priority,
        };

        // Send message to transformer actor
        let response = transformer
            .send(message)
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Actor communication failed: {}", e),
                data: None,
            })?
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Transformation failed: {}", e),
                data: None,
            })?;

        // Convert AgentResponse to MCP format
        match response {
            crate::agents::AgentResponse::Transformed(result) => Ok(json!({
                "type": "text",
                "text": format!(
                    "Transformation Results:\n\nTransformed Code:\n{}\n\nChanges: {}\n",
                    result.transformed,
                    result.changes.len()
                )
            })),
            _ => Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Unexpected response type".to_string(),
                data: None,
            }),
        }
    }
}

// Validate tool - invokes validator agent
pub struct ValidateTool {
    _registry: Arc<AgentRegistry>,
    analyzer: Option<Addr<AnalyzerActor>>,
    validator: Option<Addr<ValidatorActor>>,
}

impl ValidateTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
            analyzer: None,
            validator: None,
        }
    }

    pub fn new_with_actors(
        registry: Arc<AgentRegistry>,
        analyzer: Addr<AnalyzerActor>,
        validator: Addr<ValidatorActor>,
    ) -> Self {
        Self {
            _registry: registry,
            analyzer: Some(analyzer),
            validator: Some(validator),
        }
    }
}

#[async_trait]
impl McpTool for ValidateTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "validate".to_string(),
            description: "Validate code against quality standards".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Source code to validate"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "rules": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Validation rules to apply"
                    },
                    "thresholds": {
                        "type": "object",
                        "description": "Quality thresholds"
                    }
                },
                "required": ["code", "language"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let code = params["code"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing code parameter".to_string(),
            data: None,
        })?;

        // Get analyzer and validator actors
        let analyzer = self.analyzer.as_ref().ok_or_else(|| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: "Analyzer actor not initialized".to_string(),
            data: None,
        })?;

        let validator = self.validator.as_ref().ok_or_else(|| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: "Validator actor not initialized".to_string(),
            data: None,
        })?;

        // Step 1: Analyze code to get metrics
        let analyze_msg = AnalyzeMessage {
            code: code.to_string(),
            priority: Priority::Normal,
        };

        let analyze_response = analyzer
            .send(analyze_msg)
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Actor communication failed: {}", e),
                data: None,
            })?
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Analysis failed: {}", e),
                data: None,
            })?;

        let metrics = match analyze_response {
            crate::agents::AgentResponse::Analyzed(m) => m,
            _ => {
                return Err(McpError {
                    code: error_codes::INTERNAL_ERROR,
                    message: "Unexpected response type from analyzer".to_string(),
                    data: None,
                })
            }
        };

        // Step 2: Validate metrics with thresholds
        let thresholds = crate::modules::validator::Thresholds::default();

        let validate_msg = ValidateMessage {
            metrics: metrics.clone(),
            thresholds,
            priority: Priority::Normal,
        };

        let validate_response = validator
            .send(validate_msg)
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Actor communication failed: {}", e),
                data: None,
            })?
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Validation failed: {}", e),
                data: None,
            })?;

        // Convert AgentResponse to MCP format
        match validate_response {
            crate::agents::AgentResponse::Validated(result) => Ok(json!({
                "type": "text",
                "text": format!(
                    "Validation Results:\n\nPassed: {}\nComplexity: {}\nViolations: {}\n",
                    result.passed,
                    metrics.complexity,
                    result.violations.len()
                )
            })),
            _ => Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Unexpected response type from validator".to_string(),
                data: None,
            }),
        }
    }
}

// Orchestrate tool - invokes orchestrator for complex workflows
pub struct OrchestrateTool {
    _registry: Arc<AgentRegistry>,
}

impl OrchestrateTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for OrchestrateTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "orchestrate".to_string(),
            description: "Orchestrate complex multi-step workflows".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "steps": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "action": {"type": "string"},
                                        "params": {"type": "object"}
                                    }
                                }
                            }
                        },
                        "required": ["name", "steps"]
                    },
                    "input": {
                        "type": "object",
                        "description": "Initial input data"
                    }
                },
                "required": ["workflow"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let _workflow = params["workflow"].clone();
        let _input = params["input"].clone();

        // Orchestration is handled by the WorkflowRepository and WorkflowExecutor
        // This tool provides a simplified MCP interface to workflow execution
        // Full workflow orchestration requires:
        // 1. WorkflowBuilder to create workflow definitions
        // 2. WorkflowRepository to store workflows
        // 3. WorkflowExecutor to execute workflows with DagEngine
        // 4. Integration with agent registry for step execution

        Ok(json!({
            "type": "text",
            "text": "Orchestration via WorkflowRepository - use workflow/execute endpoint"
        }))
    }
}

// Quality gate tool
pub struct QualityGateTool {
    _registry: Arc<AgentRegistry>,
}

impl QualityGateTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for QualityGateTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "quality_gate".to_string(),
            description: "Run quality gate checks with zero tolerance".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Source code to check"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "gates": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["complexity", "satd", "efficiency", "entropy"]
                        },
                        "description": "Quality gates to run"
                    }
                },
                "required": ["code", "language"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let code = params["code"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing code parameter".to_string(),
            data: None,
        })?;

        let _language = params["language"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing language parameter".to_string(),
            data: None,
        })?;

        let gates = params["gates"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["complexity".to_string(), "satd".to_string()]);

        // Run quality gates through supervisor
        use crate::quality::complexity::ComplexityAnalyzer;
        use crate::quality::satd::SatdDetector;

        let mut results = json!({});

        for gate in gates {
            match gate.as_str() {
                "complexity" => {
                    let analyzer = ComplexityAnalyzer::new();
                    // TODO: Fix when analyze_code is implemented for language parameter
                    let complexity = analyzer.analyze_string(code).unwrap_or_default();
                    results["complexity"] = json!(complexity);
                }
                "satd" => {
                    let detector = SatdDetector::new();
                    let satd = detector.detect(code);
                    results["satd"] = json!(satd);
                }
                _ => {}
            }
        }

        Ok(json!({
            "type": "text",
            "text": serde_json::to_string_pretty(&results).unwrap()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "analyze");
        assert!(metadata.description.contains("quality metrics"));
    }

    #[test]
    fn test_transform_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "transform");
        assert!(metadata.description.contains("AST manipulation"));
    }

    #[test]
    fn test_validate_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "validate");
        assert!(metadata.description.contains("quality standards"));
    }
}
