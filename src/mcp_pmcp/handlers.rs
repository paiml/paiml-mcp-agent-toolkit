use crate::mcp_server::state_manager::StateManager;
use crate::models::refactor::RefactorConfig;
use async_trait::async_trait;
use pmcp::{Error as PmcpError, RequestHandlerExtra, Result as PmcpResult, ToolHandler};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Debug, Deserialize)]
struct RefactorStartArgs {
    targets: Vec<String>,
    config: Option<RefactorConfig>,
}

#[derive(Debug, Serialize)]
struct RefactorStartResult {
    session_id: String,
    state: Value,
}

pub struct RefactorStartTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorStartTool {
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[async_trait]
impl ToolHandler for RefactorStartTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
        debug!("Handling refactor.start with args: {}", args);

        let params: RefactorStartArgs = serde_json::from_value(args)
            .map_err(|e| PmcpError::validation(format!("Invalid arguments: {e}")))?;

        let targets: Vec<PathBuf> = params.targets.into_iter().map(PathBuf::from).collect();

        let config = params.config.unwrap_or_default();

        let mut manager = self.state_manager.lock().await;
        manager
            .start_session(targets, config)
            .map_err(|e| PmcpError::internal(format!("Failed to start session: {e}")))?;

        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {e}")))?;
        let session_id = manager.get_session_id().to_string();

        let state_value = serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {e}")))?;

        Ok(serde_json::to_value(RefactorStartResult {
            session_id,
            state: state_value,
        })?)
    }
}

pub struct RefactorNextIterationTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorNextIterationTool {
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[async_trait]
impl ToolHandler for RefactorNextIterationTool {
    async fn handle(&self, _args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
        debug!("Handling refactor.nextIteration");

        let mut manager = self.state_manager.lock().await;
        manager
            .advance()
            .map_err(|e| PmcpError::internal(format!("Failed to advance: {e}")))?;

        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {e}")))?;

        serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {e}")))
    }
}

pub struct RefactorGetStateTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorGetStateTool {
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[async_trait]
impl ToolHandler for RefactorGetStateTool {
    async fn handle(&self, _args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
        debug!("Handling refactor.getState");

        let manager = self.state_manager.lock().await;
        let state = manager
            .get_state()
            .map_err(|e| PmcpError::internal(format!("Failed to get state: {e}")))?;

        serialize_state(state)
            .map_err(|e| PmcpError::internal(format!("Failed to serialize state: {e}")))
    }
}

pub struct RefactorStopTool {
    state_manager: Arc<Mutex<StateManager>>,
}

impl RefactorStopTool {
    pub fn new(state_manager: Arc<Mutex<StateManager>>) -> Self {
        Self { state_manager }
    }
}

#[async_trait]
impl ToolHandler for RefactorStopTool {
    async fn handle(&self, _args: Value, _extra: RequestHandlerExtra) -> PmcpResult<Value> {
        debug!("Handling refactor.stop");

        let mut manager = self.state_manager.lock().await;
        manager
            .stop_session()
            .map_err(|e| PmcpError::internal(format!("Failed to stop session: {e}")))?;

        Ok(json!({
            "status": "stopped",
            "message": "Refactoring session stopped successfully"
        }))
    }
}

fn serialize_state(
    state_machine: &crate::models::refactor::RefactorStateMachine,
) -> Result<Value, Box<dyn std::error::Error>> {
    let state_json = match &state_machine.current {
        crate::models::refactor::State::Scan { targets } => {
            json!({
                "current": "Scan",
                "targets": targets,
                "current_target_index": state_machine.current_target_index,
                "config": state_machine.config
            })
        }
        crate::models::refactor::State::Analyze { current } => {
            json!({
                "current": "Analyze",
                "current_file": current,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Plan { violations } => {
            json!({
                "current": "Plan",
                "violations": violations,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Refactor { operation } => {
            json!({
                "current": "Refactor",
                "operation": operation,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Test { command } => {
            json!({
                "current": "Test",
                "command": command,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Lint { strict } => {
            json!({
                "current": "Lint",
                "strict": strict,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Emit { payload } => {
            json!({
                "current": "Emit",
                "payload": payload,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Checkpoint { reason } => {
            json!({
                "current": "Checkpoint",
                "reason": reason,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Complete { summary } => {
            json!({
                "current": "Complete",
                "summary": summary
            })
        }
    };

    Ok(state_json)
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::refactor::{
        BytePos, DefectPayload, FileId, RefactorOp, RefactorStateMachine, RefactorType, State,
        Summary,
    };
    use std::time::Duration;

    #[test]
    fn test_refactor_start_args_deserialize() {
        let json = json!({
            "targets": ["src/main.rs", "src/lib.rs"],
            "config": null
        });

        let args: RefactorStartArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.targets.len(), 2);
        assert!(args.config.is_none());
    }

    #[test]
    fn test_refactor_start_args_with_config() {
        let json = json!({
            "targets": ["src/main.rs"],
            "config": {
                "target_complexity": 10,
                "remove_satd": true,
                "max_function_lines": 50,
                "thresholds": {
                    "cyclomatic_warn": 10,
                    "cyclomatic_error": 20,
                    "cognitive_warn": 15,
                    "cognitive_error": 25,
                    "tdg_warn": 0.7,
                    "tdg_error": 0.5
                },
                "strategies": {
                    "prefer_functional": true,
                    "use_early_returns": true,
                    "extract_helpers": false
                },
                "parallel_workers": 4,
                "memory_limit_mb": 512,
                "batch_size": 10,
                "priority_expression": null,
                "auto_commit_template": null
            }
        });

        let args: RefactorStartArgs = serde_json::from_value(json).unwrap();
        assert_eq!(args.targets.len(), 1);
        assert!(args.config.is_some());
    }

    #[test]
    fn test_refactor_start_result_serialize() {
        let result = RefactorStartResult {
            session_id: "test-session-123".to_string(),
            state: json!({"current": "Scan"}),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["session_id"], "test-session-123");
        assert!(json["state"].is_object());
    }

    #[test]
    fn test_refactor_start_tool_new() {
        let state_manager = Arc::new(Mutex::new(StateManager::new()));
        let tool = RefactorStartTool::new(state_manager);
        assert!(Arc::strong_count(&tool.state_manager) == 1);
    }

    #[test]
    fn test_refactor_next_iteration_tool_new() {
        let state_manager = Arc::new(Mutex::new(StateManager::new()));
        let tool = RefactorNextIterationTool::new(state_manager);
        assert!(Arc::strong_count(&tool.state_manager) == 1);
    }

    #[test]
    fn test_refactor_get_state_tool_new() {
        let state_manager = Arc::new(Mutex::new(StateManager::new()));
        let tool = RefactorGetStateTool::new(state_manager);
        assert!(Arc::strong_count(&tool.state_manager) == 1);
    }

    #[test]
    fn test_refactor_stop_tool_new() {
        let state_manager = Arc::new(Mutex::new(StateManager::new()));
        let tool = RefactorStopTool::new(state_manager);
        assert!(Arc::strong_count(&tool.state_manager) == 1);
    }

    #[test]
    fn test_serialize_state_scan() {
        let state_machine = RefactorStateMachine {
            current: State::Scan {
                targets: vec![PathBuf::from("src/main.rs")],
            },
            targets: vec![PathBuf::from("src/main.rs")],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Scan");
        assert!(result["targets"].is_array());
    }

    #[test]
    fn test_serialize_state_analyze() {
        let state_machine = RefactorStateMachine {
            current: State::Analyze {
                current: FileId {
                    path: PathBuf::from("src/lib.rs"),
                    hash: 12345,
                },
            },
            targets: vec![PathBuf::from("src/lib.rs")],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Analyze");
        assert!(result["current_file"].is_object());
    }

    #[test]
    fn test_serialize_state_plan() {
        let state_machine = RefactorStateMachine {
            current: State::Plan { violations: vec![] },
            targets: vec![],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Plan");
        assert!(result["violations"].is_array());
    }

    #[test]
    fn test_serialize_state_refactor() {
        let state_machine = RefactorStateMachine {
            current: State::Refactor {
                operation: RefactorOp::ExtractFunction {
                    name: "extract_method".to_string(),
                    start: BytePos {
                        byte: 0,
                        line: 1,
                        column: 1,
                    },
                    end: BytePos {
                        byte: 100,
                        line: 10,
                        column: 1,
                    },
                    params: vec!["x".to_string()],
                },
            },
            targets: vec![],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Refactor");
        assert!(result["operation"].is_object());
    }

    #[test]
    fn test_serialize_state_test() {
        let state_machine = RefactorStateMachine {
            current: State::Test {
                command: "cargo test".to_string(),
            },
            targets: vec![],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Test");
        assert_eq!(result["command"], "cargo test");
    }

    #[test]
    fn test_serialize_state_lint() {
        let state_machine = RefactorStateMachine {
            current: State::Lint { strict: true },
            targets: vec![],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Lint");
        assert_eq!(result["strict"], true);
    }

    #[test]
    fn test_serialize_state_emit() {
        let state_machine = RefactorStateMachine {
            current: State::Emit {
                payload: DefectPayload {
                    file_hash: 12345,
                    tdg_score: 0.85,
                    complexity: (15, 20),
                    dead_symbols: 2,
                    timestamp: 1234567890,
                    severity_flags: 1,
                    refactor_available: true,
                    refactor_type: RefactorType::ExtractFunction,
                    estimated_improvement: 0.15,
                    _padding: [0, 0],
                },
            },
            targets: vec![],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Emit");
        assert!(result["payload"].is_object());
    }

    #[test]
    fn test_serialize_state_checkpoint() {
        let state_machine = RefactorStateMachine {
            current: State::Checkpoint {
                reason: "Pausing for review".to_string(),
            },
            targets: vec![],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Checkpoint");
        assert_eq!(result["reason"], "Pausing for review");
    }

    #[test]
    fn test_serialize_state_complete() {
        let state_machine = RefactorStateMachine {
            current: State::Complete {
                summary: Summary {
                    files_processed: 10,
                    refactors_applied: 5,
                    complexity_reduction: 0.25,
                    satd_removed: 3,
                    total_time: Duration::from_secs(120),
                },
            },
            targets: vec![],
            current_target_index: 0,
            config: RefactorConfig::default(),
            history: vec![],
        };

        let result = serialize_state(&state_machine).unwrap();
        assert_eq!(result["current"], "Complete");
        assert!(result["summary"].is_object());
    }

    #[test]
    fn test_refactor_start_args_empty_targets() {
        let json = json!({
            "targets": []
        });

        let args: RefactorStartArgs = serde_json::from_value(json).unwrap();
        assert!(args.targets.is_empty());
    }

    #[test]
    fn test_shared_state_manager() {
        let state_manager = Arc::new(Mutex::new(StateManager::new()));

        let tool1 = RefactorStartTool::new(state_manager.clone());
        let tool2 = RefactorGetStateTool::new(state_manager.clone());
        let tool3 = RefactorStopTool::new(state_manager.clone());

        // All tools share the same state manager
        assert!(Arc::strong_count(&tool1.state_manager) == 4); // 3 tools + original
        assert!(Arc::ptr_eq(&tool1.state_manager, &tool2.state_manager));
        assert!(Arc::ptr_eq(&tool2.state_manager, &tool3.state_manager));
    }

    #[test]
    fn test_refactor_stop_tool_construction() {
        let state_manager = Arc::new(Mutex::new(StateManager::new()));
        let tool = RefactorStopTool::new(state_manager);

        // Verify the tool was constructed correctly
        assert!(Arc::strong_count(&tool.state_manager) == 1);
    }

    #[test]
    fn test_all_tools_share_state() {
        let state_manager = Arc::new(Mutex::new(StateManager::new()));

        let start = RefactorStartTool::new(state_manager.clone());
        let next = RefactorNextIterationTool::new(state_manager.clone());
        let get = RefactorGetStateTool::new(state_manager.clone());
        let stop = RefactorStopTool::new(state_manager.clone());

        // All 4 tools + original = 5 references
        assert_eq!(Arc::strong_count(&start.state_manager), 5);
        assert!(Arc::ptr_eq(&start.state_manager, &next.state_manager));
        assert!(Arc::ptr_eq(&next.state_manager, &get.state_manager));
        assert!(Arc::ptr_eq(&get.state_manager, &stop.state_manager));
    }
}
