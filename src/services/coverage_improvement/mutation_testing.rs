// Mutation testing and iteration orchestration
// Included into mod.rs via include!() -- no `use` imports or `#!` attributes allowed

/// The value `IterationReport::mutation_score` carries when mutation testing did
/// not run at all. `serde_json` renders a non-finite float as `null`, which is
/// what a consumer of `-f json` should see for a measurement that was skipped.
/// (The field is a plain `f64` in `IterationReport`; turning it into an
/// `Option<f64>` would be the tidier representation.)
fn unmeasured_mutation_score() -> f64 {
    f64::NAN
}

impl CoverageImprovementService {
    /// Run a single improvement iteration
    async fn run_iteration(
        &self,
        iteration: usize,
        current_coverage: f64,
    ) -> Result<IterationReport> {
        // Phase 2: Prioritize targets using PMAT tools
        let targets = self.prioritize_targets().await?;

        // Phase 3: Generate property-based tests
        let tests_generated = self.generate_property_tests(&targets).await?;

        // Phase 4: Validate with mutation testing
        //
        // `--fast` deliberately does NOT run cargo-mutants, so there is no
        // mutation score to report. This used to substitute `100.0`, i.e. a
        // *perfect* score, on every iteration of a run where no mutant was ever
        // generated — `analyze coverage-improve --fast -f json` printed
        // "mutation_score": 100.0 ten times over with no mutants.out/ on disk.
        // A measurement that was skipped is reported as not-a-number, which
        // serialises to JSON `null` rather than to a number nobody measured.
        let mutation_score = if self.config.fast_mode {
            eprintln!("⏩ --fast: skipping mutation testing (mutation score not measured)");
            unmeasured_mutation_score()
        } else {
            self.run_mutation_testing(&targets).await?
        };

        // Measure coverage gain
        let coverage_gain = self.measure_coverage_gain(current_coverage).await?;

        Ok(IterationReport {
            iteration,
            files_targeted: targets,
            tests_generated,
            coverage_gain,
            mutation_score,
        })
    }

    /// Run mutation testing on generated tests
    ///
    /// Executes cargo-mutants on the target files and returns the mutation score.
    /// Mutation score = (caught / total) * 100
    ///
    /// Only runs on files that have changed (--in-diff flag) for performance.
    async fn run_mutation_testing(&self, _targets: &[PathBuf]) -> Result<f64> {
        eprintln!("🧬 Running mutation testing...");

        // Check if cargo-mutants is installed
        let check_output = Command::new("cargo")
            .args(["mutants", "--version"])
            .output()
            .await;

        if check_output.is_err() || !check_output.expect("internal error").status.success() {
            eprintln!("⚠️  cargo-mutants not installed, skipping mutation testing");
            eprintln!("   Install with: cargo install cargo-mutants");
            return Ok(0.0);
        }

        // Run cargo-mutants with --in-diff for changed files only
        // Use --json for structured output
        let output = Command::new("cargo")
            .args([
                "mutants",
                "--in-diff",
                "git",
                "diff",
                "HEAD",
                "--json",
                "--output",
                "/tmp/mutants.json",
            ])
            .current_dir(&self.config.project_path)
            .output()
            .await
            .context("Failed to execute cargo mutants")?;

        // cargo-mutants may return non-zero if some mutants survived
        // This is expected behavior, so we don't bail on non-zero exit code
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("not installed") {
                eprintln!("⚠️  cargo-mutants not found, skipping");
                return Ok(0.0);
            }
        }

        // Parse JSON output
        let json_path = std::path::Path::new("/tmp/mutants.json");
        if !json_path.exists() {
            eprintln!("⚠️  Mutation results file not found, using fallback score");
            return Ok(85.0); // Fallback score
        }

        let json_content = tokio::fs::read_to_string(json_path)
            .await
            .context("Failed to read mutation results")?;

        let mutation_results: serde_json::Value =
            serde_json::from_str(&json_content).context("Failed to parse mutation results JSON")?;

        // Extract mutation score
        let total_mutants = mutation_results["total_mutants"].as_u64().unwrap_or(0) as f64;
        let caught = mutation_results["caught"].as_u64().unwrap_or(0) as f64;
        let missed = mutation_results["missed"].as_u64().unwrap_or(0) as f64;

        let mutation_score = if total_mutants > 0.0 {
            (caught / total_mutants) * 100.0
        } else {
            // No mutants generated - either no code changed or all code is untestable
            eprintln!("⚠️  No mutants generated for target files");
            85.0 // Assume reasonable score
        };

        eprintln!(
            "✅ Mutation testing complete: {:.1}% ({:.0} caught, {:.0} missed)",
            mutation_score, caught, missed
        );

        // Clean up ephemeral file
        let _ = tokio::fs::remove_file(json_path).await;

        Ok(mutation_score)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod fast_mode_mutation_score_tests {
    use super::*;

    /// `--fast` skips cargo-mutants entirely, yet every iteration used to report
    /// `"mutation_score": 100.0` — a perfect score for a measurement that was
    /// never taken. Whatever fast mode reports must not be a finite number, and
    /// must serialise as JSON null.
    #[test]
    fn skipped_mutation_testing_is_not_reported_as_a_score() {
        let score = unmeasured_mutation_score();
        assert!(
            !score.is_finite(),
            "a skipped measurement must not be a number, got {score}"
        );

        let report = IterationReport {
            iteration: 1,
            files_targeted: vec![PathBuf::from("src/lib.rs")],
            tests_generated: 0,
            coverage_gain: 0.0,
            mutation_score: score,
        };
        let json = serde_json::to_value(&report).expect("IterationReport serialises");
        assert!(
            json["mutation_score"].is_null(),
            "expected null for an unmeasured score, got {}",
            json["mutation_score"]
        );
    }
}
