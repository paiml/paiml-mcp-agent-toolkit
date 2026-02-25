// WASM tests - included from wasm.rs
// NO use imports or #! inner attributes allowed (shares parent module scope)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // Simple WASM binary that adds two numbers (hand-crafted minimal example)
    const SIMPLE_WASM_BINARY: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, // WASM magic number
        0x01, 0x00, 0x00, 0x00, // Version
        0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // Type section
        0x03, 0x02, 0x01, 0x00, // Function section
        0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b, // Code section
    ];

    const SIMPLE_WAT_TEXT: &str = r#"
(module
  (func $add (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.add
  )
  (export "add" (func $add))
)
"#;

    const COMPLEX_WAT_WITH_CONTROL_FLOW: &str = r#"
(module
  (func $fibonacci (param $n i32) (result i32)
    (if (i32.lt_s (local.get $n) (i32.const 2))
      (then (local.get $n))
      (else
        (i32.add
          (call $fibonacci (i32.sub (local.get $n) (i32.const 1)))
          (call $fibonacci (i32.sub (local.get $n) (i32.const 2)))
        )
      )
    )
  )
  (export "fibonacci" (func $fibonacci))
)
"#;

    #[test]
    fn test_simple_wasm_binary_analysis() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("simple.wasm"));
        let items = analyzer
            .analyze_wasm_binary(SIMPLE_WASM_BINARY)
            .expect("Should parse simple WASM binary");

        assert!(
            !items.is_empty(),
            "Should extract at least one AST item from WASM"
        );

        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(
            function_items.len(),
            1,
            "Should extract exactly one function"
        );

        if let AstItem::Function {
            name,
            visibility,
            is_async,
            ..
        } = &function_items[0]
        {
            assert!(
                name.contains("simple"),
                "Should include module name in function name"
            );
            assert_eq!(
                visibility, "export",
                "WASM exported functions have export visibility"
            );
            assert!(!is_async, "WASM functions are not async");
        }
    }

    #[test]
    fn test_wat_text_analysis() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("add.wasm"));
        let items = analyzer
            .analyze_wat_text(SIMPLE_WAT_TEXT)
            .expect("Should parse WAT text format");

        assert!(!items.is_empty(), "Should extract AST items from WAT text");

        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(function_items.len(), 1, "Should extract the add function");

        if let AstItem::Function { name, .. } = &function_items[0] {
            assert!(
                name.contains("add"),
                "Should extract function name from WAT"
            );
        }
    }

    #[test]
    fn test_wasm_complexity_analysis() {
        let mut analyzer = WasmStackAnalyzer::new();
        let stack_complexity = analyzer
            .analyze_stack_complexity(&[0x20, 0x00, 0x20, 0x01, 0x6a])
            .expect("Should analyze WASM stack complexity");

        assert!(
            stack_complexity >= 1,
            "Should have at least complexity of 1"
        );
        assert!(
            stack_complexity <= 10,
            "Should maintain complexity ≤10 for simple WASM"
        );

        let control_flow_complexity = analyzer
            .analyze_control_flow_complexity(&[0x04, 0x40, 0x0b])
            .expect("Should analyze control flow complexity");

        assert!(
            control_flow_complexity >= 1,
            "Should have control flow complexity"
        );
    }

    #[test]
    fn test_complex_wat_control_flow() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("fibonacci.wasm"));
        let items = analyzer
            .analyze_wat_text(COMPLEX_WAT_WITH_CONTROL_FLOW)
            .expect("Should parse complex WAT with control flow");

        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(function_items.len(), 1, "Should extract fibonacci function");

        // Should detect higher complexity due to recursive calls and conditionals
        if let AstItem::Function { name, line, .. } = &function_items[0] {
            assert!(
                name.contains("fibonacci"),
                "Should extract fibonacci function name"
            );
            assert!(*line >= 1, "Should have valid line number");
        }
    }

    #[test]
    fn test_wasm_module_validation() {
        let mut validator = WasmValidator::new();
        let is_valid = validator
            .validate_wasm_module(SIMPLE_WASM_BINARY)
            .expect("Should validate WASM module");

        assert!(is_valid, "Simple WASM binary should be valid");
        assert!(
            validator.get_validation_errors().is_empty(),
            "Should have no validation errors"
        );
    }

    #[test]
    fn test_wasm_security_analysis() {
        let mut validator = WasmValidator::new();
        let security_warnings = validator
            .analyze_security(SIMPLE_WASM_BINARY)
            .expect("Should perform security analysis");

        // Simple add function should have minimal security concerns
        assert!(
            security_warnings.len() <= 2,
            "Simple WASM should have few security warnings"
        );
    }

    #[test]
    fn test_invalid_wasm_binary() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("invalid.wasm"));
        let invalid_bytes = &[0xFF, 0xFF, 0xFF, 0xFF]; // Invalid WASM magic
        let result = analyzer.analyze_wasm_binary(invalid_bytes);

        assert!(
            result.is_err(),
            "Should return error for invalid WASM binary"
        );
    }

    #[test]
    fn test_empty_wasm_module() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("empty.wasm"));
        let minimal_wasm = &[
            0x00, 0x61, 0x73, 0x6d, // WASM magic
            0x01, 0x00, 0x00, 0x00, // Version
        ];
        let items = analyzer
            .analyze_wasm_binary(minimal_wasm)
            .expect("Should handle minimal WASM module");

        assert!(
            items.is_empty(),
            "Empty WASM module should produce no function items"
        );
    }

    #[test]
    fn test_wasm_import_export_extraction() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("with_imports.wasm"));
        let items = analyzer
            .analyze_wat_text(
                r#"
(module
  (import "env" "log" (func $log (param i32)))
  (func $main (result i32)
    i32.const 42
    call $log
    i32.const 0
  )
  (export "main" (func $main))
)
"#,
            )
            .expect("Should parse WASM with imports/exports");

        let function_items: Vec<_> = items
            .iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        // Should extract both imported and local functions
        assert!(
            !function_items.is_empty(),
            "Should extract at least the main function"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::Path;

    proptest! {
        #[test]
        fn test_wasm_analyzer_handles_various_module_names(
            module_name in "[a-zA-Z_][a-zA-Z0-9_]*"
        ) {
            let file_path = format!("{}.wasm", module_name);
            let analyzer = WasmModuleAnalyzer::new(Path::new(&file_path));

            // Basic property: analyzer should be created successfully
            prop_assert_eq!(analyzer.module_name, module_name);
            prop_assert_eq!(analyzer.function_count, 0);
            prop_assert_eq!(analyzer._import_count, 0);
            prop_assert_eq!(analyzer._export_count, 0);
        }

        #[test]
        fn test_wasm_stack_analyzer_bounds(
            stack_operations in 1usize..20
        ) {
            let mut analyzer = WasmStackAnalyzer::new();

            // Test with varying numbers of stack operations
            let mut operations = Vec::new();
            for _ in 0..stack_operations {
                operations.extend_from_slice(&[0x41, 0x01]); // i32.const 1
            }
            operations.push(0x1a); // drop

            if let Ok(complexity) = analyzer.analyze_stack_complexity(&operations) {
                // Complexity should be proportional to operations but bounded
                prop_assert!(complexity >= 1);
                prop_assert!(complexity <= stack_operations as u32 + 5);
            }
        }

        #[test]
        fn test_wasm_validator_consistency(
            module_size in 8usize..100
        ) {
            let mut validator = WasmValidator::new();

            // Create a minimal valid WASM module of varying sizes
            let mut wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
            wasm_bytes.resize(module_size, 0x00);

            // Validation should be consistent - same result on repeated calls
            let result1 = validator.validate_wasm_module(&wasm_bytes);
            let result2 = validator.validate_wasm_module(&wasm_bytes);

            prop_assert_eq!(result1.is_ok(), result2.is_ok());
        }

        #[test]
        fn test_wasm_complexity_scales_reasonably(
            control_flow_depth in 1u32..8
        ) {
            let mut analyzer = WasmStackAnalyzer::new();

            // Create nested control flow (if blocks)
            let mut instructions = Vec::new();
            for _ in 0..control_flow_depth {
                instructions.extend_from_slice(&[0x04, 0x40]); // if block
            }
            #[allow(clippy::same_item_push)]
            for _ in 0..control_flow_depth {
                instructions.push(0x0b); // end - intentionally pushing same opcode
            }

            if let Ok(complexity) = analyzer.analyze_control_flow_complexity(&instructions) {
                // Complexity should scale with nesting depth
                prop_assert!(complexity >= control_flow_depth);
                prop_assert!(complexity <= control_flow_depth * 2 + 3);
            }
        }
    }
}
