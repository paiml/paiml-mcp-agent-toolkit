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
        crate::status_eprintln!("🧬 Running mutation testing...");

        // Check if cargo-mutants is installed
        let check_output = Command::new("cargo")
            .args(["mutants", "--version"])
            .output()
            .await;

        if check_output.is_err() || !check_output.expect("internal error").status.success() {
            eprintln!("⚠️  cargo-mutants not installed, mutation score NOT MEASURED");
            eprintln!("   Install with: cargo install cargo-mutants");
            return Ok(unmeasured_mutation_score());
        }

        // `--in-diff` takes a FILE containing a unified diff. This used to pass
        // it the literal string `git`, followed by `diff` and `HEAD` as
        // positional arguments; cargo-mutants answers that with
        // `error: unexpected argument 'diff' found` and runs nothing, on every
        // invocation since the code was written. It then found no results file
        // and returned a hardcoded 85.0 — a fabricated mutation score for a
        // measurement that could not physically have happened. So: write the
        // diff out, and hand over a path.
        // Not `tempfile`: that crate is an optional dependency of this one and
        // is only ever used from test code, so reaching for it here would make
        // production behaviour depend on a feature.
        let work = std::env::temp_dir().join(format!("pmat-mutation-{}", std::process::id()));
        tokio::fs::create_dir_all(&work)
            .await
            .context("Failed to create a scratch directory")?;
        let diff_path = work.join("in.diff");
        let diff = Command::new("git")
            .args(["diff", "HEAD"])
            .current_dir(&self.config.project_path)
            .output()
            .await
            .context("Failed to run git diff")?;
        tokio::fs::write(&diff_path, &diff.stdout)
            .await
            .context("Failed to write the diff cargo-mutants is to be scoped to")?;

        let out_dir = work.join("out");
        let output = Command::new("cargo")
            .args([
                "mutants".as_ref(),
                "--in-diff".as_ref(),
                diff_path.as_os_str(),
                // Body-replacement mutants leave a function's parameters unused,
                // and `src/lib.rs` denies that lint, so without this every
                // mutant is `unviable` and nothing is ever executed.
                "--cap-lints".as_ref(),
                "true".as_ref(),
                "--output".as_ref(),
                out_dir.as_os_str(),
            ])
            .current_dir(&self.config.project_path)
            .output()
            .await
            .context("Failed to execute cargo mutants")?;

        // cargo-mutants exits 2 when mutants survived, which is a RESULT. It
        // also exits 0 when every mutant was unviable, which is not. Neither
        // number decides anything here: the artifact does.
        if !output.status.success() {
            eprintln!(
                "⚠️  cargo mutants exited {}",
                output
                    .status
                    .code()
                    .map_or_else(|| "by signal".to_string(), |c| c.to_string())
            );
        }

        let outcomes = out_dir.join("mutants.out").join("outcomes.json");
        let read = tokio::fs::read_to_string(&outcomes).await;
        let Ok(json_content) = read else {
            eprintln!(
                "⚠️  No cargo-mutants report at {} — mutation score NOT MEASURED",
                outcomes.display()
            );
            return Ok(unmeasured_mutation_score());
        };

        let mutation_score = score_from_outcomes_json(&json_content);
        if mutation_score.is_finite() {
            crate::status_eprintln!("✅ Mutation testing complete: {:.1}%", mutation_score);
        } else {
            eprintln!("⚠️  cargo-mutants executed no mutant — mutation score NOT MEASURED");
        }

        Ok(mutation_score)
    }
}

/// The mutation score a cargo-mutants `outcomes.json` supports, or
/// [`unmeasured_mutation_score`] when it supports none.
///
/// The two "none" cases are the whole point. Zero mutants generated, and every
/// mutant generated but none executed (all unviable), both used to return a
/// hardcoded 85.0 here — a number nobody measured, attached to a run that tested
/// nothing. cargo-mutants exits 0 for both, so the exit code cannot tell them
/// apart from a clean sweep either; only the counts can.
fn score_from_outcomes_json(json: &str) -> f64 {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else {
        return unmeasured_mutation_score();
    };
    let total = v["total_mutants"].as_u64().unwrap_or(0);
    let caught = v["caught"].as_u64().unwrap_or(0);
    let missed = v["missed"].as_u64().unwrap_or(0);
    let timeout = v["timeout"].as_u64().unwrap_or(0);
    if total == 0 || caught + missed + timeout == 0 {
        return unmeasured_mutation_score();
    }
    #[allow(clippy::cast_precision_loss)]
    let score = (caught as f64 / total as f64) * 100.0;
    score
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

    /// PMAT-630 (#1034 EV-4). The non-fast path had the same defect the test
    /// above fixed for `--fast`, twice over, and worse: the `--in-diff`
    /// invocation was malformed (`--in-diff git diff HEAD`, which cargo-mutants
    /// rejects with `error: unexpected argument 'diff' found`), so the
    /// no-results branch — `return Ok(85.0)` — was the branch taken on EVERY
    /// run. A backend that cannot be invoked reported 85% adequacy.
    #[test]
    fn a_run_that_executed_no_mutant_is_not_a_score() {
        // Zero mutants generated.
        let none = r#"{"total_mutants":0,"caught":0,"missed":0,"timeout":0,"unviable":0}"#;
        assert!(
            !score_from_outcomes_json(none).is_finite(),
            "zero mutants was reported as a score"
        );

        // Mutants generated, none executed: this is what cargo-mutants reports
        // for this crate without `--cap-lints`, and it exits 0 for it.
        let unviable = r#"{"total_mutants":3,"caught":0,"missed":0,"timeout":0,"unviable":3}"#;
        assert!(
            !score_from_outcomes_json(unviable).is_finite(),
            "an all-unviable run was reported as a score"
        );

        // An unreadable artifact is not a measurement either.
        assert!(
            !score_from_outcomes_json("not json").is_finite(),
            "an unparseable report was reported as a score"
        );
    }

    /// The counter-test: a real run still produces a real number, or the fix
    /// above would be "report nothing, ever", which passes just as vacuously.
    #[test]
    fn counter_an_executed_run_still_yields_its_score() {
        let real = r#"{"total_mutants":10,"caught":8,"missed":2,"timeout":0,"unviable":0}"#;
        let score = score_from_outcomes_json(real);
        assert!((score - 80.0).abs() < f64::EPSILON, "got {score}");

        // Unviable mutants are not survivors; they are mutants that did not
        // compile. They stay in the denominator, but they do not make the run
        // unmeasured as long as something was executed.
        let mixed = r#"{"total_mutants":10,"caught":6,"missed":0,"timeout":0,"unviable":4}"#;
        assert!((score_from_outcomes_json(mixed) - 60.0).abs() < f64::EPSILON);
    }
}
