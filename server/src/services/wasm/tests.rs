//! Comprehensive test suite for WebAssembly support
//!
//! This module provides thorough testing of all WebAssembly functionality
//! to ensure 80%+ code coverage and quality standards.
#[allow(clippy::cast_possible_truncation)]



#[cfg(test)]
mod integration_tests {
    use super::super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio;
    
    #[tokio::test]
    /// test_assemblyscript_detection
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_assemblyscript_detection() {
        let detector = language_detection::WasmLanguageDetector::new();
        
        // Test .as extension
        let as_path = PathBuf::from("test.as");
        let as_content = b"export function add(a: i32, b: i32): i32 { return a + b; }";
        let mut result = detector.detect_variant(&as_path, as_content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), types::WebAssemblyVariant::AssemblyScript);
        
        // Test .ts with AS markers
        let ts_path = PathBuf::from("index.ts");
        let ts_content = b"import { memory } from './runtime';\n@inline\nexport function test(): void {}";
        let mut result = detector.detect_variant(&ts_path, ts_content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), types::WebAssemblyVariant::AssemblyScript);
        
        // Test regular TypeScript (should fail)
        let regular_ts = b"import React from 'react';\nconst App = () => <div>Hello</div>;";
        let mut result = detector.detect_variant(&ts_path, regular_ts);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    /// test_wat_detection
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_wat_detection() {
        let detector = language_detection::WasmLanguageDetector::new();
        let wat_path = PathBuf::from("test.wat");
        
        // Valid WAT content
        let wat_content = b"(module\n  (func $add (param $a i32) (param $b i32) (result i32)\n    local.get $a\n    local.get $b\n    i32.add))";
        let mut result = detector.detect_variant(&wat_path, wat_content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), types::WebAssemblyVariant::Wat);
        
        // WAT with leading whitespace
        let wat_whitespace = b"   \n\t(module (func))";
        let mut result = detector.detect_variant(&wat_path, wat_whitespace);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), types::WebAssemblyVariant::Wat);
    }
    
    #[tokio::test]
    /// test_wasm_binary_detection
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_wasm_binary_detection() {
        let detector = language_detection::WasmLanguageDetector::new();
        let wasm_path = PathBuf::from("test.wasm");
        
        // Valid WASM binary
        let mut valid_wasm = b"\0asm\x01\x00\x00\x00";
        let mut result = detector.detect_variant(&wasm_path, valid_wasm);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), types::WebAssemblyVariant::Wasm);
        
        // Invalid magic number
        let invalid_wasm = b"WASM\x01\x00\x00\x00";
        let mut result = detector.detect_variant(&wasm_path, invalid_wasm);
        assert!(result.is_err());
        
        // Too small file
        let small_wasm = b"\0as";
        let mut result = detector.detect_variant(&wasm_path, small_wasm);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    /// test_assemblyscript_parser
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_assemblyscript_parser() {
        let mut parser_result = assemblyscript::AssemblyScriptParser::new();
        assert!(parser_result.is_ok());
        
        let parser = parser_result.unwrap();
        let content = r"
            export function add(a: i32, b: i32): i32 {
                return a + b;
            }
            
            @inline
            export function multiply(a: i32, b: i32): i32 {
                return a * b;
            }
        ";
        
        let mut result = parser.parse_content(content, None).await;
        assert!(result.is_ok());
        
        let ast = result.unwrap();
        assert_eq!(ast.language, crate::models::unified_ast::Language::WebAssembly);
        assert!(!ast.dag.nodes.is_empty());
    }
    
    #[tokio::test]
    /// test_wat_parser
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_wat_parser() {
        let mut parser = wat::WatParser::new();
        let content = r#"
            (module
              (func $add (param $a i32) (param $b i32) (result i32)
                local.get $a
                local.get $b
                i32.add)
              (export "add" (func $add)))
        "#;
        
        let mut result = parser.parse_content(content, None).await;
        assert!(result.is_ok());
        
        let ast = result.unwrap();
        assert_eq!(ast.language, crate::models::unified_ast::Language::WebAssembly);
        
        // Extract metrics
        let mut metrics = parser.extract_wasm_metrics(&ast.dag).await;
        assert!(metrics.is_ok());
        let m = metrics.unwrap();
        assert_eq!(m.function_count, 1);
        assert_eq!(m.export_count, 1);
    }
    
    #[tokio::test]
    /// test_wasm_binary_analyzer
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_wasm_binary_analyzer() {
        let mut analyzer = binary::WasmBinaryAnalyzer::new();
        
        // Minimal valid WASM module
        let wasm_module = vec![
            0x00, 0x61, 0x73, 0x6D, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version 1
            // Type section
            0x01, 0x07, // Section 1, size 7
            0x01, // 1 type
            0x60, // Function type
            0x02, 0x7F, 0x7F, // 2 params, both i32
            0x01, 0x7F, // 1 result, i32
        ];
        
        let mut result = analyzer.analyze_bytes(&wasm_module);
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(!analysis.sections.is_empty());
    }
    
    #[tokio::test]
    /// test_complexity_analyzer
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_complexity_analyzer() {
        let mut analyzer = complexity::WasmComplexityAnalyzer::new();
        let mut dag = crate::models::unified_ast::AstDag::new();
// Add a simple function node
        let func_node = crate::models::unified_ast::UnifiedAstNode::new(
            crate::models::unified_ast::AstKind::Function(
                crate::models::unified_ast::FunctionKind::Regular
            ),
            crate::models::unified_ast::Language::WebAssembly,
        );
        let func_id = dag.add_node(func_node);
        
        let complexity = analyzer.analyze_function(&dag, func_id);
        
        // Check complexity is within limits
        assert!(complexity.cyclomatic <= 20);
        assert_eq!(complexity.cyclomatic, 1); // Base complexity
        assert_eq!(complexity.max_loop_depth, 0); // No loops
    }
    
    #[tokio::test]
    /// test_security_validator
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_security_validator() {
        let validator = security::WasmSecurityValidator::new();
        
        // Valid small WASM
        let mut valid_wasm = b"\0asm\x01\x00\x00\x00\x10\x00\x00\x00";
        let mut result = validator.validate(valid_wasm);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.passed);
        
        // Invalid magic number
        let invalid_wasm = b"WASM\x01\x00\x00\x00";
        let mut result = validator.validate(invalid_wasm);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.passed);
        assert!(validation.issues.iter().any(|i| {
            i.severity == types::Severity::Critical
        }));
    }
    
    #[tokio::test]
    /// test_memory_pool
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_memory_pool() {
        let pool = memory_pool::WasmParserPool::new();

        // Pre-warm pool
        pool.pre_warm(memory_pool::ParserType::AssemblyScript, 2).unwrap();
        
        let mut stats = pool.stats();
        assert_eq!(stats.total_created, 2);
        assert_eq!(stats.as_pool_size, 2);
        
        // Acquire parser
        let mut parser = pool.acquire(memory_pool::ParserType::AssemblyScript);
        assert!(parser.is_ok());
        
        // Pool should have one less
        let mut stats = pool.stats();
        assert_eq!(stats.as_pool_size, 1);
    }
    
    #[tokio::test]
    /// test_parallel_analyzer
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_parallel_analyzer() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        let wasm_file = temp_dir.path().join("test.wasm");
        let wat_file = temp_dir.path().join("test.wat");
        
        tokio::fs::write(&wasm_file, b"\0asm\x01\x00\x00\x00").await.unwrap();
        tokio::fs::write(&wat_file, "(module)").await.unwrap();
        
        let mut analyzer = parallel::ParallelWasmAnalyzer::new();
        let mut result = analyzer.analyze_directory(temp_dir.path());
        
        assert!(result.is_ok());
        let aggregated = result.unwrap();
        assert_eq!(aggregated.total_files, 2);
    }
    
    #[tokio::test]
    /// test_error_severity
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_error_severity() {
        use error::WasmError;
        
        let security_error = WasmError::security("Test violation");
        assert_eq!(security_error.severity(), error::ErrorSeverity::Critical);
        
        let timeout_error = WasmError::Timeout { elapsed_secs: 30 };
        assert_eq!(timeout_error.severity(), error::ErrorSeverity::Medium);
        
        let detection_error = WasmError::DetectionFailed { path: "test.wasm".into() };
        assert_eq!(detection_error.severity(), error::ErrorSeverity::Low);
    }
    
    #[tokio::test]
    /// test_opcode_conversion
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_opcode_conversion() {
        use types::WasmOpcode;
        
        assert_eq!(WasmOpcode::from(0x00), WasmOpcode::Unreachable);
        assert_eq!(WasmOpcode::from(0x01), WasmOpcode::Nop);
        assert_eq!(WasmOpcode::from(0x10), WasmOpcode::Call);
        assert_eq!(WasmOpcode::from(0x28), WasmOpcode::I32Load);
        assert_eq!(WasmOpcode::from(0xFF), WasmOpcode::Other(0xFF));
    }
    
    #[tokio::test]
    /// test_memory_cost_model
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_memory_cost_model() {
        let model = complexity::MemoryCostModel::default();
        assert_eq!(model.load_cost, 3.0);
        assert_eq!(model.store_cost, 5.0);
        assert_eq!(model.grow_cost, 100.0);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::super::*;
    
    #[tokio::test]
    /// test_webassembly_variant_display
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_webassembly_variant_display() {
        use types::WebAssemblyVariant;
        
        assert_eq!(format!("{:?}", WebAssemblyVariant::AssemblyScript), "AssemblyScript");
        assert_eq!(format!("{:?}", WebAssemblyVariant::Wat), "Wat");
        assert_eq!(format!("{:?}", WebAssemblyVariant::Wasm), "Wasm");
    }
    
    #[tokio::test]
    /// test_severity_ordering
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_severity_ordering() {
        use types::Severity;
        
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }
    
    #[tokio::test]
    /// test_difficulty_levels
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_difficulty_levels() {
        use types::Difficulty;
        
        assert_eq!(format!("{:?}", Difficulty::Easy), "Easy");
        assert_eq!(format!("{:?}", Difficulty::Medium), "Medium");
        assert_eq!(format!("{:?}", Difficulty::Hard), "Hard");
    }
    
    #[tokio::test]
    /// test_optimization_types
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_optimization_types() {
        use types::OptimizationType;
        
        let opt = OptimizationType::ReduceAllocations;
        assert_eq!(format!("{:?}", opt), "ReduceAllocations");
    }
    
    #[tokio::test]
    /// test_allocation_strategy
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_allocation_strategy() {
        use memory_pool::AllocationStrategy;
        
        let strat = AllocationStrategy::Dynamic;
        assert!(matches!(strat, AllocationStrategy::Dynamic));
    }
    
    #[tokio::test]
    /// test_parser_type
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_parser_type() {
        use memory_pool::ParserType;
        
        assert_eq!(ParserType::AssemblyScript, ParserType::AssemblyScript);
        assert_ne!(ParserType::Wat, ParserType::WasmBinary);
    }
    
    #[tokio::test]
    /// test_security_category
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_security_category() {
        use security::SecurityCategory;
        
        let cat = SecurityCategory::ResourceExhaustion;
        assert_eq!(format!("{:?}", cat), "ResourceExhaustion");
    }
    
    #[tokio::test]
    /// test_wasm_analysis_capabilities
    ///
    /// # Panics
    ///
    /// May panic if internal assertions fail
    async fn test_wasm_analysis_capabilities() {
        let caps = traits::WasmAnalysisCapabilities::default();
        
        assert!(caps.memory_analysis);
        assert!(caps.gas_estimation);
        assert!(caps.security_analysis);
        assert_eq!(caps.max_file_size, 100 * 1_024 * 1_024);
    }
}