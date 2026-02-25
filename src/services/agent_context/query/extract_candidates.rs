#![cfg_attr(coverage_nightly, coverage(off))]
//! Extract Candidates Analysis (Issue #235)
//!
//! Scans function source for I/O patterns, groups by name prefix / call graph
//! clusters, and suggests module extractions for large files.

use super::types::QueryResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── I/O Pattern Categories ──────────────────────────────────────────────────

/// I/O pattern categories with their detection strings
const IO_PATTERNS: &[(&str, &[&str])] = &[
    ("PRINT", &["println!", "print!"]),
    ("EPRINT", &["eprintln!", "eprint!"]),
    ("WRITE", &["write!", "writeln!"]),
    (
        "FS",
        &["std::fs::", "File::open", "File::create", "OpenOptions"],
    ),
    ("PROCESS", &["std::process::Command", "Command::new"]),
    ("STDIN", &["std::io::stdin"]),
    ("STDOUT", &["stdout()"]),
    ("STDERR", &["stderr()"]),
    ("HTTP", &["reqwest::", "hyper::"]),
    ("NET", &["tokio::net::", "TcpStream", "UdpSocket"]),
    ("DB", &["sqlx::", "rusqlite::", "Connection::open"]),
];

// ── Extraction Output Types ─────────────────────────────────────────────────

/// A candidate function for extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExtractionCandidate {
    pub function_name: String,
    pub file_path: String,
    pub start_line: usize,
    pub loc: u32,
    pub io_classification: String,
    pub io_patterns: Vec<String>,
    pub complexity: u32,
    pub tdg_grade: String,
}

/// A group of functions suggested for extraction into a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExtractionGroup {
    pub module_name: String,
    pub source_file: String,
    pub functions: Vec<ExtractionCandidate>,
    pub total_loc: u32,
    pub pure_count: usize,
    pub io_count: usize,
    pub grouping_signal: String,
}

// ── Included Implementation Files ───────────────────────────────────────────

include!("extract_candidates_classification.rs");
include!("extract_candidates_grouping.rs");
include!("extract_candidates_builder.rs");
include!("extract_candidates_tests.rs");
