#![cfg_attr(coverage_nightly, coverage(off))]
// Roadmap service for reading/writing roadmap.yaml files
//
// Provides file I/O operations for the unified GitHub/YAML workflow.
// Split into submodules via include!() for maintainability.

use crate::models::roadmap::{Roadmap, RoadmapItem};
use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::{self, File, OpenOptions};
// PMAT-673: the lock file stores the id high-water mark, so it is read, seeked
// and written in place rather than merely held.
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Default roadmap file location
pub const DEFAULT_ROADMAP_PATH: &str = "docs/roadmaps/roadmap.yaml";

/// Roadmap service for file operations
pub struct RoadmapService {
    roadmap_path: PathBuf,
}

impl RoadmapService {
    /// Create a new roadmap service with custom path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new<P: AsRef<Path>>(roadmap_path: P) -> Self {
        Self {
            roadmap_path: roadmap_path.as_ref().to_path_buf(),
        }
    }

    /// Create a roadmap service with default path
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn default_path() -> Self {
        Self::new(DEFAULT_ROADMAP_PATH)
    }
}

// File locking, YAML parsing, load, and save operations
include!("roadmap_service_io.rs");

// CRUD operations: upsert, remove, find, initialize
include!("roadmap_service_operations.rs");

// Unit tests, property-based tests, and edge case tests
include!("roadmap_service_tests.rs");
