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
    fn test_effort_estimation() {
        assert_eq!(estimate_effort(8), 2.0); // Medium
        assert_eq!(estimate_effort(15), 4.0); // High
        assert_eq!(estimate_effort(25), 8.0); // Critical
        assert_eq!(estimate_effort(35), 12.0); // Very critical
    }

    #[test]
    fn test_priority_calculation() {
        assert_eq!(calculate_priority(4.5), 1); // Critical
        assert_eq!(calculate_priority(3.5), 2); // High
        assert_eq!(calculate_priority(2.5), 3); // Medium
        assert_eq!(calculate_priority(1.5), 4); // Low
        assert_eq!(calculate_priority(0.5), 5); // Very low
    }
}
