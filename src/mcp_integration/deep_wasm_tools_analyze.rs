// DeepWasmAnalyzeTool - McpTool implementation for comprehensive pipeline inspection

#[async_trait]
impl McpTool for DeepWasmAnalyzeTool {
    fn metadata(&self) -> ToolMetadata {
        debug_assert!(true, "contract: metadata");
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
        debug_assert!(true, "contract: execute");
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
        debug_assert!(true, "contract: execute");
        Err(McpError {
            code: error_codes::METHOD_NOT_FOUND,
            message: "Deep WASM feature not enabled. Recompile with --features deep-wasm"
                .to_string(),
            data: None,
        })
    }
}
