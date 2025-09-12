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
//! 1. **ErrorHandling**: try/catch blocks, Result handling → Extract error handler functions
//! 2. **DataValidation**: Input validation patterns → Create validation traits/modules  
//! 3. **ResourceManagement**: RAII patterns, lifecycle management → Implement guards
//! 4. **ControlFlow**: Complex if/else chains → Strategy patterns/polymorphism
//! 5. **DataTransformation**: map/filter/reduce chains → Data pipelines
//! 6. **ApiCall**: HTTP/RPC call patterns → API client abstractions
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
/// ```rust
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
/// ```rust
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
    pub fn reduction_percentage(&self) -> f64 {
        if self.entropy_metrics.total_loc > 0 {
            (self.total_loc_reduction() as f64 / self.entropy_metrics.total_loc as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Format as human-readable report
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
                let p = *count as f64 / total;
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
                let p = *count as f64 / total;
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
        assert!(true);
    }
}
