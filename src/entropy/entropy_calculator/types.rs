#![cfg_attr(coverage_nightly, coverage(off))]
//! Type definitions for entropy calculation.
//!
//! Contains the core data structures for entropy metrics and reports.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::entropy::violation_detector::{ActionableViolation, PatternSummary};
use crate::entropy::PatternType;

/// Entropy metrics for measuring pattern diversity across different granularities
///
/// Provides detailed statistics about code patterns, duplication levels, and potential
/// for refactoring based on Shannon entropy calculations applied to AST patterns.
///
/// # Measured or absent
///
/// The four entropy figures are `Option<f64>` and are `None` (JSON `null`) whenever
/// no repeated pattern was found: Shannon entropy of an empty distribution is not
/// defined, and reporting `0.0` claimed "zero diversity", the worst possible
/// reading, for code that simply had nothing to repeat (defect #650). `total_loc`
/// is a measurement of the input — the non-blank source lines actually read — not
/// an estimate derived from the patterns.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::entropy::entropy_calculator::EntropyMetrics;
/// use pmat::entropy::PatternType;
/// use std::collections::BTreeMap;
///
/// let metrics = EntropyMetrics {
///     file_level_entropy: Some(0.85),      // High diversity within files
///     module_level_entropy: Some(0.72),    // Moderate module diversity
///     project_level_entropy: Some(0.68),   // Some cross-project duplication
///     pattern_diversity: Some(0.75),       // Good pattern distribution
///     total_patterns: 42,                  // Unique patterns found
///     total_instances: 156,                // Total pattern instances
///     total_loc: 2500,                     // Source lines analyzed (measured)
///     patterns_by_type: BTreeMap::new(),
/// };
///
/// // High entropy = low duplication (good)
/// assert!(metrics.pattern_diversity.unwrap() > 0.7);
///
/// // Pattern density calculation
/// let pattern_density = metrics.total_instances as f64 / metrics.total_loc as f64;
/// println!("Pattern density: {:.2} patterns per LOC", pattern_density);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyMetrics {
    /// Shannon entropy at the file level (0.0 = no diversity, 1.0 = maximum
    /// diversity); `None` when no pattern was detected, i.e. not measured.
    pub file_level_entropy: Option<f64>,
    /// Shannon entropy at the module level; `None` when not measured.
    pub module_level_entropy: Option<f64>,
    /// Shannon entropy at the project level; `None` when not measured.
    pub project_level_entropy: Option<f64>,
    /// Overall pattern diversity score; `None` when not measured.
    pub pattern_diversity: Option<f64>,
    /// Number of unique AST patterns identified
    pub total_patterns: usize,
    /// Total instances of all patterns across the codebase
    pub total_instances: usize,
    /// Measured non-blank source lines across every analyzed file
    pub total_loc: usize,
    /// Pattern count breakdown by pattern type (`BTreeMap` so JSON key order is
    /// fixed run to run)
    pub patterns_by_type: BTreeMap<PatternType, usize>,
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
/// use std::collections::BTreeMap;
///
/// // Create sample metrics
/// let metrics = EntropyMetrics {
///     file_level_entropy: Some(0.8),
///     module_level_entropy: Some(0.7),
///     project_level_entropy: Some(0.65),
///     pattern_diversity: Some(0.72),
///     total_patterns: 15,
///     total_instances: 63,
///     total_loc: 1500,
///     patterns_by_type: BTreeMap::new(),
/// };
///
/// // Check pattern diversity
/// match metrics.pattern_diversity {
///     Some(d) if d < 0.7 => println!("Low pattern diversity detected"),
///     Some(d) => println!("Good pattern diversity: {:.2}", d),
///     None => println!("Pattern diversity: not measured"),
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
    /// Set when the entropy figures could not be measured, explaining why, so a
    /// reader never has to guess whether a missing number means "clean" or
    /// "not computed".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measurement_note: Option<String>,
}
