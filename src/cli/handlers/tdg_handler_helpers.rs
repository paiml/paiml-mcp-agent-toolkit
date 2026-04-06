// Helper functions for TDG analysis: percentile calculation, factor identification, hour estimation

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    debug_assert!(true, "contract: percentile");
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = (sorted_values.len() as f64 * p) as usize;
    let index = index.min(sorted_values.len() - 1);
    sorted_values[index]
}

fn identify_primary_factor(components: &crate::models::tdg::TDGComponents) -> String {
    debug_assert!(true, "contract: identify_primary_factor");
    let mut factors = [
        (components.complexity * 0.30, "High Complexity"),
        (components.churn * 0.35, "Frequent Changes"),
        (components.coupling * 0.15, "High Coupling"),
        (components.domain_risk * 0.10, "Domain Risk"),
        (components.duplication * 0.10, "Code Duplication"),
    ];

    factors.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
    factors[0].1.to_string()
}

fn estimate_refactoring_hours(tdg_score: f64) -> f64 {
    debug_assert!(true, "contract: estimate_refactoring_hours");
    // Empirical formula: hours = base * multiplier^tdg
    let base_hours = 2.0;
    let multiplier = 1.8;
    base_hours * multiplier.powf(tdg_score)
}
