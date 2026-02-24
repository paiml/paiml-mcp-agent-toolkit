//! Documentation refactoring handlers
//!
//! AI-assisted documentation cleanup that identifies and removes:
//! - Temporary files (fix-*.sh, test-*.md, etc.)
//! - Outdated status files (*_STATUS.md, *_PROGRESS.md)
//! - Build artifacts (*.mmd, `optimization_state.json`)
//! - Custom patterns defined by the user
//!
//! Follows Zero Tolerance Quality Standards from CLAUDE.md:
//! - No Temporary Code: All code is production-ready or it doesn't exist

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::RefactorDocsOutputFormat;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

/// File category for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCategory {
    TemporaryScript,
    StatusReport,
    BuildArtifact,
    TestFixture,
    CustomPattern,
    Unknown,
}

impl std::fmt::Display for FileCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileCategory::TemporaryScript => write!(f, "Temporary Script"),
            FileCategory::StatusReport => write!(f, "Status Report"),
            FileCategory::BuildArtifact => write!(f, "Build Artifact"),
            FileCategory::TestFixture => write!(f, "Test Fixture"),
            FileCategory::CustomPattern => write!(f, "Custom Pattern"),
            FileCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Information about a file identified for cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CruftFile {
    pub path: PathBuf,
    pub category: FileCategory,
    pub size_bytes: u64,
    pub modified: SystemTime,
    pub age_days: u32,
    pub reason: String,
    pub pattern_matched: String,
}

/// Summary statistics for the cleanup operation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CleanupSummary {
    pub total_files_scanned: usize,
    pub cruft_files_found: usize,
    pub total_size_bytes: u64,
    pub files_by_category: HashMap<String, usize>,
    pub size_by_category: HashMap<String, u64>,
    pub oldest_file_days: u32,
    pub newest_file_days: u32,
}

/// Result of the documentation refactoring analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct RefactorDocsResult {
    pub cruft_files: Vec<CruftFile>,
    pub summary: CleanupSummary,
    pub preserved_files: Vec<PathBuf>,
    pub errors: Vec<String>,
}

/// Result for processing a single directory
struct DirectoryResult {
    cruft_files: Vec<CruftFile>,
    preserved_files: Vec<PathBuf>,
    errors: Vec<String>,
    files_scanned: usize,
    summary: CleanupSummary,
}

// --- Include submodules ---
include!("refactor_docs_orchestration.rs");
include!("refactor_docs_scanning.rs");
include!("refactor_docs_output.rs");

// Tests extracted to refactor_docs_handlers_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "refactor_docs_handlers_tests.rs"]
mod tests;
