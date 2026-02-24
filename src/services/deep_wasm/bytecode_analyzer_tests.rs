#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = BytecodeAnalyzer::new();
        assert!(analyzer.deep_analysis);
    }

    #[test]
    fn test_analyzer_shallow() {
        let analyzer = BytecodeAnalyzer::with_deep_analysis(false);
        assert!(!analyzer.deep_analysis);
    }

    #[test]
    fn test_valtype_conversion() {
        assert_eq!(valtype_to_string(&ValType::I32), "i32");
        assert_eq!(valtype_to_string(&ValType::I64), "i64");
        assert_eq!(valtype_to_string(&ValType::F32), "f32");
        assert_eq!(valtype_to_string(&ValType::F64), "f64");
    }

    #[test]
    #[ignore] // Phase 1: Basic framework only. Function-level analysis requires DWARF v5 integration (Phase 2).
              // Currently analyzer returns empty functions vec. Will be enabled after Phase 2 completion.
              // See: docs/execution/SPRINT-41-QUALITY-REMEDIATION.md (Deep WASM Phase 2)
    fn test_analyze_minimal_wasm() {
        // Minimal valid WASM module with one function
        let minimal_wasm = vec![
            0x00, 0x61, 0x73, 0x6D, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version
            // Type section: one function type () -> ()
            0x01, 0x05, 0x01, 0x60, 0x00, 0x00, // Function section: one function
            0x03, 0x02, 0x01, 0x00, // Code section: one function with just 'end'
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let analyzer = BytecodeAnalyzer::new();
        let result = analyzer.analyze(&minimal_wasm);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.functions.len(), 1);
        assert_eq!(analysis.module_stats.total_functions, 1);
    }
}
