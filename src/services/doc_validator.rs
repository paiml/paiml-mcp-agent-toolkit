//! Documentation link validator
//!
//! Validates markdown links (internal and external HTTP/HTTPS) and reports broken links.
//! Designed with EXTREME TDD principles with property tests and comprehensive coverage.

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;
#[cfg(feature = "http-client")]
use std::time::Duration;
use walkdir::WalkDir;

/// Core validator that orchestrates link checking
pub struct DocValidator {
    config: ValidatorConfig,
    #[cfg(feature = "http-client")]
    http_client: Option<reqwest::Client>,
}

/// Configuration for validation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Root directory to search for markdown files
    pub root_dir: PathBuf,

    /// Timeout for HTTP requests (milliseconds)
    pub http_timeout_ms: u64,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Delay between retries (milliseconds)
    pub retry_delay_ms: u64,

    /// Maximum concurrent HTTP requests
    pub max_concurrent_requests: usize,

    /// Patterns to exclude from validation
    pub exclude_patterns: Vec<String>,

    /// Follow redirects
    pub follow_redirects: bool,

    /// User agent string
    pub user_agent: String,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            http_timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 1000,
            max_concurrent_requests: 10,
            exclude_patterns: vec![
                "archive".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
            ],
            follow_redirects: true,
            user_agent: format!("pmat-doc-validator/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Represents a parsed markdown link
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Link {
    /// Link text
    pub text: String,

    /// Link target (URL or file path)
    pub target: String,

    /// Source file containing the link
    pub source_file: PathBuf,

    /// Line number in source file
    pub line_number: usize,

    /// Link type
    pub link_type: LinkType,
}

/// Type of link
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkType {
    /// Internal file link (relative or absolute path)
    Internal,

    /// External HTTP/HTTPS link
    ExternalHttp,

    /// Anchor link within same document
    Anchor,

    /// Email link
    Email,

    /// Other protocol
    Other(String),
}

/// Result of validating a single link
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub link: Link,
    pub status: ValidationStatus,
    pub error_message: Option<String>,
    pub http_status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
}

/// Status of link validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Link is valid and accessible
    Valid,

    /// Link returned 404 or file not found
    NotFound,

    /// Link returned other HTTP error
    HttpError(u16),

    /// Network error (timeout, connection failed, etc.)
    NetworkError,

    /// Link is malformed or invalid
    InvalidLink,

    /// Link was skipped (excluded pattern)
    Skipped,
}

/// Summary of validation run
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_files: usize,
    pub total_links: usize,
    pub valid_links: usize,
    pub broken_links: usize,
    pub skipped_links: usize,
    pub duration_ms: u64,
    pub results: Vec<ValidationResult>,
}

// --- Link parsing: extract_links, classify_link, normalize_path ---
include!("doc_validator_link_parsing.rs");

// --- DocValidator impl: constructor, validation methods, Default ---
include!("doc_validator_validation.rs");

// --- Tests: unit tests and property tests ---
include!("doc_validator_tests.rs");
