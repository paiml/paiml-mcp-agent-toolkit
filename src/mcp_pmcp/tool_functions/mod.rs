// Tool functions for MCP protocol - split for file health (CB-040)

use crate::cli::commands::{DiagnosticOutputFormat, StorageCommand, TdgCommand};
use crate::cli::handlers::tdg_diagnostic_handler;
use crate::qdd::{
    CodeType, CreateSpec, Parameter, QddOperation, QddTool, QualityProfile, RefactorSpec,
};
use crate::tdg::{
    AdaptiveThresholdFactory, SchedulerFactory, StorageBackendType, StorageConfig, TdgAnalyzer,
    TieredStorageFactory,
};
use crate::utils::path_validator::PathValidator;
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Include split modules
include!("analysis_tools.rs");
include!("quality_tools.rs");
include!("git_tools.rs");
include!("context_tools.rs");
include!("tdg_tools.rs");
include!("qdd_tools.rs");

// Tests
#[cfg(test)]
#[path = "../tool_functions_tests.rs"]
mod tests;
