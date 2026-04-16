//! Types for native bug-hunter pattern detection (PMAT-613).
//!
//! Minimal shape that matches the existing `.pmat/bug-hunter-cache/*.json`
//! schema so downstream readers (git_history_annotations, query --faults)
//! see identical data regardless of whether the cache was written by the
//! legacy batuta binary or by pmat natively.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FindingSeverity::Info => "Info",
            FindingSeverity::Low => "Low",
            FindingSeverity::Medium => "Medium",
            FindingSeverity::High => "High",
            FindingSeverity::Critical => "Critical",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectCategory {
    LogicErrors,
    MemorySafety,
    SilentDegradation,
    TestDebt,
    HiddenDebt,
    GpuKernelBugs,
    ConfigurationErrors,
    Unknown,
}

impl std::fmt::Display for DefectCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DefectCategory::LogicErrors => "LogicErrors",
            DefectCategory::MemorySafety => "MemorySafety",
            DefectCategory::SilentDegradation => "SilentDegradation",
            DefectCategory::TestDebt => "TestDebt",
            DefectCategory::HiddenDebt => "HiddenDebt",
            DefectCategory::GpuKernelBugs => "GpuKernelBugs",
            DefectCategory::ConfigurationErrors => "ConfigurationErrors",
            DefectCategory::Unknown => "Unknown",
        };
        write!(f, "{s}")
    }
}

/// Single pattern rule: literal to scan for + metadata.
#[derive(Debug, Clone, Copy)]
pub struct PatternRule {
    pub literal: &'static str,
    pub category: DefectCategory,
    pub severity: FindingSeverity,
    pub suspiciousness: f64,
}

/// A finding matches the JSON shape consumed by downstream readers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub file: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    pub title: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub category: DefectCategory,
    pub suspiciousness: f64,
    pub discovered_by: String,
}

/// Cache file body written to `.pmat/bug-hunter-cache/<hash>.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugHunterCache {
    pub findings: Vec<Finding>,
    pub mode: String,
    pub config_hash: String,
}
