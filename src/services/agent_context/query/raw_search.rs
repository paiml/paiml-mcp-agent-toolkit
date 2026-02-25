#![cfg_attr(coverage_nightly, coverage(off))]
//! Raw file search — rg-compatible line-level search across all project files.
//!
//! Used as a fallback/complement to the AST-based function index when:
//! - Searching non-code files (TOML, YAML, Markdown, etc.)
//! - Finding module-level items (use, const, static, impl blocks)
//! - User wants pure line-level grep-like results (`--raw` mode)

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A single line match from raw file search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSearchResult {
    /// File path relative to project root
    pub file_path: String,
    /// 1-based line number
    pub line_number: usize,
    /// The matching line content (trimmed trailing newline)
    pub line_content: String,
    /// Context lines before the match
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_before: Vec<String>,
    /// Context lines after the match
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_after: Vec<String>,
}

/// Options for raw search
pub struct RawSearchOptions<'a> {
    /// Regex pattern to search for
    pub pattern: &'a str,
    /// Whether to treat pattern as literal (no regex)
    pub literal: bool,
    /// Case-insensitive search
    pub case_insensitive: bool,
    /// Lines of context before match
    pub before_context: usize,
    /// Lines of context after match
    pub after_context: usize,
    /// Maximum results to return
    pub limit: usize,
    /// Filter to files matching this language extension
    pub language_filter: Option<&'a str>,
    /// Exclude files matching this glob pattern
    pub exclude_file_pattern: Option<&'a str>,
    /// Exclude results matching this content pattern
    pub exclude_pattern: Option<&'a str>,
    /// Only return file paths (like rg -l)
    pub files_with_matches: bool,
    /// Only return match counts per file (like rg -c)
    pub count_mode: bool,
}

/// Per-file match count for --count mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMatchCount {
    pub file_path: String,
    pub count: usize,
}

/// Result of a raw search operation
pub enum RawSearchOutput {
    /// Normal line matches
    Lines(Vec<RawSearchResult>),
    /// File paths only (--files-with-matches)
    Files(Vec<String>),
    /// Match counts per file (--count)
    Counts(Vec<FileMatchCount>),
}

// Search engine: file walking, pattern matching, match collection, and utilities
include!("raw_search_engine.rs");

// Tests for raw search functionality
#[cfg(test)]
include!("raw_search_tests.rs");
