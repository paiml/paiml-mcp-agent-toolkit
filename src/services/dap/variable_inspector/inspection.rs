#![cfg_attr(coverage_nightly, coverage(off))]
//! Public inspection API methods for VariableInspector

use super::{Variable, VariableInspector};
use std::path::Path;
use tree_sitter::Parser;

impl VariableInspector {
    /// Inspect variables in Rust source at the given line
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn inspect_rust(&self, source: &str, line: usize) -> Result<Vec<Variable>, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| format!("Failed to set Rust language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse Rust source".to_string())?;

        self.extract_variables_rust(&tree, source, line)
    }

    /// Inspect variables in TypeScript source at the given line
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn inspect_typescript(&self, source: &str, line: usize) -> Result<Vec<Variable>, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .map_err(|e| format!("Failed to set TypeScript language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse TypeScript source".to_string())?;

        self.extract_variables_typescript(&tree, source, line)
    }

    /// Inspect variables in Python source at the given line
    #[cfg(feature = "python-ast")]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn inspect_python(&self, source: &str, line: usize) -> Result<Vec<Variable>, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| format!("Failed to set Python language: {}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "Failed to parse Python source".to_string())?;

        self.extract_variables_python(&tree, source, line)
    }

    #[cfg(not(feature = "python-ast"))]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Inspect python.
    pub fn inspect_python(&self, _source: &str, _line: usize) -> Result<Vec<Variable>, String> {
        Err("python-ast feature is disabled".to_string())
    }

    /// Inspect variables from a file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn inspect_file(&self, path: &Path, line: usize) -> Result<Vec<Variable>, String> {
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Detect language from file extension
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| "No file extension".to_string())?;

        match ext {
            "rs" => self.inspect_rust(&source, line),
            "ts" | "tsx" => self.inspect_typescript(&source, line),
            "js" | "jsx" => self.inspect_typescript(&source, line), // Use TS parser for JS
            "py" => self.inspect_python(&source, line),
            _ => Err(format!("Unsupported file extension: {}", ext)),
        }
    }
}
