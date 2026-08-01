//! AST Pattern Extraction
//!
//! Extracts patterns from AST using pmat context system

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::EntropyConfig;

// Types: PatternType, Location, AstPattern, PatternCollection, ProjectContext
include!("pattern_extractor_types.rs");

// Core: PatternExtractor struct, extract_patterns, get_project_context, scan_source_files
include!("pattern_extractor_core.rs");

// Rust pattern extraction methods (error handling, validation, resource mgmt, control flow, etc.)
include!("pattern_extractor_rust_patterns.rs");

// Utility methods: variation scores, hashing, normalization, structural grouping
include!("pattern_extractor_utils.rs");

// Ruchy-specific pattern extraction and variation score methods
include!("pattern_extractor_ruchy.rs");

// Tests extracted to pattern_extractor_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "pattern_extractor_tests.rs"]
mod tests;

// Additional ruchy pipeline tests split into their own file to keep
// pattern_extractor_tests.rs from growing past the 500-line gate.
#[cfg(test)]
#[path = "pattern_extractor_ruchy_pipeline_tests.rs"]
mod ruchy_pipeline_tests_mod;
