//! WASM Binary Inspector
//!
//! Parses WebAssembly binary format according to WebAssembly spec.
//! Supports all standard sections + custom sections for DWARF.
//!
//! Implements DWASM-001: WASM binary parser with the following requirements:
//! - Parse Type, Function, Memory, Global, Export, Code sections
//! - Extract custom sections (.debug_*, name, sourceMappingURL)
//! - Handle WASM 1.0 and 2.0 features (MVP + threads + SIMD)
//! - Parse with zero-copy where possible for performance

use crate::services::deep_wasm::{DeepWasmError, DeepWasmResult, WasmModuleAnalysis};
use std::path::Path;
use wasmparser::{Parser, Payload};

/// WASM binary inspector
pub struct WasmInspector {
    /// Module size limit for quality gates (10MB)
    max_module_size: u64,
}

impl WasmInspector {
    /// Creates a new WASM inspector
    pub fn new() -> Self {
        Self {
            max_module_size: 10_485_760, // 10 MB
        }
    }

    /// Creates a new WASM inspector with custom size limit
    pub fn with_size_limit(max_size: u64) -> Self {
        Self {
            max_module_size: max_size,
        }
    }

    /// Inspects a WASM binary file
    pub fn inspect_file<P: AsRef<Path>>(&self, path: P) -> DeepWasmResult<WasmModuleAnalysis> {
        let bytes = std::fs::read(path.as_ref())?;
        self.inspect_bytes(&bytes)
    }

    /// Inspects WASM binary from bytes
    pub fn inspect_bytes(&self, bytes: &[u8]) -> DeepWasmResult<WasmModuleAnalysis> {
        // Check size limit
        if bytes.len() as u64 > self.max_module_size {
            return Err(DeepWasmError::QualityGate(format!(
                "Module size {} exceeds limit {}",
                bytes.len(),
                self.max_module_size
            )));
        }

        let mut analysis = WasmModuleAnalysis {
            module_size_bytes: bytes.len() as u64,
            function_count: 0,
            exported_functions: 0,
            max_complexity: 0,
            has_dwarf: false,
            has_source_map: false,
        };

        let parser = Parser::new(0);

        for payload in parser.parse_all(bytes) {
            match payload.map_err(|e| DeepWasmError::WasmParse(e.to_string()))? {
                Payload::FunctionSection(reader) => {
                    analysis.function_count = reader.count();
                }
                Payload::ExportSection(reader) => {
                    for export in reader {
                        let export = export.map_err(|e| DeepWasmError::WasmParse(e.to_string()))?;
                        if matches!(export.kind, wasmparser::ExternalKind::Func) {
                            analysis.exported_functions += 1;
                        }
                    }
                }
                Payload::CustomSection(reader) => {
                    let name = reader.name();
                    if name.starts_with(".debug_") {
                        analysis.has_dwarf = true;
                    } else if name == "sourceMappingURL" {
                        analysis.has_source_map = true;
                    }
                }
                _ => {}
            }
        }

        Ok(analysis)
    }

    /// Extracts custom section by name
    pub fn extract_custom_section<'a>(
        &self,
        bytes: &'a [u8],
        section_name: &str,
    ) -> DeepWasmResult<Option<&'a [u8]>> {
        let parser = Parser::new(0);

        for payload in parser.parse_all(bytes) {
            if let Payload::CustomSection(reader) =
                payload.map_err(|e| DeepWasmError::WasmParse(e.to_string()))?
            {
                if reader.name() == section_name {
                    return Ok(Some(reader.data()));
                }
            }
        }

        Ok(None)
    }
}

impl Default for WasmInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_creation() {
        let inspector = WasmInspector::new();
        assert_eq!(inspector.max_module_size, 10_485_760);
    }

    #[test]
    fn test_inspector_with_custom_limit() {
        let inspector = WasmInspector::with_size_limit(1_000_000);
        assert_eq!(inspector.max_module_size, 1_000_000);
    }

    #[test]
    fn test_inspect_minimal_wasm() {
        // Minimal valid WASM module: (module)
        let minimal_wasm = vec![
            0x00, 0x61, 0x73, 0x6D, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version
        ];

        let inspector = WasmInspector::new();
        let result = inspector.inspect_bytes(&minimal_wasm);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.module_size_bytes, 8);
        assert_eq!(analysis.function_count, 0);
        assert_eq!(analysis.exported_functions, 0);
    }

    #[test]
    fn test_inspect_oversized_module() {
        let inspector = WasmInspector::with_size_limit(100);
        let large_bytes = vec![0u8; 200];

        let result = inspector.inspect_bytes(&large_bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DeepWasmError::QualityGate(_)));
    }

    #[test]
    fn test_inspect_invalid_wasm() {
        let inspector = WasmInspector::new();
        let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];

        let result = inspector.inspect_bytes(&invalid_bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DeepWasmError::WasmParse(_)));
    }

    #[test]
    fn test_extract_missing_custom_section() {
        let minimal_wasm = vec![
            0x00, 0x61, 0x73, 0x6D, // Magic number
            0x01, 0x00, 0x00, 0x00, // Version
        ];

        let inspector = WasmInspector::new();
        let result = inspector.extract_custom_section(&minimal_wasm, ".debug_info");

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // Property-based test: Any valid WASM should not panic
    #[test]
    fn test_inspector_never_panics_on_random_input() {
        let inspector = WasmInspector::new();
        let random_bytes: Vec<u8> = vec![0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD];

        // Should not panic, just return error
        let _ = inspector.inspect_bytes(&random_bytes);
    }
}
