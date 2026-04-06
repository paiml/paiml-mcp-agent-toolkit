#![cfg_attr(coverage_nightly, coverage(off))]
//! Analysis type enums and quality-related enums

use clap::ValueEnum;
use serde::Serialize;
use std::fmt;

/// Duplicate type
#[derive(Clone, Debug, ValueEnum, PartialEq, Serialize)]
pub enum DuplicateType {
    /// Exact duplicates (Type 1 clones)
    Exact,
    /// Renamed duplicates (Type 2 clones)
    Renamed,
    /// Gapped duplicates (Type 3 clones)
    Gapped,
    /// Semantic duplicates using AST similarity
    Semantic,
    /// Fuzzy matching (similar but not exact)
    Fuzzy,
    /// All types of duplicates
    All,
}

impl fmt::Display for DuplicateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DuplicateType::Exact => write!(f, "exact"),
            DuplicateType::Renamed => write!(f, "renamed"),
            DuplicateType::Gapped => write!(f, "gapped"),
            DuplicateType::Semantic => write!(f, "semantic"),
            DuplicateType::Fuzzy => write!(f, "fuzzy"),
            DuplicateType::All => write!(f, "all"),
        }
    }
}

/// Quality profile for refactoring
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq, Default)]
pub enum QualityProfile {
    /// Standard quality profile
    Standard,
    /// Strict quality profile
    Strict,
    /// Extreme quality profile - RIGID standards
    #[default]
    Extreme,
}

impl fmt::Display for QualityProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QualityProfile::Standard => write!(f, "standard"),
            QualityProfile::Strict => write!(f, "strict"),
            QualityProfile::Extreme => write!(f, "extreme"),
        }
    }
}

/// Analysis type
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum AnalysisType {
    Complexity,
    DeadCode,
    Duplication,
    TechnicalDebt,
    BigO,
    All,
}

impl fmt::Display for AnalysisType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisType::Complexity => write!(f, "complexity"),
            AnalysisType::DeadCode => write!(f, "dead-code"),
            AnalysisType::Duplication => write!(f, "duplication"),
            AnalysisType::TechnicalDebt => write!(f, "technical-debt"),
            AnalysisType::BigO => write!(f, "big-o"),
            AnalysisType::All => write!(f, "all"),
        }
    }
}

/// Quality check type
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum QualityCheckType {
    DeadCode,
    Complexity,
    Coverage,
    Sections,
    Provability,
    Satd,
    Entropy,
    Security,
    Duplicates,
    All,
}

impl QualityCheckType {
    /// Returns the default checks to run
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn default_checks() -> Vec<Self> {
        vec![
            QualityCheckType::Complexity,
            QualityCheckType::DeadCode,
            QualityCheckType::Satd,
            QualityCheckType::Entropy,
            QualityCheckType::Security,
            QualityCheckType::Duplicates,
            QualityCheckType::Coverage,
            QualityCheckType::Sections,
            QualityCheckType::Provability,
        ]
    }
}

impl QualityCheckType {
    /// Get the string representation of the quality check type
    fn as_str(&self) -> &'static str {
        match self {
            QualityCheckType::DeadCode => "dead-code",
            QualityCheckType::Complexity => "complexity",
            QualityCheckType::Coverage => "coverage",
            QualityCheckType::Sections => "sections",
            QualityCheckType::Provability => "provability",
            QualityCheckType::Satd => "satd",
            QualityCheckType::Entropy => "entropy",
            QualityCheckType::Security => "security",
            QualityCheckType::Duplicates => "duplicates",
            QualityCheckType::All => "all",
        }
    }
}

impl fmt::Display for QualityCheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
