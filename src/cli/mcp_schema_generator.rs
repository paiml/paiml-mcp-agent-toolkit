#![cfg_attr(coverage_nightly, coverage(off))]
//! MCP Schema Generator - Generate MCP tool definitions from CommandRegistry
//!
//! This module generates JSON Schema for MCP tools from the single source of truth,
//! ensuring MCP tool definitions never drift from CLI implementations.
//!
//! # Architecture (Toyota Way - Poka-yoke)
//!
//! ```text
//! CommandRegistry → McpSchemaGenerator → tools/list response
//!                                            └─ JSON Schema
//! ```
//!
//! # References
//!
//! - Specification: docs/specifications/unified-cli-mcp-help-integration.md
//! - GitHub Issue: #118
//! - MCP Protocol: https://spec.modelcontextprotocol.io/

use crate::cli::registry::{CommandMetadata, CommandRegistry, ValueType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Generates MCP tool definitions from CommandRegistry.
pub struct McpSchemaGenerator {
    registry: CommandRegistry,
}

/// MCP tool definition as per protocol spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<McpToolAnnotations>,
}

/// MCP tool annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    #[serde(rename = "openWorldHint", skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// Schema consistency error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    MissingSchemaProperty {
        tool: String,
        property: String,
    },
    TypeMismatch {
        tool: String,
        property: String,
        expected: String,
        actual: String,
    },
    DuplicateToolName {
        tool_name: String,
        command1: String,
        command2: String,
    },
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSchemaProperty { tool, property } => {
                write!(
                    f,
                    "Tool '{}' missing required property '{}'",
                    tool, property
                )
            }
            Self::TypeMismatch {
                tool,
                property,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Tool '{}' property '{}' type mismatch: expected {}, got {}",
                    tool, property, expected, actual
                )
            }
            Self::DuplicateToolName {
                tool_name,
                command1,
                command2,
            } => {
                write!(
                    f,
                    "Duplicate MCP tool name '{}' in commands '{}' and '{}'",
                    tool_name, command1, command2
                )
            }
        }
    }
}

impl std::error::Error for SchemaError {}

// --- Submodule includes ---
include!("mcp_schema_generator_impl.rs");
include!("mcp_schema_generator_tests.rs");
