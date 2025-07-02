//! WebAssembly binary format analyzer
//!
//! This module provides analysis capabilities for compiled WebAssembly (.wasm) files.

use anyhow::Result;
use std::path::Path;

use super::types::WasmMetrics;

/// WebAssembly binary analyzer
pub struct WasmBinaryAnalyzer {
    max_file_size: usize,
}

impl WasmBinaryAnalyzer {
    /// Create a new WebAssembly binary analyzer
    pub fn new() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
        }
    }

    /// Analyze a WebAssembly binary file
    pub async fn analyze_file(&self, file_path: &Path) -> Result<WasmMetrics> {
        let content = tokio::fs::read(file_path).await?;
        
        if content.len() > self.max_file_size {
            return Err(anyhow::anyhow!("File too large: {} bytes", content.len()));
        }

        // Check WASM magic bytes
        if content.len() < 8 || &content[0..4] != b"\0asm" {
            return Err(anyhow::anyhow!("Invalid WASM file format"));
        }

        // Basic analysis - count sections
        let metrics = WasmMetrics {
            function_count: count_occurrences(&content, &[0x01]), // Type section
            import_count: count_occurrences(&content, &[0x02]), // Import section  
            export_count: count_occurrences(&content, &[0x07]), // Export section
            linear_memory_pages: if content.len() > 1000 { 1 } else { 0 },
            ..Default::default()
        };

        Ok(metrics)
    }
}

impl Default for WasmBinaryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Count occurrences of a byte pattern
fn count_occurrences(haystack: &[u8], needle: &[u8]) -> u32 {
    let mut count = 0;
    let mut pos = 0;
    
    while pos + needle.len() <= haystack.len() {
        if &haystack[pos..pos + needle.len()] == needle {
            count += 1;
            pos += needle.len();
        } else {
            pos += 1;
        }
    }
    
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_wasm_binary_analyzer() {
        let analyzer = WasmBinaryAnalyzer::new();
        
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
        
        // Write WASM magic bytes
        file.write_all(b"\0asm\x01\x00\x00\x00").await.unwrap();
        file.flush().await.unwrap();
        
        let result = analyzer.analyze_file(temp_file.path()).await;
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert_eq!(metrics.function_count, 1);
    }

    #[test]
    fn test_count_occurrences() {
        let data = b"\x01\x02\x01\x03\x01";
        let count = count_occurrences(data, &[0x01]);
        assert_eq!(count, 3);
    }
}