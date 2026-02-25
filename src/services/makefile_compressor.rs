#![cfg_attr(coverage_nightly, coverage(off))]
//! Makefile compression service for efficient context analysis
//!
//! This module intelligently compresses Makefiles by extracting and preserving
//! only the most critical information needed for understanding project structure
//! and build processes. It reduces large Makefiles to their essential components
//! while maintaining semantic meaning for AI analysis.
//!
//! # Compression Strategy
//!
//! The compressor identifies and preserves:
//! - **Critical Targets**: Common build targets (all, build, test, install, etc.)
//! - **Important Variables**: Project configuration and toolchain settings
//! - **Dependencies**: Target dependency chains and relationships
//! - **Key Commands**: Essential build commands and scripts
//!
//! # Features
//!
//! - **Smart Filtering**: Preserves only semantically important content
//! - **Dependency Analysis**: Maintains target dependency relationships
//! - **Variable Resolution**: Tracks variable definitions and usage
//! - **Comment Preservation**: Keeps critical documentation comments
//! - **Size Reduction**: Typically achieves 70-90% compression
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::makefile_compressor::MakefileCompressor;
//!
//! let compressor = MakefileCompressor::new();
//! let makefile_content = std::fs::read_to_string("Makefile").unwrap();
//!
//! let compressed = compressor.compress(&makefile_content);
//!
//! println!("Original size: {} bytes", makefile_content.len());
//! println!("Compressed size: {} bytes", compressed.content.len());
//! println!("Compression ratio: {:.1}%",
//!          (1.0 - compressed.content.len() as f64 / makefile_content.len() as f64) * 100.0);
//!
//! // Access extracted metadata
//! println!("Found {} targets", compressed.targets.len());
//! println!("Found {} variables", compressed.variables.len());
//!
//! for target in &compressed.targets {
//!     println!("Target: {} -> {:?}", target.name, target.deps);
//! }
//! ```ignore

use crate::models::project_meta::{CompressedMakefile, MakeTarget};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tracing::debug;

pub struct MakefileCompressor {
    critical_targets: HashSet<&'static str>,
    critical_vars: HashSet<&'static str>,
    var_pattern: Regex,
    target_pattern: Regex,
}

impl Default for MakefileCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct ParsedTarget {
    dependencies: Vec<String>,
    recipe: Vec<String>,
}

// Constructor, compress(), and is_critical_target()
include!("makefile_compressor_core.rs");

// parse_targets(), summarize_recipe(), detect_toolchain(), extract_dependencies()
include!("makefile_compressor_parsing.rs");

// Free helper functions for package name extraction
include!("makefile_compressor_helpers.rs");

// Unit tests and property tests
include!("makefile_compressor_tests.rs");
