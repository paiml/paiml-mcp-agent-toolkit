#![cfg_attr(coverage_nightly, coverage(off))]
//! CB-1000 Series: MLOps Model Quality Detection
//!
//! Header-only analysis of ML model binary files (GGUF, APR, SafeTensors).
//! Never loads tensor data — parses only metadata for quality checks.
//!
//! Based on: BUG-GGUF-001/002 (aprender), BUG-212 (safetensors sharding),
//! LAYOUT-002 (APR row-major mandate), Sculley et al. (2015) ML tech debt.

use super::types::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Directories to skip when walking for model files.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".claude",
    "node_modules",
    "target",
    ".pmat",
    "vendor",
    "build",
    "dist",
    "__pycache__",
    ".venv",
];

/// Model file extensions we recognize.
const MODEL_EXTENSIONS: &[&str] = &["gguf", "apr", "safetensors"];

/// Maximum tensor count before flagging as likely corrupt (BUG-GGUF-001).
const MAX_TENSOR_COUNT: u64 = 100_000;

/// File size threshold for "consider quantization" advisory (10 GB).
const LARGE_MODEL_THRESHOLD: u64 = 10 * 1024 * 1024 * 1024;

// =============================================================================
// Model format detection
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFormat {
    Gguf,
    Apr,
    SafeTensors,
}

impl ModelFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        debug_assert!(!ext.is_empty(), "ext must not be empty");
        match ext {
            "gguf" => Some(Self::Gguf),
            "apr" => Some(Self::Apr),
            "safetensors" => Some(Self::SafeTensors),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Gguf => "GGUF",
            Self::Apr => "APR",
            Self::SafeTensors => "SafeTensors",
        }
    }
}

/// Minimal model metadata extracted from header only.
#[derive(Debug)]
pub struct ModelMetadata {
    pub format: ModelFormat,
    pub file_size_bytes: u64,
    pub tensor_count: Option<u64>,
    pub architecture: Option<String>,
    pub has_crc: bool,
}

// =============================================================================
// Include sub-files
// =============================================================================

// File walking and model header parsing (GGUF, APR, SafeTensors)
include!("model_quality_parsing.rs");

// CB-1000 through CB-1008 detection functions
include!("model_quality_checks.rs");
