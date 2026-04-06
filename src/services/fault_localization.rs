#![cfg_attr(coverage_nightly, coverage(off))]
// Tarantula-style Spectrum-Based Fault Localization (SBFL)
// Issue #103: Fault localization integration
// Toyota Way: Start with simplest formula, evolve based on evidence
// Phase 1: Classic Tarantula + Ochiai + DStar formulas
// Muda: Avoid waste by using lightweight SBFL before expensive MBFL
// Muri: Prevent overburden by presenting only top-N suspicious statements

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Represents a code location for fault localization
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatementId {
    pub file: PathBuf,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

impl StatementId {
    pub fn new(file: impl Into<PathBuf>, line: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
}

impl std::fmt::Display for StatementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file.display(), self.line)
    }
}

/// Coverage information for a single statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementCoverage {
    pub id: StatementId,
    pub executed_by_passed: usize,
    pub executed_by_failed: usize,
}

impl StatementCoverage {
    pub fn new(id: StatementId, passed: usize, failed: usize) -> Self {
        Self {
            id,
            executed_by_passed: passed,
            executed_by_failed: failed,
        }
    }
}

/// Available fault localization formulas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SbflFormula {
    /// Original Tarantula formula (Jones & Harrold, 2005)
    #[default]
    Tarantula,
    /// Ochiai formula - often outperforms Tarantula (Abreu et al., 2009)
    Ochiai,
    /// DStar with configurable exponent (Wong et al., 2014)
    DStar { exponent: u32 },
}

impl std::fmt::Display for SbflFormula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SbflFormula::Tarantula => write!(f, "Tarantula"),
            SbflFormula::Ochiai => write!(f, "Ochiai"),
            SbflFormula::DStar { exponent } => write!(f, "DStar{}", exponent),
        }
    }
}

impl std::str::FromStr for SbflFormula {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug_assert!(!s.is_empty(), "s must not be empty");
        match s.to_lowercase().as_str() {
            "tarantula" => Ok(SbflFormula::Tarantula),
            "ochiai" => Ok(SbflFormula::Ochiai),
            "dstar2" => Ok(SbflFormula::DStar { exponent: 2 }),
            "dstar3" => Ok(SbflFormula::DStar { exponent: 3 }),
            other => Err(anyhow!("Unknown SBFL formula: {}", other)),
        }
    }
}

/// Individual suspiciousness ranking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousnessRanking {
    pub rank: usize,
    pub statement: StatementId,
    pub suspiciousness: f32,
    pub scores: HashMap<String, f32>,
    pub explanation: String,
    pub failed_coverage: usize,
    pub passed_coverage: usize,
}

/// Result of fault localization analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultLocalizationResult {
    pub rankings: Vec<SuspiciousnessRanking>,
    pub formula_used: SbflFormula,
    pub confidence: f32,
    pub total_passed_tests: usize,
    pub total_failed_tests: usize,
}

/// Spectrum-Based Fault Localizer
///
/// Implements the core SBFL algorithms following Toyota Way principles:
/// - Start simple (Tarantula baseline)
/// - Measure and evolve (compare formulas)
/// - Eliminate waste (skip expensive analysis when simple works)
pub struct SbflLocalizer {
    formula: SbflFormula,
    top_n: usize,
    include_explanations: bool,
    min_confidence_threshold: f32,
}

impl Default for SbflLocalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Report output format for fault localization results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportFormat {
    #[default]
    Terminal,
    Json,
    Yaml,
}

impl std::str::FromStr for ReportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug_assert!(!s.is_empty(), "s must not be empty");
        match s.to_lowercase().as_str() {
            "terminal" | "text" => Ok(ReportFormat::Terminal),
            "json" => Ok(ReportFormat::Json),
            "yaml" => Ok(ReportFormat::Yaml),
            other => Err(anyhow!("Unknown format: {}", other)),
        }
    }
}

/// LCOV coverage data parser for cargo-llvm-cov integration
#[derive(Debug, Default)]
pub struct LcovParser;

/// Tarantula integration wrapper
///
/// Provides high-level interface for fault localization that integrates
/// with cargo-llvm-cov for coverage and pmat for TDG enrichment.
pub struct FaultLocalizer;

// SBFL scoring formulas (tarantula, ochiai, dstar) and SbflLocalizer methods
include!("fault_localization_formulas.rs");

// LCOV parsing (LcovParser) and high-level integration (FaultLocalizer)
include!("fault_localization_parsing.rs");

// Unit tests
include!("fault_localization_tests.rs");
