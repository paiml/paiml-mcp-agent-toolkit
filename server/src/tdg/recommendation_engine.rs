//! Recommendation engine for TDG --explain mode (Issue #78)
//!
//! Generates actionable refactoring recommendations based on function complexity analysis.

use super::explain::{
    ActionableRecommendation, ComplexitySeverity, ExplainedTDGScore, RecommendationType,
};

/// Generate actionable recommendations from complexity analysis
///
/// # Algorithm
///
/// 1. Identify high-complexity functions (severity >= Medium)
/// 2. Generate ReduceComplexity recommendations for each
/// 3. Prioritize by TDG impact (higher = priority 1)
/// 4. Estimate effort based on complexity level
/// 5. Estimate expected impact based on current TDG impact
///
/// # Arguments
///
/// * `explained` - ExplainedTDGScore with function complexity data
///
/// # Returns
///
/// Vector of ActionableRecommendation sorted by priority and impact
pub fn generate_recommendations(explained: &ExplainedTDGScore) -> Vec<ActionableRecommendation> {
    let mut recommendations = Vec::new();

    // Generate recommendations for each function
    for func in &explained.functions {
        // Only recommend for Medium+ severity (complexity >= 6)
        if matches!(
            func.severity,
            ComplexitySeverity::Medium | ComplexitySeverity::High | ComplexitySeverity::Critical
        ) {
            let recommendation = ActionableRecommendation {
                rec_type: RecommendationType::ReduceComplexity,
                action: format!(
                    "Reduce complexity in '{}' (currently {})",
                    func.name, func.cyclomatic
                ),
                lines: vec![func.line_number],
                expected_impact: estimate_impact(func.tdg_impact, func.cyclomatic),
                estimated_hours: estimate_effort(func.cyclomatic),
                priority: calculate_priority(func.tdg_impact),
                pattern: format!("high_complexity_{}", func.severity).to_lowercase(),
            };

            recommendations.push(recommendation);
        }
    }

    // Sort by priority (ascending), then by expected impact (descending)
    recommendations.sort_by(|a, b| match a.priority.cmp(&b.priority) {
        std::cmp::Ordering::Equal => b
            .expected_impact
            .partial_cmp(&a.expected_impact)
            .unwrap_or(std::cmp::Ordering::Equal),
        other => other,
    });

    recommendations
}

/// Estimate expected TDG score improvement from reducing complexity
///
/// Formula: impact scales with current TDG impact and reduction potential
fn estimate_impact(tdg_impact: f64, cyclomatic: u32) -> f64 {
    // Higher TDG impact = more room for improvement
    // Assumes reducing to reasonable complexity (~5-10) gives proportional gains
    let reduction_ratio = if cyclomatic > 20 {
        0.7 // Critical: can reduce significantly
    } else if cyclomatic > 10 {
        0.5 // High: moderate reduction
    } else {
        0.3 // Medium: small reduction
    };

    tdg_impact * reduction_ratio
}

/// Estimate effort hours based on cyclomatic complexity
///
/// Formula: More complex functions need more refactoring time
fn estimate_effort(cyclomatic: u32) -> f64 {
    match cyclomatic {
        0..=10 => 2.0,  // Medium: 2 hours
        11..=20 => 4.0, // High: 4 hours
        21..=30 => 8.0, // Critical: 8 hours
        _ => 12.0,      // Very critical: 12+ hours
    }
}

/// Calculate priority based on TDG impact
///
/// Priority 1 = highest impact (tackle first)
/// Priority 5 = lowest impact (tackle last)
fn calculate_priority(tdg_impact: f64) -> u8 {
    if tdg_impact >= 4.0 {
        1 // Critical impact - highest priority
    } else if tdg_impact >= 3.0 {
        2 // High impact
    } else if tdg_impact >= 2.0 {
        3 // Medium impact
    } else if tdg_impact >= 1.0 {
        4 // Low impact
    } else {
        5 // Very low impact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::{FunctionComplexity, TdgScore};

    // Helper to create a function with specific values
    fn create_function(
        name: &str,
        line: usize,
        cyclomatic: u32,
        tdg_impact: f64,
        severity: ComplexitySeverity,
    ) -> FunctionComplexity {
        FunctionComplexity {
            name: name.to_string(),
            line_number: line,
            cyclomatic,
            cognitive: cyclomatic + 5,
            tdg_impact,
            severity,
        }
    }

    // === generate_recommendations Tests ===

    #[test]
    fn test_generate_recommendations_for_high_complexity() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(FunctionComplexity {
            name: "complex_function".to_string(),
            line_number: 100,
            cyclomatic: 25,
            cognitive: 30,
            tdg_impact: 4.5,
            severity: ComplexitySeverity::Critical,
        });

        let recommendations = generate_recommendations(&explained);

        assert_eq!(recommendations.len(), 1);
        assert_eq!(
            recommendations[0].rec_type,
            RecommendationType::ReduceComplexity
        );
        assert!(recommendations[0].lines.contains(&100));
        assert!(recommendations[0].expected_impact > 0.0);
        assert!(recommendations[0].estimated_hours > 0.0);
        assert_eq!(recommendations[0].priority, 1); // Highest priority for critical impact
    }

    #[test]
    fn test_no_recommendations_for_low_complexity() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(FunctionComplexity {
            name: "simple_function".to_string(),
            line_number: 10,
            cyclomatic: 3,
            cognitive: 4,
            tdg_impact: 0.5,
            severity: ComplexitySeverity::Low,
        });

        let recommendations = generate_recommendations(&explained);

        assert_eq!(
            recommendations.len(),
            0,
            "Should not recommend for low complexity"
        );
    }

    #[test]
    fn test_recommendations_sorted_by_priority() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(FunctionComplexity {
            name: "medium_impact".to_string(),
            line_number: 50,
            cyclomatic: 12,
            cognitive: 15,
            tdg_impact: 2.5,
            severity: ComplexitySeverity::High,
        });

        explained.add_function(FunctionComplexity {
            name: "high_impact".to_string(),
            line_number: 100,
            cyclomatic: 25,
            cognitive: 30,
            tdg_impact: 4.5,
            severity: ComplexitySeverity::Critical,
        });

        let recommendations = generate_recommendations(&explained);

        assert_eq!(recommendations.len(), 2);
        // First should be high_impact (priority 1)
        assert_eq!(recommendations[0].priority, 1);
        assert!(recommendations[0].lines.contains(&100));
        // Second should be medium_impact (priority 3)
        assert_eq!(recommendations[1].priority, 3);
        assert!(recommendations[1].lines.contains(&50));
    }

    #[test]
    fn test_generate_recommendations_empty() {
        let explained = ExplainedTDGScore::new(TdgScore::default());
        let recommendations = generate_recommendations(&explained);
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_generate_recommendations_medium_severity() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(create_function(
            "medium_func",
            50,
            8,
            1.5,
            ComplexitySeverity::Medium,
        ));

        let recommendations = generate_recommendations(&explained);
        assert_eq!(recommendations.len(), 1);
    }

    #[test]
    fn test_generate_recommendations_pattern_format() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(create_function(
            "test_func",
            10,
            25,
            4.0,
            ComplexitySeverity::Critical,
        ));

        let recommendations = generate_recommendations(&explained);
        assert!(recommendations[0].pattern.starts_with("high_complexity_"));
        assert!(recommendations[0].pattern.contains("critical"));
    }

    #[test]
    fn test_generate_recommendations_action_message() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(create_function(
            "my_function",
            100,
            15,
            3.0,
            ComplexitySeverity::High,
        ));

        let recommendations = generate_recommendations(&explained);
        assert!(recommendations[0].action.contains("my_function"));
        assert!(recommendations[0].action.contains("15"));
    }

    #[test]
    fn test_generate_recommendations_same_priority_sorted_by_impact() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        // Same priority (1) but different impacts
        explained.add_function(create_function(
            "lower_impact",
            10,
            25,
            4.0,
            ComplexitySeverity::Critical,
        ));

        explained.add_function(create_function(
            "higher_impact",
            20,
            30,
            5.0,
            ComplexitySeverity::Critical,
        ));

        let recommendations = generate_recommendations(&explained);

        // Both have priority 1, so sorted by impact descending
        assert_eq!(recommendations.len(), 2);
        assert!(recommendations[0].expected_impact >= recommendations[1].expected_impact);
    }

    #[test]
    fn test_generate_recommendations_multiple_severities() {
        let mut explained = ExplainedTDGScore::new(TdgScore::default());

        explained.add_function(create_function(
            "medium",
            10,
            8,
            1.5,
            ComplexitySeverity::Medium,
        ));
        explained.add_function(create_function(
            "high",
            20,
            15,
            3.0,
            ComplexitySeverity::High,
        ));
        explained.add_function(create_function(
            "critical",
            30,
            25,
            4.5,
            ComplexitySeverity::Critical,
        ));
        explained.add_function(create_function(
            "low",
            40,
            3,
            0.5,
            ComplexitySeverity::Low,
        ));

        let recommendations = generate_recommendations(&explained);

        // Low severity excluded
        assert_eq!(recommendations.len(), 3);
    }

    // === estimate_effort Tests ===

    #[test]
    fn test_effort_estimation() {
        assert_eq!(estimate_effort(8), 2.0); // Medium
        assert_eq!(estimate_effort(15), 4.0); // High
        assert_eq!(estimate_effort(25), 8.0); // Critical
        assert_eq!(estimate_effort(35), 12.0); // Very critical
    }

    #[test]
    fn test_effort_estimation_boundaries() {
        // Boundary at 10
        assert_eq!(estimate_effort(10), 2.0);
        assert_eq!(estimate_effort(11), 4.0);

        // Boundary at 20
        assert_eq!(estimate_effort(20), 4.0);
        assert_eq!(estimate_effort(21), 8.0);

        // Boundary at 30
        assert_eq!(estimate_effort(30), 8.0);
        assert_eq!(estimate_effort(31), 12.0);
    }

    #[test]
    fn test_effort_estimation_minimum() {
        assert_eq!(estimate_effort(0), 2.0);
        assert_eq!(estimate_effort(1), 2.0);
    }

    // === calculate_priority Tests ===

    #[test]
    fn test_priority_calculation() {
        assert_eq!(calculate_priority(4.5), 1); // Critical
        assert_eq!(calculate_priority(3.5), 2); // High
        assert_eq!(calculate_priority(2.5), 3); // Medium
        assert_eq!(calculate_priority(1.5), 4); // Low
        assert_eq!(calculate_priority(0.5), 5); // Very low
    }

    #[test]
    fn test_priority_calculation_boundaries() {
        // Boundary at 4.0
        assert_eq!(calculate_priority(4.0), 1);
        assert_eq!(calculate_priority(3.99), 2);

        // Boundary at 3.0
        assert_eq!(calculate_priority(3.0), 2);
        assert_eq!(calculate_priority(2.99), 3);

        // Boundary at 2.0
        assert_eq!(calculate_priority(2.0), 3);
        assert_eq!(calculate_priority(1.99), 4);

        // Boundary at 1.0
        assert_eq!(calculate_priority(1.0), 4);
        assert_eq!(calculate_priority(0.99), 5);
    }

    #[test]
    fn test_priority_calculation_zero() {
        assert_eq!(calculate_priority(0.0), 5);
    }

    #[test]
    fn test_priority_calculation_very_high() {
        assert_eq!(calculate_priority(10.0), 1);
        assert_eq!(calculate_priority(100.0), 1);
    }

    // === estimate_impact Tests ===

    #[test]
    fn test_estimate_impact_critical() {
        let impact = estimate_impact(5.0, 25);
        // Critical: 70% reduction ratio
        assert!((impact - 3.5).abs() < 0.01);
    }

    #[test]
    fn test_estimate_impact_high() {
        let impact = estimate_impact(4.0, 15);
        // High: 50% reduction ratio
        assert!((impact - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_estimate_impact_medium() {
        let impact = estimate_impact(3.0, 8);
        // Medium: 30% reduction ratio
        assert!((impact - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_estimate_impact_boundaries() {
        // Boundary at cyclomatic 20
        let impact_20 = estimate_impact(4.0, 20);
        let impact_21 = estimate_impact(4.0, 21);
        assert!(impact_21 > impact_20); // Critical should have higher ratio

        // Boundary at cyclomatic 10
        let impact_10 = estimate_impact(4.0, 10);
        let impact_11 = estimate_impact(4.0, 11);
        assert!(impact_11 > impact_10); // High should have higher ratio
    }

    #[test]
    fn test_estimate_impact_zero_tdg() {
        let impact = estimate_impact(0.0, 25);
        assert_eq!(impact, 0.0);
    }
}
