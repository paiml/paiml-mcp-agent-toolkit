#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::ExecutionMode;
use crate::handlers::tools::handle_tool_call;
use crate::models::mcp::{McpRequest, McpResponse};
#[cfg(feature = "git-lib")]
use crate::services::git_clone::{CloneError, GitCloner};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(feature = "git-lib")]
use std::time::Duration;
use std::time::Instant;
#[cfg(feature = "git-lib")]
use tokio::time::sleep;

// Type definitions: DemoRunner, DemoStep, DemoReport, DemoAnalysisResult, Component
include!("runner_types.rs");

impl DemoRunner {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(server: Arc<StatelessTemplateServer>) -> Self {
        Self {
            server,
            execution_log: Vec::new(),
        }
    }
}

// Git clone and repository preparation
include!("runner_clone.rs");

// System diagram generation (Mermaid)
include!("runner_diagram.rs");

// Demo execution pipeline (all demo_* methods)
include!("runner_pipeline.rs");

// DemoReport rendering (CLI and JSON output)
include!("runner_report.rs");

// Repository resolution and detection (free functions)
include!("runner_repository.rs");

// Tests extracted to runner_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
