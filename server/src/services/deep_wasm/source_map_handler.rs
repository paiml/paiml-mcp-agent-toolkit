//! Source Map Handler
//!
//! Parses JavaScript-style source maps adapted for WASM.
//! Supports both inline and external source maps.

use crate::services::deep_wasm::{DeepWasmError, DeepWasmResult, SourceMapEntry};
use sourcemap::SourceMap;
use std::path::Path;

/// Source map handler
pub struct SourceMapHandler;

impl SourceMapHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> DeepWasmResult<SourceMap> {
        let contents = std::fs::read_to_string(path)?;
        SourceMap::from_slice(contents.as_bytes())
            .map_err(|e| DeepWasmError::SourceMap(e.to_string()))
    }

    pub fn parse_mappings(&self, source_map: &SourceMap) -> Vec<SourceMapEntry> {
        let mut entries = Vec::new();

        for token in source_map.tokens() {
            if let Some(source) = token.get_source() {
                entries.push(SourceMapEntry {
                    generated_line: token.get_dst_line(),
                    generated_column: token.get_dst_col(),
                    original_line: token.get_src_line(),
                    original_column: token.get_src_col(),
                    source: source.to_string(),
                    name: token.get_name().map(|n| n.to_string()),
                });
            }
        }

        entries
    }
}

impl Default for SourceMapHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_creation() {
        let _handler = SourceMapHandler::new();
    }

    #[test]
    fn test_load_missing_file() {
        let handler = SourceMapHandler::new();
        let result = handler.load_from_file("/nonexistent/file.map");
        assert!(result.is_err());
    }
}
