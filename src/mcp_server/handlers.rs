use crate::mcp_server::state_manager::StateManager;
use crate::models::refactor::RefactorConfig;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

// Refactor operation handlers: start, next_iteration, get_state, stop
include!("handlers_refactor_ops.rs");

// Parameter parsing and state serialization helpers
include!("handlers_parsing.rs");
