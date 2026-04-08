#![allow(unused)]
// Parallel Louvain community detection algorithm
// Implementation based on Blondel et al. (2008) with parallel optimization
// Complexity: All functions <= 10
// SATD: Zero tolerance
#![cfg_attr(coverage_nightly, coverage(off))]
use super::types::UndirectedGraph;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// Parallel Louvain community detection algorithm.
///
/// This implementation uses Rayon for parallel processing of the local moving phase.
/// The algorithm follows the standard Louvain approach:
/// 1. Initialize each node in its own community
/// 2. Local moving phase: Move nodes to maximize modularity gain (parallel)
/// 3. Aggregation phase: Create super-nodes from communities
/// 4. Repeat until no improvement
#[derive(Debug, Clone)]
pub struct ParallelLouvain {
    /// Resolution parameter (gamma) for modularity calculation
    /// Higher values lead to smaller communities
    pub resolution: f64,
    /// Maximum number of outer iterations
    pub max_iterations: usize,
    /// Minimum modularity improvement threshold to continue
    pub min_improvement: f64,
    /// Number of parallel threads (0 = use all available)
    pub num_threads: usize,
}

impl Default for ParallelLouvain {
    fn default() -> Self {
        ParallelLouvain {
            resolution: 1.0,
            max_iterations: 100,
            min_improvement: 1e-6,
            num_threads: 0,
        }
    }
}

// Internal graph representation optimized for Louvain algorithm
include!("parallel_louvain_graph_data.rs");

// Aggregated community data for efficient calculations
include!("parallel_louvain_community_data.rs");

// Core algorithm implementation (detect, modularity, local moving phase)
include!("parallel_louvain_algorithm.rs");

// Tests: builder, detection, modularity
include!("parallel_louvain_tests.rs");

// Tests: internal data structures, renumbering, edge cases
include!("parallel_louvain_tests_internals.rs");
