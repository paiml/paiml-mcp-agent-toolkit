#![cfg_attr(coverage_nightly, coverage(off))]
//! Perfection Score Calculator implementation.

use crate::services::popper_score::orchestrator::PopperOrchestrator;
use crate::services::repo_score::aggregator::ScoreAggregator;
use crate::services::repo_score::scorers::ScorerConfig;
use crate::services::rust_project_score::models::ScoringMode;
use crate::services::rust_project_score::orchestrator::RustProjectScoreOrchestrator;
use std::path::Path;
use std::time::Duration;

use super::types::{CategoryScore, CategoryWeights, PerfectionScoreResult};

/// A category's 0-100 score, or the reason it could not be measured.
///
/// The `Err` side always names the artifact that would make the category
/// measurable, so "not measured" is representable and a reader is never handed
/// a plausible constant instead. This is the same shape `pmat score` uses for
/// its dimensions; #938 is what happens without it — `Mutation Testing`,
/// `Test Coverage` and `Performance` each started from a bare `50.0` and added
/// file-existence bonuses, so four empty files (`mutants.toml`, `.mutants/`,
/// `benches/`, a `criterion` dev-dependency) moved the 200-point total by
/// +15.65 and took Mutation Testing from F to A- without a mutant being run.
pub(super) type Measured = Result<f64, String>;

/// Append a category, or an explicit N/A entry that costs nothing and earns
/// nothing.
///
/// An unmeasured category leaves the denominator (its `max_points` is 0), the
/// treatment `calculate_inner` already applied to the fast-mode mutation skip.
fn push_category(categories: &mut Vec<CategoryScore>, name: &str, weight: u16, score: Measured) {
    match score {
        Ok(value) => categories.push(CategoryScore::new(name, value, weight)),
        Err(why) => {
            let mut skipped = CategoryScore::new(name, 0.0, 0)
                .with_details(&format!("Not measured — {why} (excluded from total)"));
            skipped.grade = "N/A".to_string();
            categories.push(skipped);
        }
    }
}

/// Perfection Score Calculator
pub struct PerfectionScoreCalculator {
    pub(super) weights: CategoryWeights,
    pub(super) fast_mode: bool,
}

impl Default for PerfectionScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfectionScoreCalculator {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            weights: CategoryWeights::default(),
            fast_mode: false,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Fast mode.
    pub fn fast_mode(mut self, fast: bool) -> Self {
        self.fast_mode = fast;
        self
    }

    /// Calculate perfection score for a project.
    ///
    /// Categories 1-4 (TDG, repo-score, rust-project-score, popper-score) run in
    /// parallel via `tokio::join!`. The entire calculation is wrapped in a 120-second
    /// timeout to prevent runaway CPU usage from unbounded `git log` subprocesses.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn calculate(&self, project_path: &Path) -> anyhow::Result<PerfectionScoreResult> {
        match tokio::time::timeout(Duration::from_secs(120), self.calculate_inner(project_path))
            .await
        {
            Ok(result) => result,
            Err(_elapsed) => {
                eprintln!("⚠️  Perfection score calculation timed out after 120s");
                // Return a partial result with timeout details
                let categories = vec![
                    CategoryScore::new("Technical Debt Grade", 0.0, self.weights.tdg)
                        .with_details("Timed out"),
                    CategoryScore::new("Repository Health", 0.0, self.weights.repo_score)
                        .with_details("Timed out"),
                    CategoryScore::new("Rust Project Quality", 0.0, self.weights.rust_score)
                        .with_details("Timed out"),
                    CategoryScore::new("Popperian Falsifiability", 0.0, self.weights.popper_score)
                        .with_details("Timed out"),
                    CategoryScore::new("Test Coverage", 0.0, self.weights.test_coverage)
                        .with_details("Timed out"),
                    CategoryScore::new("Mutation Testing", 0.0, self.weights.mutation)
                        .with_details("Timed out"),
                    CategoryScore::new("Documentation", 0.0, self.weights.documentation)
                        .with_details("Timed out"),
                    CategoryScore::new("Performance", 0.0, self.weights.performance)
                        .with_details("Timed out"),
                ];
                Ok(PerfectionScoreResult::new(categories))
            }
        }
    }

    /// Inner calculation logic, called within the timeout wrapper.
    async fn calculate_inner(&self, project_path: &Path) -> anyhow::Result<PerfectionScoreResult> {
        // Categories 1-4 are expensive and independent — run in parallel
        let (tdg_score, repo_score, rust_score, popper_score) = tokio::join!(
            self.get_tdg_score(project_path),
            self.get_repo_score(project_path),
            self.get_rust_project_score(project_path),
            self.get_popper_score(project_path),
        );

        let mut categories = Vec::new();
        push_category(
            &mut categories,
            "Technical Debt Grade",
            self.weights.tdg,
            tdg_score,
        );
        push_category(
            &mut categories,
            "Repository Health",
            self.weights.repo_score,
            repo_score,
        );
        push_category(
            &mut categories,
            "Rust Project Quality",
            self.weights.rust_score,
            rust_score,
        );
        push_category(
            &mut categories,
            "Popperian Falsifiability",
            self.weights.popper_score,
            popper_score,
        );

        // Categories 5-8 are cheap filesystem checks — run sequentially

        // 5. Test Coverage (25 pts)
        let coverage_score = self.get_coverage_score(project_path).await;
        push_category(
            &mut categories,
            "Test Coverage",
            self.weights.test_coverage,
            coverage_score,
        );

        // 6. Mutation Score (20 pts) - Skip in fast mode
        //
        // `--fast` cannot run mutation testing, but this used to hand the category
        // a flat 50.0 ("default credit in fast mode") while its own details string
        // admitted "Skipped (fast mode)". That put 10 unearned points inside a
        // total presented as a grade, identically for a real repo, an empty
        // directory and a path that does not exist. A measurement that never ran
        // earns nothing and is removed from the denominator instead.
        //
        // #938: the non-fast branch kept the constant this comment describes.
        // It read `let mut score: f64 = 50.0;` and added 20 for a `mutants.toml`,
        // 20 for a `.mutants/` directory and 10 for the substring "mutants" in
        // any Cargo.toml — no process was ever spawned, so an empty two-file
        // crate that ran zero mutants scored 90.0 (A-). Both branches now
        // report N/A unless a real cargo-mutants run left results behind.
        let mutation_score = if self.fast_mode {
            Err("--fast skips mutation testing".to_string())
        } else {
            self.get_mutation_score(project_path).await
        };
        push_category(
            &mut categories,
            "Mutation Testing",
            self.weights.mutation,
            mutation_score,
        );

        // 7. Documentation (15 pts)
        let doc_score = self.get_documentation_score(project_path).await;
        categories.push(CategoryScore::new(
            "Documentation",
            doc_score,
            self.weights.documentation,
        ));

        // 8. Performance (15 pts)
        let perf_score = self.get_performance_score(project_path).await;
        push_category(
            &mut categories,
            "Performance",
            self.weights.performance,
            perf_score,
        );

        let mut result = PerfectionScoreResult::new(categories);

        // The denominator must match the categories that were actually measured,
        // otherwise a skipped category silently costs the project its own weight
        // (or, as before, lends it half of it for free).
        let measured_max: u16 = result.categories.iter().map(|c| c.max_points).sum();
        if measured_max > 0 && measured_max != result.max_score {
            result.max_score = measured_max;
            // calculate_overall_grade normalises against the full 200-point scale,
            // so rescale the total to that scale before grading it.
            let scaled = result.total_score * f64::from(super::types::MAX_PERFECTION_SCORE)
                / f64::from(measured_max);
            result.grade = PerfectionScoreResult::calculate_overall_grade(scaled);
        }

        Ok(result)
    }

    /// Technical Debt Grade (40 pts) — the same measurement `pmat tdg` and
    /// `pmat analyze tdg` report.
    ///
    /// #941: this used to run a *different* implementation —
    /// `TDGCalculator::analyze_directory().average_tdg` on a 0-5 debt scale,
    /// converted as `100 - average_tdg * 20` — so the heaviest category in the
    /// score (40 of 200 points) contradicted the command the tool tells users
    /// to run for TDG. On a fixture with a cyclomatic-241 function, 40
    /// `unwrap()`/`panic!` functions and 43 TODO/FIXME/HACK comments, `pmat tdg`
    /// said 0.0/100 (F) while perfection-score said 77.2 (C+, 30.9/40 points);
    /// inside a git repo the same fixture read 87.3 (B+). That was the third
    /// TDG scale in the binary after the pair reconciled in #870. There is now
    /// one: [`TdgAnalyzer`], consumed on its own 0-100 scale.
    pub(super) async fn get_tdg_score(&self, project_path: &Path) -> Measured {
        let analyzer = crate::tdg::TdgAnalyzer::new()
            .map_err(|e| format!("the TDG analyzer could not start: {e}"))?;

        let project = analyzer
            .analyze_project(project_path)
            .await
            .map_err(|e| format!("TDG analysis failed: {e}"))?;

        // `average_score` is `None` when nothing was graded (GH #704) — that is
        // not a zero, and must not be graded as one.
        project.average_score.map(f64::from).ok_or_else(|| {
            format!(
                "no gradable source files under {} (`pmat tdg` reports the same)",
                project_path.display()
            )
        })
    }

    pub(super) async fn get_repo_score(&self, project_path: &Path) -> Measured {
        // Repo Score: 0-100 scale
        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig {
            verbose: false,
            timeout_seconds: 60,
            skip_slow_checks: self.fast_mode,
            deep: !self.fast_mode,
        };

        // A failed sub-scorer used to fall back to a flat 50.0 — the same
        // "absence rendered as a plausible constant" #938 is about, just on the
        // error path. A category that could not be computed is N/A.
        aggregator
            .aggregate(project_path, &config)
            .await
            .map(|score| score.total_score)
            .map_err(|e| format!("repo-score failed: {e}"))
    }

    pub(super) async fn get_rust_project_score(&self, project_path: &Path) -> Measured {
        // Rust Project Score: raw points, normalize to 0-100 using the
        // orchestrator-reported max (289 in v3.0). Never hardcode the scale —
        // it grows as scorers are added (was 134, now 289).
        let orchestrator = RustProjectScoreOrchestrator::new();
        let mode = if self.fast_mode {
            ScoringMode::Quick
        } else {
            ScoringMode::Fast
        };

        orchestrator
            .score_with_mode(project_path, mode)
            .map(|score| normalize_rps_percentage(score.total_earned, score.total_possible))
            .map_err(|e| format!("rust-project-score failed: {e}"))
    }

    pub(super) async fn get_popper_score(&self, project_path: &Path) -> Measured {
        // Popper Score: 0-100 scale
        let orchestrator = PopperOrchestrator::new();

        orchestrator
            .score(project_path)
            .map(|result| result.normalized_score)
            .map_err(|e| format!("popper-score failed: {e}"))
    }

    /// Test Coverage (25 pts) — read from a real coverage run, or N/A.
    ///
    /// #938: when no coverage cache existed this returned
    /// `50.0 + test_count * 0.1 + density * 5` (capped at 95), or a flat 70.0
    /// for a project with no Rust files at all. Counting `#[test]` attributes
    /// measures how many tests were *written*, not how much code they execute,
    /// and it started from the same 50.0 floor every other fabricated category
    /// used.
    pub(super) async fn get_coverage_score(&self, project_path: &Path) -> Measured {
        // Look for cached coverage data in multiple locations (workspace-aware)
        let cache_paths = [
            project_path.join(".pmat-metrics").join("coverage.json"),
            project_path.join("server/.pmat-metrics/coverage.json"),
        ];

        for metrics_file in &cache_paths {
            if metrics_file.exists() {
                if let Ok(content) = std::fs::read_to_string(metrics_file) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(coverage) = json.get("coverage").and_then(|v| v.as_f64()) {
                            return Ok(coverage);
                        }
                    }
                }
            }
        }

        Err(format!(
            "no coverage data — run `cargo llvm-cov` (or `make coverage`) to write {}",
            project_path.join(".pmat-metrics/coverage.json").display()
        ))
    }

    /// Mutation Testing (20 pts) — read from a real cargo-mutants run, or N/A.
    ///
    /// #938: this used to start at 50.0 and add points for the mere existence
    /// of `mutants.toml`, a `.mutants/` directory and the substring "mutants"
    /// in a Cargo.toml. `touch mutants.toml && mkdir .mutants` took an empty
    /// crate from F to A- (90.0) without a single mutant being generated, let
    /// alone caught. The score now comes from cargo-mutants' own outcome file.
    pub(super) async fn get_mutation_score(&self, project_path: &Path) -> Measured {
        for root in [project_path.to_path_buf(), project_path.join("server")] {
            let out_dir = root.join("mutants.out");
            if let Some((caught, viable)) = read_mutation_outcomes(&out_dir) {
                if viable == 0 {
                    return Err(format!("{} records no viable mutants", out_dir.display()));
                }
                return Ok((caught as f64 / viable as f64) * 100.0);
            }
        }

        Err(format!(
            "no mutation results — run `cargo mutants` to produce {}",
            project_path.join("mutants.out/outcomes.json").display()
        ))
    }

    pub(super) async fn get_documentation_score(&self, project_path: &Path) -> f64 {
        // Check for common documentation files
        let has_readme =
            project_path.join("README.md").exists() || project_path.join("readme.md").exists();
        let has_changelog = project_path.join("CHANGELOG.md").exists();
        let has_docs_dir = project_path.join("docs").exists();
        let has_contributing = project_path.join("CONTRIBUTING.md").exists();

        let mut score: f64 = 0.0;
        if has_readme {
            score += 40.0;
        }
        if has_changelog {
            score += 20.0;
        }
        if has_docs_dir {
            score += 25.0;
        }
        if has_contributing {
            score += 15.0;
        }

        score.min(100.0)
    }

    /// Performance (15 pts) — read from a real benchmark run, or N/A.
    ///
    /// #938: this used to start at 50.0, add 30 for a `benches/` directory and
    /// 20 for the substring "criterion" in a Cargo.toml, so `mkdir benches` and
    /// a dev-dependency line scored a perfect 100.0 (A+) on a crate whose
    /// benchmarks had never been run — and would score the same if every
    /// benchmark in it regressed tenfold. It is now the share of criterion
    /// benchmarks that did not regress against their stored baseline, and N/A
    /// until `cargo bench` has produced comparisons.
    pub(super) async fn get_performance_score(&self, project_path: &Path) -> Measured {
        for root in [project_path.to_path_buf(), project_path.join("server")] {
            let criterion_dir = root.join("target").join("criterion");
            if let Some((regressed, compared)) = read_criterion_changes(&criterion_dir) {
                return Ok(((compared - regressed) as f64 / compared as f64) * 100.0);
            }
        }

        Err(format!(
            "no benchmark comparisons — run `cargo bench` twice to produce {}",
            project_path
                .join("target/criterion/<bench>/change/estimates.json")
                .display()
        ))
    }
}

/// `(caught, viable)` mutant counts from a cargo-mutants run, or `None` when
/// the directory holds no readable outcome.
///
/// `outcomes.json` is preferred; the per-outcome text files cargo-mutants also
/// writes are the fallback. Unviable mutants (those that do not compile) are
/// excluded from the denominator, matching cargo-mutants' own reporting.
fn read_mutation_outcomes(out_dir: &Path) -> Option<(usize, usize)> {
    if let Some(counts) = read_mutation_outcomes_json(&out_dir.join("outcomes.json")) {
        return Some(counts);
    }

    let count_lines = |name: &str| -> Option<usize> {
        std::fs::read_to_string(out_dir.join(name))
            .ok()
            .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
    };
    let caught = count_lines("caught.txt");
    let missed = count_lines("missed.txt");
    let timeout = count_lines("timeout.txt").unwrap_or(0);
    match (caught, missed) {
        (None, None) => None,
        (caught, missed) => {
            let caught = caught.unwrap_or(0);
            Some((caught, caught + missed.unwrap_or(0) + timeout))
        }
    }
}

fn read_mutation_outcomes_json(path: &Path) -> Option<(usize, usize)> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let outcomes = json.get("outcomes")?.as_array()?;

    let mut caught = 0;
    let mut viable = 0;
    for outcome in outcomes {
        // The baseline build is not a mutant.
        if outcome
            .get("scenario")
            .and_then(|s| s.as_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("baseline"))
        {
            continue;
        }
        match outcome.get("summary").and_then(|s| s.as_str()) {
            Some("CaughtMutant") => {
                caught += 1;
                viable += 1;
            }
            Some("MissedMutant" | "Timeout") => viable += 1,
            _ => {}
        }
    }
    (viable > 0).then_some((caught, viable))
}

/// `(regressed, compared)` criterion benchmark counts, or `None` when no
/// benchmark comparison exists under `criterion_dir`.
///
/// A benchmark counts as regressed when the mean of its stored `change`
/// estimate is more than 5% slower than the baseline — criterion's own noise
/// threshold.
fn read_criterion_changes(criterion_dir: &Path) -> Option<(usize, usize)> {
    if !criterion_dir.is_dir() {
        return None;
    }

    const REGRESSION_THRESHOLD: f64 = 0.05;
    let mut regressed = 0;
    let mut compared = 0;

    for entry in walkdir::WalkDir::new(criterion_dir)
        .max_depth(6)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_name() == "estimates.json")
        .filter(|e| {
            e.path()
                .parent()
                .and_then(|p| p.file_name())
                .is_some_and(|n| n == "change")
        })
    {
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        let Some(mean) = json
            .get("mean")
            .and_then(|m| m.get("point_estimate"))
            .and_then(serde_json::Value::as_f64)
        else {
            continue;
        };
        compared += 1;
        if mean > REGRESSION_THRESHOLD {
            regressed += 1;
        }
    }

    (compared > 0).then_some((regressed, compared))
}

/// Convert raw rust-project-score points to a 0-100 percentage.
///
/// `total_earned` is raw points out of `total_possible` (289 in RPS v3.0).
/// Feeding raw points directly into `CategoryScore::new` (which expects a
/// 0-100 percentage) inflated the category past its max — e.g. raw 184
/// scaled to 55.2/30 and clamped the total at 200/200 A+.
pub(super) fn normalize_rps_percentage(total_earned: f64, total_possible: f64) -> f64 {
    if total_possible > 0.0 {
        ((total_earned / total_possible) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    }
}
