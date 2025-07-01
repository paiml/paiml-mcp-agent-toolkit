//! Parallel WebAssembly analysis
//!
//! This module provides parallel processing capabilities for analyzing
//! multiple WebAssembly files efficiently using thread pools.
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

use super::error::{WasmError, WasmResult};
use super::language_detection::WasmLanguageDetector;
use super::memory_pool::{ParserType, WasmParserPool};
use super::types::{WasmComplexity, WasmMetrics, WebAssemblyVariant};

/// Configuration for parallel analysis
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads (0 = number of CPUs)
    pub thread_count: usize,

    /// Chunk size for batch processing
    pub chunk_size: usize,

    /// Maximum depth for directory traversal
    pub max_depth: usize,

    /// File size threshold for sequential processing
    pub sequential_threshold: usize,

    /// Enable progress reporting
    pub enable_progress: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            thread_count: 0, // Use all CPUs
            chunk_size: 100,
            max_depth: 10,
            sequential_threshold: 10 * 1_024 * 1_024, // 10MB
            enable_progress: false,
        }
    }
}

/// `Result` of analyzing a single file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileAnalysisResult {
    pub path: PathBuf,
    pub variant: WebAssemblyVariant,
    pub metrics: WasmMetrics,
    pub complexity: WasmComplexity,
    pub size_bytes: u64,
    pub parse_time_ms: u64,
    pub errors: Vec<String>,
}

/// Aggregated analysis results
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AggregatedAnalysis {
    pub total_files: usize,
    pub successful_analyses: usize,
    pub failed_analyses: usize,
    pub total_functions: u32,
    pub total_imports: u32,
    pub total_exports: u32,
    pub average_complexity: f32,
    pub total_parse_time_ms: u64,
    pub file_results: Vec<FileAnalysisResult>,
    pub errors_by_type: HashMap<String, usize>,
}

/// Parallel WebAssembly analyzer
pub struct ParallelWasmAnalyzer {
    config: ParallelConfig,
    parser_pool: Arc<Mutex<WasmParserPool>>,
    detector: WasmLanguageDetector,
}

impl ParallelWasmAnalyzer {
    ///
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - May panic on out-of-bounds array/slice access
    /// - Panics if the value is `None` or Err
    /// Create a new parallel analyzer
//! Parallel WebAssembly analysis
//!
//! This module provides parallel processing capabilities for analyzing
//! multiple WebAssembly files efficiently using thread pools.
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

use super::error::{WasmError, WasmResult};
use super::language_detection::WasmLanguageDetector;
use super::memory_pool::{ParserType, WasmParserPool};
use super::types::{WasmComplexity, WasmMetrics, WebAssemblyVariant};

/// Configuration for parallel analysis
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads (0 = number of CPUs)
    pub thread_count: usize,

    /// Chunk size for batch processing
    pub chunk_size: usize,

    /// Maximum depth for directory traversal
    pub max_depth: usize,

    /// File size threshold for sequential processing
    pub sequential_threshold: usize,

    /// Enable progress reporting
    pub enable_progress: bool,
}
