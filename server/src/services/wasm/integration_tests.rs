//! Integration tests for WASM modules to ensure proper coverage tracking
//!
//! These tests use the public API to ensure LLVM coverage properly tracks execution.

#[cfg(test)]
mod tests {
    use crate::services::wasm;
    
    #[test]
    fn test_wasm_error_integration() {
        // Test error creation through public API
        let parse_err = wasm::WasmError::parse("Test parse error");
        assert_eq!(parse_err.to_string(), "Parse error: Test parse error");
        
        let format_err = wasm::WasmError::format("Test format error");
        assert_eq!(format_err.to_string(), "Invalid format: Test format error");
        
        let analysis_err = wasm::WasmError::analysis("Test analysis error");
        assert_eq!(analysis_err.to_string(), "Analysis error: Test analysis error");
        
        // Test error conversions
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let wasm_io_err: wasm::WasmError = io_err.into();
        assert!(wasm_io_err.to_string().contains("File not found"));
        
        let anyhow_err = anyhow::anyhow!("Generic error");
        let wasm_other_err: wasm::WasmError = anyhow_err.into();
        assert_eq!(wasm_other_err.to_string(), "Other error: Generic error");
        
        // Test Result type
        let ok_result: wasm::WasmResult<i32> = Ok(42);
        assert!(ok_result.is_ok());
        
        let err_result: wasm::WasmResult<i32> = Err(wasm::WasmError::parse("Failed"));
        assert!(err_result.is_err());
    }
    
    #[test]
    fn test_memory_pool_integration() {
        use crate::services::wasm::memory_pool::MemoryPool;
        
        // Test pool creation and usage
        let pool = MemoryPool::new(1024 * 1024);
        assert_eq!(pool.max_size(), 1024 * 1024);
        
        let default_pool = MemoryPool::default();
        assert_eq!(default_pool.max_size(), 64 * 1024 * 1024);
        
        // Test different pool sizes
        let small_pool = MemoryPool::new(512);
        assert_eq!(small_pool.max_size(), 512);
        
        let large_pool = MemoryPool::new(256 * 1024 * 1024);
        assert_eq!(large_pool.max_size(), 256 * 1024 * 1024);
    }
    
    #[test]
    fn test_types_integration() {
        use crate::services::wasm::types::*;
        
        // Test WebAssemblyVariant
        let variants = vec![
            WebAssemblyVariant::AssemblyScript,
            WebAssemblyVariant::Wat,
            WebAssemblyVariant::Wasm,
        ];
        for variant in &variants {
            assert_eq!(variant, variant);
            let _ = format!("{:?}", variant);
        }
        
        // Test WasmMetrics
        let mut metrics = WasmMetrics {
            function_count: 10,
            import_count: 5,
            ..Default::default()
        };
        metrics.instruction_histogram.insert(WasmOpcode::Call, 20);
        assert_eq!(metrics.function_count, 10);
        assert_eq!(*metrics.instruction_histogram.get(&WasmOpcode::Call).unwrap(), 20);
        
        // Test MemoryOpStats
        let stats = MemoryOpStats {
            loads: 100,
            stores: 50,
            grows: 2,
            atomic_ops: 10,
            simd_ops: 5,
            bulk_ops: 3,
        };
        assert_eq!(stats.loads, 100);
        assert_eq!(stats.stores, 50);
        
        // Test WasmComplexity
        let complexity = WasmComplexity {
            cyclomatic: 15,
            memory_pressure: 80.5,
            indirect_call_overhead: 2.5,
            estimated_gas: 2000.0,
            cognitive: 20,
            hot_path_score: 0.9,
            max_loop_depth: 4,
        };
        assert_eq!(complexity.cyclomatic, 15);
        assert_eq!(complexity.memory_pressure, 80.5);
        
        // Test Severity
        assert_eq!(Severity::Low.to_string(), "Low");
        assert_eq!(Severity::Medium.to_string(), "Medium");
        assert_eq!(Severity::High.to_string(), "High");
        assert_eq!(Severity::Critical.to_string(), "Critical");
        assert!(Severity::Low < Severity::Critical);
        
        // Test WasmOpcode conversions
        assert_eq!(WasmOpcode::from(0x00), WasmOpcode::Unreachable);
        assert_eq!(WasmOpcode::from(0x10), WasmOpcode::Call);
        assert_eq!(WasmOpcode::from(0x28), WasmOpcode::I32Load);
        assert_eq!(WasmOpcode::from(0xFF), WasmOpcode::Other(0xFF));
    }
    
    #[test]
    fn test_traits_integration() {
        use crate::services::wasm::traits::*;
        use crate::models::unified_ast::{AstDag, Language};
        use std::collections::HashMap;
        
        // Test ParsedAst
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());
        
        let ast = ParsedAst {
            language: Language::WebAssembly,
            dag: AstDag::new(),
            source_file: Some(std::path::PathBuf::from("test.wasm")),
            parse_errors: vec!["Error 1".to_string(), "Error 2".to_string()],
            metadata,
        };
        assert_eq!(ast.language, Language::WebAssembly);
        assert_eq!(ast.parse_errors.len(), 2);
        assert!(ast.source_file.is_some());
        assert_eq!(ast.metadata.get("version"), Some(&"1.0".to_string()));
        
        // Test WasmAnalysisCapabilities
        let default_caps = WasmAnalysisCapabilities::default();
        assert!(default_caps.memory_analysis);
        assert!(default_caps.gas_estimation);
        assert!(default_caps.security_analysis);
        assert!(default_caps.optimization_hints);
        assert!(default_caps.streaming_support);
        assert!(!default_caps.simd_analysis);
        assert!(!default_caps.multi_memory);
        assert_eq!(default_caps.max_file_size, 100 * 1024 * 1024);
        
        let custom_caps = WasmAnalysisCapabilities {
            memory_analysis: false,
            gas_estimation: true,
            security_analysis: true,
            optimization_hints: false,
            streaming_support: false,
            simd_analysis: true,
            multi_memory: true,
            max_file_size: 200 * 1024 * 1024,
        };
        assert!(!custom_caps.memory_analysis);
        assert!(custom_caps.simd_analysis);
        assert_eq!(custom_caps.max_file_size, 200 * 1024 * 1024);
    }
    
    #[test]
    fn test_security_integration() {
        use crate::services::wasm::security::*;
        use crate::services::wasm::types::Severity;
        
        // Test validator creation
        let validator = WasmSecurityValidator::new();
        
        // Test valid WASM
        let valid_wasm = b"\0asm\x01\x00\x00\x00\x10\x00\x00\x00extra_data";
        let result = validator.validate(valid_wasm).unwrap();
        assert!(result.passed);
        assert!(result.issues.is_empty());
        
        // Test invalid magic number
        let invalid_wasm = b"WASM\x01\x00\x00\x00";
        let result = validator.validate(invalid_wasm).unwrap();
        assert!(!result.passed);
        assert!(!result.issues.is_empty());
        assert_eq!(result.issues[0].severity, Severity::Critical);
        assert_eq!(result.issues[0].category, SecurityCategory::InvalidFormat);
        
        // Test too small file
        let tiny_wasm = b"\0a";
        let result = validator.validate(tiny_wasm).unwrap();
        assert!(!result.passed);
        
        // Test large file
        let mut large_wasm = vec![0, b'a', b's', b'm', 1, 0, 0, 0];
        large_wasm.resize(101 * 1024 * 1024, 0);
        let result = validator.validate(&large_wasm).unwrap();
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| {
            i.severity == Severity::High && i.category == SecurityCategory::ResourceExhaustion
        }));
        
        // Test AST validation
        let ast = crate::models::unified_ast::AstDag::new();
        let result = validator.validate_ast(&ast);
        assert!(result.is_ok());
        
        // Test text validation
        let result = validator.validate_text("(module)");
        assert!(result.is_ok());
        
        // Test SecurityIssue
        let issue = SecurityIssue {
            severity: Severity::Medium,
            description: "Test security issue".to_string(),
            category: SecurityCategory::MemorySafety,
        };
        assert_eq!(issue.severity, Severity::Medium);
        assert_eq!(issue.description, "Test security issue");
        
        // Test all SecurityCategory variants
        let categories = vec![
            SecurityCategory::InvalidFormat,
            SecurityCategory::MemorySafety,
            SecurityCategory::ResourceExhaustion,
            SecurityCategory::CodeInjection,
            SecurityCategory::Other,
        ];
        for cat in categories {
            let _ = format!("{:?}", cat);
        }
    }
    
    #[test]
    fn test_complexity_integration() {
        use crate::services::wasm::complexity::*;
        use crate::models::unified_ast::{AstDag, UnifiedAstNode, AstKind, FunctionKind, Language};
        
        // Test MemoryCostModel
        let default_model = MemoryCostModel::default();
        assert_eq!(default_model.load_cost, 3.0);
        assert_eq!(default_model.store_cost, 5.0);
        assert_eq!(default_model.grow_cost, 100.0);
        
        let custom_model = MemoryCostModel {
            load_cost: 5.0,
            store_cost: 10.0,
            grow_cost: 150.0,
        };
        assert_eq!(custom_model.load_cost, 5.0);
        
        // Test WasmComplexityAnalyzer
        let analyzer = WasmComplexityAnalyzer::new();
        
        // Test AST analysis
        let ast = AstDag::new();
        let result = analyzer.analyze_ast(&ast);
        assert!(result.is_ok());
        let complexity = result.unwrap();
        assert_eq!(complexity.cyclomatic, 5); // Base complexity
        
        // Test function analysis
        let mut dag = AstDag::new();
        let func_node = UnifiedAstNode::new(
            AstKind::Function(FunctionKind::Regular),
            Language::WebAssembly,
        );
        let func_id = dag.add_node(func_node);
        
        let complexity = analyzer.analyze_function(&dag, func_id);
        assert_eq!(complexity.cyclomatic, 1);
        assert_eq!(complexity.cognitive, 1);
        assert_eq!(complexity.max_loop_depth, 0);
        
        // Test text analysis
        let simple_module = "(module)";
        let result = analyzer.analyze_text(simple_module);
        assert!(result.is_ok());
        
        let complex_module = r#"
        (module
          (func $add (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add)
          (func $sub (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.sub)
        )"#;
        let result = analyzer.analyze_text(complex_module);
        assert!(result.is_ok());
        let complexity = result.unwrap();
        assert!(complexity.cyclomatic > 0);
    }
}