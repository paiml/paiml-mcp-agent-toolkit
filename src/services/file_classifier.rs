#![cfg_attr(coverage_nightly, coverage(off))]
//! File classification and filtering for code analysis.
//!
//! This module provides intelligent file classification to determine which files
//! should be analyzed, skipped, or specially handled. It detects vendor code,
//! build artifacts, generated files, and minified content to ensure analysis
//! focuses on human-written source code.
//!
//! # Classification Rules
//!
//! - **Vendor Detection**: Identifies third-party dependencies and libraries
//! - **Build Artifacts**: Skips compiled output and build directories
//! - **Minified Files**: Detects compressed/minified code via entropy analysis
//! - **Large Files**: Handles files exceeding size thresholds
//! - **Binary Detection**: Identifies non-text files
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::file_classifier::{FileClassifier, FileClassifierConfig};
//! use std::path::Path;
//!
//! let config = FileClassifierConfig {
//!     skip_vendor: true,
//!     max_line_length: 10_000,
//!     max_file_size: 1_048_576,
//! };
//!
//! let classifier = FileClassifier::from_config(&config);
//!
//! // Check if a file should be analyzed
//! let path = Path::new("src/main.rs");
//! match classifier.classify(path) {
//!     pmat::services::file_classifier::FileDecision::Parse => {
//!         println!("File should be analyzed");
//!     }
//!     pmat::services::file_classifier::FileDecision::Skip(reason) => {
//!         println!("Skipping file: {:?}", reason);
//!     }
//! }
//! ```ignore

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassifierConfig {
    pub skip_vendor: bool,
    pub max_line_length: usize,
    pub max_file_size: usize,
}

/// Maximum line length before considering a file unparseable
const DEFAULT_MAX_LINE_LENGTH: usize = 10_000;

/// Maximum file size for AST parsing (1MB)
pub const DEFAULT_MAX_FILE_SIZE: usize = 1_048_576;

/// Maximum file size before considering it a "large file" (500KB)
/// Large files are likely minified/generated and should be skipped by default
pub const LARGE_FILE_THRESHOLD: usize = 512_000;

/// Shannon entropy threshold for minified content detection
const MINIFIED_ENTROPY_THRESHOLD: f64 = 6.0;

// ---------------------------------------------------------------------------
// Static patterns (deterministic ordering)
// ---------------------------------------------------------------------------

lazy_static! {
    /// Deterministic vendor detection rules
    static ref VENDOR_RULES: VendorRules = VendorRules {
        // Deterministic ordering for consistent results
        path_patterns: vec![
            "vendor/",
            "node_modules/",
            "third_party/",
            "external/",
            ".yarn/",
            "bower_components/",
            ".min.",
            ".bundle.",
        ],
        file_patterns: vec![
            r"\.min\.(js|css)$",
            r"\.bundle\.js$",
            r"-min\.js$",
            r"\.packed\.js$",
            r"\.dist\.js$",
            r"\.production\.js$",
        ],
        // Content signatures (first 256 bytes)
        content_signatures: vec![
            b"/*! jQuery" as &[u8],
            b"/*! * Bootstrap" as &[u8],
            b"!function(e,t){" as &[u8],  // Common minification pattern
            b"/*! For license information" as &[u8],
            b"/** @license React" as &[u8],
        ],
    };

    /// Build artifact patterns - separate from vendor patterns for clarity
    static ref BUILD_PATTERNS: Vec<&'static str> = vec![
        "target/debug/",
        "target/release/",
        "target/thumbv",
        "/target/debug/",
        "/target/release/",
        "build/",
        "/build/",
        "dist/",
        "/dist/",
        "/.next/",
        "__pycache__/",
        "/__pycache__/",
        "venv/",
        "/venv/",
        ".tox/",
        "/.tox/",
        "cmake-build-",
        "/cmake-build-",
        "/.gradle/",
        ".gradle/",
    ];
}

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassifier {
    pub max_line_length: usize,
    pub max_file_size: usize,
    pub vendor_patterns: Vec<String>,
    pub skip_vendor: bool,
}

impl Default for FileClassifier {
    fn default() -> Self {
        Self {
            max_line_length: DEFAULT_MAX_LINE_LENGTH,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            vendor_patterns: VENDOR_RULES
                .path_patterns
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            skip_vendor: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseDecision {
    Parse,
    Skip(SkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipReason {
    VendorDirectory,
    MinifiedContent,
    LineTooLong,
    FileTooLarge,
    BinaryContent,
    EmptyFile,
    BuildArtifact,
    LargeFile,
}

// ---------------------------------------------------------------------------
// Included implementation files
// ---------------------------------------------------------------------------

include!("file_classifier_classification.rs");
include!("file_classifier_debug_reporter.rs");
include!("file_classifier_tests.rs");
