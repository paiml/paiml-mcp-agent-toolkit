use super::*;
use crate::agents::registry::AgentRegistry;
use serde_json::json;
use std::sync::Arc;

// Analyze tool - invokes analyzer agent
pub struct AnalyzeTool {
    registry: Arc<AgentRegistry>,
}

impl AnalyzeTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
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

        let language = params["language"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing language parameter".to_string(),
            data: None,
        })?;

        // TODO: Create analyzer request when ModuleRequest is defined
        // let request = ModuleRequest::Analyze {
        //     code: code.to_string(),
        //     language: language.to_string(),
        // };

        // TODO: Implement agent processing after agent system is complete
        Ok(json!({
            "type": "text",
            "text": "Analysis not yet implemented"
        }))
    }
}

// Transform tool - invokes transformer agent
pub struct TransformTool {
    registry: Arc<AgentRegistry>,
}

impl TransformTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
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

        let transformation = params["transformation"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing transformation parameter".to_string(),
            data: None,
        })?;

        // TODO: Create transform request when ModuleRequest is defined
        // let request = ModuleRequest::Transform {
        //     ast: json!({"code": code}),
        //     operation: transformation.to_string(),
        // };

        // TODO: Implement transformer after agent system is complete
        /*
        if let Some(transformer) = self.registry.get_agent("transformer").await {
            match transformer.process(request).await {
                Ok(ModuleResponse::Transformation(result)) => Ok(json!({
                    "type": "text",
                    "text": result.code
                })),
                Err(e) => Err(McpError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("Transformation failed: {}", e),
                    data: None,
                }),
            }
        } else {
            Err(McpError {
                code: error_codes::INTERNAL_ERROR,
                message: "Transformer agent not found".to_string(),
                data: None,
            })
        }
        */
        Ok(json!({
            "type": "text",
            "text": "Transformation not yet implemented"
        }))
    }
}

// Validate tool - invokes validator agent
pub struct ValidateTool {
    registry: Arc<AgentRegistry>,
}

impl ValidateTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
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

        // TODO: Create validation request when ModuleRequest is defined
        // let request = ModuleRequest::Validate {
        //     data: json!({"code": code}),
        //     rules: vec![],
        // };

        // TODO: Implement validator after agent system is complete
        Ok(json!({
            "type": "text",
            "text": "Validation not yet implemented"
        }))
    }
}

// Orchestrate tool - invokes orchestrator for complex workflows
pub struct OrchestrateTool {
    registry: Arc<AgentRegistry>,
}

impl OrchestrateTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
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
        let workflow = params["workflow"].clone();
        let input = params["input"].clone();

        // TODO: Create orchestration request when ModuleRequest is defined
        // let request = ModuleRequest::Orchestrate {
        //     workflow,
        //     context: input.unwrap_or(json!({})),
        // };

        // TODO: Implement orchestrator after agent system is complete
        Ok(json!({
            "type": "text",
            "text": "Orchestration not yet implemented"
        }))
    }
}

// Quality gate tool
pub struct QualityGateTool {
    registry: Arc<AgentRegistry>,
}

impl QualityGateTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
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

        let language = params["language"].as_str().ok_or_else(|| McpError {
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
        use crate::quality::gate::QualityGateRunner;
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
