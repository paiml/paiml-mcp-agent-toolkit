//! Coverage tests for WASM modules with 0% coverage
//!
//! This module provides comprehensive unit tests to ensure complete coverage
//! of error handling, type conversions, and utility functions.

#[cfg(test)]
mod error_coverage_tests {
    use crate::services::wasm::error::{WasmError, WasmResult};
    use std::io;

    #[test]
    fn test_wasm_error_parse() {
        let err = WasmError::parse("Invalid syntax");
        assert!(matches!(err, WasmError::ParseError(_)));
        assert_eq!(err.to_string(), "Parse error: Invalid syntax");
    }

    #[test]
    fn test_wasm_error_format() {
        let err = WasmError::format("Bad magic number");
        assert!(matches!(err, WasmError::InvalidFormat(_)));
        assert_eq!(err.to_string(), "Invalid format: Bad magic number");
    }

    #[test]
    fn test_wasm_error_analysis() {
        let err = WasmError::analysis("Complexity too high");
        assert!(matches!(err, WasmError::AnalysisError(_)));
        assert_eq!(err.to_string(), "Analysis error: Complexity too high");
    }

    #[test]
    fn test_wasm_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let wasm_err = WasmError::from(io_err);
        assert!(matches!(wasm_err, WasmError::IoError(_)));
        assert!(wasm_err.to_string().contains("File not found"));
    }

    #[test]
    fn test_wasm_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("Generic error");
        let wasm_err = WasmError::from(anyhow_err);
        assert!(matches!(wasm_err, WasmError::Other(_)));
        assert_eq!(wasm_err.to_string(), "Other error: Generic error");
    }

    #[test]
    fn test_wasm_result_type() {
        let result: WasmResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap(), &42);

        let error_result: WasmResult<i32> = Err(WasmError::parse("Failed"));
        assert!(error_result.is_err());
    }
}

#[cfg(test)]
mod memory_pool_coverage_tests {
    use crate::services::wasm::memory_pool::MemoryPool;

    #[test]
    fn test_memory_pool_new() {
        let pool = MemoryPool::new(1024 * 1024);
        assert_eq!(pool.max_size(), 1024 * 1024);
    }

    #[test]
    fn test_memory_pool_default() {
        let pool = MemoryPool::default();
        assert_eq!(pool.max_size(), 64 * 1024 * 1024);
    }

    #[test]
    fn test_memory_pool_custom_sizes() {
        let small_pool = MemoryPool::new(1024);
        assert_eq!(small_pool.max_size(), 1024);

        let large_pool = MemoryPool::new(256 * 1024 * 1024);
        assert_eq!(large_pool.max_size(), 256 * 1024 * 1024);
    }
}

#[cfg(test)]
mod security_coverage_tests {
    use crate::services::wasm::security::{
        SecurityCategory, SecurityIssue, SecurityValidation, WasmSecurityValidator,
    };
    use crate::services::wasm::types::Severity;

    #[test]
    fn test_security_validator_new() {
        let validator = WasmSecurityValidator::new();
        // Ensure it creates without panic
        let _ = validator;
    }

    #[test]
    fn test_security_validator_default() {
        let validator = WasmSecurityValidator;
        // Ensure it creates without panic
        let _ = validator;
    }

    #[test]
    fn test_validate_valid_wasm() {
        let validator = WasmSecurityValidator::new();
        let valid_wasm = b"\0asm\x01\x00\x00\x00extra_data_here";
        let result = validator.validate(valid_wasm).unwrap();
        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_invalid_magic() {
        let validator = WasmSecurityValidator::new();
        let invalid_wasm = b"WASM\x01\x00\x00\x00";
        let result = validator.validate(invalid_wasm).unwrap();
        assert!(!result.passed);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].severity, Severity::Critical);
        assert_eq!(result.issues[0].category, SecurityCategory::InvalidFormat);
    }

    #[test]
    fn test_validate_too_small() {
        let validator = WasmSecurityValidator::new();
        let tiny_wasm = b"\0as";
        let result = validator.validate(tiny_wasm).unwrap();
        assert!(!result.passed);
        assert!(result
            .issues
            .iter()
            .any(|i| i.severity == Severity::Critical));
    }

    #[test]
    fn test_validate_large_file() {
        let validator = WasmSecurityValidator::new();
        let mut large_wasm = vec![0, b'a', b's', b'm', 1, 0, 0, 0];
        large_wasm.resize(101 * 1024 * 1024, 0);
        let result = validator.validate(&large_wasm).unwrap();
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| {
            i.severity == Severity::High && i.category == SecurityCategory::ResourceExhaustion
        }));
    }

    #[test]
    fn test_validate_ast() {
        let validator = WasmSecurityValidator::new();
        let ast = crate::models::unified_ast::AstDag::new();
        let result = validator.validate_ast(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_text() {
        let validator = WasmSecurityValidator::new();
        let result = validator.validate_text("(module)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_issue_creation() {
        let issue = SecurityIssue {
            severity: Severity::High,
            description: "Test issue".to_string(),
            category: SecurityCategory::MemorySafety,
        };
        assert_eq!(issue.severity, Severity::High);
        assert_eq!(issue.description, "Test issue");
        assert_eq!(issue.category, SecurityCategory::MemorySafety);
    }

    #[test]
    fn test_security_validation_creation() {
        let validation = SecurityValidation {
            passed: true,
            issues: vec![],
        };
        assert!(validation.passed);
        assert!(validation.issues.is_empty());
    }

    #[test]
    fn test_security_categories() {
        let categories = vec![
            SecurityCategory::InvalidFormat,
            SecurityCategory::MemorySafety,
            SecurityCategory::ResourceExhaustion,
            SecurityCategory::CodeInjection,
            SecurityCategory::Other,
        ];

        for cat in categories {
            // Ensure Debug trait works
            let _ = format!("{:?}", cat);
        }
    }
}

#[cfg(test)]
mod traits_coverage_tests {
    use crate::models::unified_ast::{AstDag, Language};
    use crate::services::wasm::traits::{ParsedAst, WasmAnalysisCapabilities};
    use std::collections::HashMap;

    #[test]
    fn test_parsed_ast_creation() {
        let ast = ParsedAst {
            language: Language::WebAssembly,
            dag: AstDag::new(),
            source_file: Some(std::path::PathBuf::from("test.wasm")),
            parse_errors: vec!["Error 1".to_string()],
            metadata: HashMap::new(),
        };
        assert_eq!(ast.language, Language::WebAssembly);
        assert_eq!(ast.parse_errors.len(), 1);
        assert!(ast.source_file.is_some());
    }

    #[test]
    fn test_wasm_analysis_capabilities_default() {
        let caps = WasmAnalysisCapabilities::default();
        assert!(caps.memory_analysis);
        assert!(caps.gas_estimation);
        assert!(caps.security_analysis);
        assert!(caps.optimization_hints);
        assert!(caps.streaming_support);
        assert!(!caps.simd_analysis);
        assert!(!caps.multi_memory);
        assert_eq!(caps.max_file_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_wasm_analysis_capabilities_custom() {
        let caps = WasmAnalysisCapabilities {
            memory_analysis: false,
            gas_estimation: false,
            security_analysis: true,
            optimization_hints: false,
            streaming_support: true,
            simd_analysis: true,
            multi_memory: true,
            max_file_size: 50 * 1024 * 1024,
        };
        assert!(!caps.memory_analysis);
        assert!(caps.simd_analysis);
        assert_eq!(caps.max_file_size, 50 * 1024 * 1024);
    }
}

#[cfg(test)]
mod types_coverage_tests {
    use crate::services::wasm::types::*;
    use std::collections::HashMap;

    #[test]
    fn test_webassembly_variant() {
        let variants = vec![
            WebAssemblyVariant::AssemblyScript,
            WebAssemblyVariant::Wat,
            WebAssemblyVariant::Wasm,
        ];

        for variant in variants {
            // Test Debug and PartialEq
            let _ = format!("{:?}", variant);
            assert_eq!(variant, variant.clone());
        }
    }

    #[test]
    fn test_wasm_metrics_default() {
        let metrics = WasmMetrics::default();
        assert_eq!(metrics.memory_sections, 0);
        assert_eq!(metrics.function_count, 0);
        assert_eq!(metrics.import_count, 0);
    }

    #[test]
    fn test_memory_op_stats() {
        let stats = MemoryOpStats {
            loads: 10,
            stores: 5,
            grows: 1,
            atomic_ops: 2,
            simd_ops: 3,
            bulk_ops: 4,
        };
        assert_eq!(stats.loads, 10);
        assert_eq!(stats.simd_ops, 3);
    }

    #[test]
    fn test_wasm_complexity() {
        let complexity = WasmComplexity {
            cyclomatic: 10,
            memory_pressure: 75.5,
            indirect_call_overhead: 1.5,
            estimated_gas: 1000.0,
            cognitive: 15,
            hot_path_score: 0.8,
            max_loop_depth: 3,
        };
        assert_eq!(complexity.cyclomatic, 10);
        assert_eq!(complexity.memory_pressure, 75.5);
    }

    #[test]
    fn test_memory_analysis() {
        let analysis = MemoryAnalysis {
            peak_usage_bytes: 1024 * 1024,
            allocation_patterns: vec![],
            leak_risk_score: 25.0,
            optimization_hints: vec![],
            max_stack_depth: 100,
            alignment_issues: vec![],
        };
        assert_eq!(analysis.peak_usage_bytes, 1024 * 1024);
        assert_eq!(analysis.leak_risk_score, 25.0);
    }

    #[test]
    fn test_allocation_pattern() {
        let pattern = AllocationPattern {
            pattern_type: "linear_growth".to_string(),
            location: SourceLocation {
                file: "test.wasm".to_string(),
                line: 10,
                column: 5,
                offset: 100,
            },
            severity: Severity::Medium,
            description: "Memory grows linearly".to_string(),
        };
        assert_eq!(pattern.pattern_type, "linear_growth");
        assert_eq!(pattern.location.line, 10);
    }

    #[test]
    fn test_memory_optimization_hint() {
        let hint = MemoryOptimizationHint {
            hint_type: OptimizationType::ReduceAllocations,
            expected_improvement: 15.5,
            difficulty: Difficulty::Medium,
            suggestion: "Reduce allocations in hot loop".to_string(),
        };
        assert_eq!(hint.expected_improvement, 15.5);
        assert!(matches!(hint.difficulty, Difficulty::Medium));
    }

    #[test]
    fn test_alignment_issue() {
        let issue = AlignmentIssue {
            offset: 100,
            required_alignment: 8,
            actual_alignment: 4,
            performance_impact: 10.5,
        };
        assert_eq!(issue.offset, 100);
        assert_eq!(issue.required_alignment, 8);
    }

    #[test]
    fn test_source_location() {
        let loc = SourceLocation {
            file: "module.wasm".to_string(),
            line: 42,
            column: 8,
            offset: 1024,
        };
        assert_eq!(loc.file, "module.wasm");
        assert_eq!(loc.line, 42);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Low.to_string(), "Low");
        assert_eq!(Severity::Medium.to_string(), "Medium");
        assert_eq!(Severity::High.to_string(), "High");
        assert_eq!(Severity::Critical.to_string(), "Critical");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_difficulty_enum() {
        let difficulties = vec![Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
        for diff in difficulties {
            let _ = format!("{:?}", diff);
        }
    }

    #[test]
    fn test_optimization_type_enum() {
        let types = vec![
            OptimizationType::ReduceAllocations,
            OptimizationType::ImproveAlignment,
            OptimizationType::UseStackMemory,
            OptimizationType::PoolAllocations,
            OptimizationType::CompactDataStructures,
            OptimizationType::EliminateLeaks,
            OptimizationType::ReduceFragmentation,
        ];
        for opt_type in types {
            let _ = format!("{:?}", opt_type);
        }
    }

    #[test]
    fn test_wasm_opcode_from_u8() {
        assert_eq!(WasmOpcode::from(0x00), WasmOpcode::Unreachable);
        assert_eq!(WasmOpcode::from(0x01), WasmOpcode::Nop);
        assert_eq!(WasmOpcode::from(0x10), WasmOpcode::Call);
        assert_eq!(WasmOpcode::from(0x28), WasmOpcode::I32Load);
        assert_eq!(WasmOpcode::from(0x40), WasmOpcode::MemoryGrow);
        assert_eq!(WasmOpcode::from(0xFF), WasmOpcode::Other(0xFF));
    }

    #[test]
    fn test_wasm_opcode_all_variants() {
        let opcodes = vec![
            WasmOpcode::Unreachable,
            WasmOpcode::Nop,
            WasmOpcode::Block,
            WasmOpcode::Loop,
            WasmOpcode::If,
            WasmOpcode::Else,
            WasmOpcode::End,
            WasmOpcode::Br,
            WasmOpcode::BrIf,
            WasmOpcode::BrTable,
            WasmOpcode::Return,
            WasmOpcode::Call,
            WasmOpcode::CallIndirect,
            WasmOpcode::I32Load,
            WasmOpcode::I64Load,
            WasmOpcode::F32Load,
            WasmOpcode::F64Load,
            WasmOpcode::I32Store,
            WasmOpcode::I64Store,
            WasmOpcode::F32Store,
            WasmOpcode::F64Store,
            WasmOpcode::MemorySize,
            WasmOpcode::MemoryGrow,
            WasmOpcode::I32Const,
            WasmOpcode::I64Const,
            WasmOpcode::F32Const,
            WasmOpcode::F64Const,
            WasmOpcode::LocalGet,
            WasmOpcode::LocalSet,
            WasmOpcode::LocalTee,
            WasmOpcode::GlobalGet,
            WasmOpcode::GlobalSet,
            WasmOpcode::Other(0x99),
        ];

        for opcode in opcodes {
            // Test PartialEq and Hash traits
            assert_eq!(opcode, opcode);
            let mut map = HashMap::new();
            map.insert(opcode, 1);
            assert_eq!(map.get(&opcode), Some(&1));
        }
    }

    #[test]
    fn test_wasm_metrics_with_histogram() {
        let mut metrics = WasmMetrics::default();
        metrics.instruction_histogram.insert(WasmOpcode::Call, 10);
        metrics.instruction_histogram.insert(WasmOpcode::I32Load, 5);

        assert_eq!(
            metrics.instruction_histogram.get(&WasmOpcode::Call),
            Some(&10)
        );
        assert_eq!(
            metrics.instruction_histogram.get(&WasmOpcode::I32Load),
            Some(&5)
        );
    }
}

#[cfg(test)]
mod complexity_coverage_tests {
    use crate::models::unified_ast::{AstDag, AstKind, FunctionKind, Language, UnifiedAstNode};
    use crate::services::wasm::complexity::{MemoryCostModel, WasmComplexityAnalyzer};

    #[test]
    fn test_memory_cost_model_default() {
        let model = MemoryCostModel::default();
        assert_eq!(model.load_cost, 3.0);
        assert_eq!(model.store_cost, 5.0);
        assert_eq!(model.grow_cost, 100.0);
    }

    #[test]
    fn test_memory_cost_model_custom() {
        let model = MemoryCostModel {
            load_cost: 10.0,
            store_cost: 15.0,
            grow_cost: 200.0,
        };
        assert_eq!(model.load_cost, 10.0);
        assert_eq!(model.store_cost, 15.0);
        assert_eq!(model.grow_cost, 200.0);
    }

    #[test]
    fn test_complexity_analyzer_default() {
        let analyzer = WasmComplexityAnalyzer::default();
        // Just ensure it creates without panic
        let _ = analyzer;
    }

    #[test]
    fn test_analyze_ast() {
        let analyzer = WasmComplexityAnalyzer::new();
        let ast = AstDag::new();
        let result = analyzer.analyze_ast(&ast);
        assert!(result.is_ok());
        let complexity = result.unwrap();
        assert_eq!(complexity.cyclomatic, 5);
        assert_eq!(complexity.cognitive, 5);
    }

    #[test]
    fn test_analyze_function() {
        let analyzer = WasmComplexityAnalyzer::new();
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
        assert_eq!(complexity.memory_pressure, 0.1);
        assert_eq!(complexity.hot_path_score, 1.0);
        assert_eq!(complexity.estimated_gas, 1000.0);
        assert_eq!(complexity.indirect_call_overhead, 1.0);
    }

    #[test]
    fn test_analyze_text_simple() {
        let analyzer = WasmComplexityAnalyzer::new();
        let content = "(module)";
        let result = analyzer.analyze_text(content);
        assert!(result.is_ok());
        let complexity = result.unwrap();
        assert_eq!(complexity.cyclomatic, 0);
        assert_eq!(complexity.max_loop_depth, 1);
    }

    #[test]
    fn test_analyze_text_with_functions() {
        let analyzer = WasmComplexityAnalyzer::new();
        let content = r#"
(module
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
  (func $mul (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.mul)
)"#;
        let result = analyzer.analyze_text(content);
        assert!(result.is_ok());
        let complexity = result.unwrap();
        assert!(complexity.cyclomatic > 0);
        assert_eq!(complexity.cyclomatic, 5); // Actual calculated value
    }

    #[test]
    fn test_analyze_text_large() {
        let analyzer = WasmComplexityAnalyzer::new();
        let mut content = String::from("(module\n");
        for i in 0..100 {
            content.push_str(&format!("  (func $f{} (result i32) i32.const {})\n", i, i));
        }
        content.push(')');

        let result = analyzer.analyze_text(&content);
        assert!(result.is_ok());
        let complexity = result.unwrap();
        assert!(complexity.cyclomatic > 100);
        assert!(complexity.memory_pressure > 0.0);
    }
}
