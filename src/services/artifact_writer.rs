//! Content-Addressable Artifact Storage System
//!
//! This module implements deterministic artifact storage with content-addressable
//! organization and atomic write operations.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::models::error::TemplateError;
use crate::services::unified_ast_engine::{ArtifactTree, MermaidArtifacts, Template};
use blake3::Hash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Content-addressable artifact writer with atomic operations
pub struct ArtifactWriter {
    root: PathBuf,
    manifest: BTreeMap<String, ArtifactMetadata>,
}

/// Metadata for each artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub path: PathBuf,
    pub hash: String,
    pub size: usize,
    pub generated_at: DateTime<Utc>,
    pub artifact_type: ArtifactType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Type classification for artifact.
pub enum ArtifactType {
    DogfoodingMarkdown,
    DogfoodingJson,
    MermaidDiagram,
    Template,
    Manifest,
}

/// Verification report for artifact integrity
#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub total_artifacts: usize,
    pub verified: usize,
    pub failed: Vec<IntegrityFailure>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone)]
/// Integrity failure.
pub struct IntegrityFailure {
    pub artifact: String,
    pub expected_hash: String,
    pub actual_hash: String,
}

/// Statistics about stored artifacts
#[derive(Debug, Clone)]
pub struct ArtifactStatistics {
    pub total_artifacts: usize,
    pub total_size: usize,
    pub by_type: BTreeMap<String, TypeStatistics>,
    pub oldest: Option<DateTime<Utc>>,
    pub newest: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
/// Type statistics.
pub struct TypeStatistics {
    pub count: usize,
    pub size: usize,
}

/// Report from cleanup operation
#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub removed: Vec<String>,
    pub failed: Vec<(String, String)>, // (artifact_name, error_message)
}

// Core implementation: new, write, verify, stats, cleanup
include!("artifact_writer_core.rs");

// Unit tests and property tests
include!("artifact_writer_tests.rs");
