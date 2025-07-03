//! WebAssembly binary format analyzer
//!
//! This module provides analysis capabilities for compiled WebAssembly (.wasm) files.

use anyhow::Result;
use std::path::Path;

use super::types::WasmMetrics;

/// WebAssembly binary analysis result
#[derive(Debug, Clone)]
pub struct WasmAnalysis {
    /// Parsed sections from the binary
    pub sections: Vec<WasmSection>,
}

/// WebAssembly section information
#[derive(Debug, Clone)]
pub struct WasmSection {
    /// Section ID
    pub id: u8,
    /// Section size in bytes
    pub size: usize,
}

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
            import_count: count_occurrences(&content, &[0x02]),   // Import section
            export_count: count_occurrences(&content, &[0x07]),   // Export section
            linear_memory_pages: if content.len() > 1000 { 1 } else { 0 },
            ..Default::default()
        };

        Ok(metrics)
    }

    /// Analyze raw WASM bytes
    pub fn analyze_bytes(&self, data: &[u8]) -> Result<WasmAnalysis> {
        // Check minimum size and magic bytes
        if data.len() < 8 {
            return Err(anyhow::anyhow!("File too small to be valid WASM"));
        }

        if &data[0..4] != b"\0asm" {
            return Err(anyhow::anyhow!("Invalid WASM magic number"));
        }

        let mut sections = Vec::new();
        let mut pos = 8; // Skip magic and version

        // Parse sections
        while pos < data.len() {
            if pos + 2 > data.len() {
                break;
            }

            let section_id = data[pos];
            pos += 1;

            // Decode LEB128 section size
            let mut size = 0u64;
            let mut shift = 0;
            loop {
                if pos >= data.len() {
                    break;
                }
                let byte = data[pos];
                pos += 1;

                size |= ((byte & 0x7F) as u64) << shift;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
                if shift > 35 {
                    return Err(anyhow::anyhow!("Invalid LEB128 encoding"));
                }
            }

            sections.push(WasmSection {
                id: section_id,
                size: size as usize,
            });

            // Skip section content
            pos += size as usize;
            if pos > data.len() {
                break;
            }
        }

        Ok(WasmAnalysis { sections })
    }
}

impl Default for WasmBinaryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Count occurrences of a byte pattern
pub fn count_occurrences(haystack: &[u8], needle: &[u8]) -> u32 {
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
