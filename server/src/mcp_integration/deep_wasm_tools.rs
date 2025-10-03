//! MCP tools for Deep WASM analysis
//!
//! Provides AI-powered WebAssembly pipeline inspection tools for MCP clients.

use super::*;
use crate::agents::registry::AgentRegistry;
use serde_json::json;
use std::sync::Arc;

#[cfg(feature = "deep-wasm")]
use crate::services::deep_wasm::{
    AnalysisFocus, DeepWasmAnalysisRequest, DeepWasmService, ReportGenerator, SourceLanguage,
};

/// Deep WASM analysis tool - comprehensive pipeline inspection
pub struct DeepWasmAnalyzeTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmAnalyzeTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for DeepWasmAnalyzeTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "deep_wasm_analyze".to_string(),
            description: "Perform deep inspection of Rust/Ruchy → WASM → JS pipeline".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "source_path": {
                        "type": "string",
                        "description": "Path to source code file"
                    },
                    "wasm_path": {
                        "type": "string",
                        "description": "Path to WASM binary (optional)"
                    },
                    "dwarf_path": {
                        "type": "string",
                        "description": "Path to DWARF debug symbols (optional)"
                    },
                    "source_map_path": {
                        "type": "string",
                        "description": "Path to source map file (optional)"
                    },
                    "language": {
                        "type": "string",
                        "enum": ["rust", "ruchy"],
                        "description": "Source language (auto-detected if not specified)"
                    },
                    "focus": {
                        "type": "string",
                        "enum": ["full", "source", "compilation", "runtime", "interop"],
                        "description": "Analysis focus area",
                        "default": "full"
                    },
                    "strict": {
                        "type": "boolean",
                        "description": "Enable strict quality gates",
                        "default": false
                    }
                },
                "required": ["source_path"]
            }),
        }
    }

    #[cfg(feature = "deep-wasm")]
    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use std::path::PathBuf;

        let source_path = params["source_path"]
            .as_str()
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing source_path parameter".to_string(),
                data: None,
            })?
            .to_string();

        let language = match params["language"].as_str() {
            Some("rust") => SourceLanguage::Rust,
            Some("ruchy") => SourceLanguage::Ruchy,
            _ => {
                // Auto-detect
                if source_path.ends_with(".rs") {
                    SourceLanguage::Rust
                } else {
                    SourceLanguage::Ruchy
                }
            }
        };

        let focus = match params["focus"].as_str().unwrap_or("full") {
            "source" => AnalysisFocus::Source,
            "compilation" => AnalysisFocus::Compilation,
            "runtime" => AnalysisFocus::Runtime,
            "interop" => AnalysisFocus::Interop,
            _ => AnalysisFocus::Full,
        };

        let request = DeepWasmAnalysisRequest {
            source_path: PathBuf::from(source_path),
            wasm_path: params["wasm_path"].as_str().map(PathBuf::from),
            dwarf_path: params["dwarf_path"].as_str().map(PathBuf::from),
            source_map_path: params["source_map_path"].as_str().map(PathBuf::from),
            language,
            analysis_focus: focus,
        };

        let mut service = DeepWasmService::new();

        if params["strict"].as_bool().unwrap_or(false) {
            use crate::services::deep_wasm::WasmQualityGates;
            let mut gates = WasmQualityGates::default();
            gates.max_module_size = 5_242_880; // 5MB strict limit
            gates.max_wasm_complexity = 15;
            gates.min_source_map_coverage = 0.99;
            service = service.with_quality_gates(gates);
        }

        let report = service.analyze(request).await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Analysis failed: {}", e),
            data: None,
        })?;

        let generator = ReportGenerator::new();
        let markdown = generator.generate_markdown(&report).map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Report generation failed: {}", e),
            data: None,
        })?;

        Ok(json!({
            "type": "text",
            "text": markdown,
            "metadata": {
                "project_name": report.project_name,
                "module_size": report.wasm_module_analysis.module_size_bytes,
                "quality_passed": report.quality_gate_results.passed,
                "violations": report.quality_gate_results.violations.len(),
            }
        }))
    }

    #[cfg(not(feature = "deep-wasm"))]
    async fn execute(&self, _params: Value) -> Result<Value, McpError> {
        Err(McpError {
            code: error_codes::METHOD_NOT_FOUND,
            message: "Deep WASM feature not enabled. Recompile with --features deep-wasm".to_string(),
            data: None,
        })
    }
}

/// Query source-to-WASM mappings
pub struct DeepWasmQueryMappingTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmQueryMappingTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for DeepWasmQueryMappingTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "deep_wasm_query_mapping".to_string(),
            description: "Query source-to-WASM bidirectional mappings".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "wasm_path": {
                        "type": "string",
                        "description": "Path to WASM binary"
                    },
                    "source_file": {
                        "type": "string",
                        "description": "Source file to query mappings for (optional)"
                    },
                    "function_name": {
                        "type": "string",
                        "description": "Function name to query (optional)"
                    },
                    "line": {
                        "type": "integer",
                        "description": "Source line number (optional)"
                    }
                },
                "required": ["wasm_path"]
            }),
        }
    }

    async fn execute(&self, _params: Value) -> Result<Value, McpError> {
        Ok(json!({
            "type": "text",
            "text": "Mapping query not yet implemented - coming in Phase 2"
        }))
    }
}

/// Trace execution through pipeline layers
pub struct DeepWasmTraceExecutionTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmTraceExecutionTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for DeepWasmTraceExecutionTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "deep_wasm_trace_execution".to_string(),
            description: "Trace execution flow through Source → WASM → JS layers".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "wasm_path": {
                        "type": "string",
                        "description": "Path to WASM binary"
                    },
                    "entry_point": {
                        "type": "string",
                        "description": "Entry point function name"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "Maximum trace depth",
                        "default": 100
                    }
                },
                "required": ["wasm_path", "entry_point"]
            }),
        }
    }

    async fn execute(&self, _params: Value) -> Result<Value, McpError> {
        Ok(json!({
            "type": "text",
            "text": "Execution tracing not yet implemented - coming in Phase 3"
        }))
    }
}

/// Compare optimization levels
pub struct DeepWasmCompareOptimizationsTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmCompareOptimizationsTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for DeepWasmCompareOptimizationsTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "deep_wasm_compare_optimizations".to_string(),
            description: "Compare WASM binaries at different optimization levels".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "wasm_paths": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Paths to WASM binaries to compare",
                        "minItems": 2
                    },
                    "metrics": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["size", "complexity", "performance", "security"]
                        },
                        "description": "Metrics to compare"
                    }
                },
                "required": ["wasm_paths"]
            }),
        }
    }

    async fn execute(&self, _params: Value) -> Result<Value, McpError> {
        Ok(json!({
            "type": "text",
            "text": "Optimization comparison not yet implemented - coming in Phase 2"
        }))
    }
}

/// Detect WASM-specific issues
pub struct DeepWasmDetectIssuesTool {
    _registry: Arc<AgentRegistry>,
}

impl DeepWasmDetectIssuesTool {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }
}

#[async_trait]
impl McpTool for DeepWasmDetectIssuesTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "deep_wasm_detect_issues".to_string(),
            description: "Detect WASM-specific quality issues and anti-patterns".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "wasm_path": {
                        "type": "string",
                        "description": "Path to WASM binary"
                    },
                    "issue_types": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": [
                                "unreachable_code",
                                "unbounded_loop",
                                "stack_overflow",
                                "memory_leak",
                                "undefined_behavior",
                                "type_unsafety"
                            ]
                        },
                        "description": "Types of issues to detect"
                    },
                    "zero_tolerance": {
                        "type": "boolean",
                        "description": "Fail on any issue found",
                        "default": true
                    }
                },
                "required": ["wasm_path"]
            }),
        }
    }

    async fn execute(&self, _params: Value) -> Result<Value, McpError> {
        Ok(json!({
            "type": "text",
            "text": "Issue detection not yet implemented - coming in Phase 2"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_wasm_analyze_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = DeepWasmAnalyzeTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "deep_wasm_analyze");
        assert!(metadata.description.contains("pipeline"));
    }

    #[test]
    fn test_deep_wasm_query_mapping_metadata() {
        let registry = Arc::new(AgentRegistry::new());
        let tool = DeepWasmQueryMappingTool::new(registry);
        let metadata = tool.metadata();

        assert_eq!(metadata.name, "deep_wasm_query_mapping");
        assert!(metadata.description.contains("bidirectional"));
    }

    #[test]
    fn test_all_tools_have_schemas() {
        let registry = Arc::new(AgentRegistry::new());

        let tools: Vec<Box<dyn McpTool>> = vec![
            Box::new(DeepWasmAnalyzeTool::new(registry.clone())),
            Box::new(DeepWasmQueryMappingTool::new(registry.clone())),
            Box::new(DeepWasmTraceExecutionTool::new(registry.clone())),
            Box::new(DeepWasmCompareOptimizationsTool::new(registry.clone())),
            Box::new(DeepWasmDetectIssuesTool::new(registry)),
        ];

        for tool in tools {
            let metadata = tool.metadata();
            assert!(!metadata.name.is_empty());
            assert!(!metadata.description.is_empty());
            assert!(metadata.input_schema.is_object());
        }
    }
}
