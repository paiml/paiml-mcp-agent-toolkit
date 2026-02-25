// DeepWasmQueryMappingTool - McpTool implementation for source-to-WASM mapping queries

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
        let source_path = PathBuf::from(&wasm_path).with_extension("rs");

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
                let source_type =
                    if mapping.dwarf_die.is_some() && mapping.source_map_entry.is_some() {
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
            message: "Deep WASM feature not enabled. Recompile with --features deep-wasm"
                .to_string(),
            data: None,
        })
    }
}
