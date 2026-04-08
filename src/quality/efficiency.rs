#![cfg_attr(coverage_nightly, coverage(off))]
use syn::{self, visit::Visit};

/// Efficiency analyzer.
pub struct EfficiencyAnalyzer {
    _max_loop_depth: u32,
    _recursive_calls: u32,
}

impl Default for EfficiencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of efficiency operation.
pub struct EfficiencyResult {
    pub time_complexity: String,
    pub space_complexity: String,
}

struct EfficiencyVisitor {
    current_loop_depth: u32,
    max_loop_depth: u32,
    has_recursion: bool,
}

struct SpaceComplexityVisitor {
    allocations: u32,
    recursive_depth: u32,
}

include!("efficiency_analysis.rs");
include!("efficiency_tests.rs");
