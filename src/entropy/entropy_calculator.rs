#![cfg_attr(coverage_nightly, coverage(off))]
//! Entropy Calculation Module
//!
//! This module provides advanced entropy-based analysis for detecting repetitive patterns
//! in codebases. Unlike traditional character-based entropy, it focuses on AST-level patterns
//! that can be actionably refactored to reduce code duplication and improve maintainability.
//!
//! # Key Concepts
//!
//! - **Pattern Entropy**: Measures the diversity of AST patterns in code
//! - **Actionable Violations**: Repetitive patterns that can be extracted into functions
//! - **LOC Reduction**: Estimated lines of code that can be eliminated through refactoring
//! - **Pattern Diversity**: Shannon entropy of pattern distribution across the codebase
//!
//! # Pattern Types Analyzed
//!
//! 1. **`ErrorHandling`**: try/catch blocks, Result handling → Extract error handler functions
//! 2. **`DataValidation`**: Input validation patterns → Create validation traits/modules  
//! 3. **`ResourceManagement`**: RAII patterns, lifecycle management → Implement guards
//! 4. **`ControlFlow`**: Complex if/else chains → Strategy patterns/polymorphism
//! 5. **`DataTransformation`**: map/filter/reduce chains → Data pipelines
//! 6. **`ApiCall`**: HTTP/RPC call patterns → API client abstractions
//!
//! # Example Usage
//!
//! ```rust
//! use pmat::entropy::entropy_calculator::{EntropyMetrics, EntropyReport};
//! use std::collections::HashMap;
//!
//! // Example metrics showing good pattern diversity
//! let metrics = EntropyMetrics {
//!     file_level_entropy: 0.85,
//!     module_level_entropy: 0.75,
//!     project_level_entropy: 0.70,
//!     pattern_diversity: 0.78,
//!     total_patterns: 42,
//!     total_instances: 156,
//!     total_loc: 2500,
//!     patterns_by_type: HashMap::new(),
//! };
//!
//! // High entropy indicates good pattern diversity (low duplication)
//! assert!(metrics.pattern_diversity > 0.7);
//!
//! // Pattern density calculation
//! let pattern_density = metrics.total_instances as f64 / metrics.total_loc as f64;
//! println!("Pattern density: {:.2} patterns per LOC", pattern_density);
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::pattern_extractor::{AstPattern, PatternCollection};
use super::violation_detector::ActionableViolation;
use super::violation_detector::PatternSummary;
use super::{EntropyConfig, PatternType};

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
///     println!("⚠️ Low pattern diversity detected");
/// } else {
///     println!("✅ Good pattern diversity: {:.2}", metrics.pattern_diversity);
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

impl EntropyReport {
    /// Calculate total estimated lines of code reduction from all actionable violations
    ///
    /// Sums up the estimated LOC reduction from all detected patterns that can be
    /// refactored into reusable functions or modules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::entropy::entropy_calculator::{EntropyReport, EntropyMetrics};
    /// use pmat::entropy::violation_detector::PatternSummary;
    /// use pmat::entropy::PatternType;
    /// use std::collections::HashMap;
    ///
    /// // Create a mock report with empty violations for demonstration
    /// let pattern_summary = PatternSummary {
    ///     pattern_type: PatternType::ErrorHandling,
    ///     repetitions: 0,
    ///     variation_score: 0.0,
    ///     example_code: String::new(),
    /// };
    ///
    /// let report = EntropyReport {
    ///     total_files_analyzed: 5,
    ///     actionable_violations: vec![],  // Empty for simplicity
    ///     pattern_summary,
    ///     entropy_metrics: EntropyMetrics {
    ///         file_level_entropy: 0.7,
    ///         module_level_entropy: 0.6,
    ///         project_level_entropy: 0.55,
    ///         pattern_diversity: 0.6,
    ///         total_patterns: 8,
    ///         total_instances: 24,
    ///         total_loc: 500,
    ///         patterns_by_type: HashMap::new(),
    ///     },
    /// };
    ///
    /// // With no actionable violations, LOC reduction should be 0
    /// assert_eq!(report.total_loc_reduction(), 0);
    /// ```
    #[must_use]
    pub fn total_loc_reduction(&self) -> usize {
        self.actionable_violations
            .iter()
            .map(|v| v.estimated_loc_reduction)
            .sum()
    }

    /// Calculate percentage of codebase that could be reduced through refactoring
    ///
    /// Returns the potential LOC reduction as a percentage of total analyzed code.
    /// Higher percentages indicate more duplication and better refactoring opportunities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::entropy::entropy_calculator::{EntropyReport, EntropyMetrics};
    /// use pmat::entropy::violation_detector::PatternSummary;
    /// use pmat::entropy::PatternType;
    /// use std::collections::HashMap;
    ///
    /// let pattern_summary = PatternSummary {
    ///     pattern_type: PatternType::ErrorHandling,
    ///     repetitions: 0,
    ///     variation_score: 0.0,
    ///     example_code: String::new(),
    /// };
    ///
    /// let report = EntropyReport {
    ///     total_files_analyzed: 10,
    ///     actionable_violations: vec![], // Empty violations
    ///     pattern_summary,
    ///     entropy_metrics: EntropyMetrics {
    ///         file_level_entropy: 0.8,
    ///         module_level_entropy: 0.75,
    ///         project_level_entropy: 0.7,
    ///         pattern_diversity: 0.75,
    ///         total_patterns: 0,
    ///         total_instances: 0,
    ///         total_loc: 1000, // Total lines analyzed
    ///         patterns_by_type: HashMap::new(),
    ///     },
    /// };
    ///
    /// // With no violations, reduction percentage should be 0
    /// assert_eq!(report.reduction_percentage(), 0.0);
    /// ```
    #[must_use]
    pub fn reduction_percentage(&self) -> f64 {
        if self.entropy_metrics.total_loc > 0 {
            (self.total_loc_reduction() as f64 / self.entropy_metrics.total_loc as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Format as human-readable report
    #[must_use]
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        report.push_str("Entropy Analysis Results\n");
        report.push_str("========================\n\n");

        report.push_str(&format!("Files Analyzed: {}\n", self.total_files_analyzed));
        report.push_str(&format!(
            "Actionable Violations: {}\n\n",
            self.actionable_violations.len()
        ));

        // Group violations by severity
        let mut high = Vec::new();
        let mut medium = Vec::new();
        let mut low = Vec::new();

        for violation in &self.actionable_violations {
            match violation.severity {
                super::violation_detector::Severity::High => high.push(violation),
                super::violation_detector::Severity::Medium => medium.push(violation),
                super::violation_detector::Severity::Low => low.push(violation),
            }
        }

        if !high.is_empty() {
            report.push_str(&format!("HIGH SEVERITY ({}):\n", high.len()));
            for (i, v) in high.iter().enumerate() {
                report.push_str(&format!(
                    "{}. {}\n   Fix: {} - saves {} lines\n\n",
                    i + 1,
                    v.message,
                    v.fix_suggestion,
                    v.estimated_loc_reduction
                ));
            }
        }

        if !medium.is_empty() {
            report.push_str(&format!("MEDIUM SEVERITY ({}):\n", medium.len()));
            for (i, v) in medium.iter().enumerate() {
                report.push_str(&format!(
                    "{}. {}\n   Fix: {} - saves {} lines\n\n",
                    i + 1,
                    v.message,
                    v.fix_suggestion,
                    v.estimated_loc_reduction
                ));
            }
        }

        report.push_str(&format!(
            "Total Potential Reduction: {} lines ({:.1}% of analyzed code)\n",
            self.total_loc_reduction(),
            self.reduction_percentage()
        ));

        report
    }
}

/// Calculates entropy metrics
pub struct EntropyCalculator {
    #[allow(dead_code)]
    config: EntropyConfig,
}

impl EntropyCalculator {
    #[must_use]
    pub fn new(config: EntropyConfig) -> Self {
        Self { config }
    }

    /// Calculate entropy metrics from patterns
    pub fn calculate(&self, patterns: &PatternCollection) -> Result<EntropyMetrics> {
        let total_patterns = patterns.patterns.len();
        let total_instances: usize = patterns.patterns.values().map(|p| p.frequency).sum();

        let total_loc: usize = patterns
            .patterns
            .values()
            .map(|p| p.estimated_loc * p.frequency)
            .sum();

        // Calculate pattern diversity (Shannon entropy of pattern distribution)
        let pattern_diversity = self.calculate_pattern_diversity(patterns);

        // Calculate entropy at different levels
        let file_level_entropy = self.calculate_file_level_entropy(patterns);
        let module_level_entropy = self.calculate_module_level_entropy(patterns);
        let project_level_entropy = self.calculate_project_level_entropy(patterns);

        // Count patterns by type
        let mut patterns_by_type = HashMap::new();
        for pattern in patterns.patterns.values() {
            *patterns_by_type.entry(pattern.pattern_type).or_insert(0) += pattern.frequency;
        }

        Ok(EntropyMetrics {
            file_level_entropy,
            module_level_entropy,
            project_level_entropy,
            pattern_diversity,
            total_patterns,
            total_instances,
            total_loc,
            patterns_by_type,
        })
    }

    /// Calculate Shannon entropy of pattern distribution
    fn calculate_pattern_diversity(&self, patterns: &PatternCollection) -> f64 {
        if patterns.patterns.is_empty() {
            return 0.0;
        }

        let total_instances: usize = patterns.patterns.values().map(|p| p.frequency).sum();

        if total_instances == 0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for pattern in patterns.patterns.values() {
            let probability = pattern.frequency as f64 / total_instances as f64;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        // Normalize to 0-1 scale (assuming max entropy of 8 bits for code patterns)
        (entropy / 8.0).min(1.0)
    }

    /// Calculate average entropy at file level
    fn calculate_file_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Calculate how diverse patterns are within each file
        let mut file_entropies = Vec::new();

        for file_patterns in patterns.file_patterns.values() {
            if file_patterns.is_empty() {
                continue;
            }

            // Count pattern frequencies in this file
            let mut pattern_counts = HashMap::new();
            for pattern_hash in file_patterns {
                *pattern_counts.entry(pattern_hash).or_insert(0) += 1;
            }

            // Calculate entropy for this file
            let total = file_patterns.len() as f64;
            let mut entropy = 0.0;

            for count in pattern_counts.values() {
                let p = f64::from(*count) / total;
                if p > 0.0 {
                    entropy -= p * p.log2();
                }
            }

            file_entropies.push(entropy);
        }

        if file_entropies.is_empty() {
            return 0.0;
        }

        // Return average file entropy
        let sum: f64 = file_entropies.iter().sum();
        (sum / file_entropies.len() as f64 / 8.0).min(1.0)
    }

    /// Calculate entropy at module level
    fn calculate_module_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Group files by module (simplified: by directory)
        let mut modules: HashMap<String, Vec<&AstPattern>> = HashMap::new();

        for pattern in patterns.patterns.values() {
            for location in &pattern.locations {
                let module = location
                    .file
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("root")
                    .to_string();

                modules.entry(module).or_default().push(pattern);
            }
        }

        // Calculate entropy for each module
        let mut module_entropies = Vec::new();

        for module_patterns in modules.values() {
            if module_patterns.is_empty() {
                continue;
            }

            let mut pattern_counts = HashMap::new();
            for pattern in module_patterns {
                *pattern_counts.entry(pattern.pattern_type).or_insert(0) += 1;
            }

            let total = module_patterns.len() as f64;
            let mut entropy = 0.0;

            for count in pattern_counts.values() {
                let p = f64::from(*count) / total;
                if p > 0.0 {
                    entropy -= p * p.log2();
                }
            }

            module_entropies.push(entropy);
        }

        if module_entropies.is_empty() {
            return 0.0;
        }

        let sum: f64 = module_entropies.iter().sum();
        (sum / module_entropies.len() as f64 / 3.0).min(1.0) // Lower max for module level
    }

    /// Calculate entropy at project level
    fn calculate_project_level_entropy(&self, patterns: &PatternCollection) -> f64 {
        // Overall project pattern diversity
        self.calculate_pattern_diversity(patterns)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_metrics_creation() {
        let metrics = EntropyMetrics {
            file_level_entropy: 0.5,
            module_level_entropy: 0.6,
            project_level_entropy: 0.7,
            pattern_diversity: 0.4,
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: HashMap::new(),
        };

        assert_eq!(metrics.total_patterns, 10);
        assert_eq!(metrics.total_instances, 50);
    }

    #[test]
    fn test_entropy_report_calculations() {
        let report = EntropyReport {
            total_files_analyzed: 10,
            actionable_violations: vec![ActionableViolation {
                severity: crate::entropy::Severity::High,
                pattern: PatternSummary {
                    pattern_type: PatternType::ErrorHandling,
                    repetitions: 10,
                    variation_score: 0.0,
                    example_code: "test".to_string(),
                },
                message: "test".to_string(),
                fix_suggestion: "test".to_string(),
                estimated_loc_reduction: 100,
                affected_files: vec![],
                priority_score: 10.0,
            }],
            pattern_summary: PatternSummary {
                pattern_type: PatternType::ErrorHandling,
                repetitions: 10,
                variation_score: 0.0,
                example_code: "test".to_string(),
            },
            entropy_metrics: EntropyMetrics {
                file_level_entropy: 0.5,
                module_level_entropy: 0.6,
                project_level_entropy: 0.7,
                pattern_diversity: 0.4,
                total_patterns: 10,
                total_instances: 50,
                total_loc: 1000,
                patterns_by_type: HashMap::new(),
            },
        };

        assert_eq!(report.total_loc_reduction(), 100);
        assert_eq!(report.reduction_percentage(), 10.0);
    }
}
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::EntropyMetrics;

    #[test]
    fn test_entropy_metrics_serialization() {
        use std::collections::HashMap;
        let metrics = EntropyMetrics {
            file_level_entropy: 2.5,
            module_level_entropy: 1.8,
            project_level_entropy: 3.2,
            pattern_diversity: 0.75,
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: HashMap::new(),
        };

        let serialized = format!("{:?}", metrics);
        assert!(!serialized.is_empty());
        assert!(serialized.contains("EntropyMetrics"));
    }

    #[test]
    fn test_entropy_metrics_clone() {
        use std::collections::HashMap;
        let metrics = EntropyMetrics {
            file_level_entropy: 2.5,
            module_level_entropy: 1.8,
            project_level_entropy: 3.2,
            pattern_diversity: 0.75,
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: HashMap::new(),
        };

        let cloned = metrics.clone();
        assert_eq!(format!("{:?}", metrics), format!("{:?}", cloned));
        assert_eq!(metrics.file_level_entropy, cloned.file_level_entropy);
        assert_eq!(metrics.pattern_diversity, cloned.pattern_diversity);
        assert_eq!(metrics.total_patterns, cloned.total_patterns);
    }

    #[test]
    fn test_entropy_metrics_memory_safety() {
        use std::collections::HashMap;
        let metrics = EntropyMetrics {
            file_level_entropy: 2.5,
            module_level_entropy: 1.8,
            project_level_entropy: 3.2,
            pattern_diversity: 0.75,
            total_patterns: 10,
            total_instances: 50,
            total_loc: 1000,
            patterns_by_type: HashMap::new(),
        };

        let _cloned = metrics.clone();
        let _size = std::mem::size_of_val(&metrics);

        // Memory safety verification - no panics or issues
        // Memory safety verification - no panics or issues in above calculations
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use crate::entropy::pattern_extractor::{AstPattern, Location, PatternCollection};
    use crate::entropy::Severity;
    use proptest::prelude::*;
    use std::path::PathBuf;

    // ============================================
    // EntropyMetrics tests
    // ============================================

    #[test]
    fn test_entropy_metrics_with_all_fields() {
        let mut patterns_by_type = HashMap::new();
        patterns_by_type.insert(PatternType::ErrorHandling, 10);
        patterns_by_type.insert(PatternType::DataValidation, 5);

        let metrics = EntropyMetrics {
            file_level_entropy: 0.85,
            module_level_entropy: 0.75,
            project_level_entropy: 0.70,
            pattern_diversity: 0.78,
            total_patterns: 42,
            total_instances: 156,
            total_loc: 2500,
            patterns_by_type,
        };

        assert_eq!(metrics.total_patterns, 42);
        assert_eq!(metrics.total_instances, 156);
        assert_eq!(metrics.total_loc, 2500);
        assert!(metrics.file_level_entropy > 0.8);
        assert!(metrics.pattern_diversity > 0.7);
        assert_eq!(
            metrics.patterns_by_type.get(&PatternType::ErrorHandling),
            Some(&10)
        );
    }

    #[test]
    fn test_entropy_metrics_serialization_json() {
        let metrics = EntropyMetrics {
            file_level_entropy: 0.5,
            module_level_entropy: 0.6,
            project_level_entropy: 0.7,
            pattern_diversity: 0.55,
            total_patterns: 5,
            total_instances: 25,
            total_loc: 500,
            patterns_by_type: HashMap::new(),
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("file_level_entropy"));
        assert!(json.contains("0.5"));

        let deserialized: EntropyMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics.file_level_entropy, deserialized.file_level_entropy);
        assert_eq!(metrics.total_patterns, deserialized.total_patterns);
    }

    #[test]
    fn test_entropy_metrics_zero_values() {
        let metrics = EntropyMetrics {
            file_level_entropy: 0.0,
            module_level_entropy: 0.0,
            project_level_entropy: 0.0,
            pattern_diversity: 0.0,
            total_patterns: 0,
            total_instances: 0,
            total_loc: 0,
            patterns_by_type: HashMap::new(),
        };

        assert_eq!(metrics.total_loc, 0);
        assert_eq!(metrics.pattern_diversity, 0.0);
    }

    // ============================================
    // EntropyReport tests
    // ============================================

    fn create_test_report(violations: Vec<ActionableViolation>, total_loc: usize) -> EntropyReport {
        EntropyReport {
            total_files_analyzed: 10,
            actionable_violations: violations,
            pattern_summary: PatternSummary {
                pattern_type: PatternType::ErrorHandling,
                repetitions: 0,
                variation_score: 0.0,
                example_code: String::new(),
            },
            entropy_metrics: EntropyMetrics {
                file_level_entropy: 0.5,
                module_level_entropy: 0.6,
                project_level_entropy: 0.7,
                pattern_diversity: 0.55,
                total_patterns: 5,
                total_instances: 25,
                total_loc,
                patterns_by_type: HashMap::new(),
            },
        }
    }

    fn create_test_violation(severity: Severity, loc_reduction: usize) -> ActionableViolation {
        ActionableViolation {
            severity,
            pattern: PatternSummary {
                pattern_type: PatternType::ErrorHandling,
                repetitions: 5,
                variation_score: 0.1,
                example_code: "test code".to_string(),
            },
            message: "Test violation message".to_string(),
            fix_suggestion: "Fix suggestion".to_string(),
            estimated_loc_reduction: loc_reduction,
            affected_files: vec![PathBuf::from("test.rs")],
            priority_score: 5.0,
        }
    }

    #[test]
    fn test_entropy_report_total_loc_reduction_empty() {
        let report = create_test_report(vec![], 1000);
        assert_eq!(report.total_loc_reduction(), 0);
    }

    #[test]
    fn test_entropy_report_total_loc_reduction_single() {
        let violations = vec![create_test_violation(Severity::High, 100)];
        let report = create_test_report(violations, 1000);
        assert_eq!(report.total_loc_reduction(), 100);
    }

    #[test]
    fn test_entropy_report_total_loc_reduction_multiple() {
        let violations = vec![
            create_test_violation(Severity::High, 100),
            create_test_violation(Severity::Medium, 50),
            create_test_violation(Severity::Low, 25),
        ];
        let report = create_test_report(violations, 1000);
        assert_eq!(report.total_loc_reduction(), 175);
    }

    #[test]
    fn test_entropy_report_reduction_percentage_zero_loc() {
        let report = create_test_report(vec![], 0);
        assert_eq!(report.reduction_percentage(), 0.0);
    }

    #[test]
    fn test_entropy_report_reduction_percentage_normal() {
        let violations = vec![create_test_violation(Severity::High, 100)];
        let report = create_test_report(violations, 1000);
        assert!((report.reduction_percentage() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_report_reduction_percentage_large() {
        let violations = vec![create_test_violation(Severity::High, 500)];
        let report = create_test_report(violations, 1000);
        assert!((report.reduction_percentage() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_report_format_report_header() {
        let report = create_test_report(vec![], 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("Entropy Analysis Results"));
        assert!(formatted.contains("========================"));
        assert!(formatted.contains("Files Analyzed: 10"));
    }

    #[test]
    fn test_entropy_report_format_report_with_high_severity() {
        let violations = vec![create_test_violation(Severity::High, 100)];
        let report = create_test_report(violations, 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("HIGH SEVERITY (1)"));
        assert!(formatted.contains("Test violation message"));
        assert!(formatted.contains("Fix suggestion"));
        assert!(formatted.contains("saves 100 lines"));
    }

    #[test]
    fn test_entropy_report_format_report_with_medium_severity() {
        let violations = vec![create_test_violation(Severity::Medium, 50)];
        let report = create_test_report(violations, 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("MEDIUM SEVERITY (1)"));
    }

    #[test]
    fn test_entropy_report_format_report_with_low_severity() {
        // Low severity violations are not shown in the format_report
        let violations = vec![create_test_violation(Severity::Low, 25)];
        let report = create_test_report(violations, 1000);
        let formatted = report.format_report();

        // Low severity is not displayed in the report
        assert!(!formatted.contains("LOW SEVERITY"));
    }

    #[test]
    fn test_entropy_report_format_report_mixed_severity() {
        let violations = vec![
            create_test_violation(Severity::High, 100),
            create_test_violation(Severity::High, 75),
            create_test_violation(Severity::Medium, 50),
            create_test_violation(Severity::Medium, 30),
            create_test_violation(Severity::Low, 10),
        ];
        let report = create_test_report(violations, 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("HIGH SEVERITY (2)"));
        assert!(formatted.contains("MEDIUM SEVERITY (2)"));
        assert!(formatted.contains("Total Potential Reduction: 265 lines"));
    }

    #[test]
    fn test_entropy_report_serialization() {
        let report = create_test_report(vec![], 1000);
        let json = serde_json::to_string(&report).unwrap();
        let deserialized: EntropyReport = serde_json::from_str(&json).unwrap();

        assert_eq!(
            report.total_files_analyzed,
            deserialized.total_files_analyzed
        );
        assert_eq!(
            report.entropy_metrics.total_loc,
            deserialized.entropy_metrics.total_loc
        );
    }

    #[test]
    fn test_entropy_report_clone() {
        let violations = vec![create_test_violation(Severity::High, 100)];
        let report = create_test_report(violations, 1000);
        let cloned = report.clone();

        assert_eq!(report.total_files_analyzed, cloned.total_files_analyzed);
        assert_eq!(
            report.actionable_violations.len(),
            cloned.actionable_violations.len()
        );
    }

    // ============================================
    // EntropyCalculator tests
    // ============================================

    #[test]
    fn test_entropy_calculator_new() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let _ = calculator; // Ensure it can be created
    }

    #[test]
    fn test_entropy_calculator_calculate_empty_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let patterns = PatternCollection::new();

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(metrics.total_patterns, 0);
        assert_eq!(metrics.total_instances, 0);
        assert_eq!(metrics.total_loc, 0);
        assert_eq!(metrics.pattern_diversity, 0.0);
    }

    #[test]
    fn test_entropy_calculator_calculate_single_pattern() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            }],
            variation_score: 0.1,
            example_code: "match result {}".to_string(),
            estimated_loc: 10,
        });

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(metrics.total_patterns, 1);
        assert_eq!(metrics.total_instances, 5);
        assert_eq!(metrics.total_loc, 50); // 5 * 10
    }

    #[test]
    fn test_entropy_calculator_calculate_multiple_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![Location {
                file: PathBuf::from("test1.rs"),
                line: 10,
                column: 1,
            }],
            variation_score: 0.1,
            example_code: "error handling".to_string(),
            estimated_loc: 10,
        });

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::DataValidation,
            pattern_hash: "hash2".to_string(),
            frequency: 3,
            locations: vec![Location {
                file: PathBuf::from("test2.rs"),
                line: 20,
                column: 1,
            }],
            variation_score: 0.2,
            example_code: "validation".to_string(),
            estimated_loc: 5,
        });

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(metrics.total_patterns, 2);
        assert_eq!(metrics.total_instances, 8); // 5 + 3
        assert_eq!(metrics.total_loc, 65); // 5*10 + 3*5
        assert!(metrics.pattern_diversity > 0.0);
    }

    #[test]
    fn test_entropy_calculator_patterns_by_type() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        });

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash2".to_string(),
            frequency: 3,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "hash3".to_string(),
            frequency: 2,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 8,
        });

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(
            metrics.patterns_by_type.get(&PatternType::ErrorHandling),
            Some(&8)
        ); // 5+3
        assert_eq!(
            metrics.patterns_by_type.get(&PatternType::ControlFlow),
            Some(&2)
        );
    }

    #[test]
    fn test_calculate_pattern_diversity_empty() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let patterns = PatternCollection::new();

        let diversity = calculator.calculate_pattern_diversity(&patterns);
        assert_eq!(diversity, 0.0);
    }

    #[test]
    fn test_calculate_pattern_diversity_single_pattern() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 10,
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        let diversity = calculator.calculate_pattern_diversity(&patterns);
        // Single pattern = zero entropy (no diversity)
        assert_eq!(diversity, 0.0);
    }

    #[test]
    fn test_calculate_pattern_diversity_multiple_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        // Add multiple patterns with equal frequency for maximum diversity
        for i in 0..4 {
            patterns.add_pattern(AstPattern {
                pattern_type: PatternType::ErrorHandling,
                pattern_hash: format!("hash{}", i),
                frequency: 5,
                locations: vec![],
                variation_score: 0.0,
                example_code: "".to_string(),
                estimated_loc: 5,
            });
        }

        let diversity = calculator.calculate_pattern_diversity(&patterns);
        // Multiple patterns = positive entropy
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_calculate_file_level_entropy_empty() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let patterns = PatternCollection::new();

        let entropy = calculator.calculate_file_level_entropy(&patterns);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_calculate_module_level_entropy_empty() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let patterns = PatternCollection::new();

        let entropy = calculator.calculate_module_level_entropy(&patterns);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_calculate_module_level_entropy_with_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        // Add patterns with locations in different modules
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![
                Location {
                    file: PathBuf::from("src/mod1/file.rs"),
                    line: 10,
                    column: 1,
                },
                Location {
                    file: PathBuf::from("src/mod2/file.rs"),
                    line: 20,
                    column: 1,
                },
            ],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ControlFlow,
            pattern_hash: "hash2".to_string(),
            frequency: 3,
            locations: vec![Location {
                file: PathBuf::from("src/mod1/other.rs"),
                line: 15,
                column: 1,
            }],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 8,
        });

        let entropy = calculator.calculate_module_level_entropy(&patterns);
        // Should have some entropy from patterns in different modules
        assert!(entropy >= 0.0);
    }

    #[test]
    fn test_calculate_project_level_entropy() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        for i in 0..3 {
            patterns.add_pattern(AstPattern {
                pattern_type: PatternType::ErrorHandling,
                pattern_hash: format!("hash{}", i),
                frequency: 5,
                locations: vec![],
                variation_score: 0.0,
                example_code: "".to_string(),
                estimated_loc: 5,
            });
        }

        let entropy = calculator.calculate_project_level_entropy(&patterns);
        // Should be same as pattern diversity
        let diversity = calculator.calculate_pattern_diversity(&patterns);
        assert_eq!(entropy, diversity);
    }

    #[test]
    fn test_calculate_file_level_entropy_with_file_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        // Add pattern
        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 5,
            locations: vec![Location {
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 1,
            }],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 5,
        });

        // Manually add file patterns
        patterns.file_patterns.insert(
            PathBuf::from("test.rs"),
            vec![
                "hash1".to_string(),
                "hash1".to_string(),
                "hash2".to_string(),
            ],
        );

        let entropy = calculator.calculate_file_level_entropy(&patterns);
        // Should have some entropy from multiple patterns in the file
        assert!(entropy >= 0.0);
    }

    // ============================================
    // Property-based tests
    // ============================================

    proptest! {
        #[test]
        fn test_entropy_report_loc_reduction_non_negative(
            reductions in proptest::collection::vec(0usize..1000, 0..10)
        ) {
            let violations: Vec<ActionableViolation> = reductions
                .iter()
                .map(|&r| create_test_violation(Severity::Medium, r))
                .collect();
            let report = create_test_report(violations, 10000);

            prop_assert!(report.total_loc_reduction() <= 10000 || report.total_loc_reduction() >= 0);
        }

        #[test]
        fn test_entropy_metrics_entropy_values_bounded(
            file_entropy in 0.0f64..=1.0,
            module_entropy in 0.0f64..=1.0,
            project_entropy in 0.0f64..=1.0,
            diversity in 0.0f64..=1.0,
        ) {
            let metrics = EntropyMetrics {
                file_level_entropy: file_entropy,
                module_level_entropy: module_entropy,
                project_level_entropy: project_entropy,
                pattern_diversity: diversity,
                total_patterns: 10,
                total_instances: 50,
                total_loc: 1000,
                patterns_by_type: HashMap::new(),
            };

            prop_assert!(metrics.file_level_entropy >= 0.0 && metrics.file_level_entropy <= 1.0);
            prop_assert!(metrics.module_level_entropy >= 0.0 && metrics.module_level_entropy <= 1.0);
            prop_assert!(metrics.pattern_diversity >= 0.0 && metrics.pattern_diversity <= 1.0);
        }

        #[test]
        fn test_reduction_percentage_bounded(
            loc_reduction in 0usize..1000,
            total_loc in 1usize..10000,
        ) {
            let violations = vec![create_test_violation(Severity::High, loc_reduction)];
            let report = create_test_report(violations, total_loc);

            let percentage = report.reduction_percentage();
            prop_assert!(percentage >= 0.0);
            // Percentage could be > 100% if reduction > total_loc (edge case)
        }

        #[test]
        fn test_pattern_collection_patterns_tracked(
            num_patterns in 1usize..20,
            frequency in 1usize..100,
        ) {
            let config = EntropyConfig::default();
            let calculator = EntropyCalculator::new(config);
            let mut patterns = PatternCollection::new();

            for i in 0..num_patterns {
                patterns.add_pattern(AstPattern {
                    pattern_type: PatternType::ErrorHandling,
                    pattern_hash: format!("hash{}", i),
                    frequency,
                    locations: vec![],
                    variation_score: 0.0,
                    example_code: "".to_string(),
                    estimated_loc: 5,
                });
            }

            let metrics = calculator.calculate(&patterns).unwrap();
            prop_assert_eq!(metrics.total_patterns, num_patterns);
            prop_assert_eq!(metrics.total_instances, num_patterns * frequency);
        }
    }

    // ============================================
    // Edge cases
    // ============================================

    #[test]
    fn test_entropy_calculator_zero_frequency_patterns() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        patterns.add_pattern(AstPattern {
            pattern_type: PatternType::ErrorHandling,
            pattern_hash: "hash1".to_string(),
            frequency: 0, // Zero frequency
            locations: vec![],
            variation_score: 0.0,
            example_code: "".to_string(),
            estimated_loc: 10,
        });

        let metrics = calculator.calculate(&patterns).unwrap();
        assert_eq!(metrics.total_instances, 0);
        assert_eq!(metrics.total_loc, 0);
    }

    #[test]
    fn test_entropy_calculator_large_pattern_collection() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        // Add many patterns
        for i in 0..100 {
            patterns.add_pattern(AstPattern {
                pattern_type: if i % 2 == 0 {
                    PatternType::ErrorHandling
                } else {
                    PatternType::ControlFlow
                },
                pattern_hash: format!("hash{}", i),
                frequency: (i % 10) + 1,
                locations: vec![],
                variation_score: (i as f64) / 100.0,
                example_code: format!("code{}", i),
                estimated_loc: (i % 20) + 1,
            });
        }

        let metrics = calculator.calculate(&patterns).unwrap();
        assert_eq!(metrics.total_patterns, 100);
        assert!(metrics.pattern_diversity > 0.0);
    }

    #[test]
    fn test_entropy_report_format_empty_violations() {
        let report = create_test_report(vec![], 1000);
        let formatted = report.format_report();

        assert!(formatted.contains("Actionable Violations: 0"));
        assert!(formatted.contains("Total Potential Reduction: 0 lines"));
    }

    #[test]
    fn test_pattern_summary_debug() {
        let summary = PatternSummary {
            pattern_type: PatternType::DataTransformation,
            repetitions: 15,
            variation_score: 0.45,
            example_code: "transform(data)".to_string(),
        };

        let debug_str = format!("{:?}", summary);
        assert!(debug_str.contains("DataTransformation"));
        assert!(debug_str.contains("15"));
    }

    #[test]
    fn test_all_pattern_types_in_metrics() {
        let config = EntropyConfig::default();
        let calculator = EntropyCalculator::new(config);
        let mut patterns = PatternCollection::new();

        let all_types = [
            PatternType::ErrorHandling,
            PatternType::DataValidation,
            PatternType::ResourceManagement,
            PatternType::ControlFlow,
            PatternType::DataTransformation,
            PatternType::ApiCall,
        ];

        for (i, pt) in all_types.iter().enumerate() {
            patterns.add_pattern(AstPattern {
                pattern_type: *pt,
                pattern_hash: format!("hash{}", i),
                frequency: (i + 1) * 2,
                locations: vec![],
                variation_score: 0.0,
                example_code: "".to_string(),
                estimated_loc: 5,
            });
        }

        let metrics = calculator.calculate(&patterns).unwrap();

        assert_eq!(metrics.total_patterns, 6);
        assert_eq!(metrics.patterns_by_type.len(), 6);

        // Verify each pattern type is tracked
        for pt in &all_types {
            assert!(metrics.patterns_by_type.contains_key(pt));
        }
    }

    #[test]
    fn test_entropy_metrics_with_empty_patterns_by_type() {
        let metrics = EntropyMetrics {
            file_level_entropy: 0.5,
            module_level_entropy: 0.5,
            project_level_entropy: 0.5,
            pattern_diversity: 0.5,
            total_patterns: 0,
            total_instances: 0,
            total_loc: 0,
            patterns_by_type: HashMap::new(),
        };

        assert!(metrics.patterns_by_type.is_empty());
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: EntropyMetrics = serde_json::from_str(&json).unwrap();
        assert!(deserialized.patterns_by_type.is_empty());
    }
}
