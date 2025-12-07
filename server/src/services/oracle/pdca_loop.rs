//! PDCA (Plan-Do-Check-Act) Execution Loop
//!
//! Implements the core improvement cycle with CITL learning.

use super::signal_collector::AggregatedCollector;
use super::types::*;
use anyhow::{bail, Result};
use std::path::Path;

/// Result of a PDCA iteration
#[derive(Debug, Clone)]
pub struct PdcaIterationResult {
    pub iteration: usize,
    pub defects_found: usize,
    pub defects_fixed: usize,
    pub defects_skipped: usize,
    pub metrics_before: ProjectMetrics,
    pub metrics_after: ProjectMetrics,
    pub converged: bool,
}

/// PDCA Loop executor
pub struct PdcaLoop {
    config: OracleConfig,
    targets: ConvergenceTargets,
    collector: AggregatedCollector,
}

impl PdcaLoop {
    /// Create a new PDCA loop with default configuration
    pub fn new() -> Self {
        Self {
            config: OracleConfig::default(),
            targets: ConvergenceTargets::default(),
            collector: AggregatedCollector::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: OracleConfig, targets: ConvergenceTargets) -> Self {
        Self {
            config,
            targets,
            collector: AggregatedCollector::new(),
        }
    }

    /// Get max iterations configured
    pub fn max_iterations(&self) -> usize {
        self.config.max_iterations
    }

    /// Run the PDCA loop until convergence or max iterations
    pub async fn run(&self, project_path: &Path) -> Result<Vec<PdcaIterationResult>> {
        if !project_path.exists() {
            bail!("Project path does not exist: {}", project_path.display());
        }

        let mut results = Vec::new();
        let mut stagnation_count = 0;
        let mut previous_defect_count = usize::MAX;

        for iteration in 1..=self.config.max_iterations {
            // PLAN: Collect signals and analyze
            let metrics_before = self.collect_metrics(project_path).await?;
            let signals = self.collector.collect_all(project_path).await?;
            let mut defects = self.collector.signals_to_defects(signals);

            // Update decisions based on thresholds
            for defect in &mut defects {
                defect.update_decision(
                    self.config.auto_apply_threshold,
                    self.config.review_threshold,
                );
            }

            // Check for stagnation (Andon principle)
            if defects.len() == previous_defect_count {
                stagnation_count += 1;
                if stagnation_count >= self.config.stagnation_threshold
                    && self.config.andon_enabled
                {
                    eprintln!(
                        "ANDON: Stagnation detected after {} iterations with {} defects",
                        stagnation_count,
                        defects.len()
                    );
                    // Continue but note the stagnation
                }
            } else {
                stagnation_count = 0;
            }
            previous_defect_count = defects.len();

            // DO: Apply fixes (for auto-apply defects only in this implementation)
            let auto_apply_defects: Vec<_> = defects
                .iter()
                .filter(|d| d.decision == OracleDecision::AutoApply)
                .take(self.config.batch_size)
                .collect();

            let defects_fixed = self.apply_fixes(&auto_apply_defects, project_path).await?;

            // CHECK: Verify fixes worked
            let metrics_after = self.collect_metrics(project_path).await?;

            // Detect regression (Andon)
            if self.config.andon_enabled {
                self.check_regression(&metrics_before, &metrics_after)?;
            }

            // ACT: Learn from results (pattern capture would go here)
            let status = self.targets.check(&metrics_after);
            let converged = matches!(status, ConvergenceStatus::Converged);

            let result = PdcaIterationResult {
                iteration,
                defects_found: defects.len(),
                defects_fixed,
                defects_skipped: defects.len() - auto_apply_defects.len(),
                metrics_before,
                metrics_after,
                converged,
            };

            results.push(result.clone());

            if converged {
                eprintln!("CONVERGED at iteration {}", iteration);
                break;
            }

            // Check for sufficient progress
            if iteration > 1 {
                let progress = self.calculate_progress(&results);
                if progress < self.config.min_progress_per_iteration {
                    eprintln!(
                        "Insufficient progress ({:.4} < {:.4}), continuing...",
                        progress, self.config.min_progress_per_iteration
                    );
                }
            }
        }

        Ok(results)
    }

    /// Run a single PDCA iteration (for CI/CD)
    pub async fn run_single(&self, project_path: &Path) -> Result<PdcaIterationResult> {
        let results = self.run_iterations(project_path, 1).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No iteration result"))
    }

    /// Run N iterations
    pub async fn run_iterations(
        &self,
        project_path: &Path,
        max_iterations: usize,
    ) -> Result<Vec<PdcaIterationResult>> {
        let limited_config = OracleConfig {
            max_iterations,
            ..self.config.clone()
        };

        let limited_loop = PdcaLoop {
            config: limited_config,
            targets: self.targets.clone(),
            collector: AggregatedCollector::new(),
        };

        limited_loop.run(project_path).await
    }

    /// Collect current project metrics
    async fn collect_metrics(&self, project_path: &Path) -> Result<ProjectMetrics> {
        // Simplified metrics collection - would integrate with actual PMAT commands
        let signals = self.collector.collect_all(project_path).await?;

        let compiler_errors = signals
            .iter()
            .filter(|s| s.source == SignalSource::Rustc)
            .count();

        let clippy_warnings = signals
            .iter()
            .filter(|s| s.source == SignalSource::Clippy)
            .count();

        let test_failures = signals
            .iter()
            .filter(|s| s.source == SignalSource::CargoTest)
            .count();

        Ok(ProjectMetrics {
            compiler_errors,
            clippy_warnings,
            test_failures,
            // Other metrics would be collected from respective tools
            ..Default::default()
        })
    }

    /// Apply fixes for defects
    async fn apply_fixes(
        &self,
        defects: &[&DefectReport],
        project_path: &Path,
    ) -> Result<usize> {
        let mut fixed_count = 0;

        for defect in defects {
            for fix in &defect.suggested_fixes {
                match &fix.fix_type {
                    FixType::ClippyAutoFix => {
                        // Run cargo clippy --fix
                        let output = std::process::Command::new("cargo")
                            .args(["clippy", "--fix", "--allow-dirty", "--allow-staged"])
                            .current_dir(project_path)
                            .output()?;

                        if output.status.success() {
                            fixed_count += 1;
                        }
                    }
                    FixType::Replacement { old, new } => {
                        // Apply text replacement
                        let content =
                            std::fs::read_to_string(&defect.location.file_path)?;
                        let updated = content.replace(old, new);
                        std::fs::write(&defect.location.file_path, updated)?;
                        fixed_count += 1;
                    }
                    _ => {
                        // Other fix types not yet implemented
                    }
                }
            }
        }

        Ok(fixed_count)
    }

    /// Check for regression after applying fixes
    fn check_regression(
        &self,
        before: &ProjectMetrics,
        after: &ProjectMetrics,
    ) -> Result<()> {
        // Coverage should not decrease significantly
        if after.test_coverage < before.test_coverage - 0.01 {
            bail!(
                "ANDON: Coverage decreased: {:.1}% → {:.1}%",
                before.test_coverage * 100.0,
                after.test_coverage * 100.0
            );
        }

        // Compiler errors should not increase
        if after.compiler_errors > before.compiler_errors {
            bail!(
                "ANDON: Compiler errors increased: {} → {}",
                before.compiler_errors,
                after.compiler_errors
            );
        }

        // Test failures should not increase
        if after.test_failures > before.test_failures {
            bail!(
                "ANDON: Test failures increased: {} → {}",
                before.test_failures,
                after.test_failures
            );
        }

        Ok(())
    }

    /// Calculate progress across iterations
    fn calculate_progress(&self, results: &[PdcaIterationResult]) -> f32 {
        if results.len() < 2 {
            return 1.0;
        }

        let first = &results[0];
        let last = results.last().unwrap();

        let initial_defects = first.defects_found as f32;
        let current_defects = last.defects_found as f32;

        if initial_defects == 0.0 {
            return 1.0;
        }

        (initial_defects - current_defects) / initial_defects
    }
}

impl Default for PdcaLoop {
    fn default() -> Self {
        Self::new()
    }
}
