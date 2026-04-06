#![cfg_attr(coverage_nightly, coverage(off))]
//! TDG --explain Mode (Issue #78)
//!
//! Function-level complexity analysis with actionable recommendations.
//!
//! # Usage
//!
//! ```bash
//! # Show function-level breakdown
//! pmat tdg src/vector.rs --explain
//!
//! # Filter to functions with complexity >= 15
//! pmat tdg src/vector.rs --explain --threshold 15
//!
//! # Compare against baseline
//! pmat tdg src/vector.rs --explain --baseline main
//!
//! # JSON output for CI
//! pmat tdg src/vector.rs --explain --format json
//! ```

use serde::{Deserialize, Serialize};

use super::TdgScore;

/// Function-level complexity analysis
///
/// Captures cyclomatic complexity, cognitive complexity, and TDG impact
/// for individual functions within a file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionComplexity {
    /// Function name
    pub name: String,

    /// Line number where function starts
    pub line_number: usize,

    /// Cyclomatic complexity (McCabe)
    pub cyclomatic: u32,

    /// Cognitive complexity (SonarSource)
    pub cognitive: u32,

    /// Estimated impact on TDG score (0.0-5.0)
    ///
    /// Higher values indicate functions that have the most negative impact
    /// on the overall TDG score. Reducing complexity in these functions
    /// will yield the highest score improvement.
    pub tdg_impact: f64,

    /// Severity classification based on complexity
    pub severity: ComplexitySeverity,
}

/// Complexity severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplexitySeverity {
    /// Cyclomatic complexity <= 5
    Low,

    /// Cyclomatic complexity 6-10
    Medium,

    /// Cyclomatic complexity 11-20
    High,

    /// Cyclomatic complexity > 20
    Critical,
}

impl ComplexitySeverity {
    /// Classify severity based on cyclomatic complexity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_cyclomatic(complexity: u32) -> Self {
        match complexity {
            0..=5 => Self::Low,
            6..=10 => Self::Medium,
            11..=20 => Self::High,
            _ => Self::Critical,
        }
    }
}

impl std::fmt::Display for ComplexitySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Actionable recommendation for TDG improvement
///
/// Provides specific refactoring steps with estimated impact
/// on TDG score and effort in hours.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionableRecommendation {
    /// Recommendation type (e.g., "extract_macro", "split_module")
    pub rec_type: RecommendationType,

    /// Human-readable action description
    pub action: String,

    /// Line numbers affected by this recommendation
    pub lines: Vec<usize>,

    /// Expected TDG score improvement (points)
    pub expected_impact: f64,

    /// Estimated effort in hours
    pub estimated_hours: f64,

    /// Priority ranking (1 = highest, 5 = lowest)
    pub priority: u8,

    /// Pattern detected (e.g., "match_dispatch", "oversized_module")
    pub pattern: String,
}

/// Types of recommendations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationType {
    /// Extract repeated code pattern into macro
    ExtractMacro,

    /// Split oversized module into smaller modules
    SplitModule,

    /// Reduce function complexity via refactoring
    ReduceComplexity,

    /// Extract nested logic into helper functions
    ExtractFunction,

    /// Replace switch/match with polymorphism
    ReplaceConditional,
}

impl std::fmt::Display for RecommendationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExtractMacro => write!(f, "extract_macro"),
            Self::SplitModule => write!(f, "split_module"),
            Self::ReduceComplexity => write!(f, "reduce_complexity"),
            Self::ExtractFunction => write!(f, "extract_function"),
            Self::ReplaceConditional => write!(f, "replace_conditional"),
        }
    }
}

/// TDG score with function-level breakdown and recommendations
///
/// Extends the base TdgScore with detailed explain information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainedTDGScore {
    /// Base TDG score
    pub score: TdgScore,

    /// Function-level complexity breakdown
    pub functions: Vec<FunctionComplexity>,

    /// Actionable recommendations (priority-sorted)
    pub recommendations: Vec<ActionableRecommendation>,

    /// Optional baseline comparison
    pub baseline_comparison: Option<ExplainBaselineComparison>,
}

/// Baseline comparison for explain mode
///
/// Extends the standard baseline comparison with explain-specific fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainBaselineComparison {
    /// Baseline git ref (commit SHA, tag, or branch)
    pub baseline_ref: String,

    /// Baseline TDG score
    pub baseline_score: f32,

    /// Current TDG score
    pub current_score: f32,

    /// Delta (current - baseline)
    pub delta: f32,

    /// Recommendations that were completed since baseline
    pub completed: Vec<String>,

    /// Recommendations still pending
    pub pending: Vec<String>,
}

impl ExplainedTDGScore {
    /// Create a new explained TDG score
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(score: TdgScore) -> Self {
        Self {
            score,
            functions: Vec::new(),
            recommendations: Vec::new(),
            baseline_comparison: None,
        }
    }

    /// Add function complexity entry
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_function(&mut self, func: FunctionComplexity) {
        self.functions.push(func);
    }

    /// Add recommendation
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_recommendation(&mut self, rec: ActionableRecommendation) {
        self.recommendations.push(rec);
    }

    /// Sort functions by TDG impact (descending)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn sort_functions_by_impact(&mut self) {
        self.functions.sort_by(|a, b| {
            b.tdg_impact
                .partial_cmp(&a.tdg_impact)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Sort recommendations by priority (ascending) and expected impact (descending)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn sort_recommendations(&mut self) {
        self.recommendations
            .sort_by(|a, b| match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => b
                    .expected_impact
                    .partial_cmp(&a.expected_impact)
                    .unwrap_or(std::cmp::Ordering::Equal),
                other => other,
            });
    }

    /// Filter functions by complexity threshold
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn filter_functions_by_threshold(&mut self, threshold: u32) {
        self.functions.retain(|f| f.cyclomatic >= threshold);
    }

    /// Get total number of functions analyzed
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_functions(&self) -> usize {
        self.functions.len()
    }

    /// Get total number of recommendations
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn total_recommendations(&self) -> usize {
        self.recommendations.len()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_severity_classification() {
        assert_eq!(
            ComplexitySeverity::from_cyclomatic(3),
            ComplexitySeverity::Low
        );
        assert_eq!(
            ComplexitySeverity::from_cyclomatic(7),
            ComplexitySeverity::Medium
        );
        assert_eq!(
            ComplexitySeverity::from_cyclomatic(15),
            ComplexitySeverity::High
        );
        assert_eq!(
            ComplexitySeverity::from_cyclomatic(25),
            ComplexitySeverity::Critical
        );
    }

    #[test]
    fn test_explained_score_function_sorting() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(FunctionComplexity {
            name: "low_impact".to_string(),
            line_number: 10,
            cyclomatic: 5,
            cognitive: 6,
            tdg_impact: 1.0,
            severity: ComplexitySeverity::Low,
        });

        explained.add_function(FunctionComplexity {
            name: "high_impact".to_string(),
            line_number: 50,
            cyclomatic: 20,
            cognitive: 25,
            tdg_impact: 4.5,
            severity: ComplexitySeverity::Critical,
        });

        explained.add_function(FunctionComplexity {
            name: "medium_impact".to_string(),
            line_number: 30,
            cyclomatic: 12,
            cognitive: 15,
            tdg_impact: 2.5,
            severity: ComplexitySeverity::High,
        });

        explained.sort_functions_by_impact();

        assert_eq!(explained.functions[0].name, "high_impact");
        assert_eq!(explained.functions[1].name, "medium_impact");
        assert_eq!(explained.functions[2].name, "low_impact");
    }

    #[test]
    fn test_explained_score_threshold_filtering() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(FunctionComplexity {
            name: "simple".to_string(),
            line_number: 10,
            cyclomatic: 5,
            cognitive: 6,
            tdg_impact: 1.0,
            severity: ComplexitySeverity::Low,
        });

        explained.add_function(FunctionComplexity {
            name: "complex".to_string(),
            line_number: 50,
            cyclomatic: 20,
            cognitive: 25,
            tdg_impact: 4.5,
            severity: ComplexitySeverity::Critical,
        });

        explained.filter_functions_by_threshold(10);

        assert_eq!(explained.total_functions(), 1);
        assert_eq!(explained.functions[0].name, "complex");
    }

    #[test]
    fn test_recommendation_sorting() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_recommendation(ActionableRecommendation {
            rec_type: RecommendationType::ExtractMacro,
            action: "Low priority, low impact".to_string(),
            lines: vec![10],
            expected_impact: 2.0,
            estimated_hours: 3.0,
            priority: 3,
            pattern: "test".to_string(),
        });

        explained.add_recommendation(ActionableRecommendation {
            rec_type: RecommendationType::SplitModule,
            action: "High priority, high impact".to_string(),
            lines: vec![100],
            expected_impact: 8.5,
            estimated_hours: 5.0,
            priority: 1,
            pattern: "test".to_string(),
        });

        explained.add_recommendation(ActionableRecommendation {
            rec_type: RecommendationType::ReduceComplexity,
            action: "High priority, medium impact".to_string(),
            lines: vec![50],
            expected_impact: 5.0,
            estimated_hours: 4.0,
            priority: 1,
            pattern: "test".to_string(),
        });

        explained.sort_recommendations();

        // Should be sorted by priority (ascending), then by impact (descending)
        assert_eq!(explained.recommendations[0].expected_impact, 8.5);
        assert_eq!(explained.recommendations[0].priority, 1);
        assert_eq!(explained.recommendations[1].expected_impact, 5.0);
        assert_eq!(explained.recommendations[1].priority, 1);
        assert_eq!(explained.recommendations[2].expected_impact, 2.0);
        assert_eq!(explained.recommendations[2].priority, 3);
    }
}
