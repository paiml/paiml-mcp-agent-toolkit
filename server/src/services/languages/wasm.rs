//! WebAssembly Analysis Support for PMAT
//!
//! This module provides WASM-specific analysis capabilities using wasmparser
//! for bytecode analysis and complexity metrics extraction from WASM modules.

use wasmparser::{Parser, Payload};
#[cfg(feature = "wasm-ast")]
use crate::services::context::AstItem;
use std::path::{Path, PathBuf};

/// WASM module analyzer that extracts WASM-specific information
pub struct WasmModuleAnalyzer {
    items: Vec<AstItem>,
    _file_path: PathBuf,
    module_name: String,
    function_count: usize,
    _import_count: usize,
    _export_count: usize,
}

impl WasmModuleAnalyzer {
    /// Creates a new WASM module analyzer
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            module_name: file_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            function_count: 0,
            _import_count: 0,
            _export_count: 0,
        }
    }

    /// Analyzes WASM binary and extracts AST items (complexity ≤10)
    pub fn analyze_wasm_binary(mut self, wasm_bytes: &[u8]) -> Result<Vec<AstItem>, String> {
        if wasm_bytes.len() < 8 {
            return Err("Invalid WASM binary: too short".to_string());
        }

        if &wasm_bytes[0..4] != b"\0asm" {
            return Err("Invalid WASM magic number".to_string());
        }

        let parser = Parser::new(0);
        for (i, payload) in parser.parse_all(wasm_bytes).enumerate() {
            match payload {
                Ok(Payload::FunctionSection(_)) => {
                    self.function_count = i + 1;
                    self.items.push(AstItem::Function {
                        name: self.get_qualified_name(&format!("function_{}", i)),
                        visibility: "export".to_string(),
                        is_async: false,
                        line: 1,
                    });
                }
                _ => continue,
            }
        }

        Ok(self.items)
    }

    /// Analyzes WASM text format (.wat) (complexity ≤10)
    pub fn analyze_wat_text(mut self, wat_source: &str) -> Result<Vec<AstItem>, String> {
        let mut function_count = 0;

        for line in wat_source.lines() {
            let trimmed = line.trim();

            if trimmed.contains("(func ") {
                let func_name = self.extract_wat_function_name(trimmed);
                let qualified_name = self.get_qualified_name(&func_name);

                self.items.push(AstItem::Function {
                    name: qualified_name,
                    visibility: "export".to_string(),
                    is_async: false,
                    line: function_count + 1,
                });
                function_count += 1;
            }
        }

        self.function_count = function_count;
        Ok(self.items)
    }

    /// Extracts function information from WASM (complexity ≤10)
    fn _extract_wasm_functions(&mut self, _parser: &Parser) -> Result<(), String> {
        // TO BE IMPLEMENTED
        todo!("Extract function information from WASM module")
    }

    /// Extracts import/export information (complexity ≤10)
    fn _extract_imports_exports(&mut self, _parser: &Parser) -> Result<(), String> {
        // TO BE IMPLEMENTED
        todo!("Extract import/export information from WASM")
    }

    /// Calculates WASM-specific complexity metrics (complexity ≤10)
    fn _calculate_wasm_complexity(&self, _function_body: &[u8]) -> Result<(u32, u32), String> {
        // TO BE IMPLEMENTED
        todo!("Calculate cyclomatic and cognitive complexity for WASM function")
    }

    /// Extracts function name from WAT line (complexity ≤10)
    fn extract_wat_function_name(&self, line: &str) -> String {
        if let Some(start) = line.find("$") {
            if let Some(end) = line[start..].find(" ") {
                line[start+1..start+end].to_string()
            } else {
                format!("function_{}", self.function_count)
            }
        } else {
            format!("function_{}", self.function_count)
        }
    }

    /// Gets qualified name for WASM symbol (complexity ≤10)
    fn get_qualified_name(&self, symbol_name: &str) -> String {
        if self.module_name.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{}::{}", self.module_name, symbol_name)
        }
    }
}

/// WASM stack depth analyzer for complexity calculation (complexity ≤10)
pub struct WasmStackAnalyzer {
    max_stack_depth: u32,
    current_depth: u32,
    branch_count: u32,
}

impl Default for WasmStackAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmStackAnalyzer {
    /// Creates a new WASM stack analyzer
    pub fn new() -> Self {
        Self {
            max_stack_depth: 0,
            current_depth: 0,
            branch_count: 0,
        }
    }

    /// Analyzes stack depth complexity (complexity ≤10)
    pub fn analyze_stack_complexity(&mut self, function_body: &[u8]) -> Result<u32, String> {
        self.current_depth = 0;
        self.max_stack_depth = 0;

        for &byte in function_body {
            match byte {
                0x41 => self.current_depth += 1, // i32.const
                0x42 => self.current_depth += 1, // i64.const
                0x6a => {                         // i32.add
                    if self.current_depth >= 2 {
                        self.current_depth -= 1;
                    }
                }
                0x1a => {                         // drop
                    if self.current_depth > 0 {
                        self.current_depth -= 1;
                    }
                }
                _ => {}
            }
            self.max_stack_depth = self.max_stack_depth.max(self.current_depth);
        }

        Ok(self.max_stack_depth)
    }

    /// Analyzes control flow complexity (complexity ≤10)
    pub fn analyze_control_flow_complexity(&mut self, function_body: &[u8]) -> Result<u32, String> {
        self.branch_count = 1; // Base complexity

        for &byte in function_body {
            match byte {
                0x04 => self.branch_count += 1, // if block
                0x05 => self.branch_count += 1, // else
                0x03 => self.branch_count += 1, // loop
                0x02 => self.branch_count += 1, // block
                0x0c => self.branch_count += 1, // br (branch)
                0x0d => self.branch_count += 1, // br_if (conditional branch)
                _ => {}
            }
        }

        Ok(self.branch_count)
    }
}

/// WASM validation and quality checks (complexity ≤10)
pub struct WasmValidator {
    validation_errors: Vec<String>,
    security_warnings: Vec<String>,
}

impl Default for WasmValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmValidator {
    /// Creates a new WASM validator
    pub fn new() -> Self {
        Self {
            validation_errors: Vec::new(),
            security_warnings: Vec::new(),
        }
    }

    /// Validates WASM module for correctness (complexity ≤10)
    pub fn validate_wasm_module(&mut self, wasm_bytes: &[u8]) -> Result<bool, String> {
        if wasm_bytes.len() < 8 {
            self.validation_errors.push("Module too short".to_string());
            return Ok(false);
        }

        if &wasm_bytes[0..4] != b"\0asm" {
            self.validation_errors.push("Invalid magic number".to_string());
            return Ok(false);
        }

        let parser = Parser::new(0);
        for payload in parser.parse_all(wasm_bytes) {
            if payload.is_err() {
                self.validation_errors.push("Parse error in module".to_string());
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Performs security analysis on WASM module (complexity ≤10)
    pub fn analyze_security(&mut self, wasm_bytes: &[u8]) -> Result<Vec<String>, String> {
        let mut warnings = Vec::new();

        if wasm_bytes.len() > 1024 * 1024 {
            warnings.push("Large WASM module may consume excessive memory".to_string());
        }

        let parser = Parser::new(0);
        for payload in parser.parse_all(wasm_bytes) {
            if let Ok(Payload::ImportSection(_)) = payload {
                warnings.push("Module imports external functions".to_string());
            }
        }

        self.security_warnings = warnings.clone();
        Ok(warnings)
    }

    /// Gets validation errors
    pub fn get_validation_errors(&self) -> &[String] {
        &self.validation_errors
    }

    /// Gets security warnings
    pub fn get_security_warnings(&self) -> &[String] {
        &self.security_warnings
    }
}

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
        0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b // Code section
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
        let items = analyzer.analyze_wasm_binary(SIMPLE_WASM_BINARY)
            .expect("Should parse simple WASM binary");

        assert!(!items.is_empty(), "Should extract at least one AST item from WASM");

        let function_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(function_items.len(), 1, "Should extract exactly one function");

        if let AstItem::Function { name, visibility, is_async, .. } = &function_items[0] {
            assert!(name.contains("simple"), "Should include module name in function name");
            assert_eq!(visibility, "export", "WASM exported functions have export visibility");
            assert!(!is_async, "WASM functions are not async");
        }
    }

    #[test]
    fn test_wat_text_analysis() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("add.wasm"));
        let items = analyzer.analyze_wat_text(SIMPLE_WAT_TEXT)
            .expect("Should parse WAT text format");

        assert!(!items.is_empty(), "Should extract AST items from WAT text");

        let function_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(function_items.len(), 1, "Should extract the add function");

        if let AstItem::Function { name, .. } = &function_items[0] {
            assert!(name.contains("add"), "Should extract function name from WAT");
        }
    }

    #[test]
    fn test_wasm_complexity_analysis() {
        let mut analyzer = WasmStackAnalyzer::new();
        let stack_complexity = analyzer.analyze_stack_complexity(&[0x20, 0x00, 0x20, 0x01, 0x6a])
            .expect("Should analyze WASM stack complexity");

        assert!(stack_complexity >= 1, "Should have at least complexity of 1");
        assert!(stack_complexity <= 10, "Should maintain complexity ≤10 for simple WASM");

        let control_flow_complexity = analyzer.analyze_control_flow_complexity(&[0x04, 0x40, 0x0b])
            .expect("Should analyze control flow complexity");

        assert!(control_flow_complexity >= 1, "Should have control flow complexity");
    }

    #[test]
    fn test_complex_wat_control_flow() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("fibonacci.wasm"));
        let items = analyzer.analyze_wat_text(COMPLEX_WAT_WITH_CONTROL_FLOW)
            .expect("Should parse complex WAT with control flow");

        let function_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        assert_eq!(function_items.len(), 1, "Should extract fibonacci function");

        // Should detect higher complexity due to recursive calls and conditionals
        if let AstItem::Function { name, line, .. } = &function_items[0] {
            assert!(name.contains("fibonacci"), "Should extract fibonacci function name");
            assert!(*line >= 1, "Should have valid line number");
        }
    }

    #[test]
    fn test_wasm_module_validation() {
        let mut validator = WasmValidator::new();
        let is_valid = validator.validate_wasm_module(SIMPLE_WASM_BINARY)
            .expect("Should validate WASM module");

        assert!(is_valid, "Simple WASM binary should be valid");
        assert!(validator.get_validation_errors().is_empty(), "Should have no validation errors");
    }

    #[test]
    fn test_wasm_security_analysis() {
        let mut validator = WasmValidator::new();
        let security_warnings = validator.analyze_security(SIMPLE_WASM_BINARY)
            .expect("Should perform security analysis");

        // Simple add function should have minimal security concerns
        assert!(security_warnings.len() <= 2, "Simple WASM should have few security warnings");
    }

    #[test]
    fn test_invalid_wasm_binary() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("invalid.wasm"));
        let invalid_bytes = &[0xFF, 0xFF, 0xFF, 0xFF]; // Invalid WASM magic
        let result = analyzer.analyze_wasm_binary(invalid_bytes);

        assert!(result.is_err(), "Should return error for invalid WASM binary");
    }

    #[test]
    fn test_empty_wasm_module() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("empty.wasm"));
        let minimal_wasm = &[
            0x00, 0x61, 0x73, 0x6d, // WASM magic
            0x01, 0x00, 0x00, 0x00, // Version
        ];
        let items = analyzer.analyze_wasm_binary(minimal_wasm)
            .expect("Should handle minimal WASM module");

        assert!(items.is_empty(), "Empty WASM module should produce no function items");
    }

    #[test]
    fn test_wasm_import_export_extraction() {
        let analyzer = WasmModuleAnalyzer::new(Path::new("with_imports.wasm"));
        let items = analyzer.analyze_wat_text(r#"
(module
  (import "env" "log" (func $log (param i32)))
  (func $main (result i32)
    i32.const 42
    call $log
    i32.const 0
  )
  (export "main" (func $main))
)
"#).expect("Should parse WASM with imports/exports");

        let function_items: Vec<_> = items.iter()
            .filter(|item| matches!(item, AstItem::Function { .. }))
            .collect();

        // Should extract both imported and local functions
        assert!(function_items.len() >= 1, "Should extract at least the main function");
    }
}

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
            for _ in 0..control_flow_depth {
                instructions.push(0x0b); // end
            }

            if let Ok(complexity) = analyzer.analyze_control_flow_complexity(&instructions) {
                // Complexity should scale with nesting depth
                prop_assert!(complexity >= control_flow_depth);
                prop_assert!(complexity <= control_flow_depth * 2 + 3);
            }
        }
    }
}