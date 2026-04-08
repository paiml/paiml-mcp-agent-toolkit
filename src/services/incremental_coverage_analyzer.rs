#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Incremental code coverage analyzer for CI/CD pipelines
//!
//! This module provides efficient incremental coverage analysis by tracking
//! only changed code and its dependencies. It maintains persistent state across
//! runs to minimize recomputation and provides detailed coverage metrics for
//! modified code paths, enabling fast feedback in continuous integration.
//!
//! # Key Features
//!
//! - **Incremental Analysis**: Only analyzes changed files and their dependencies
//! - **Persistent State**: Caches AST and coverage data across runs
//! - **Call Graph Tracking**: Identifies which tests need re-running
//! - **Delta Coverage**: Reports coverage specifically for changed code
//! - **CI/CD Optimized**: Minimal overhead for build pipelines
//!
//! # Coverage Metrics
//!
//! - **Line Coverage**: Percentage of executed lines
//! - **Branch Coverage**: Percentage of executed branches
//! - **Function Coverage**: Percentage of called functions
//! - **Delta Coverage**: Coverage of newly added/modified code
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::incremental_coverage_analyzer::{
//!     IncrementalCoverageAnalyzer, CoverageConfig
//! };
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let analyzer = IncrementalCoverageAnalyzer::new(Path::new(".coverage_db"))?;
//!
//! // Analyze coverage for changed files
//! let changed_files = vec![
//!     Path::new("src/lib.rs"),
//!     Path::new("src/main.rs"),
//! ];
//!
//! let coverage = analyzer.analyze_incremental(&changed_files).await?;
//!
//! println!("Delta coverage: {:.1}%", coverage.delta_coverage.percentage);
//! println!("Files needing re-testing: {}", coverage.files_to_test.len());
//!
//! // Get impacted test list
//! for test in &coverage.impacted_tests {
//!     println!("Re-run test: {}", test);
//! }
//! # Ok(())
//! # }
//! ```ignore

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

/// Incremental Coverage Analysis with Persistent State
/// Optimized for CI/CD environments with minimal overhead
pub struct IncrementalCoverageAnalyzer {
    coverage_cache: Arc<DashMap<Vec<u8>, Vec<u8>>>, // Simple in-memory cache for now
    ast_cache: Arc<DashMap<FileId, (u64, AstNode)>>,
    call_graph: Arc<CallGraph>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileId {
    pub path: PathBuf,
    pub hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub functions: Vec<FunctionInfo>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub complexity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageUpdate {
    pub file_coverage: HashMap<FileId, FileCoverage>,
    pub aggregate_coverage: AggregateCoverage,
    pub delta_coverage: DeltaCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub covered_lines: Vec<usize>,
    pub total_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateCoverage {
    pub line_percentage: f64,
    pub branch_percentage: f64,
    pub function_percentage: f64,
    pub total_files: usize,
    pub covered_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaCoverage {
    pub new_lines_covered: usize,
    pub new_lines_total: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub modified_files: Vec<FileId>,
    pub added_files: Vec<FileId>,
    pub deleted_files: Vec<FileId>,
}

pub struct CallGraph {
    edges: DashMap<String, HashSet<String>>,
    reverse_edges: DashMap<String, HashSet<String>>,
}

// Core analysis: new(), analyze_changes(), compute_affected_files(),
// analyze_file_coverage(), compute_coverage(), compute_file_hash()
include!("incremental_coverage_analyzer_analysis.rs");

// File parsing: parse_file(), parse_rust_file(), parse_typescript_file(),
// parse_python_file(), extract_function_name()
include!("incremental_coverage_analyzer_parsing.rs");

// Cache operations, aggregation, Clone impl, CallGraph impl
include!("incremental_coverage_analyzer_cache.rs");

// Unit tests and property tests
include!("incremental_coverage_analyzer_tests.rs");
