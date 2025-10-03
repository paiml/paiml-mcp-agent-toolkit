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
            let gates = WasmQualityGates {
                max_module_size: 5_242_880, // 5MB strict limit
                max_wasm_complexity: 15,
                min_source_map_coverage: 0.99,
                ..Default::default()
            };
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
                "correlation_count": report.correlations.len(),
                "has_dwarf": report.wasm_module_analysis.has_dwarf,
                "has_source_map": report.wasm_module_analysis.has_source_map,
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
                    "dwarf_path": {
                        "type": "string",
                        "description": "Path to DWARF debug symbols (optional)"
                    },
                    "source_map_path": {
                        "type": "string",
                        "description": "Path to source map file (optional)"
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

    #[cfg(feature = "deep-wasm")]
    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        use std::path::PathBuf;

        let wasm_path = params["wasm_path"]
            .as_str()
            .ok_or_else(|| McpError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing wasm_path parameter".to_string(),
                data: None,
            })?
            .to_string();

        // Derive source path from wasm path (assume .rs or .ruchy in same directory)
        let source_path = PathBuf::from(&wasm_path)
            .with_extension("rs");

        let dwarf_path = params["dwarf_path"].as_str().map(PathBuf::from);
        let source_map_path = params["source_map_path"].as_str().map(PathBuf::from);

        let language = if source_path.ends_with(".rs") {
            SourceLanguage::Rust
        } else {
            SourceLanguage::Ruchy
        };

        let request = DeepWasmAnalysisRequest {
            source_path,
            wasm_path: Some(PathBuf::from(&wasm_path)),
            dwarf_path,
            source_map_path,
            language,
            analysis_focus: AnalysisFocus::Full,
        };

        let service = DeepWasmService::new();
        let report = service.analyze(request).await.map_err(|e| McpError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Analysis failed: {}", e),
            data: None,
        })?;

        // Filter correlations based on query parameters
        let mut filtered_correlations = report.correlations.clone();

        if let Some(source_file) = params["source_file"].as_str() {
            filtered_correlations.retain(|m| {
                m.source_file
                    .file_name()
                    .and_then(|f| f.to_str())
                    .map(|f| f == source_file)
                    .unwrap_or(false)
            });
        }

        if let Some(function_name) = params["function_name"].as_str() {
            filtered_correlations.retain(|m| {
                m.dwarf_die
                    .as_ref()
                    .and_then(|die| die.name.as_deref())
                    .map(|name| name == function_name)
                    .unwrap_or(false)
                    || m.source_map_entry
                        .as_ref()
                        .and_then(|e| e.name.as_deref())
                        .map(|name| name == function_name)
                        .unwrap_or(false)
            });
        }

        if let Some(line) = params["line"].as_i64() {
            let target_line = line as u32;
            filtered_correlations.retain(|m| m.source_location.line == target_line);
        }

        // Build response
        let mut output = String::new();
        output.push_str(&format!(
            "# Source-to-WASM Mappings\n\nFound {} mappings",
            filtered_correlations.len()
        ));

        if filtered_correlations.is_empty() {
            output.push_str("\n\nNo correlations found matching the query criteria.");
        } else {
            output.push_str(":\n\n");
            output.push_str("| Source | Line:Col | WASM Fn | Confidence | Source |\n");
            output.push_str("|--------|----------|---------|------------|--------|\n");

            for mapping in filtered_correlations.iter().take(50) {
                let source_file = mapping
                    .source_file
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let source_type = if mapping.dwarf_die.is_some() && mapping.source_map_entry.is_some() {
                    "Both"
                } else if mapping.dwarf_die.is_some() {
                    "DWARF"
                } else {
                    "SourceMap"
                };
                output.push_str(&format!(
                    "| {} | {}:{} | {} | {:.0}% | {} |\n",
                    source_file,
                    mapping.source_location.line,
                    mapping.source_location.column,
                    mapping.wasm_function_idx,
                    mapping.confidence * 100.0,
                    source_type
                ));
            }

            if filtered_correlations.len() > 50 {
                output.push_str(&format!(
                    "\n\n... and {} more mappings (showing top 50)\n",
                    filtered_correlations.len() - 50
                ));
            }

            // Statistics
            let avg_confidence = filtered_correlations
                .iter()
                .map(|m| m.confidence)
                .sum::<f64>()
                / filtered_correlations.len() as f64;

            output.push_str(&format!(
                "\n\n## Statistics\n\n- Average Confidence: {:.1}%\n",
                avg_confidence * 100.0
            ));
        }

        Ok(json!({
            "type": "text",
            "text": output,
            "metadata": {
                "total_correlations": report.correlations.len(),
                "filtered_correlations": filtered_correlations.len(),
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
