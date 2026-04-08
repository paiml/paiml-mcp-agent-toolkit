#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// =============================================================================
// CB-081: Dependency Count Detection (Enhanced v2.9)
// Per rust-project-score spec: Too many dependencies degrades build times,
// increases supply chain risk, and bloats binaries.
//
// Enhancements:
// - CB-081-A: Base dependency count scoring
// - CB-081-B: Duplicate crate detection
// - CB-081-C: Feature flag hygiene analysis
// - CB-081-D: Sovereign stack bonus
// - CB-081-E: Trend tracking
// =============================================================================

/// Sovereign stack crates (batuta ecosystem)
const SOVEREIGN_CRATES: &[&str] = &[
    "aprender",
    "trueno",
    "trueno-graph",
    "trueno-db",
    "trueno-rag",
    "trueno-viz",
    "trueno-zram-core",
    "pmcp",
    "presentar-core",
    "renacer",
    "certeza",
    "bashrs",
    "probar",
    "ruchy",
];

/// Dependency count analysis result (enhanced)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyCountReport {
    pub direct_count: usize,
    /// Total packages in Cargo.lock (includes dev-dep transitive)
    pub transitive_count: usize,
    /// Production-only transitive count (excludes dev-dep transitive), used for scoring
    pub prod_transitive_count: Option<usize>,
    pub score: u8, // 0-5 points based on rust-project-score thresholds
    /// Crates with multiple versions in Cargo.lock
    pub duplicate_crates: Vec<DuplicateCrate>,
    /// Dependencies using default-features = false
    pub feature_gated_count: usize,
    pub feature_gated_pct: f64,
    /// Sovereign stack crates used
    pub sovereign_crates: Vec<String>,
    pub sovereign_bonus: u8, // 0-3 bonus points
    /// Delta from previous check (if available)
    pub trend: Option<DependencyTrend>,
    pub violations: Vec<CbPatternViolation>,
}

/// Duplicate crate info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateCrate {
    pub name: String,
    pub versions: Vec<String>,
}

/// Trend tracking data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyTrend {
    pub direct_delta: i32,
    pub transitive_delta: i32,
    pub previous_timestamp: String,
}

/// CB-081 Cache for O(1) dependency analysis (issue #148 fix)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct DependencyCache {
    /// Cargo.lock file mtime (unix timestamp)
    cargo_lock_mtime: u64,
    /// Cached transitive count (all packages from Cargo.lock)
    transitive_count: usize,
    /// Cached production-only transitive count (via cargo tree --no-dev)
    #[serde(default)]
    prod_transitive_count: Option<usize>,
    /// Cached duplicate crates
    duplicate_crates: Vec<DuplicateCrate>,
}

// =============================================================================
// CB-130: Agent Context Adoption (PMAT-470)
// =============================================================================

/// CB-130 Agent Context Adoption report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContextReport {
    /// Whether the RAG index exists
    pub index_exists: bool,
    /// Index age in hours (None if no index)
    pub index_age_hours: Option<f64>,
    /// Whether index is considered stale (>24h)
    pub index_stale: bool,
    /// Number of functions indexed
    pub function_count: usize,
    /// Whether CLAUDE.md mentions pmat_query_code
    pub claude_md_configured: bool,
    /// Required patterns missing from CLAUDE.md
    pub missing_required_patterns: Vec<String>,
    /// Forbidden patterns found in CLAUDE.md
    pub forbidden_patterns_found: Vec<ForbiddenPatternMatch>,
}

/// A forbidden pattern match with location info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenPatternMatch {
    /// The pattern that was found
    pub pattern: String,
    /// Line number where it was found
    pub line: usize,
    /// The actual line content (truncated)
    pub context: String,
}

// --- Include implementation files ---
include!("dependency_checks_cache.rs");
include!("dependency_checks_analysis.rs");
include!("dependency_checks_agent_context.rs");
