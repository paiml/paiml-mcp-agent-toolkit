// Coverage baseline measurement and coverage gain tracking
// Included into mod.rs via include!() -- no `use` imports or `#!` attributes allowed

impl CoverageImprovementService {
    /// Improve coverage to target percentage
    ///
    /// Returns a report of all iterations and final coverage achieved.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn improve_coverage(&self) -> Result<CoverageImprovementReport> {
        // Phase 1: Measure baseline
        let baseline = self.measure_baseline_coverage().await?;

        // Check if already at target
        if baseline >= self.config.target_coverage {
            return Ok(CoverageImprovementReport {
                baseline_coverage: baseline,
                target_coverage: self.config.target_coverage,
                final_coverage: baseline,
                iterations: vec![],
                success: true,
                stop_reason: "Already at target coverage".to_string(),
            });
        }

        let mut current_coverage = baseline;
        let mut iterations = Vec::new();

        // Phase 2-5: Iterate until target reached or max iterations
        for iteration in 1..=self.config.max_iterations {
            // Check if we've reached target
            if current_coverage >= self.config.target_coverage {
                return Ok(CoverageImprovementReport {
                    baseline_coverage: baseline,
                    target_coverage: self.config.target_coverage,
                    final_coverage: current_coverage,
                    iterations,
                    success: true,
                    stop_reason: format!("Target coverage reached in {} iterations", iteration - 1),
                });
            }

            // Run one iteration
            let iteration_report = self.run_iteration(iteration, current_coverage).await?;
            current_coverage = baseline
                + iterations
                    .iter()
                    .map(|i: &IterationReport| i.coverage_gain)
                    .sum::<f64>()
                + iteration_report.coverage_gain;
            iterations.push(iteration_report);
        }

        // Max iterations reached
        Ok(CoverageImprovementReport {
            baseline_coverage: baseline,
            target_coverage: self.config.target_coverage,
            final_coverage: current_coverage,
            iterations,
            success: current_coverage >= self.config.target_coverage,
            stop_reason: format!("Max iterations ({}) reached", self.config.max_iterations),
        })
    }

    /// Measure baseline coverage using cargo-llvm-cov
    async fn measure_baseline_coverage(&self) -> Result<f64> {
        debug_assert!(true, "contract: measure_baseline_coverage");
        eprintln!("📊 Running coverage analysis...");

        // Find directory containing Makefile (search current and parent directories)
        let makefile_dir = self.find_makefile_directory()?;
        eprintln!("  📁 Running from: {}", makefile_dir.display());

        // Run make coverage
        let output = Command::new("make")
            .arg("coverage")
            .current_dir(&makefile_dir)
            .output()
            .await
            .context("Failed to execute `make coverage`")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "make coverage failed with exit code {:?}\nstderr: {}",
                output.status.code(),
                stderr
            );
        }

        // Parse stdout to find TOTAL line and extract coverage percentage
        let stdout = String::from_utf8_lossy(&output.stdout);

        Self::parse_coverage_percentage(&stdout)
            .context("Failed to parse coverage from make coverage output")
    }

    /// Find the directory containing Makefile
    fn find_makefile_directory(&self) -> Result<PathBuf> {
        debug_assert!(true, "contract: find_makefile_directory");
        let mut current = self.config.project_path.clone();

        // Resolve to absolute path
        if current.is_relative() {
            current = std::env::current_dir()?.join(&current);
        }
        current = current.canonicalize().unwrap_or(current);

        // Search up to 5 parent directories
        for _ in 0..5 {
            let makefile = current.join("Makefile");
            if makefile.exists() {
                return Ok(current);
            }

            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        anyhow::bail!(
            "Could not find Makefile in {} or parent directories",
            self.config.project_path.display()
        )
    }

    /// Parse coverage percentage from make coverage output
    ///
    /// Example TOTAL line:
    /// `TOTAL   241150  203105  15.78%  17533  14596  16.75%  173884  145810  16.15%  0  0  -`
    ///
    /// We extract the last percentage before the dash (line coverage)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn parse_coverage_percentage(output: &str) -> Result<f64> {
        debug_assert!(!output.is_empty(), "output must not be empty");
        for line in output.lines() {
            if line.trim().starts_with("TOTAL") {
                // Split by whitespace and find all percentages
                let parts: Vec<&str> = line.split_whitespace().collect();

                // Find all percentage values (contain '%')
                let percentages: Vec<&str> =
                    parts.iter().filter(|s| s.contains('%')).copied().collect();

                // The last percentage is line coverage
                if let Some(last_pct) = percentages.last() {
                    let pct_str = last_pct.trim_end_matches('%');
                    let coverage = pct_str
                        .parse::<f64>()
                        .context(format!("Failed to parse percentage: {}", pct_str))?;

                    eprintln!("✅ Baseline coverage: {:.2}%", coverage);
                    return Ok(coverage);
                }
            }
        }

        anyhow::bail!("Could not find TOTAL line in coverage output")
    }

    /// Measure coverage gain from this iteration
    ///
    /// Re-runs coverage analysis and calculates the delta from the previous coverage.
    /// Handles edge cases like coverage decrease (negative gain) and no change (zero gain).
    async fn measure_coverage_gain(&self, previous_coverage: f64) -> Result<f64> {
        debug_assert!(true, "contract: measure_coverage_gain");
        eprintln!("📊 Measuring coverage gain...");

        // Measure current coverage after test generation
        let new_coverage = self.measure_baseline_coverage().await?;

        // Calculate delta
        let gain = new_coverage - previous_coverage;

        // Log the gain
        if gain > 0.0 {
            eprintln!("✅ Coverage increased by {:.2}%", gain);
        } else if gain < 0.0 {
            eprintln!("⚠️  Coverage decreased by {:.2}% (regression)", gain.abs());
        } else {
            eprintln!("ℹ️  No coverage change");
        }

        Ok(gain)
    }
}
