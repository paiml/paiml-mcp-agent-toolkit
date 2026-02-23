#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for entropy calculation.
//!
//! Contains the core data structures for entropy metrics and reports.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::entropy::violation_detector::{ActionableViolation, PatternSummary};
use crate::entropy::PatternType;

/// Entropy metrics for measuring pattern diversity across different granularities
///
/// Provides detailed statistics about code patterns, duplication levels, and potential
/// for refactoring based on Shannon entropy calculations applied to AST patterns.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::entropy::entropy_calculator::EntropyMetrics;
/// use pmat::entropy::PatternType;
/// use std::collections::HashMap;
///
/// let metrics = EntropyMetrics {
///     file_level_entropy: 0.85,      // High diversity within files
///     module_level_entropy: 0.72,     // Moderate module diversity
///     project_level_entropy: 0.68,    // Some cross-project duplication
///     pattern_diversity: 0.75,        // Good pattern distribution
///     total_patterns: 42,             // Unique patterns found
///     total_instances: 156,           // Total pattern instances
///     total_loc: 2500,                // Lines of code analyzed
///     patterns_by_type: HashMap::new(),
/// };
///
/// // High entropy = low duplication (good)
/// assert!(metrics.pattern_diversity > 0.7);
///
/// // Pattern density calculation
/// let pattern_density = metrics.total_instances as f64 / metrics.total_loc as f64;
/// println!("Pattern density: {:.2} patterns per LOC", pattern_density);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyMetrics {
    /// Shannon entropy at the file level (0.0 = no diversity, 1.0 = maximum diversity)
    pub file_level_entropy: f64,
    /// Shannon entropy at the module level
    pub module_level_entropy: f64,
    /// Shannon entropy at the project level
    pub project_level_entropy: f64,
    /// Overall pattern diversity score (weighted average of all levels)
    pub pattern_diversity: f64,
    /// Number of unique AST patterns identified
    pub total_patterns: usize,
    /// Total instances of all patterns across the codebase
    pub total_instances: usize,
    /// Total lines of code analyzed
    pub total_loc: usize,
    /// Pattern count breakdown by pattern type
    pub patterns_by_type: HashMap<PatternType, usize>,
}

/// Comprehensive report of entropy-based pattern analysis
///
/// Contains all findings from entropy analysis including actionable violations,
/// pattern statistics, and refactoring recommendations with LOC reduction estimates.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::entropy::entropy_calculator::{EntropyReport, EntropyMetrics};
/// use std::collections::HashMap;
///
/// // Create sample metrics
/// let metrics = EntropyMetrics {
///     file_level_entropy: 0.8,
///     module_level_entropy: 0.7,
///     project_level_entropy: 0.65,
///     pattern_diversity: 0.72,
///     total_patterns: 15,
///     total_instances: 63,
///     total_loc: 1500,
///     patterns_by_type: HashMap::new(),
/// };
///
/// // Check pattern diversity
/// if metrics.pattern_diversity < 0.7 {
///     println!("Low pattern diversity detected");
/// } else {
///     println!("Good pattern diversity: {:.2}", metrics.pattern_diversity);
/// }
///
/// assert_eq!(metrics.total_patterns, 15);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyReport {
    /// Number of source files processed in the analysis
    pub total_files_analyzed: usize,
    /// List of repetitive patterns that can be refactored with specific suggestions
    pub actionable_violations: Vec<ActionableViolation>,
    /// Statistical summary of all patterns found in the codebase
    pub pattern_summary: PatternSummary,
    /// Detailed entropy measurements at different granularities
    pub entropy_metrics: EntropyMetrics,
}
