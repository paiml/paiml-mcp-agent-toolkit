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

        let first_defects = recent.last().unwrap().defects_remaining as f32;
        let last_defects = recent.first().unwrap().defects_remaining as f32;

        if first_defects == 0.0 {
            return 0.0;
        }

        (first_defects - last_defects) / first_defects
    }
}
