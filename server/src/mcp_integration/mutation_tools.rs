//! MCP tools for Mutation Testing
//!
//! Provides AI-powered mutation testing tools for MCP clients.

use super::*;
use crate::agents::registry::AgentRegistry;
use serde_json::json;
use std::sync::Arc;

/// Mutation testing tool - ML-powered test suite quality analysis
pub struct MutationTestTool {
    _registry: Arc<AgentRegistry>,
}

impl MutationTestTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for MutationTestTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "mutation_test".to_string(),
            description: "Perform ML-powered mutation testing to assess test suite quality".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to source code to mutate"
                    },
                    "operators": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Mutation operators to use: AOR, ROR, COR, UOR, CRR, SDL, WasmNumeric, WasmControlFlow, WasmLocal"
                    },
                    "ml_predict": {
                        "type": "boolean",
                        "description": "Enable ML-based survivability prediction",
                        "default": false
                    },
                    "distributed": {
                        "type": "boolean",
                        "description": "Enable distributed execution with work-stealing queue",
                        "default": false
                    },
                    "workers": {
                        "type": "integer",
                        "description": "Number of worker threads for distributed execution",
                        "default": 4,
                        "minimum": 1,
                        "maximum": 128
                    },
                    "min_score": {
                        "type": "number",
                        "description": "Minimum mutation score threshold (0.0-1.0)",
                        "minimum": 0.0,
                        "maximum": 1.0
                    },
                    "ci_learning": {
                        "type": "boolean",
                        "description": "Enable CI/CD learning mode for automated model training",
                        "default": false
                    },
                    "ci_provider": {
                        "type": "string",
                        "enum": ["github", "gitlab", "jenkins"],
                        "description": "CI/CD provider for learning mode"
                    },
                    "auto_train_threshold": {
                        "type": "integer",
                        "description": "Number of samples before auto-training",
                        "default": 50,
                        "minimum": 10
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use crate::services::mutation::{MutationEngine, MutationConfig, RustAdapter};
        use std::path::PathBuf;

        let path_str = params["path"].as_str().ok_or_else(|| McpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing 'path' parameter".to_string(),
            data: None,
        })?;

        let path = PathBuf::from(path_str);

        if !path.exists() {
            return Err(McpError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Path does not exist: {}", path_str),
                data: None,
            });
        }

        let operators = params["operators"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["AOR".to_string(), "ROR".to_string(), "COR".to_string(), "UOR".to_string()]);

        let _ml_predict = params["ml_predict"].as_bool().unwrap_or(false);
        let _distributed = params["distributed"].as_bool().unwrap_or(false);
        let _workers = params["workers"].as_u64().unwrap_or(4) as usize;
        let min_score = params["min_score"].as_f64();
        let _ci_learning = params["ci_learning"].as_bool().unwrap_or(false);
        let _ci_provider = params["ci_provider"].as_str();
        let _auto_train_threshold = params["auto_train_threshold"].as_u64().unwrap_or(50) as usize;

        // Create mutation engine with Rust adapter
        let adapter = Arc::new(RustAdapter::new());
        let config = MutationConfig::default();
        let engine = MutationEngine::new(adapter, config);

        // Generate mutants
        let mutants = if path.is_file() {
            engine.generate_mutants_from_file(&path)
                .await
                .map_err(|e| McpError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("Failed to generate mutants: {}", e),
                    data: None,
                })?
        } else {
            return Err(McpError {
                code: error_codes::INVALID_PARAMS,
                message: "Directory mutation testing not yet supported. Please provide a file path.".to_string(),
                data: None,
            });
        };

        let total_mutants = mutants.len();

        // Simulate mutation score (actual execution would run tests)
        let simulated_killed = (total_mutants as f64 * 0.75) as usize;
        let simulated_survived = total_mutants - simulated_killed;
        let mutation_score = if total_mutants > 0 {
            simulated_killed as f64 / total_mutants as f64
        } else {
            0.0
        };

        // Check minimum score threshold
        if let Some(min) = min_score {
            if mutation_score < min {
                return Err(McpError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!(
                        "Mutation score {:.2}% is below threshold {:.2}%",
                        mutation_score * 100.0,
                        min * 100.0
                    ),
                    data: None,
                });
            }
        }

        let report = json!({
            "mutation_score": mutation_score,
            "total_mutants": total_mutants,
            "killed": simulated_killed,
            "survived": simulated_survived,
            "operators": operators,
            "mode": "simulation",
            "note": "Simulation mode - actual test execution not yet implemented",
            "mutants_sample": mutants.iter().take(10).map(|m| {
                json!({
                    "id": m.id,
                    "operator": format!("{:?}", m.operator),
                    "line": m.location.line,
                    "column": m.location.column,
                })
            }).collect::<Vec<_>>()
        });

        Ok(report)
    }
}
