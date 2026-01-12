//! Convergence criteria and status tracking
//!
//! Implements quality gates for the "perfect" project state.

use super::types::*;
use serde::{Deserialize, Serialize};

/// Convergence tracker for monitoring progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConvergenceTracker {
    pub iterations: usize,
    pub history: Vec<ConvergenceSnapshot>,
    pub best_metrics: Option<ProjectMetrics>,
    pub current_status: Option<ConvergenceStatus>,
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceSnapshot {
    pub iteration: usize,
    pub metrics: ProjectMetrics,
    pub defects_remaining: usize,
    pub status: ConvergenceStatus,
}

impl ConvergenceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new iteration
    pub fn record(
        &mut self,
        metrics: ProjectMetrics,
        defects_remaining: usize,
        targets: &ConvergenceTargets,
    ) {
        self.iterations += 1;
        let status = targets.check(&metrics);

        // Track best metrics seen
        if self.best_metrics.is_none() || self.is_better(&metrics) {
            self.best_metrics = Some(metrics.clone());
        }

        self.history.push(ConvergenceSnapshot {
            iteration: self.iterations,
            metrics,
            defects_remaining,
            status: status.clone(),
        });

        self.current_status = Some(status);
    }

    /// Check if new metrics are better than best
    fn is_better(&self, metrics: &ProjectMetrics) -> bool {
        let Some(best) = &self.best_metrics else {
            return true;
        };

        // Score based on multiple factors
        let new_score = self.calculate_score(metrics);
        let best_score = self.calculate_score(best);

        new_score > best_score
    }

    /// Calculate a composite quality score
    fn calculate_score(&self, metrics: &ProjectMetrics) -> f32 {
        let mut score = 0.0;

        // Coverage contribution (weight: 0.25)
        score += metrics.test_coverage * 0.25;

        // Mutation score contribution (weight: 0.15)
        score += metrics.mutation_score * 0.15;

        // No compiler errors is critical (weight: 0.20)
        if metrics.compiler_errors == 0 {
            score += 0.20;
        }

        // No clippy warnings (weight: 0.10)
        if metrics.clippy_warnings == 0 {
            score += 0.10;
        }

        // No test failures (weight: 0.15)
        if metrics.test_failures == 0 {
            score += 0.15;
        }

        // TDG score contribution (weight: 0.10)
        score += (metrics.tdg_score / 100.0) * 0.10;

        // Rust project score contribution (weight: 0.05)
        score += (metrics.rust_project_score as f32 / 106.0) * 0.05;

        score
    }

    /// Get convergence percentage (0.0 - 1.0)
    pub fn convergence_percentage(&self, targets: &ConvergenceTargets) -> f32 {
        let Some(best) = &self.best_metrics else {
            return 0.0;
        };

        let mut achieved = 0.0;
        let mut total = 0.0;

        // Coverage
        total += 1.0;
        if best.test_coverage >= targets.test_coverage {
            achieved += 1.0;
        } else {
            achieved += best.test_coverage / targets.test_coverage;
        }

        // Mutation score
        total += 1.0;
        if best.mutation_score >= targets.mutation_score {
            achieved += 1.0;
        } else {
            achieved += best.mutation_score / targets.mutation_score;
        }

        // Compiler errors
        total += 1.0;
        if best.compiler_errors <= targets.max_compiler_errors {
            achieved += 1.0;
        }

        // Clippy warnings
        total += 1.0;
        if best.clippy_warnings <= targets.max_clippy_warnings {
            achieved += 1.0;
        }

        // Test failures
        total += 1.0;
        if best.test_failures <= targets.max_test_failures {
            achieved += 1.0;
        }

        // TDG score
        total += 1.0;
        if best.tdg_score >= targets.min_tdg_score {
            achieved += 1.0;
        } else {
            achieved += best.tdg_score / targets.min_tdg_score;
        }

        // Rust project score
        total += 1.0;
        if best.rust_project_score >= targets.min_rust_project_score {
            achieved += 1.0;
        } else {
            achieved += best.rust_project_score as f32 / targets.min_rust_project_score as f32;
        }

        // SATD markers
        total += 1.0;
        if best.satd_markers <= targets.max_satd_markers {
            achieved += 1.0;
        }

        // Dead code
        total += 1.0;
        if best.dead_code_items <= targets.max_dead_code {
            achieved += 1.0;
        }

        achieved / total
    }

    /// Check if converged
    pub fn is_converged(&self) -> bool {
        matches!(self.current_status, Some(ConvergenceStatus::Converged))
    }

    /// Get remaining failures
    pub fn remaining_failures(&self) -> Vec<String> {
        match &self.current_status {
            Some(ConvergenceStatus::NotConverged { remaining }) => remaining.clone(),
            _ => Vec::new(),
        }
    }

    /// Get improvement trend (positive = improving)
    pub fn trend(&self) -> f32 {
        if self.history.len() < 2 {
            return 0.0;
        }

        let recent: Vec<_> = self.history.iter().rev().take(5).collect();
        if recent.len() < 2 {
            return 0.0;
        }

        let first_defects = recent.last().expect("internal error").defects_remaining as f32;
        let last_defects = recent.first().expect("internal error").defects_remaining as f32;

        if first_defects == 0.0 {
            return 0.0;
        }

        (first_defects - last_defects) / first_defects
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_metrics() -> ProjectMetrics {
        ProjectMetrics {
            test_coverage: 0.90,
            mutation_score: 0.80,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 90.0,
            rust_project_score: 85,
            satd_markers: 0,
            dead_code_items: 0,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 15,
            build_time: Duration::from_secs(30),
        }
    }

    fn create_poor_metrics() -> ProjectMetrics {
        ProjectMetrics {
            test_coverage: 0.50,
            mutation_score: 0.40,
            compiler_errors: 5,
            clippy_warnings: 10,
            test_failures: 3,
            tdg_score: 50.0,
            rust_project_score: 40,
            satd_markers: 5,
            dead_code_items: 10,
            max_cyclomatic_complexity: 30,
            max_cognitive_complexity: 50,
            build_time: Duration::from_secs(120),
        }
    }

    // ==================== ConvergenceTracker Basic Tests ====================

    #[test]
    fn test_convergence_tracker_new() {
        let tracker = ConvergenceTracker::new();

        assert_eq!(tracker.iterations, 0);
        assert!(tracker.history.is_empty());
        assert!(tracker.best_metrics.is_none());
        assert!(tracker.current_status.is_none());
    }

    #[test]
    fn test_convergence_tracker_default() {
        let tracker = ConvergenceTracker::default();

        assert_eq!(tracker.iterations, 0);
        assert!(tracker.history.is_empty());
    }

    // ==================== Record Tests ====================

    #[test]
    fn test_record_first_iteration() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();
        let metrics = create_test_metrics();

        tracker.record(metrics.clone(), 10, &targets);

        assert_eq!(tracker.iterations, 1);
        assert_eq!(tracker.history.len(), 1);
        assert!(tracker.best_metrics.is_some());
        assert!(tracker.current_status.is_some());
    }

    #[test]
    fn test_record_multiple_iterations() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        tracker.record(create_poor_metrics(), 20, &targets);
        tracker.record(create_test_metrics(), 10, &targets);
        tracker.record(create_test_metrics(), 5, &targets);

        assert_eq!(tracker.iterations, 3);
        assert_eq!(tracker.history.len(), 3);
    }

    #[test]
    fn test_record_updates_best_metrics() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // First record poor metrics
        tracker.record(create_poor_metrics(), 20, &targets);
        let first_best = tracker.best_metrics.clone();

        // Then record better metrics
        tracker.record(create_test_metrics(), 10, &targets);
        let second_best = tracker.best_metrics.clone();

        // Best should be updated
        assert!(second_best.is_some());
        assert_ne!(
            first_best.as_ref().map(|m| m.test_coverage),
            second_best.as_ref().map(|m| m.test_coverage)
        );
    }

    // ==================== Score Calculation Tests ====================

    #[test]
    fn test_calculate_score_perfect_metrics() {
        let tracker = ConvergenceTracker::new();
        let metrics = ProjectMetrics {
            test_coverage: 1.0,
            mutation_score: 1.0,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 100.0,
            rust_project_score: 106,
            ..Default::default()
        };

        let score = tracker.calculate_score(&metrics);
        // Should be close to max score (0.25 + 0.15 + 0.20 + 0.10 + 0.15 + 0.10 + 0.05 = 1.0)
        assert!(score > 0.95);
    }

    #[test]
    fn test_calculate_score_zero_metrics() {
        let tracker = ConvergenceTracker::new();
        let metrics = ProjectMetrics::default();

        let score = tracker.calculate_score(&metrics);
        // With all zeros but no errors, should still get some points
        assert!(score >= 0.0);
    }

    #[test]
    fn test_calculate_score_with_errors() {
        let tracker = ConvergenceTracker::new();
        let metrics = ProjectMetrics {
            compiler_errors: 5,
            clippy_warnings: 10,
            test_failures: 3,
            ..Default::default()
        };

        let score = tracker.calculate_score(&metrics);
        // Should be lower due to errors
        assert!(score < 0.5);
    }

    // ==================== Convergence Percentage Tests ====================

    #[test]
    fn test_convergence_percentage_no_metrics() {
        let tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        let percentage = tracker.convergence_percentage(&targets);
        assert_eq!(percentage, 0.0);
    }

    #[test]
    fn test_convergence_percentage_perfect_metrics() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.95,
            mutation_score: 0.85,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 95.0,
            rust_project_score: 90,
            satd_markers: 0,
            dead_code_items: 0,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 20,
            build_time: Duration::from_secs(30),
        };

        tracker.record(metrics, 0, &targets);
        let percentage = tracker.convergence_percentage(&targets);

        // Should be 100%
        assert!((percentage - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convergence_percentage_partial() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.50, // 50% of target (0.95)
            mutation_score: 0.50,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 50.0,
            rust_project_score: 45,
            satd_markers: 0,
            dead_code_items: 0,
            ..Default::default()
        };

        tracker.record(metrics, 10, &targets);
        let percentage = tracker.convergence_percentage(&targets);

        // Should be between 0 and 1
        assert!(percentage > 0.0);
        assert!(percentage < 1.0);
    }

    // ==================== Is Converged Tests ====================

    #[test]
    fn test_is_converged_not_started() {
        let tracker = ConvergenceTracker::new();
        assert!(!tracker.is_converged());
    }

    #[test]
    fn test_is_converged_false() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        tracker.record(create_poor_metrics(), 20, &targets);
        assert!(!tracker.is_converged());
    }

    #[test]
    fn test_is_converged_true() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.96,
            mutation_score: 0.86,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 96.0,
            rust_project_score: 91,
            satd_markers: 0,
            dead_code_items: 0,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 20,
            build_time: Duration::from_secs(30),
        };

        tracker.record(metrics, 0, &targets);
        assert!(tracker.is_converged());
    }

    // ==================== Remaining Failures Tests ====================

    #[test]
    fn test_remaining_failures_not_started() {
        let tracker = ConvergenceTracker::new();
        let failures = tracker.remaining_failures();
        assert!(failures.is_empty());
    }

    #[test]
    fn test_remaining_failures_converged() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        let metrics = ProjectMetrics {
            test_coverage: 0.96,
            mutation_score: 0.86,
            compiler_errors: 0,
            clippy_warnings: 0,
            test_failures: 0,
            tdg_score: 96.0,
            rust_project_score: 91,
            satd_markers: 0,
            dead_code_items: 0,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 20,
            build_time: Duration::from_secs(30),
        };

        tracker.record(metrics, 0, &targets);
        let failures = tracker.remaining_failures();
        assert!(failures.is_empty());
    }

    #[test]
    fn test_remaining_failures_not_converged() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        tracker.record(create_poor_metrics(), 20, &targets);
        let failures = tracker.remaining_failures();

        assert!(!failures.is_empty());
        // Should have multiple failures
        assert!(failures.len() > 1);
    }

    // ==================== Trend Tests ====================

    #[test]
    fn test_trend_empty() {
        let tracker = ConvergenceTracker::new();
        assert_eq!(tracker.trend(), 0.0);
    }

    #[test]
    fn test_trend_single_iteration() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        tracker.record(create_test_metrics(), 10, &targets);
        assert_eq!(tracker.trend(), 0.0);
    }

    #[test]
    fn test_trend_improving() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // Start with 20 defects, end with 10 = 50% improvement
        tracker.record(create_poor_metrics(), 20, &targets);
        tracker.record(create_test_metrics(), 10, &targets);

        let trend = tracker.trend();
        assert!(trend > 0.0);
        assert!((trend - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_trend_degrading() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // Start with 10 defects, end with 20 = negative trend
        tracker.record(create_test_metrics(), 10, &targets);
        tracker.record(create_poor_metrics(), 20, &targets);

        let trend = tracker.trend();
        assert!(trend < 0.0);
    }

    #[test]
    fn test_trend_stable() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // Same defects = 0 trend
        tracker.record(create_test_metrics(), 10, &targets);
        tracker.record(create_test_metrics(), 10, &targets);

        let trend = tracker.trend();
        assert_eq!(trend, 0.0);
    }

    #[test]
    fn test_trend_zero_start() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // Start with 0, end with something = 0 trend (avoid division by zero)
        tracker.record(create_test_metrics(), 0, &targets);
        tracker.record(create_test_metrics(), 5, &targets);

        let trend = tracker.trend();
        assert_eq!(trend, 0.0);
    }

    // ==================== ConvergenceSnapshot Tests ====================

    #[test]
    fn test_convergence_snapshot_creation() {
        let snapshot = ConvergenceSnapshot {
            iteration: 1,
            metrics: create_test_metrics(),
            defects_remaining: 5,
            status: ConvergenceStatus::NotConverged {
                remaining: vec!["Coverage too low".to_string()],
            },
        };

        assert_eq!(snapshot.iteration, 1);
        assert_eq!(snapshot.defects_remaining, 5);
    }

    #[test]
    fn test_convergence_snapshot_serialization() {
        let snapshot = ConvergenceSnapshot {
            iteration: 1,
            metrics: create_test_metrics(),
            defects_remaining: 5,
            status: ConvergenceStatus::Converged,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let parsed: ConvergenceSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(snapshot.iteration, parsed.iteration);
        assert_eq!(snapshot.defects_remaining, parsed.defects_remaining);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_convergence_workflow() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        // Simulate improvement over iterations
        let mut defects = 20;
        for i in 0..5 {
            let coverage = 0.7 + (i as f32 * 0.05);
            let metrics = ProjectMetrics {
                test_coverage: coverage,
                mutation_score: 0.7 + (i as f32 * 0.03),
                compiler_errors: std::cmp::max(0, 3 - i as usize),
                clippy_warnings: std::cmp::max(0, 5 - i as usize),
                test_failures: std::cmp::max(0, 2 - i as usize),
                tdg_score: 70.0 + (i as f32 * 5.0),
                rust_project_score: 60 + (i * 5) as u32,
                satd_markers: std::cmp::max(0, 3 - i as usize),
                dead_code_items: std::cmp::max(0, 5 - i as usize),
                ..Default::default()
            };

            defects = std::cmp::max(0, defects - 4);
            tracker.record(metrics, defects, &targets);
        }

        assert_eq!(tracker.iterations, 5);
        assert_eq!(tracker.history.len(), 5);

        // Trend should be positive (improving)
        let trend = tracker.trend();
        assert!(trend > 0.0);

        // Convergence percentage should have increased
        let percentage = tracker.convergence_percentage(&targets);
        assert!(percentage > 0.5);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut tracker = ConvergenceTracker::new();
        let targets = ConvergenceTargets::default();

        tracker.record(create_test_metrics(), 10, &targets);
        tracker.record(create_poor_metrics(), 15, &targets);

        let json = serde_json::to_string(&tracker).unwrap();
        let parsed: ConvergenceTracker = serde_json::from_str(&json).unwrap();

        assert_eq!(tracker.iterations, parsed.iterations);
        assert_eq!(tracker.history.len(), parsed.history.len());
    }
}
