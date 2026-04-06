#![cfg_attr(coverage_nightly, coverage(off))]
//! Filter, scope, and severity enums

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Explain level for code explanations
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ExplainLevel {
    Brief,
    Detailed,
    Verbose,
}

impl fmt::Display for ExplainLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExplainLevel::Brief => write!(f, "brief"),
            ExplainLevel::Detailed => write!(f, "detailed"),
            ExplainLevel::Verbose => write!(f, "verbose"),
        }
    }
}

/// Search scope
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq)]
pub enum SearchScope {
    /// Search function names
    Functions,
    /// Search type/class names
    Types,
    /// Search variable names
    Variables,
    /// Search all identifiers
    All,
}

impl fmt::Display for SearchScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchScope::Functions => write!(f, "functions"),
            SearchScope::Types => write!(f, "types"),
            SearchScope::Variables => write!(f, "variables"),
            SearchScope::All => write!(f, "all"),
        }
    }
}

/// Symbol type filter
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SymbolTypeFilter {
    /// Functions and methods
    Functions,
    /// Classes only
    Classes,
    /// Types, structs, and classes
    Types,
    /// Variables and constants
    Variables,
    /// Modules and namespaces
    Modules,
    /// All symbols
    All,
}

impl fmt::Display for SymbolTypeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolTypeFilter::Functions => write!(f, "functions"),
            SymbolTypeFilter::Classes => write!(f, "classes"),
            SymbolTypeFilter::Types => write!(f, "types"),
            SymbolTypeFilter::Variables => write!(f, "variables"),
            SymbolTypeFilter::Modules => write!(f, "modules"),
            SymbolTypeFilter::All => write!(f, "all"),
        }
    }
}

/// Property type filter
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum PropertyTypeFilter {
    MemorySafety,
    ThreadSafety,
    DataRaceFreeze,
    Termination,
    FunctionalCorrectness,
    ResourceBounds,
    All,
}

impl PropertyTypeFilter {
    /// Get the string representation of the property type filter
    fn as_str(&self) -> &'static str {
        debug_assert!(true, "contract: as_str");
        match self {
            PropertyTypeFilter::MemorySafety => "memory-safety",
            PropertyTypeFilter::ThreadSafety => "thread-safety",
            PropertyTypeFilter::DataRaceFreeze => "data-race-freeze",
            PropertyTypeFilter::Termination => "termination",
            PropertyTypeFilter::FunctionalCorrectness => "functional-correctness",
            PropertyTypeFilter::ResourceBounds => "resource-bounds",
            PropertyTypeFilter::All => "all",
        }
    }
}

impl fmt::Display for PropertyTypeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Verification method filter
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum VerificationMethodFilter {
    FormalProof,
    ModelChecking,
    StaticAnalysis,
    AbstractInterpretation,
    BorrowChecker,
    All,
}

impl fmt::Display for VerificationMethodFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationMethodFilter::FormalProof => write!(f, "formal-proof"),
            VerificationMethodFilter::ModelChecking => write!(f, "model-checking"),
            VerificationMethodFilter::StaticAnalysis => write!(f, "static-analysis"),
            VerificationMethodFilter::AbstractInterpretation => {
                write!(f, "abstract-interpretation")
            }
            VerificationMethodFilter::BorrowChecker => write!(f, "borrow-checker"),
            VerificationMethodFilter::All => write!(f, "all"),
        }
    }
}

/// SATD severity levels
#[derive(Clone, Debug, ValueEnum, PartialEq, PartialOrd, Ord, Eq)]
pub enum SatdSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for SatdSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SatdSeverity::Low => write!(f, "low"),
            SatdSeverity::Medium => write!(f, "medium"),
            SatdSeverity::High => write!(f, "high"),
            SatdSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Entropy violation severity levels
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, Deserialize, Serialize)]
pub enum EntropySeverity {
    /// Low severity violations
    Low,
    /// Medium severity violations
    Medium,
    /// High severity violations
    High,
}

impl fmt::Display for EntropySeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntropySeverity::Low => write!(f, "low"),
            EntropySeverity::Medium => write!(f, "medium"),
            EntropySeverity::High => write!(f, "high"),
        }
    }
}
