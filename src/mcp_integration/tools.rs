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
            .map(|p| match p {
                "critical" => Priority::Critical,
                "high" => Priority::High,
                "low" => Priority::Low,
                _ => Priority::Normal,
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

    pub fn new_with_actor(
        registry: Arc<AgentRegistry>,
        transformer: Addr<TransformerActor>,
    ) -> Self {
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
            .map(|p| match p {
                "critical" => Priority::Critical,
                "high" => Priority::High,
                "low" => Priority::Low,
                _ => Priority::Normal,
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
    registry: Arc<AgentRegistry>,
    executor: Arc<dyn crate::workflow::WorkflowExecutor>,
}

impl OrchestrateTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        let executor = Arc::new(crate::workflow::executor::DefaultWorkflowExecutor::new(
            registry.clone(),
        ));
        Self { registry, executor }
    }

    pub fn new_with_executor(
        registry: Arc<AgentRegistry>,
        executor: Arc<dyn crate::workflow::WorkflowExecutor>,
    ) -> Self {
        Self { registry, executor }
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
        use crate::workflow::{Workflow, WorkflowContext, WorkflowState};
        use parking_lot::RwLock;
        use std::time::Instant;
        use uuid::Uuid;

        let workflow_def = params["workflow"].as_object().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing workflow parameter".to_string(),
            data: None,
        })?;

        let input = params["input"].clone();

        // Parse workflow from JSON
        let workflow: Workflow = serde_json::from_value(json!({
            "id": Uuid::new_v4().to_string(),
            "name": workflow_def.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("mcp_workflow"),
            "description": workflow_def.get("description")
                .and_then(|v| v.as_str()),
            "version": "1.0.0",
            "steps": workflow_def.get("steps")
                .ok_or_else(|| McpError {
                    code: error_codes::INVALID_PARAMS,
                    message: "Missing steps in workflow".to_string(),
                    data: None,
                })?,
            "error_strategy": "fail_fast",
            "timeout": null,
            "metadata": {}
        }))
        .map_err(|e| McpError {
            code: error_codes::INVALID_PARAMS,
            message: format!("Invalid workflow definition: {}", e),
            data: None,
        })?;

        // Create workflow context
        let context = WorkflowContext {
            workflow_id: workflow.id,
            execution_id: Uuid::new_v4(),
            variables: Arc::new(RwLock::new(
                input
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            )),
            step_results: Arc::new(RwLock::new(std::collections::HashMap::new())),
            state: Arc::new(RwLock::new(WorkflowState::Running)),
            started_at: Instant::now(),
            agent_registry: self.registry.clone(),
        };

        // Execute workflow
        let result = self
            .executor
            .execute(&workflow, &context)
            .await
            .map_err(|e| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Workflow execution failed: {}", e),
                data: None,
            })?;

        // Return results in MCP format
        Ok(json!({
            "type": "text",
            "text": format!(
                "Workflow Execution Results:\n\nWorkflow: {}\nExecution ID: {}\n\nResult:\n{}\n",
                workflow.name,
                context.execution_id,
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            )
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
        use crate::quality::satd::SatdDetector;

        let mut results = json!({});

        for gate in gates {
            match gate.as_str() {
                "complexity" => {
                    // Note: ComplexityAnalyzer currently only supports Rust (uses syn::parse_file)
                    // For non-Rust languages, this will fail gracefully and return default metrics
                    let analyzer = ComplexityAnalyzer::new();
                    let complexity = if language.to_lowercase() == "rust" {
                        analyzer.analyze_string(code).unwrap_or_default()
                    } else {
                        // Return default/zero metrics for unsupported languages
                        Default::default()
                    };
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
            "text": serde_json::to_string_pretty(&results).expect("internal error")
        }))
    }
}

// ============================================================================
// Semantic Search Tool Adapters (Sprint 33: PMAT-SEARCH-012)
// ============================================================================
// These adapters bridge the semantic search tools (crate::mcp::McpTool)
// to the mcp_integration tool system (mcp_integration::McpTool)

use crate::services::semantic::HybridSearchEngine;

/// Adapter for semantic_search tool
pub struct SemanticSearchToolAdapter {
    inner: crate::mcp::tools::semantic_search_tools::SemanticSearchTool,
}

impl SemanticSearchToolAdapter {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self {
            inner: crate::mcp::tools::semantic_search_tools::SemanticSearchTool::new(engine),
        }
    }
}

#[async_trait]
impl McpTool for SemanticSearchToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        // Extract description from schema
        let description = schema["description"]
            .as_str()
            .unwrap_or("Search code by natural language query")
            .to_string();

        // Extract input_schema from schema parameters
        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for find_similar_code tool
pub struct FindSimilarCodeToolAdapter {
    inner: crate::mcp::tools::semantic_search_tools::FindSimilarCodeTool,
}

impl FindSimilarCodeToolAdapter {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self {
            inner: crate::mcp::tools::semantic_search_tools::FindSimilarCodeTool::new(engine),
        }
    }
}

#[async_trait]
impl McpTool for FindSimilarCodeToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Find similar code files using vector similarity")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for cluster_code tool
pub struct ClusterCodeToolAdapter {
    inner: crate::mcp::tools::semantic_search_tools::ClusterCodeTool,
}

impl ClusterCodeToolAdapter {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self {
            inner: crate::mcp::tools::semantic_search_tools::ClusterCodeTool::new(engine),
        }
    }
}

#[async_trait]
impl McpTool for ClusterCodeToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Cluster code by semantic similarity")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for analyze_topics tool
pub struct AnalyzeTopicsToolAdapter {
    inner: crate::mcp::tools::semantic_search_tools::AnalyzeTopicsTool,
}

impl AnalyzeTopicsToolAdapter {
    pub fn new(engine: Arc<HybridSearchEngine>) -> Self {
        Self {
            inner: crate::mcp::tools::semantic_search_tools::AnalyzeTopicsTool::new(engine),
        }
    }
}

#[async_trait]
impl McpTool for AnalyzeTopicsToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Extract semantic topics from codebase")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::semantic_search_tools::McpTool as SimpleMcpTool;

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

// ============================================================================
// Agent Context Tool Adapters (PMAT-470)
// ============================================================================
// These adapters bridge the agent context tools (crate::mcp::tools::agent_context_tools)
// to the mcp_integration tool system (mcp_integration::McpTool)

use crate::mcp::tools::agent_context_tools::IndexManager;

/// Adapter for pmat_query_code tool
pub struct QueryCodeToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::QueryCodeTool,
}

impl QueryCodeToolAdapter {
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self {
            inner: crate::mcp::tools::agent_context_tools::QueryCodeTool::new(manager),
        }
    }
}

#[async_trait]
impl McpTool for QueryCodeToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Search code functions by natural language query")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for pmat_get_function tool
pub struct GetFunctionToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::GetFunctionTool,
}

impl GetFunctionToolAdapter {
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self {
            inner: crate::mcp::tools::agent_context_tools::GetFunctionTool::new(manager),
        }
    }
}

#[async_trait]
impl McpTool for GetFunctionToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Get detailed information about a specific function")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for pmat_find_similar tool
pub struct FindSimilarToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::FindSimilarTool,
}

impl FindSimilarToolAdapter {
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self {
            inner: crate::mcp::tools::agent_context_tools::FindSimilarTool::new(manager),
        }
    }
}

#[async_trait]
impl McpTool for FindSimilarToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Find functions similar to a reference function")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

/// Adapter for pmat_index_stats tool
pub struct IndexStatsToolAdapter {
    inner: crate::mcp::tools::agent_context_tools::IndexStatsTool,
}

impl IndexStatsToolAdapter {
    pub fn new(manager: Arc<IndexManager>) -> Self {
        Self {
            inner: crate::mcp::tools::agent_context_tools::IndexStatsTool::new(manager),
        }
    }
}

#[async_trait]
impl McpTool for IndexStatsToolAdapter {
    fn metadata(&self) -> ToolMetadata {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;
        let name = self.inner.name().to_string();
        let schema = self.inner.schema();

        let description = schema["description"]
            .as_str()
            .unwrap_or("Get statistics about the code index")
            .to_string();

        let input_schema = schema["parameters"].clone();

        ToolMetadata {
            name,
            description,
            input_schema,
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::mcp::tools::agent_context_tools::McpTool as SimpleMcpTool;

        self.inner
            .execute(params)
            .await
            .map_err(|err_msg| McpError {
                code: error_codes::INTERNAL_ERROR,
                message: err_msg,
                data: None,
            })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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

    #[test]
    fn test_orchestrate_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "orchestrate");
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_quality_gate_tool_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = QualityGateTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "quality_gate");
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_analyze_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);
        assert!(tool.analyzer.is_none());
    }

    #[test]
    fn test_transform_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);
        assert!(tool.transformer.is_none());
    }

    #[test]
    fn test_validate_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);
        assert!(tool.validator.is_none());
    }

    #[test]
    fn test_orchestrate_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);
        // Just verify construction works
        assert!(!tool.metadata().name.is_empty());
    }

    #[test]
    fn test_quality_gate_tool_new() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = QualityGateTool::new(registry);
        assert!(!tool.metadata().name.is_empty());
    }

    #[test]
    fn test_analyze_tool_schema_has_required_fields() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["code"].is_object());
        assert!(schema["properties"]["language"].is_object());
        assert_eq!(schema["required"], json!(["code", "language"]));
    }

    #[test]
    fn test_transform_tool_schema_has_required_fields() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["code"].is_object());
        assert!(schema["properties"]["transformation"].is_object());
    }

    #[test]
    fn test_validate_tool_schema_has_required_fields() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["code"].is_object());
    }

    #[test]
    fn test_orchestrate_tool_schema() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["workflow"].is_object());
    }

    #[test]
    fn test_quality_gate_tool_schema() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = QualityGateTool::new(registry);
        let metadata = tool.metadata();
        let schema = &metadata.input_schema;

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["code"].is_object());
        assert!(schema["properties"]["language"].is_object());
    }

    #[actix_rt::test]
    async fn test_analyze_tool_missing_code() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);

        let result = tool.execute(json!({ "language": "rust" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("code"));
    }

    #[actix_rt::test]
    async fn test_analyze_tool_missing_language() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);

        let result = tool.execute(json!({ "code": "fn main() {}" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("language"));
    }

    #[actix_rt::test]
    async fn test_transform_tool_missing_code() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);

        let result = tool.execute(json!({ "transform": "refactor" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("code"));
    }

    #[actix_rt::test]
    async fn test_transform_tool_missing_transform() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);

        let result = tool.execute(json!({ "code": "fn main() {}" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("transform"));
    }

    #[actix_rt::test]
    async fn test_validate_tool_missing_code() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("code"));
    }

    #[actix_rt::test]
    async fn test_orchestrate_tool_missing_workflow() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("workflow"));
    }

    #[actix_rt::test]
    async fn test_quality_gate_tool_missing_code() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = QualityGateTool::new(registry);

        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("code"));
    }

    #[actix_rt::test]
    async fn test_analyze_tool_no_actor() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = AnalyzeTool::new(registry);

        let result = tool
            .execute(json!({
                "code": "fn main() {}",
                "language": "rust"
            }))
            .await;

        // Should fail because no actor is initialized
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("not initialized"));
    }

    #[actix_rt::test]
    async fn test_transform_tool_no_actor() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = TransformTool::new(registry);

        let result = tool
            .execute(json!({
                "code": "fn main() {}",
                "transformation": "refactor"
            }))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("not initialized"));
    }

    #[actix_rt::test]
    async fn test_validate_tool_no_actor() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = ValidateTool::new(registry);

        let result = tool.execute(json!({ "code": "fn main() {}" })).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("not initialized"));
    }

    #[actix_rt::test]
    async fn test_orchestrate_tool_missing_workflow_name() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = OrchestrateTool::new(registry);

        // Workflow without name should still work (uses default)
        let result = tool
            .execute(json!({
                "workflow": {
                    "steps": []
                }
            }))
            .await;

        // Should either succeed or fail with workflow-related error
        // The orchestrator doesn't check for actor initialization
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(
                err.message.contains("workflow") || err.message.contains("Workflow"),
                "Error should be workflow-related: {}",
                err.message
            );
        }
    }

    #[test]
    fn test_tool_metadata_descriptions_not_empty() {
        let registry = Arc::new(AgentRegistry::new());

        let analyze = AnalyzeTool::new(registry.clone());
        assert!(!analyze.metadata().description.is_empty());

        let transform = TransformTool::new(registry.clone());
        assert!(!transform.metadata().description.is_empty());

        let validate = ValidateTool::new(registry.clone());
        assert!(!validate.metadata().description.is_empty());

        let orchestrate = OrchestrateTool::new(registry.clone());
        assert!(!orchestrate.metadata().description.is_empty());

        let quality_gate = QualityGateTool::new(registry);
        assert!(!quality_gate.metadata().description.is_empty());
    }

    #[test]
    fn test_all_tools_have_object_schema() {
        let registry = Arc::new(AgentRegistry::new());

        let tools: Vec<Box<dyn McpTool + Send + Sync>> = vec![
            Box::new(AnalyzeTool::new(registry.clone())),
            Box::new(TransformTool::new(registry.clone())),
            Box::new(ValidateTool::new(registry.clone())),
            Box::new(OrchestrateTool::new(registry.clone())),
            Box::new(QualityGateTool::new(registry)),
        ];

        for tool in tools {
            let metadata = tool.metadata();
            assert_eq!(metadata.input_schema["type"], "object");
        }
    }

    // ============================================================
    // Agent Context Tool Adapter Tests (PMAT-470)
    // ============================================================

    #[test]
    fn test_query_code_adapter_metadata() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));
        let tool = QueryCodeToolAdapter::new(manager);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "pmat_query_code");
        assert!(metadata.description.contains("Search code functions"));
        assert_eq!(metadata.input_schema["type"], "object");
    }

    #[test]
    fn test_get_function_adapter_metadata() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));
        let tool = GetFunctionToolAdapter::new(manager);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "pmat_get_function");
        assert!(metadata.description.contains("function"));
        assert_eq!(metadata.input_schema["type"], "object");
    }

    #[test]
    fn test_find_similar_adapter_metadata() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));
        let tool = FindSimilarToolAdapter::new(manager);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "pmat_find_similar");
        assert!(metadata.description.contains("similar"));
        assert_eq!(metadata.input_schema["type"], "object");
    }

    #[test]
    fn test_index_stats_adapter_metadata() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));
        let tool = IndexStatsToolAdapter::new(manager);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "pmat_index_stats");
        assert!(metadata.description.contains("statistics"));
        assert_eq!(metadata.input_schema["type"], "object");
    }

    #[test]
    fn test_agent_context_adapters_have_object_schema() {
        let manager = Arc::new(IndexManager::new(std::path::PathBuf::from("/tmp/test")));

        let tools: Vec<Box<dyn McpTool + Send + Sync>> = vec![
            Box::new(QueryCodeToolAdapter::new(manager.clone())),
            Box::new(GetFunctionToolAdapter::new(manager.clone())),
            Box::new(FindSimilarToolAdapter::new(manager.clone())),
            Box::new(IndexStatsToolAdapter::new(manager)),
        ];

        for tool in tools {
            let metadata = tool.metadata();
            assert_eq!(metadata.input_schema["type"], "object");
            assert!(!metadata.name.is_empty());
            assert!(!metadata.description.is_empty());
        }
    }
}
