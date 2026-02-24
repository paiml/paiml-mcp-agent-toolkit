//! AST Pattern Extraction
//!
//! Extracts patterns from AST using pmat context system

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::EntropyConfig;

// Types: PatternType, Location, AstPattern, PatternCollection, ProjectContext
include!("pattern_extractor_types.rs");

// Core: PatternExtractor struct, extract_patterns, get_project_context, scan_directory_fallback
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
