#![cfg_attr(coverage_nightly, coverage(off))]

use crate::agents::registry::AgentRegistry;
use crate::mcp_integration::{error_codes, McpError, McpTool, ToolMetadata};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

// Orchestrate tool - invokes orchestrator for complex workflows
pub struct OrchestrateTool {
    pub(super) registry: Arc<AgentRegistry>,
    pub(super) executor: Arc<dyn crate::workflow::WorkflowExecutor>,
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
    pub(super) _registry: Arc<AgentRegistry>,
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
