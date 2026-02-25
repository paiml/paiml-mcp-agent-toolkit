#![cfg_attr(coverage_nightly, coverage(off))]
//! MCP-AGENTS.md Bridge
//!
//! Bidirectional bridge between AGENTS.md and MCP protocols.

use super::{AgentsMdDocument, Command};
use serde_json::{json, Value as JsonValue};

/// MCP-AGENTS.md protocol bridge
pub struct McpAgentsMdBridge {
    /// Protocol configuration
    config: BridgeConfig,

    /// Tool registry
    tool_registry: Vec<McpTool>,
}

/// Bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Enable bidirectional sync
    pub bidirectional: bool,

    /// Auto-discover AGENTS.md files
    pub auto_discover: bool,

    /// Quality enforcement level
    pub quality_level: QualityLevel,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            bidirectional: true,
            auto_discover: true,
            quality_level: QualityLevel::Standard,
        }
    }
}

/// Quality enforcement levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QualityLevel {
    None,
    Basic,
    Standard,
    Strict,
    Extreme,
}

/// MCP tool representation
#[derive(Debug, Clone)]
pub struct McpTool {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema
    pub input_schema: JsonValue,

    /// Output schema
    pub output_schema: JsonValue,

    /// Handler function
    pub handler: ToolHandler,
}

/// Tool handler type
#[derive(Debug, Clone)]
pub enum ToolHandler {
    Command(Command),
    Function(String),
    External(String),
}

/// Request types
#[derive(Debug, Clone)]
pub enum Request {
    AgentsMd(AgentsMdRequest),
    Mcp(McpRequest),
}

/// AGENTS.md request
#[derive(Debug, Clone)]
pub struct AgentsMdRequest {
    /// Request type
    pub request_type: String,

    /// Parameters
    pub params: JsonValue,
}

/// MCP request
#[derive(Debug, Clone)]
pub struct McpRequest {
    /// Method name
    pub method: String,

    /// Parameters
    pub params: JsonValue,
}

/// Translated request
#[derive(Debug, Clone)]
pub struct TranslatedRequest {
    /// Original request
    pub original: Request,

    /// Translated format
    pub translated: Request,

    /// Translation metadata
    pub metadata: TranslationMetadata,
}

/// Translation metadata
#[derive(Debug, Clone)]
pub struct TranslationMetadata {
    /// Translation timestamp
    pub timestamp: std::time::SystemTime,

    /// Quality checks applied
    pub quality_checks: Vec<String>,

    /// Warnings
    pub warnings: Vec<String>,
}

/// Response types
#[derive(Debug, Clone)]
pub enum Response {
    AgentsMd(AgentsMdResponse),
    Mcp(McpResponse),
}

/// AGENTS.md response
#[derive(Debug, Clone)]
pub struct AgentsMdResponse {
    /// Success status
    pub success: bool,

    /// Result data
    pub result: JsonValue,

    /// Error if any
    pub error: Option<String>,
}

/// MCP response
#[derive(Debug, Clone)]
pub struct McpResponse {
    /// Result data
    pub result: JsonValue,

    /// Error if any
    pub error: Option<JsonValue>,
}

/// Unified response
#[derive(Debug, Clone)]
pub struct UnifiedResponse {
    /// Original response
    pub original: Response,

    /// Unified format
    pub unified: JsonValue,

    /// Quality report
    pub quality_report: Option<QualityReport>,
}

/// Quality report for responses
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// Quality score
    pub score: f64,

    /// Issues found
    pub issues: Vec<String>,

    /// Suggestions
    pub suggestions: Vec<String>,
}

impl Default for McpAgentsMdBridge {
    fn default() -> Self {
        Self::new()
    }
}

// Conversion, translation, and quality methods
include!("bridge_conversion.rs");

// Unit tests
include!("bridge_tests.rs");
