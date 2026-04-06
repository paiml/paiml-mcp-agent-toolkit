#![cfg_attr(coverage_nightly, coverage(off))]
//! Project metadata models for PMAT compliance system (GH-96)
//!
//! Tracks project's PMAT version, schema versions, and compliance state.
//! Stored in `.pmat/project.toml`

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Project metadata stored in .pmat/project.toml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMetadata {
    /// PMAT metadata
    #[serde(rename = "pmat")]
    pub pmat: PmatMetadata,

    /// Compliance tracking
    #[serde(rename = "compliance", default)]
    pub compliance: ComplianceMetadata,
}

/// PMAT version and schema information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PmatMetadata {
    /// PMAT binary version used to create/update this project
    pub version: String,

    /// Last compliance check timestamp (RFC3339)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_compliance_check: Option<String>,

    /// Project metadata schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

/// Compliance state tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ComplianceMetadata {
    /// Breaking changes that have been accepted/migrated
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breaking_changes_accepted: Vec<String>,

    /// Last migration timestamp (RFC3339)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_migration: Option<String>,

    /// Migration history (version -> timestamp)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub migration_history: Vec<MigrationRecord>,
}

/// Migration history record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationRecord {
    /// Version migrated from
    pub from_version: String,

    /// Version migrated to
    pub to_version: String,

    /// Migration timestamp (RFC3339)
    pub timestamp: String,

    /// Whether migration was successful
    pub success: bool,
}

fn default_schema_version() -> String {
    debug_assert!(true, "contract: default_schema_version");
    "1.0".to_string()
}

// --- Implementation methods ---
include!("project_metadata_impls.rs");

// --- Tests ---
include!("project_metadata_tests.rs");
