/// Convergence targets for the oracle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceTargets {
    pub test_coverage: f32,
    pub mutation_score: f32,
    pub max_compiler_errors: usize,
    pub max_clippy_warnings: usize,
    pub max_test_failures: usize,
    pub min_tdg_score: f32,
    pub min_rust_project_score: u32,
    pub max_satd_markers: usize,
    pub max_dead_code: usize,
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub max_build_time: Duration,
}

impl Default for ConvergenceTargets {
    fn default() -> Self {
        Self {
            test_coverage: 0.95,
            mutation_score: 0.85,
            max_compiler_errors: 0,
            max_clippy_warnings: 0,
            max_test_failures: 0,
            min_tdg_score: 95.0,
            min_rust_project_score: 90,
            max_satd_markers: 0,
            max_dead_code: 0,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 25,
            max_build_time: Duration::from_secs(60),
        }
    }
}

/// Current project metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub test_coverage: f32,
    pub mutation_score: f32,
    pub compiler_errors: usize,
    pub clippy_warnings: usize,
    pub test_failures: usize,
    pub tdg_score: f32,
    pub rust_project_score: u32,
    pub satd_markers: usize,
    pub dead_code_items: usize,
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub build_time: Duration,
}

/// Convergence status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvergenceStatus {
    Converged,
    NotConverged { remaining: Vec<String> },
}

impl ConvergenceTargets {
    /// Check if metrics meet convergence criteria
    pub fn check(&self, metrics: &ProjectMetrics) -> ConvergenceStatus {
        let mut failures = Vec::new();

        if metrics.test_coverage < self.test_coverage {
            failures.push(format!(
                "Coverage: {:.1}% < {:.1}%",
                metrics.test_coverage * 100.0,
                self.test_coverage * 100.0
            ));
        }

        if metrics.mutation_score < self.mutation_score {
            failures.push(format!(
                "Mutation score: {:.1}% < {:.1}%",
                metrics.mutation_score * 100.0,
                self.mutation_score * 100.0
            ));
        }

        if metrics.compiler_errors > self.max_compiler_errors {
            failures.push(format!(
                "Compiler errors: {} > {}",
                metrics.compiler_errors, self.max_compiler_errors
            ));
        }

        if metrics.clippy_warnings > self.max_clippy_warnings {
            failures.push(format!(
                "Clippy warnings: {} > {}",
                metrics.clippy_warnings, self.max_clippy_warnings
            ));
        }

        if metrics.test_failures > self.max_test_failures {
            failures.push(format!(
                "Test failures: {} > {}",
                metrics.test_failures, self.max_test_failures
            ));
        }

        if metrics.tdg_score < self.min_tdg_score {
            failures.push(format!(
                "TDG score: {:.1} < {:.1}",
                metrics.tdg_score, self.min_tdg_score
            ));
        }

        if metrics.rust_project_score < self.min_rust_project_score {
            failures.push(format!(
                "Rust project score: {} < {}",
                metrics.rust_project_score, self.min_rust_project_score
            ));
        }

        if metrics.satd_markers > self.max_satd_markers {
            failures.push(format!(
                "SATD markers: {} > {}",
                metrics.satd_markers, self.max_satd_markers
            ));
        }

        if metrics.dead_code_items > self.max_dead_code {
            failures.push(format!(
                "Dead code items: {} > {}",
                metrics.dead_code_items, self.max_dead_code
            ));
        }

        if metrics.max_cyclomatic_complexity > self.max_cyclomatic_complexity {
            failures.push(format!(
                "Cyclomatic complexity: {} > {}",
                metrics.max_cyclomatic_complexity, self.max_cyclomatic_complexity
            ));
        }

        if metrics.max_cognitive_complexity > self.max_cognitive_complexity {
            failures.push(format!(
                "Cognitive complexity: {} > {}",
                metrics.max_cognitive_complexity, self.max_cognitive_complexity
            ));
        }

        if failures.is_empty() {
            ConvergenceStatus::Converged
        } else {
            ConvergenceStatus::NotConverged {
                remaining: failures,
            }
        }
    }
}
