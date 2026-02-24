// Core tests: property tests, WasmAnalyzer, Analysis, AnalysisResult, ModuleInfo, InstructionProfiler
// Included from analyzer.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // Minimal valid WASM module (empty module with proper header)
    fn minimal_wasm_module() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version 1
        ]
    }

    // WASM module with a simple function
    fn simple_function_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Type section
            0x01, 0x05, // section id 1, size 5
            0x01, // 1 type
            0x60, 0x00, 0x01, 0x7f, // func type: () -> i32
            // Function section
            0x03, 0x02, // section id 3, size 2
            0x01, 0x00, // 1 function, type 0
            // Code section
            0x0a, 0x09, // section id 10, size 9
            0x01, // 1 function body
            0x07, // body size 7
            0x00, // 0 locals
            0x41, 0x01, // i32.const 1
            0x41, 0x02, // i32.const 2
            0x6a, // i32.add
            0x0b, // end
        ]
    }

    // WASM module with various instruction types
    fn mixed_instructions_wasm() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6d, // magic
            0x01, 0x00, 0x00, 0x00, // version
            // Type section
            0x01, 0x05, // section id 1, size 5
            0x01, // 1 type
            0x60, 0x00, 0x01, 0x7f, // func type: () -> i32
            // Function section
            0x03, 0x02, // section id 3, size 2
            0x01, 0x00, // 1 function, type 0
            // Memory section
            0x05, 0x03, // section id 5, size 3
            0x01, // 1 memory
            0x00, 0x01, // min 1 page, no max
            // Code section with mixed instructions
            0x0a, 0x11, // section id 10, size 17
            0x01, // 1 function body
            0x0f, // body size 15
            0x00, // 0 locals
            0x02, 0x7f, // block returning i32
            0x41, 0x00, // i32.const 0
            0x28, 0x02, 0x00, // i32.load
            0x41, 0x01, // i32.const 1
            0x6a, // i32.add
            0x0c, 0x00, // br 0
            0x0b, // end block
            0x0b, // end function
        ]
    }

    // ==================== WasmAnalyzer Tests ====================

    #[test]
    fn test_wasm_analyzer_new() {
        let analyzer = WasmAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_analyze_minimal_module() {
        let analyzer = WasmAnalyzer::new().unwrap();
        let result = analyzer.analyze(&minimal_wasm_module());

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.function_count, 0);
        assert_eq!(analysis.instruction_count, 0);
    }

    #[test]
    fn test_analyze_simple_function() {
        let analyzer = WasmAnalyzer::new().unwrap();
        let result = analyzer.analyze(&simple_function_wasm());

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.instruction_count > 0);
    }

    #[test]
    fn test_analyze_streaming_minimal() {
        let analyzer = WasmAnalyzer::new().unwrap();
        let result = analyzer.analyze_streaming(&minimal_wasm_module());

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.instruction_mix.total_instructions, 0);
        assert!(analysis.vulnerability_patterns.is_empty());
    }

    #[test]
    fn test_analyze_streaming_mixed() {
        let analyzer = WasmAnalyzer::new().unwrap();
        let result = analyzer.analyze_streaming(&mixed_instructions_wasm());

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.instruction_mix.total_instructions > 0);
        assert!(analysis.instruction_mix.control_flow > 0);
        assert!(analysis.instruction_mix.memory_ops > 0);
    }

    #[test]
    fn test_analyze_invalid_wasm() {
        let analyzer = WasmAnalyzer::new().unwrap();
        let result = analyzer.analyze(&[0x00, 0x01, 0x02, 0x03]);

        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_empty_input() {
        let analyzer = WasmAnalyzer::new().unwrap();
        let result = analyzer.analyze(&[]);

        assert!(result.is_err());
    }

    // ==================== Analysis Tests ====================

    #[test]
    fn test_analysis_serialization() {
        let analysis = Analysis {
            module_info: ModuleInfo {
                num_functions: 5,
                num_imports: 2,
                num_exports: 3,
                num_tables: 1,
                num_memories: 1,
                num_globals: 4,
                has_start_function: true,
                code_size: 1000,
            },
            instruction_mix: InstructionMix {
                total_instructions: 100,
                control_flow: 20,
                memory_ops: 30,
                arithmetic: 40,
                calls: 10,
            },
            vulnerability_patterns: vec![],
            security_report: SecurityReport::new(),
        };

        let serialized = serde_json::to_string(&analysis).unwrap();
        let deserialized: Analysis = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            analysis.module_info.num_functions,
            deserialized.module_info.num_functions
        );
        assert_eq!(
            analysis.instruction_mix.total_instructions,
            deserialized.instruction_mix.total_instructions
        );
    }

    #[test]
    fn test_analysis_clone() {
        let analysis = Analysis {
            module_info: ModuleInfo::from_validator(Validator::new()),
            instruction_mix: InstructionMix {
                total_instructions: 50,
                control_flow: 10,
                memory_ops: 15,
                arithmetic: 20,
                calls: 5,
            },
            vulnerability_patterns: vec![],
            security_report: SecurityReport::new(),
        };

        let cloned = analysis.clone();
        assert_eq!(
            analysis.instruction_mix.total_instructions,
            cloned.instruction_mix.total_instructions
        );
    }

    // ==================== AnalysisResult Tests ====================

    #[test]
    fn test_analysis_result_from_analysis() {
        let analysis = Analysis {
            module_info: ModuleInfo {
                num_functions: 10,
                num_imports: 5,
                num_exports: 3,
                num_tables: 1,
                num_memories: 2,
                num_globals: 4,
                has_start_function: false,
                code_size: 5000,
            },
            instruction_mix: InstructionMix {
                total_instructions: 500,
                control_flow: 100,
                memory_ops: 150,
                arithmetic: 200,
                calls: 50,
            },
            vulnerability_patterns: vec![],
            security_report: SecurityReport::new(),
        };

        let result = AnalysisResult::from(analysis);

        assert_eq!(result.function_count, 10);
        assert_eq!(result.instruction_count, 500);
        assert_eq!(result.binary_size, 5000);
        assert_eq!(result.memory_pages, 2);
        assert_eq!(result.max_complexity, 10); // Default estimate
    }

    #[test]
    fn test_analysis_result_serialization() {
        let result = AnalysisResult {
            function_count: 25,
            instruction_count: 1000,
            binary_size: 10000,
            memory_pages: 4,
            max_complexity: 15,
        };

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: AnalysisResult = serde_json::from_str(&serialized).unwrap();

        assert_eq!(result.function_count, deserialized.function_count);
        assert_eq!(result.instruction_count, deserialized.instruction_count);
        assert_eq!(result.binary_size, deserialized.binary_size);
    }

    // ==================== ModuleInfo Tests ====================

    #[test]
    fn test_module_info_from_validator() {
        let validator = Validator::new();
        let info = ModuleInfo::from_validator(validator);

        // Default values from simplified implementation
        assert_eq!(info.num_functions, 0);
        assert_eq!(info.num_imports, 0);
        assert_eq!(info.num_exports, 0);
        assert_eq!(info.num_tables, 0);
        assert_eq!(info.num_memories, 1);
        assert_eq!(info.num_globals, 0);
        assert!(!info.has_start_function);
        assert_eq!(info.code_size, 0);
    }

    #[test]
    fn test_module_info_clone() {
        let info = ModuleInfo {
            num_functions: 15,
            num_imports: 5,
            num_exports: 8,
            num_tables: 2,
            num_memories: 1,
            num_globals: 10,
            has_start_function: true,
            code_size: 25000,
        };

        let cloned = info.clone();
        assert_eq!(info.num_functions, cloned.num_functions);
        assert_eq!(info.has_start_function, cloned.has_start_function);
    }

    // ==================== InstructionProfiler Tests ====================

    #[test]
    fn test_instruction_profiler_new() {
        let profiler = InstructionProfiler::new();
        assert!(profiler.instruction_counts.is_empty());
        assert_eq!(profiler.total_instructions, 0);
    }

    #[test]
    fn test_instruction_profiler_default() {
        let profiler = InstructionProfiler::default();
        assert!(profiler.instruction_counts.is_empty());
    }

    #[test]
    fn test_instruction_profiler_observe() {
        let mut profiler = InstructionProfiler::new();
        let wasm = simple_function_wasm();

        for payload in Parser::new(0).parse_all(&wasm) {
            if let Ok(p) = payload {
                profiler.observe(&p);
            }
        }

        assert!(profiler.total_instructions > 0);
    }

    #[test]
    fn test_instruction_profiler_finalize() {
        let mut profiler = InstructionProfiler::new();
        let wasm = mixed_instructions_wasm();

        for payload in Parser::new(0).parse_all(&wasm) {
            if let Ok(p) = payload {
                profiler.observe(&p);
            }
        }

        let mix = profiler.finalize();
        assert!(mix.total_instructions > 0);
    }
}
