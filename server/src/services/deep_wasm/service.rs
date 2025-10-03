//! Deep WASM Service
//!
//! Main service that coordinates all deep WASM inspection components.

use crate::services::deep_wasm::{
    CorrelationEngine, DeepWasmAnalysisRequest, DeepWasmReport, DeepWasmResult, DwarfParser,
    PipelineOverview, ReportGenerator, SourceMapHandler, SourceMetrics,
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
    report_generator: ReportGenerator,
}

impl DeepWasmService {
    pub fn new() -> Self {
        Self {
            wasm_inspector: WasmInspector::new(),
            dwarf_parser: DwarfParser::new(),
            source_map_handler: SourceMapHandler::new(),
            correlation_engine: CorrelationEngine::new(),
            quality_gates: WasmQualityGates::new(),
            report_generator: ReportGenerator::new(),
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

        // Analyze source code for WASM constructs (Rust only for Phase 1)
        let source_metrics = self.analyze_source_code(&request.source_path)?;

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
                debug_symbols: None,
            },
            source_metrics,
            wasm_module_analysis: wasm_analysis,
            correlations: vec![],
            type_flows: vec![],
            hotspots: vec![],
            quality_gate_results: quality_results,
        })
    }

    /// Analyze source code for WASM-specific constructs
    fn analyze_source_code(&self, source_path: &Path) -> DeepWasmResult<SourceMetrics> {
        use crate::services::deep_wasm::DeepWasmError;

        // Read source file
        let source_code = fs::read_to_string(source_path)
            .map_err(|e| DeepWasmError::Io(e))?;

        // Count lines of code
        let lines_of_code = source_code.lines().count();

        // Parse as Rust code (for Phase 1, only Rust is supported)
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
    async fn test_analyze_minimal_request() {
        let service = DeepWasmService::new();
        let request = DeepWasmAnalysisRequest {
            source_path: PathBuf::from("test.rs"),
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
}
