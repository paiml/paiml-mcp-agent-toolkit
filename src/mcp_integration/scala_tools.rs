use crate::mcp_integration::ast_item_helpers::{extract_complexity, extract_kind, extract_name};
use crate::mcp_integration::{McpError, McpTool, ToolMetadata};
// Import the ScalaAstVisitor when available
use crate::services::languages::scala::ScalaAstVisitor;
use crate::utils::path_validator::PathValidator;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn};

// ScalaAnalysisTool: struct definition and McpTool impl
include!("scala_tools_analysis.rs");

// Free functions: analyze_scala_file, analyze_scala_directory,
// find_scala_files, calculate_functional_percentage
include!("scala_tools_helpers.rs");

// ScalaMutationTool: struct definition and McpTool impl
include!("scala_tools_mutation.rs");

// Tests
include!("scala_tools_tests.rs");
