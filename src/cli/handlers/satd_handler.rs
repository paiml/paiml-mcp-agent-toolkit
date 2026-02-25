//! SATD (Self-Admitted Technical Debt) Analysis Handler
//!
//! Refactored handler using the service facade pattern.

use crate::cli::{SatdOutputFormat, SatdSeverity};
use crate::services::facades::satd_facade::{SatdAnalysisRequest, SatdAnalysisResult, SatdFacade};
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// Configuration for SATD analysis
#[derive(Debug, Clone)]
pub struct SatdAnalysisConfig {
    pub path: PathBuf,
    pub format: SatdOutputFormat,
    pub severity: Option<SatdSeverity>,
    pub critical_only: bool,
    pub include_tests: bool,
    pub strict: bool,
    pub evolution: bool,
    pub days: u32,
    pub metrics: bool,
    pub output: Option<PathBuf>,
    pub top_files: usize,
    pub fail_on_violation: bool,
    pub timeout: u64,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    /// Extended mode: detects euphemisms like placeholder, stub, "for now" (issue #149)
    pub extended: bool,
}

include!("satd_handler_analysis.rs");
include!("satd_handler_formatting.rs");
include!("satd_handler_tests.rs");
