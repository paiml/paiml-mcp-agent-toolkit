#![cfg_attr(coverage_nightly, coverage(off))]
//! Sovereign Stack Compression Adapter
//!
//! Wraps trueno-zram-core's PAGE_SIZE-based compression API with a variable-length
//! interface compatible with lz4_flex.
//!
//! # Architecture
//!
//! trueno-zram-core is optimized for 4KB pages (zram use case). This adapter:
//! 1. For small data (≤4KB): Pads to PAGE_SIZE, compresses, stores original length
//! 2. For large data (>4KB): Chunks into pages, compresses each, combines with header
//!
//! # Performance
//!
//! - 2-4x speedup on AVX2/AVX-512 (auto-detected at runtime)
//! - NEON acceleration on ARM64
//! - Fallback to scalar on other platforms

use std::io;

/// Page size used by trueno-zram-core (4KB)
pub const PAGE_SIZE: usize = 4096;

/// Magic bytes for compressed data identification
#[cfg(feature = "sovereign-compression")]
const MAGIC: [u8; 4] = [b'T', b'Z', b'R', b'C']; // Trueno Zram Compressed

/// Header for variable-length compressed data
/// Format: MAGIC (4) + original_len (4) + num_pages (4) + page_sizes (num_pages * 4)
#[cfg(feature = "sovereign-compression")]
#[derive(Debug, Clone)]
struct CompressionHeader {
    original_len: u32,
    num_pages: u32,
    page_sizes: Vec<u32>,
}

#[cfg(feature = "sovereign-compression")]
impl CompressionHeader {
    fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(12 + self.page_sizes.len() * 4);
        result.extend_from_slice(&MAGIC);
        result.extend_from_slice(&self.original_len.to_le_bytes());
        result.extend_from_slice(&self.num_pages.to_le_bytes());
        for &size in &self.page_sizes {
            result.extend_from_slice(&size.to_le_bytes());
        }
        result
    }

    fn decode(data: &[u8]) -> io::Result<(Self, usize)> {
        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Header too short",
            ));
        }

        if &data[0..4] != &MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic bytes - not trueno-zram-core compressed data",
            ));
        }

        let original_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let num_pages = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

        let header_size = 12 + num_pages as usize * 4;
        if data.len() < header_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Header incomplete",
            ));
        }

        let mut page_sizes = Vec::with_capacity(num_pages as usize);
        for i in 0..num_pages as usize {
            let offset = 12 + i * 4;
            let size = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            page_sizes.push(size);
        }

        Ok((
            Self {
                original_len,
                num_pages,
                page_sizes,
            },
            header_size,
        ))
    }
}

/// Compress variable-length data using trueno-zram-core SIMD
///
/// This is the sovereign stack replacement for lz4_flex::compress_prepend_size.
///
/// # Arguments
/// * `input` - Data to compress (any length)
///
/// # Returns
/// Compressed data with header for decompression
#[cfg(feature = "sovereign-compression")]
pub fn compress(input: &[u8]) -> io::Result<Vec<u8>> {
    use trueno_zram_core::lz4;

    if input.is_empty() {
        // Empty input: just return header with 0 pages
        let header = CompressionHeader {
            original_len: 0,
            num_pages: 0,
            page_sizes: vec![],
        };
        return Ok(header.encode());
    }

    let num_pages = (input.len() + PAGE_SIZE - 1) / PAGE_SIZE;
    let mut page_sizes = Vec::with_capacity(num_pages);
    let mut compressed_pages = Vec::new();

    for chunk in input.chunks(PAGE_SIZE) {
        // Pad to PAGE_SIZE if needed
        let mut page = [0u8; PAGE_SIZE];
        page[..chunk.len()].copy_from_slice(chunk);

        // Compress using SIMD-accelerated LZ4
        let compressed = lz4::compress(&page)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e}")))?;
        page_sizes.push(compressed.len() as u32);
        compressed_pages.extend_from_slice(&compressed);
    }

    let header = CompressionHeader {
        original_len: input.len() as u32,
        num_pages: num_pages as u32,
        page_sizes,
    };

    let mut result = header.encode();
    result.extend_from_slice(&compressed_pages);
    Ok(result)
}

/// Decompress data compressed with trueno-zram-core
///
/// This is the sovereign stack replacement for lz4_flex::decompress_size_prepended.
///
/// # Arguments
/// * `input` - Compressed data with header
///
/// # Returns
/// Original uncompressed data
#[cfg(feature = "sovereign-compression")]
pub fn decompress(input: &[u8]) -> io::Result<Vec<u8>> {
    use trueno_zram_core::lz4;

    let (header, header_size) = CompressionHeader::decode(input)?;

    if header.original_len == 0 {
        return Ok(Vec::new());
    }

    let mut result = Vec::with_capacity(header.original_len as usize);
    let mut offset = header_size;

    for (i, &compressed_size) in header.page_sizes.iter().enumerate() {
        let compressed_end = offset + compressed_size as usize;
        if compressed_end > input.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Page {} compressed data extends beyond input (offset={}, size={}, input_len={})",
                    i, offset, compressed_size, input.len()
                ),
            ));
        }

        let compressed = &input[offset..compressed_end];

        // Decompress to PAGE_SIZE buffer
        let mut page = [0u8; PAGE_SIZE];
        let decompressed_len = lz4::decompress(compressed, &mut page)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e}")))?;

        // Calculate how much of this page contains actual data
        let remaining = header.original_len as usize - result.len();
        let take = remaining.min(decompressed_len).min(PAGE_SIZE);
        result.extend_from_slice(&page[..take]);

        offset = compressed_end;
    }

    Ok(result)
}

/// Fallback implementation using lz4_flex when sovereign-compression is disabled
#[cfg(not(feature = "sovereign-compression"))]
pub fn compress(input: &[u8]) -> io::Result<Vec<u8>> {
    Ok(lz4_flex::compress_prepend_size(input))
}

/// Fallback implementation using lz4_flex when sovereign-compression is disabled
#[cfg(not(feature = "sovereign-compression"))]
pub fn decompress(input: &[u8]) -> io::Result<Vec<u8>> {
    lz4_flex::decompress_size_prepended(input)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_empty() {
        let data = b"";
        let compressed = compress(data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_roundtrip_small() {
        let data = b"Hello, sovereign stack!";
        let compressed = compress(data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_roundtrip_page_size() {
        let data = vec![0x42u8; PAGE_SIZE];
        let compressed = compress(&data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_roundtrip_multi_page() {
        let data = vec![0x42u8; PAGE_SIZE * 3 + 1234];
        let compressed = compress(&data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_roundtrip_random() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Generate pseudo-random data
        let mut data = Vec::with_capacity(10000);
        for i in 0..10000 {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            data.push(hasher.finish() as u8);
        }

        let compressed = compress(&data).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compression_ratio() {
        // Highly compressible data
        let data = vec![0u8; PAGE_SIZE * 2];
        let compressed = compress(&data).unwrap();

        // Should achieve good compression
        assert!(
            compressed.len() < data.len(),
            "Compressed {} bytes -> {} bytes (no compression!)",
            data.len(),
            compressed.len()
        );
    }

    #[test]
    fn test_header_decode_invalid_magic() {
        let bad_data = [0u8; 16];
        let result = decompress(&bad_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_header_decode_too_short() {
        let result = decompress(&[0, 1, 2]);
        assert!(result.is_err());
    }
}
