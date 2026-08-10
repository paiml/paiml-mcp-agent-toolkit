#![cfg_attr(coverage_nightly, coverage(off))]
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
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
        }
    }

    /// Analyze a WebAssembly binary file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_file(&self, file_path: &Path) -> Result<WasmMetrics> {
        let content = tokio::fs::read(file_path).await?;

        if content.len() > self.max_file_size {
            return Err(anyhow::anyhow!("File too large: {} bytes", content.len()));
        }

        // Check WASM magic bytes
        if content.len() < 8 || &content[0..4] != b"\0asm" {
            return Err(anyhow::anyhow!("Invalid WASM file format"));
        }

        // Counts come from the decoded section stream. They used to come from
        // byte frequencies over the whole file: function_count was "how many
        // 0x01 bytes are in this file", import_count the 0x02 bytes, export_count
        // the 0x07 bytes, and linear_memory_pages was literally `file size >
        // 1000`. A module of magic+version and nothing else reported 1 function,
        // and a hand-assembled module with exactly 1 function / 1 export
        // reported 7 functions and 2 imports.
        let sections = decode_sections(&content);

        let metrics = WasmMetrics {
            // Defined functions live in the Function section (id 3); imported
            // functions are counted in import_count, not here.
            function_count: vector_count_in_sections(&sections, SECTION_FUNCTION),
            import_count: vector_count_in_sections(&sections, SECTION_IMPORT),
            export_count: vector_count_in_sections(&sections, SECTION_EXPORT),
            linear_memory_pages: linear_memory_pages(&sections),
            memory_sections: section_occurrences(&sections, SECTION_MEMORY),
            ..Default::default()
        };

        Ok(metrics)
    }

    /// Analyze raw WASM bytes
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

                size |= u64::from(byte & 0x7F) << shift;
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

/// WebAssembly section ids used by the binary metrics.
const SECTION_IMPORT: u8 = 2;
const SECTION_FUNCTION: u8 = 3;
const SECTION_MEMORY: u8 = 5;
const SECTION_EXPORT: u8 = 7;

/// Decode an unsigned LEB128 integer at `pos`, advancing `pos`.
/// Returns `None` for a truncated or over-long encoding rather than guessing.
fn read_uleb128(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    let mut shift = 0u32;
    loop {
        let byte = *data.get(*pos)?;
        *pos += 1;
        value |= u64::from(byte & 0x7F) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }
}

/// Decode the section stream of a WASM module into `(id, payload)` pairs.
///
/// Decoding stops at the first section whose declared size does not fit in the
/// file: a truncated stream contributes only the sections that decoded cleanly,
/// so nothing is reported for bytes that were never parsed.
fn decode_sections(data: &[u8]) -> Vec<(u8, &[u8])> {
    let mut sections = Vec::new();
    let mut pos = 8; // magic (4) + version (4)

    while pos < data.len() {
        let id = data[pos];
        pos += 1;
        let Some(size) = read_uleb128(data, &mut pos) else {
            break;
        };
        let Ok(size) = usize::try_from(size) else {
            break;
        };
        let Some(end) = pos.checked_add(size) else {
            break;
        };
        if end > data.len() {
            break;
        }
        sections.push((id, &data[pos..end]));
        pos = end;
    }

    sections
}

/// Number of sections with the given id (0 when the module has none).
fn section_occurrences(sections: &[(u8, &[u8])], id: u8) -> u32 {
    sections.iter().filter(|(sid, _)| *sid == id).count() as u32
}

/// Every non-custom section body starts with a LEB128 vector length; sum those
/// lengths for the requested section id. A section whose length cannot be
/// decoded contributes nothing.
fn vector_count_in_sections(sections: &[(u8, &[u8])], id: u8) -> u32 {
    sections
        .iter()
        .filter(|(sid, _)| *sid == id)
        .filter_map(|(_, payload)| {
            let mut pos = 0;
            read_uleb128(payload, &mut pos)
        })
        .map(|count| u32::try_from(count).unwrap_or(u32::MAX))
        .fold(0u32, u32::saturating_add)
}

/// Initial linear memory size in 64KiB pages, summed over the memories the
/// Memory section actually declares. A module with no Memory section has no
/// linear memory, so this is 0 — not a function of the file's size.
fn linear_memory_pages(sections: &[(u8, &[u8])]) -> u32 {
    let mut pages = 0u32;
    for (_, payload) in sections.iter().filter(|(id, _)| *id == SECTION_MEMORY) {
        let mut pos = 0;
        let Some(count) = read_uleb128(payload, &mut pos) else {
            continue;
        };
        for _ in 0..count {
            // limits := flags:u8 min:u32 [max:u32]
            let Some(flags) = read_uleb128(payload, &mut pos) else {
                break;
            };
            let Some(min) = read_uleb128(payload, &mut pos) else {
                break;
            };
            pages = pages.saturating_add(u32::try_from(min).unwrap_or(u32::MAX));
            if flags & 0x01 != 0 && read_uleb128(payload, &mut pos).is_none() {
                break;
            }
        }
    }
    pages
}

/// Count occurrences of a byte pattern
/// Counts non-overlapping occurrences of a byte pattern in data
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::wasm::binary::count_occurrences;
///
/// let data = b"hello world hello";
/// let pattern = b"hello";
///
/// let count = count_occurrences(data, pattern);
/// assert_eq!(count, 2);
///
/// // Pattern larger than data returns 0
/// assert_eq!(count_occurrences(b"hi", b"hello"), 0);
///
/// // Single byte pattern
/// assert_eq!(count_occurrences(b"aaa", b"a"), 3);
/// ```
#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;

    /// A module of magic+version and nothing else declares nothing. This test
    /// used to assert `function_count == 1`, which was the byte-frequency count
    /// of the 0x01 version byte, not a function.
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
        assert_eq!(metrics.function_count, 0);
        assert_eq!(metrics.import_count, 0);
        assert_eq!(metrics.export_count, 0);
        assert_eq!(metrics.linear_memory_pages, 0);
    }

    /// Hand-assembled module: 1 type, 1 function, 1 export "f", no imports,
    /// no memory. Byte counting reported 7 functions / 2 imports / 1 export
    /// for this module.
    fn one_function_module() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, // magic + version
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // type: 1 × () -> ()
            0x03, 0x02, 0x01, 0x00, // function: 1 function, type 0
            0x07, 0x05, 0x01, 0x01, 0x66, 0x00, 0x00, // export: 1 × "f" func 0
            0x0A, 0x04, 0x01, 0x02, 0x00, 0x0B, // code
        ]
    }

    #[tokio::test]
    async fn test_analyze_file_counts_decoded_sections_not_bytes() {
        let analyzer = WasmBinaryAnalyzer::new();
        let temp_file = NamedTempFile::new().unwrap();
        tokio::fs::write(temp_file.path(), one_function_module())
            .await
            .unwrap();

        let metrics = analyzer.analyze_file(temp_file.path()).await.unwrap();
        assert_eq!(metrics.function_count, 1);
        assert_eq!(metrics.import_count, 0);
        assert_eq!(metrics.export_count, 1);
        assert_eq!(metrics.linear_memory_pages, 0);
        assert_eq!(metrics.memory_sections, 0);
    }

    /// Padding a sectionless module with 4KB of zeros must not conjure a page of
    /// linear memory: `linear_memory_pages` used to be `file size > 1000`.
    #[tokio::test]
    async fn test_analyze_file_memory_pages_come_from_the_memory_section() {
        let analyzer = WasmBinaryAnalyzer::new();

        let mut padded = b"\0asm\x01\x00\x00\x00".to_vec();
        padded.extend(vec![0u8; 4096]);
        let padded_file = NamedTempFile::new().unwrap();
        tokio::fs::write(padded_file.path(), &padded).await.unwrap();
        let metrics = analyzer.analyze_file(padded_file.path()).await.unwrap();
        assert_eq!(metrics.linear_memory_pages, 0);

        // Same module plus a Memory section declaring min = 3 pages.
        let mut with_memory = b"\0asm\x01\x00\x00\x00".to_vec();
        with_memory.extend([0x05, 0x03, 0x01, 0x00, 0x03]);
        let mem_file = NamedTempFile::new().unwrap();
        tokio::fs::write(mem_file.path(), &with_memory)
            .await
            .unwrap();
        let metrics = analyzer.analyze_file(mem_file.path()).await.unwrap();
        assert_eq!(metrics.linear_memory_pages, 3);
        assert_eq!(metrics.memory_sections, 1);
    }

    #[test]
    fn test_decode_sections_stops_at_truncated_section() {
        // Type section declares 4 payload bytes but only 2 are present.
        let data = b"\0asm\x01\x00\x00\x00\x01\x04\x60\x00";
        let sections = decode_sections(data);
        assert!(
            sections.is_empty(),
            "a section that runs off the end of the file must not be decoded"
        );
    }

    #[test]
    fn test_decode_sections_payload_boundaries() {
        let module = one_function_module();
        let sections = decode_sections(&module);
        let ids: Vec<u8> = sections.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec![1, 3, 7, 10]);
        assert_eq!(vector_count_in_sections(&sections, SECTION_FUNCTION), 1);
        assert_eq!(vector_count_in_sections(&sections, SECTION_IMPORT), 0);
        assert_eq!(vector_count_in_sections(&sections, SECTION_EXPORT), 1);
    }

    #[test]
    fn test_read_uleb128_multibyte_and_truncated() {
        let mut pos = 0;
        assert_eq!(read_uleb128(&[0xE5, 0x8E, 0x26], &mut pos), Some(624_485));
        assert_eq!(pos, 3);

        let mut pos = 0;
        assert_eq!(read_uleb128(&[0x80], &mut pos), None);
    }

    #[test]
    fn test_count_occurrences() {
        let data = b"\x01\x02\x01\x03\x01";
        let count = count_occurrences(data, &[0x01]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_occurrences_empty_haystack() {
        assert_eq!(count_occurrences(b"", &[0x01]), 0);
    }

    #[test]
    fn test_count_occurrences_needle_larger_than_haystack() {
        assert_eq!(count_occurrences(b"hi", b"hello"), 0);
    }

    #[test]
    fn test_count_occurrences_no_match() {
        assert_eq!(count_occurrences(b"hello world", b"xyz"), 0);
    }

    #[test]
    fn test_count_occurrences_multi_byte() {
        let data = b"hello world hello";
        assert_eq!(count_occurrences(data, b"hello"), 2);
    }

    #[test]
    fn test_wasm_binary_analyzer_default() {
        let analyzer = WasmBinaryAnalyzer::default();
        assert_eq!(analyzer.max_file_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_analyze_bytes_valid() {
        let analyzer = WasmBinaryAnalyzer::new();
        // Minimal valid WASM with no sections
        let data = b"\0asm\x01\x00\x00\x00";
        let result = analyzer.analyze_bytes(data);
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.sections.is_empty());
    }

    #[test]
    fn test_analyze_bytes_with_section() {
        let analyzer = WasmBinaryAnalyzer::new();
        // WASM with a type section (id=1, size=0)
        let data = b"\0asm\x01\x00\x00\x00\x01\x00";
        let result = analyzer.analyze_bytes(data);
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.sections.len(), 1);
        assert_eq!(analysis.sections[0].id, 1);
        assert_eq!(analysis.sections[0].size, 0);
    }

    #[test]
    fn test_analyze_bytes_too_small() {
        let analyzer = WasmBinaryAnalyzer::new();
        let data = b"\0asm";
        let result = analyzer.analyze_bytes(data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_analyze_bytes_invalid_magic() {
        let analyzer = WasmBinaryAnalyzer::new();
        let data = b"invalid\x00";
        let result = analyzer.analyze_bytes(data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("magic number"));
    }

    #[test]
    fn test_wasm_section_clone() {
        let section = WasmSection { id: 5, size: 100 };
        let cloned = section.clone();
        assert_eq!(cloned.id, 5);
        assert_eq!(cloned.size, 100);
    }

    #[test]
    fn test_wasm_analysis_clone() {
        let analysis = WasmAnalysis {
            sections: vec![WasmSection { id: 1, size: 10 }],
        };
        let cloned = analysis.clone();
        assert_eq!(cloned.sections.len(), 1);
    }

    #[tokio::test]
    async fn test_analyze_file_invalid_format() {
        let analyzer = WasmBinaryAnalyzer::new();
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
        file.write_all(b"not wasm content").await.unwrap();
        file.flush().await.unwrap();

        let result = analyzer.analyze_file(temp_file.path()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid WASM"));
    }
}
