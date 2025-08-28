//! MCP server implementation using uniform contracts
//! This ensures MCP tools use exactly the same contracts as CLI and HTTP

use super::*;
use super::service::ContractService;
use anyhow::Result;
use pmcp::{
    Server, ServerBuilder, Tool, ToolHandler, ToolResult,
    types::{JsonValue, ToolDefinition},
};
use serde_json::{json, Value};
use std::sync::Arc;
use async_trait::async_trait;

/// MCP server that uses uniform contracts
pub struct ContractMcpServer {
    server: Server,
    service: Arc<ContractService>,
}

impl ContractMcpServer {
    pub async fn new() -> Result<Self> {
        let service = Arc::new(ContractService::new()?);
        
        let server = ServerBuilder::new()
            .name("pmat")
            .version(env!("CARGO_PKG_VERSION"))
            .tool(create_analyze_complexity_tool())
            .tool(create_analyze_satd_tool())
            .tool(create_analyze_dead_code_tool())
            .tool(create_analyze_tdg_tool())
            .tool(create_analyze_lint_hotspot_tool())
            .tool(create_quality_gate_tool())
            .tool(create_refactor_auto_tool())
            .build()
            .await?;
        
        Ok(Self { server, service })
    }
    
    pub async fn run(self) -> Result<()> {
        self.server.run().await
    }
    
    /// Handle tool calls using uniform contracts
    pub async fn handle_tool_call(&self, name: &str, params: Value) -> Result<ToolResult> {
        // Apply backward compatibility mapping
        let params = super::adapter::BackwardCompatibility::map_json_params(params);
        
        match name {
            "analyze_complexity" => self.handle_analyze_complexity(params).await,
            "analyze_satd" => self.handle_analyze_satd(params).await,
            "analyze_dead_code" => self.handle_analyze_dead_code(params).await,
            "analyze_tdg" => self.handle_analyze_tdg(params).await,
            "analyze_lint_hotspot" => self.handle_analyze_lint_hotspot(params).await,
            "quality_gate" => self.handle_quality_gate(params).await,
            "refactor_auto" => self.handle_refactor_auto(params).await,
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
    
    async fn handle_analyze_complexity(&self, params: Value) -> Result<ToolResult> {
        let contract = serde_json::from_value::<AnalyzeComplexityContract>(params)?;
        let result = self.service.analyze_complexity(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_satd(&self, params: Value) -> Result<ToolResult> {
        let contract = serde_json::from_value::<AnalyzeSatdContract>(params)?;
        let result = self.service.analyze_satd(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_dead_code(&self, params: Value) -> Result<ToolResult> {
        let contract = serde_json::from_value::<AnalyzeDeadCodeContract>(params)?;
        let result = self.service.analyze_dead_code(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_tdg(&self, params: Value) -> Result<ToolResult> {
        let contract = serde_json::from_value::<AnalyzeTdgContract>(params)?;
        let result = self.service.analyze_tdg(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_analyze_lint_hotspot(&self, params: Value) -> Result<ToolResult> {
        let contract = serde_json::from_value::<AnalyzeLintHotspotContract>(params)?;
        let result = self.service.analyze_lint_hotspot(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_quality_gate(&self, params: Value) -> Result<ToolResult> {
        let contract = serde_json::from_value::<QualityGateContract>(params)?;
        let result = self.service.quality_gate(contract).await?;
        Ok(ToolResult::Success(result))
    }
    
    async fn handle_refactor_auto(&self, params: Value) -> Result<ToolResult> {
        let contract = serde_json::from_value::<RefactorAutoContract>(params)?;
        let result = self.service.refactor_auto(contract).await?;
        Ok(ToolResult::Success(result))
    }
}

/// Create tool definition for analyze_complexity
fn create_analyze_complexity_tool() -> ToolDefinition {
    ToolDefinition {
        name: "analyze_complexity".to_string(),
        description: "Analyze code complexity metrics".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "max_cyclomatic": {
                    "type": "integer",
                    "description": "Maximum cyclomatic complexity"
                },
                "max_cognitive": {
                    "type": "integer",
                    "description": "Maximum cognitive complexity"
                },
                "max_halstead": {
                    "type": "number",
                    "description": "Maximum Halstead difficulty"
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_satd
fn create_analyze_satd_tool() -> ToolDefinition {
    ToolDefinition {
        name: "analyze_satd".to_string(),
        description: "Analyze Self-Admitted Technical Debt in comments".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "severity": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "critical"],
                    "description": "Filter by severity"
                },
                "critical_only": {
                    "type": "boolean",
                    "description": "Show only critical items",
                    "default": false
                },
                "strict": {
                    "type": "boolean",
                    "description": "Use strict mode",
                    "default": false
                },
                "fail_on_violation": {
                    "type": "boolean",
                    "description": "Fail if violations found",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_dead_code
fn create_analyze_dead_code_tool() -> ToolDefinition {
    ToolDefinition {
        name: "analyze_dead_code".to_string(),
        description: "Analyze dead and unreachable code".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "include_unreachable": {
                    "type": "boolean",
                    "description": "Include unreachable code",
                    "default": false
                },
                "min_dead_lines": {
                    "type": "integer",
                    "description": "Minimum dead lines to report",
                    "default": 10
                },
                "max_percentage": {
                    "type": "number",
                    "description": "Maximum allowed percentage",
                    "default": 15.0
                },
                "fail_on_violation": {
                    "type": "boolean",
                    "description": "Fail if violations found",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_tdg
fn create_analyze_tdg_tool() -> ToolDefinition {
    ToolDefinition {
        name: "analyze_tdg".to_string(),
        description: "Analyze Technical Debt Gradient scores".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "threshold": {
                    "type": "number",
                    "description": "TDG threshold for filtering",
                    "default": 1.5
                },
                "include_components": {
                    "type": "boolean",
                    "description": "Include component breakdown",
                    "default": false
                },
                "critical_only": {
                    "type": "boolean",
                    "description": "Show only critical files",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for analyze_lint_hotspot
fn create_analyze_lint_hotspot_tool() -> ToolDefinition {
    ToolDefinition {
        name: "analyze_lint_hotspot".to_string(),
        description: "Find files with highest defect density".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "file": {
                    "type": "string",
                    "description": "Specific file to analyze"
                },
                "max_density": {
                    "type": "number",
                    "description": "Maximum defect density",
                    "default": 5.0
                },
                "min_confidence": {
                    "type": "number",
                    "description": "Minimum confidence for fixes",
                    "default": 0.8
                },
                "enforce": {
                    "type": "boolean",
                    "description": "Enforce quality standards",
                    "default": false
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Dry run mode",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for quality_gate
fn create_quality_gate_tool() -> ToolDefinition {
    ToolDefinition {
        name: "quality_gate".to_string(),
        description: "Run quality gate checks".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to analyze"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "top_files": {
                    "type": "integer",
                    "description": "Number of top files to show",
                    "default": 10
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                },
                "profile": {
                    "type": "string",
                    "enum": ["standard", "strict", "extreme", "toyota"],
                    "description": "Quality profile",
                    "default": "standard"
                },
                "file": {
                    "type": "string",
                    "description": "Specific file to check"
                },
                "fail_on_violation": {
                    "type": "boolean",
                    "description": "Fail if violations found",
                    "default": false
                },
                "verbose": {
                    "type": "boolean",
                    "description": "Verbose output",
                    "default": false
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create tool definition for refactor_auto
fn create_refactor_auto_tool() -> ToolDefinition {
    ToolDefinition {
        name: "refactor_auto".to_string(),
        description: "Automatically refactor code to reduce complexity".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "File to refactor"
                },
                "format": {
                    "type": "string",
                    "enum": ["table", "json", "yaml", "markdown", "csv", "summary"],
                    "default": "table"
                },
                "output": {
                    "type": "string",
                    "description": "Output file path"
                },
                "target_complexity": {
                    "type": "integer",
                    "description": "Target complexity",
                    "default": 8
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Dry run mode",
                    "default": false
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 60
                }
            },
            "required": ["file"]
        }),
    }
}