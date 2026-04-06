#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};

/// Dependency category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepCategory {
    /// Core - essential, cannot be removed
    Core,
    /// Sovereign - already part of trueno/paiml ecosystem
    Sovereign,
    /// Replaceable - can be replaced with Sovereign stack
    Replaceable,
    /// Heavy - large dependency that adds significant bloat
    Heavy,
    /// DevOnly - only needed for development/testing
    DevOnly,
    /// Removable - can likely be removed entirely
    Removable,
}

/// Dependency analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepAnalysis {
    pub name: String,
    pub version: String,
    pub category: DepCategory,
    pub replacement: Option<String>,
    pub reason: String,
    pub transitive_count: usize,
    pub estimated_size_kb: usize,
    // Graph metrics
    pub pagerank_score: f32,
    pub in_degree: usize,  // How many deps depend on this
    pub out_degree: usize, // How many deps this brings in
    pub is_bridge: bool,   // Connects otherwise disconnected clusters
    pub is_orphan: bool,   // Nothing depends on this (easy to remove)
}

/// Full audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepsAuditReport {
    pub total_deps: usize,
    pub direct_deps: usize,
    pub transitive_deps: usize,
    pub sovereign_deps: usize,
    pub replaceable_deps: usize,
    pub removable_deps: usize,
    pub heavy_deps: usize,
    pub orphan_deps: usize,
    pub bridge_deps: usize,
    pub estimated_savings_kb: usize,
    pub dependencies: Vec<DepAnalysis>,
    pub recommendations: Vec<String>,
    // Graph analysis
    pub top_critical: Vec<(String, f32)>, // Top deps by PageRank
    pub removal_candidates: Vec<String>,  // Orphans that are safe to remove
}

/// Pareto analysis result - ROI for removing a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoEntry {
    pub name: String,
    pub transitive_deps: usize,
    pub effort: ParetoEffort,
    pub roi: f32, // transitive_deps / effort_multiplier
    pub reason: String,
    pub category: DepCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ParetoEffort {
    /// Easy: Just remove from Cargo.toml
    Low = 1,
    /// Medium: Requires small code changes
    Medium = 2,
    /// Hard: Requires significant refactoring
    High = 3,
}

impl ParetoEffort {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn multiplier(&self) -> f32 {
        match self {
            ParetoEffort::Low => 1.0,
            ParetoEffort::Medium => 2.0,
            ParetoEffort::High => 3.0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn label(&self) -> &'static str {
        match self {
            ParetoEffort::Low => "Low",
            ParetoEffort::Medium => "Medium",
            ParetoEffort::High => "High",
        }
    }
}

/// Sort mode for deps-audit output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Sort by transitive dependency count (most deps first)
    Transitive,
    /// Sort by estimated binary size (largest first)
    Size,
    /// Sort by PageRank criticality score (most critical first)
    PageRank,
    /// Sort alphabetically by name
    Name,
    /// Sort by category priority (removable first)
    Category,
}

impl SortMode {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parse(s: &str) -> Self {
        debug_assert!(!s.is_empty(), "s must not be empty");
        match s.to_lowercase().as_str() {
            "size" | "binary" | "kb" => SortMode::Size,
            "pagerank" | "rank" | "critical" => SortMode::PageRank,
            "name" | "alpha" | "alphabetical" => SortMode::Name,
            "category" | "cat" => SortMode::Category,
            _ => SortMode::Transitive, // default
        }
    }
}

/// Dependency edge from Cargo.lock
#[derive(Debug, Clone)]
pub struct DepEdge {
    pub from: String,
    pub to: String,
}

/// Graph analysis results
pub struct GraphAnalysis {
    pub pagerank_scores: std::collections::HashMap<String, f32>,
    pub in_degrees: std::collections::HashMap<String, usize>,
    pub out_degrees: std::collections::HashMap<String, usize>,
    pub bridges: std::collections::HashSet<String>,
    pub orphans: std::collections::HashSet<String>,
    pub transitive_counts: std::collections::HashMap<String, usize>,
}
