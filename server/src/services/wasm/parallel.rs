//! Parallel WebAssembly analysis
//!
//! This module provides parallel processing capabilities for analyzing
//! multiple WebAssembly files efficiently using thread pools.

use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

use super::language_detection::WasmLanguageDetector;

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

/// Result of analyzing a single file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileAnalysisResult {
    pub path: PathBuf,
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
    pub total_parse_time_ms: u64,
    pub file_results: Vec<FileAnalysisResult>,
    pub errors_by_type: HashMap<String, usize>,
}

/// Parallel WebAssembly analyzer
pub struct ParallelWasmAnalyzer {
    _config: ParallelConfig,
    _detector: WasmLanguageDetector,
}

impl ParallelWasmAnalyzer {
    /// Create a new parallel analyzer
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            _config: config,
            _detector: WasmLanguageDetector::new(),
        }
    }

    /// Analyze files in parallel
    pub async fn analyze_directory(&self, dir_path: &Path) -> Result<AggregatedAnalysis> {
        let _start_time = Instant::now();
        let mut aggregated = AggregatedAnalysis::default();

        // Find all relevant files
        let files: Vec<PathBuf> = WalkDir::new(dir_path)
            .max_depth(self._config.max_depth)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.path().to_path_buf())
            .filter(|path| self.is_relevant_file(path))
            .collect();

        aggregated.total_files = files.len();

        // Process files in parallel
        let results: Vec<FileAnalysisResult> = files
            .into_par_iter()
            .map(|path| self.analyze_file(&path))
            .collect();

        // Aggregate results
        for result in results {
            if result.errors.is_empty() {
                aggregated.successful_analyses += 1;
            } else {
                aggregated.failed_analyses += 1;
                for error in &result.errors {
                    *aggregated.errors_by_type.entry(error.clone()).or_insert(0) += 1;
                }
            }
            aggregated.total_parse_time_ms += result.parse_time_ms;
            aggregated.file_results.push(result);
        }

        Ok(aggregated)
    }

    /// Analyze a single file
    fn analyze_file(&self, path: &Path) -> FileAnalysisResult {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut size_bytes = 0;

        // Get file size
        if let Ok(metadata) = std::fs::metadata(path) {
            size_bytes = metadata.len();
        } else {
            errors.push("Failed to read file metadata".to_string());
        }

        // Basic analysis (simplified for compilation)
        if let Err(e) = std::fs::read_to_string(path) {
            errors.push(format!("Failed to read file: {}", e));
        }

        let parse_time_ms = start_time.elapsed().as_millis() as u64;

        FileAnalysisResult {
            path: path.to_path_buf(),
            size_bytes,
            parse_time_ms,
            errors,
        }
    }

    /// Check if file is relevant for analysis
    fn is_relevant_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            matches!(extension.to_str(), Some("wasm" | "wat" | "ts"))
        } else {
            false
        }
    }
}

impl Default for ParallelWasmAnalyzer {
    fn default() -> Self {
        Self::new(ParallelConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_parallel_analyzer() {
        let analyzer = ParallelWasmAnalyzer::default();
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test.wasm"), b"\0asm\x01\x00\x00\x00").unwrap();
        fs::write(temp_dir.path().join("test.wat"), "(module)").unwrap();

        let result = analyzer.analyze_directory(temp_dir.path()).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.total_files, 2);
    }

    #[test]
    fn test_file_relevance() {
        let analyzer = ParallelWasmAnalyzer::default();

        assert!(analyzer.is_relevant_file(Path::new("test.wasm")));
        assert!(analyzer.is_relevant_file(Path::new("test.wat")));
        assert!(analyzer.is_relevant_file(Path::new("test.ts")));
        assert!(!analyzer.is_relevant_file(Path::new("test.txt")));
    }
}
