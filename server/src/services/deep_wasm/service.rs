//! Deep WASM Service
//!
//! Main service that coordinates all deep WASM inspection components.

use crate::services::deep_wasm::{
    CorrelationEngine, DeepWasmAnalysisRequest, DeepWasmReport, DeepWasmResult, DwarfParser,
    PipelineOverview, SourceLanguage, SourceMapHandler, SourceMetrics,
    WasmInspector, WasmModuleAnalysis, WasmQualityGates,
};
use crate::services::rust_wasm_analyzer;
use std::fs;
use std::path::Path;

/// Deep WASM analysis service
pub struct DeepWasmService {
    wasm_inspector: WasmInspector,
    dwarf_parser: DwarfParser,
    source_map_handler: SourceMapHandler,
    correlation_engine: CorrelationEngine,
    quality_gates: WasmQualityGates,
}

impl DeepWasmService {
    pub fn new() -> Self {
        Self {
            wasm_inspector: WasmInspector::new(),
            dwarf_parser: DwarfParser::new(),
            source_map_handler: SourceMapHandler::new(),
            correlation_engine: CorrelationEngine::new(),
            quality_gates: WasmQualityGates::new(),
        }
    }

    pub fn with_quality_gates(mut self, gates: WasmQualityGates) -> Self {
        self.quality_gates = gates;
        self
    }

    pub async fn analyze(&self, request: DeepWasmAnalysisRequest) -> DeepWasmResult<DeepWasmReport> {
        // Analyze WASM binary if provided
        let wasm_analysis = if let Some(ref wasm_path) = request.wasm_path {
            self.wasm_inspector.inspect_file(wasm_path)?
        } else {
            WasmModuleAnalysis {
                module_size_bytes: 0,
                function_count: 0,
                exported_functions: 0,
                max_complexity: 0,
                has_dwarf: false,
                has_source_map: false,
            }
        };

        // Analyze source code for WASM constructs
        let source_metrics = self.analyze_source_code(&request.source_path, request.language.clone())?;

        // Parse DWARF debug information if provided
        let dwarf_entries = if let Some(ref dwarf_path) = request.dwarf_path {
            let dwarf_data = fs::read(dwarf_path)
                .map_err(crate::services::deep_wasm::DeepWasmError::Io)?;
            self.dwarf_parser.parse_dwarf_sections(&dwarf_data, None, None)?
        } else {
            vec![]
        };

        // Parse source map if provided
        let source_map_entries = if let Some(ref source_map_path) = request.source_map_path {
            self.source_map_handler.parse_source_map(source_map_path)?
        } else {
            vec![]
        };

        // Create source-to-WASM correlations
        let correlations = if !dwarf_entries.is_empty() || !source_map_entries.is_empty() {
            self.correlation_engine.correlate(&dwarf_entries, &source_map_entries)?
        } else {
            vec![]
        };

        let quality_results = self.quality_gates.evaluate(&wasm_analysis)?;

        Ok(DeepWasmReport {
            project_name: request
                .source_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            pipeline_overview: PipelineOverview {
                source_language: request.language,
                source_version: String::new(),
                target: "wasm32-unknown-unknown".to_string(),
                optimization_level: String::new(),
                debug_symbols: Some(if dwarf_entries.is_empty() {
                    "none".to_string()
                } else {
                    format!("{} DWARF entries", dwarf_entries.len())
                }),
            },
            source_metrics,
            wasm_module_analysis: wasm_analysis,
            correlations,
            type_flows: vec![],
            hotspots: vec![],
            quality_gate_results: quality_results,
        })
    }

    /// Analyze source code for WASM-specific constructs
    fn analyze_source_code(&self, source_path: &Path, language: SourceLanguage) -> DeepWasmResult<SourceMetrics> {
        use crate::services::deep_wasm::DeepWasmError;

        // Read source file
        let source_code = fs::read_to_string(source_path)
            .map_err(DeepWasmError::Io)?;

        // Count lines of code
        let lines_of_code = source_code.lines().count();

        match language {
            SourceLanguage::Rust => {
                // Parse as Rust code
                let syntax_tree = syn::parse_file(&source_code)
                    .map_err(|e| DeepWasmError::Analysis(format!("Failed to parse Rust source: {}", e)))?;

                // Analyze WASM constructs
                let wasm_analysis = rust_wasm_analyzer::analyze_wasm_constructs(&syntax_tree);

                // Calculate total function count (including non-WASM functions)
                let mut total_functions = 0;
                let mut max_complexity = 0;

                for item in &syntax_tree.items {
                    match item {
                        syn::Item::Fn(_) => total_functions += 1,
                        syn::Item::Impl(impl_block) => {
                            for impl_item in &impl_block.items {
                                if let syn::ImplItem::Fn(_) = impl_item {
                                    total_functions += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Estimate max complexity from WASM boundary functions
                // (For Phase 1, this is a simplified estimation)
                for boundary_fn in &wasm_analysis.boundary_functions {
                    let complexity_estimate = if boundary_fn.has_unsafe { 5 } else { 3 };
                    max_complexity = max_complexity.max(complexity_estimate);
                }

                Ok(SourceMetrics {
                    lines_of_code,
                    function_count: total_functions,
                    max_complexity,
                    wasm_boundary_functions: wasm_analysis.boundary_functions.len(),
                })
            }
            SourceLanguage::Ruchy => {
                // For Ruchy, use simplified analysis (Phase 1)
                // Count functions using simple pattern matching
                let mut total_functions = 0;
                let mut max_complexity = 0;

                for line in source_code.lines() {
                    let trimmed = line.trim();

                    // Detect function declarations: "fun name(...)" or "async fun name(...)"
                    if (trimmed.starts_with("fun ") || trimmed.starts_with("async fun ")) && trimmed.contains('(') {
                        total_functions += 1;

                        // Simple complexity heuristic
                        // Functions with "unsafe", "async", or complex patterns get higher complexity
                        if trimmed.contains("unsafe") || trimmed.contains("async") {
                            max_complexity = max_complexity.max(5);
                        } else {
                            max_complexity = max_complexity.max(3);
                        }
                    }
                }

                Ok(SourceMetrics {
                    lines_of_code,
                    function_count: total_functions,
                    max_complexity,
                    wasm_boundary_functions: 0, // Ruchy Phase 1: no WASM-specific analysis yet
                })
            }
        }
    }
}

impl Default for DeepWasmService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::deep_wasm::{AnalysisFocus, SourceLanguage};
    use std::path::PathBuf;

    #[test]
    fn test_service_creation() {
        let _service = DeepWasmService::new();
    }

    #[test]
    fn test_service_with_custom_gates() {
        let gates = WasmQualityGates::new();
        let _service = DeepWasmService::new().with_quality_gates(gates);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_analyze_minimal_request() {
        let service = DeepWasmService::new();
        let request = DeepWasmAnalysisRequest {
            source_path: PathBuf::from("tests/fixtures/test.rs"),
            wasm_path: None,
            dwarf_path: None,
            source_map_path: None,
            language: SourceLanguage::Rust,
            analysis_focus: AnalysisFocus::Full,
        };

        let result = service.analyze(request).await;
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.project_name.is_empty());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_analyze_ruchy_file() {
        let service = DeepWasmService::new();
        let request = DeepWasmAnalysisRequest {
            source_path: PathBuf::from("tests/fixtures/deep_wasm_ruchy_test.ruchy"),
            wasm_path: None,
            dwarf_path: None,
            source_map_path: None,
            language: SourceLanguage::Ruchy,
            analysis_focus: AnalysisFocus::Source,
        };

        let result = service.analyze(request).await;
        assert!(result.is_ok());
        let report = result.unwrap();

        // Verify report has correct language
        assert_eq!(report.pipeline_overview.source_language, SourceLanguage::Ruchy);

        // Verify source metrics were calculated
        assert!(report.source_metrics.lines_of_code > 0);
        assert_eq!(report.source_metrics.function_count, 3); // fibonacci, factorial, fetch_data
        assert!(report.source_metrics.max_complexity > 0);

        // Phase 1: No WASM boundary analysis for Ruchy yet
        assert_eq!(report.source_metrics.wasm_boundary_functions, 0);
    }
}
