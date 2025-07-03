//! WebAssembly language detection
//!
//! This module provides language detection for WebAssembly variants.

/// WebAssembly language detector
pub struct WasmLanguageDetector;

impl WasmLanguageDetector {
    /// Create a new language detector
    pub fn new() -> Self {
        Self
    }

    /// Detect if content is AssemblyScript
    pub fn is_assemblyscript(&self, content: &str) -> bool {
        // Check for AssemblyScript-specific keywords and patterns
        content.contains("@global")
            || content.contains("@inline")
            || content.contains("@external")
            || content.contains("i32")
            || content.contains("f64")
            || content.contains("memory.")
            || (content.contains("export") && content.contains("function"))
    }

    /// Detect if content is WebAssembly Text Format
    pub fn is_wat(&self, content: &str) -> bool {
        content.trim_start().starts_with('(')
            && (content.contains("module") || content.contains("func"))
    }

    /// Detect if binary data is WebAssembly
    pub fn is_wasm_binary(&self, data: &[u8]) -> bool {
        data.len() >= 8 && &data[0..4] == b"\0asm"
    }
}

impl Default for WasmLanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemblyscript_detection() {
        let detector = WasmLanguageDetector::new();

        let as_content = "export function test(): i32 { return 42; }";
        assert!(detector.is_assemblyscript(as_content));

        let js_content = "function test() { return 42; }";
        assert!(!detector.is_assemblyscript(js_content));
    }

    #[test]
    fn test_wat_detection() {
        let detector = WasmLanguageDetector::new();

        let wat_content = "(module (func $test (result i32) i32.const 42))";
        assert!(detector.is_wat(wat_content));

        let js_content = "function test() { return 42; }";
        assert!(!detector.is_wat(js_content));
    }

    #[test]
    fn test_wasm_binary_detection() {
        let detector = WasmLanguageDetector::new();

        let wasm_data = b"\0asm\x01\x00\x00\x00";
        assert!(detector.is_wasm_binary(wasm_data));

        let text_data = b"not wasm binary";
        assert!(!detector.is_wasm_binary(text_data));
    }
}
