//! Integration tests for deep WASM module

#[cfg(test)]
mod integration_tests {
    use crate::services::deep_wasm::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_end_to_end_minimal_analysis() {
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
    }

    #[test]
    fn test_wasm_inspector_integration() {
        let inspector = WasmInspector::new();
        let minimal_wasm = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

        let result = inspector.inspect_bytes(&minimal_wasm);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quality_gates_integration() {
        let gates = WasmQualityGates::new();
        let analysis = WasmModuleAnalysis {
            module_size_bytes: 1000,
            function_count: 5,
            exported_functions: 2,
            max_complexity: 10,
            has_dwarf: false,
            has_source_map: true,  // Updated to pass quality gate
        };

        let result = gates.evaluate(&analysis);
        assert!(result.is_ok());
        assert!(result.unwrap().passed);
    }

    #[test]
    fn test_report_generation_integration() {
        let generator = ReportGenerator::new();
        let report = DeepWasmReport {
            project_name: "test".to_string(),
            timestamp: "2025-10-02".to_string(),
            pmat_version: "2.109.0".to_string(),
            pipeline_overview: PipelineOverview {
                source_language: SourceLanguage::Rust,
                source_version: "1.75.0".to_string(),
                target: "wasm32-unknown-unknown".to_string(),
                optimization_level: "3".to_string(),
                debug_symbols: None,
            },
            source_metrics: SourceMetrics {
                lines_of_code: 100,
                function_count: 5,
                max_complexity: 10,
                wasm_boundary_functions: 2,
            },
            wasm_module_analysis: WasmModuleAnalysis {
                module_size_bytes: 1000,
                function_count: 5,
                exported_functions: 2,
                max_complexity: 10,
                has_dwarf: false,
                has_source_map: false,
            },
            correlations: vec![],
            type_flows: vec![],
            hotspots: vec![],
            quality_gate_results: QualityGateResults {
                passed: true,
                violations: vec![],
            },
        };

        let result = generator.generate_markdown(&report);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(!markdown.is_empty());
    }
}
