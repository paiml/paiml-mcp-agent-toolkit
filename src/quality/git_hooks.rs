#![cfg_attr(coverage_nightly, coverage(off))]
use crate::quality::gate::QualityGateRunner;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitHookManager {
    repo_path: PathBuf,
    quality_runner: QualityGateRunner,
}

impl GitHookManager {
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
            quality_runner: QualityGateRunner::strict(),
        }
    }
}

#[derive(Debug)]
pub struct QualityReport {
    pub file: String,
    pub passed: bool,
    pub violations: Vec<String>,
}

pub struct IncrementalChecker {
    cache: HashMap<String, FileChecksum>,
}

#[derive(Debug, Clone)]
struct FileChecksum {
    _hash: String,
    last_checked: std::time::SystemTime,
    _passed: bool,
}

// Hook installation methods (pre-commit, commit-msg, pre-push)
include!("git_hooks_install.rs");

// Validation, incremental checking, and CI config generation
include!("git_hooks_validate.rs");

// Tests
include!("git_hooks_tests.rs");
