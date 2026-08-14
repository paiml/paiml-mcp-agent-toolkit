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
pub(super) fn push_category(
    categories: &mut Vec<CategoryScore>,
    name: &str,
    weight: u16,
    score: Measured,
) {
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

/// Wall-clock budget for one expensive category.
///
/// The four expensive categories run concurrently, so this is per category, not
/// a share of the total.
pub(super) const CATEGORY_BUDGET: Duration = Duration::from_secs(100);

/// Backstop for the whole calculation.
pub(super) const TOTAL_BUDGET: Duration = Duration::from_secs(120);

/// Apply the whole-run backstop.
///
/// Nothing survives this deadline — the inner future is dropped entire — so the
/// only honest answer is a refusal. The previous answer was eight categories of
/// `0.0` out of their full weight with the detail "Timed out": a run that
/// measured nothing, reported as a run that measured everything and found it
/// worthless, right down to the F.
pub(super) async fn guard_total<F>(
    project_path: &Path,
    budget: Duration,
    inner: F,
) -> anyhow::Result<PerfectionScoreResult>
where
    F: std::future::Future<Output = anyhow::Result<PerfectionScoreResult>>,
{
    match tokio::time::timeout(budget, inner).await {
        Ok(result) => result,
        Err(_elapsed) => Err(anyhow::anyhow!(
            "perfection-score measured nothing: the run exceeded its {}s budget on {}. \
             No score is reported — a category that never ran is not a category that scored zero. \
             Re-run with --fast, or on a smaller path.",
            budget.as_secs(),
            project_path.display(),
        )),
    }
}

/// Run one category under [`CATEGORY_BUDGET`], turning an overrun into a named
/// "not measured" rather than a zero.
pub(super) async fn within_budget<F: std::future::Future<Output = Measured>>(fut: F) -> Measured {
    match tokio::time::timeout(CATEGORY_BUDGET, fut).await {
        Ok(measured) => measured,
        Err(_elapsed) => Err(format!(
            "it did not finish within {}s",
            CATEGORY_BUDGET.as_secs()
        )),
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
    /// parallel via `tokio::join!`, each under its own [`CATEGORY_BUDGET`], so a
    /// runaway `git log` costs its own category and nothing else.
    ///
    /// The whole calculation keeps a [`TOTAL_BUDGET`] backstop. It used to
    /// answer that backstop with eight categories of `0.0` out of their full
    /// weight and the detail "Timed out" — a measurement that never ran,
    /// rendered as a measured zero, dragging the total to 0/200 F exactly as if
    /// every category had been examined and found worthless. Nothing is
    /// measurable once the backstop fires (the inner future is dropped whole),
    /// so this refuses instead of grading.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn calculate(&self, project_path: &Path) -> anyhow::Result<PerfectionScoreResult> {
        guard_total(
            project_path,
            TOTAL_BUDGET,
            self.calculate_inner(project_path),
        )
        .await
    }

    /// Inner calculation logic, called within the timeout wrapper.
    async fn calculate_inner(&self, project_path: &Path) -> anyhow::Result<PerfectionScoreResult> {
        // Categories 1-4 are expensive and independent — run in parallel, each
        // bounded on its own so one slow category is excluded and disclosed
        // rather than taking the whole run down with it.
        let (tdg_score, repo_score, rust_score, popper_score) = tokio::join!(
            within_budget(self.get_tdg_score(project_path)),
            within_budget(self.get_repo_score(project_path)),
            within_budget(self.get_rust_project_score(project_path)),
            within_budget(self.get_popper_score(project_path)),
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

        // 7. Documentation (15 pts) — always measurable: the files are either
        // there with content in them, or they are not.
        let (doc_score, doc_details) = self.get_documentation_score(project_path).await;
        categories.push(
            CategoryScore::new("Documentation", doc_score, self.weights.documentation)
                .with_details(&doc_details),
        );

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

    /// Documentation (15 pts) — what the documentation *contains*.
    ///
    /// This was pure file existence: `README.md` 40 + `CHANGELOG.md` 20 +
    /// `docs/` 25 + `CONTRIBUTING.md` 15, awarded on `Path::exists()`. Four
    /// `touch`ed empty files and an empty `docs/` directory scored 100.0 (A+),
    /// the same family as #938's file-existence bonuses — a project with no
    /// documentation at all was indistinguishable from a documented one.
    ///
    /// Each artifact now has to hold something: nothing for absent or blank,
    /// 40% for less than a paragraph, 70% when it is substantive but fails its
    /// own structural test (a README with no example, a CHANGELOG with no
    /// versioned entry), full marks otherwise. `docs/` is scored by how many
    /// non-empty documentation files it actually contains. The breakdown is
    /// returned alongside the score and printed as the category's details, so
    /// the reader can see exactly what was read.
    pub(super) async fn get_documentation_score(&self, project_path: &Path) -> (f64, String) {
        let readme = if project_path.join("README.md").exists() {
            project_path.join("README.md")
        } else {
            project_path.join("readme.md")
        };

        let parts = [
            score_doc_file("README", &readme, 40.0, readme_has_structure),
            score_doc_file(
                "CHANGELOG",
                &project_path.join("CHANGELOG.md"),
                20.0,
                has_version_entry,
            ),
            score_docs_dir(&project_path.join("docs"), 25.0),
            score_doc_file(
                "CONTRIBUTING",
                &project_path.join("CONTRIBUTING.md"),
                15.0,
                has_heading,
            ),
        ];

        let score = parts.iter().fold(0.0_f64, |acc, p| acc + p.earned);
        let breakdown: Vec<String> = parts.iter().map(DocPart::to_string).collect();
        (
            score.min(100.0),
            format!(
                "content, not filenames: {} (an empty file earns nothing)",
                breakdown.join(", ")
            ),
        )
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

/// One documentation artifact's contribution, with the reason for it.
pub(super) struct DocPart {
    label: &'static str,
    earned: f64,
    max: f64,
    why: String,
}

impl std::fmt::Display for DocPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:.0}/{:.0} ({})",
            self.label, self.earned, self.max, self.why
        )
    }
}

/// Below this many non-whitespace characters a file is a placeholder, not a
/// document — roughly a short paragraph.
pub(super) const MIN_SUBSTANTIVE_CHARS: usize = 200;

/// Fraction of the maximum a present-but-thin file earns.
const THIN_CREDIT: f64 = 0.4;

/// Fraction a substantive file earns when it fails its structural test.
const UNSTRUCTURED_CREDIT: f64 = 0.7;

/// Score one documentation file by what is inside it.
pub(super) fn score_doc_file(
    label: &'static str,
    path: &Path,
    max: f64,
    structure: fn(&str) -> bool,
) -> DocPart {
    let part = |earned: f64, why: &str| DocPart {
        label,
        earned,
        max,
        why: why.to_string(),
    };

    let Ok(text) = std::fs::read_to_string(path) else {
        return part(0.0, "missing");
    };
    let chars = text.chars().filter(|c| !c.is_whitespace()).count();
    if chars == 0 {
        return part(0.0, "empty");
    }
    if chars < MIN_SUBSTANTIVE_CHARS {
        return part(max * THIN_CREDIT, "under a paragraph");
    }
    if !structure(&text) {
        return part(max * UNSTRUCTURED_CREDIT, "prose only, no structure");
    }
    part(max, "substantive")
}

/// Score `docs/` by the number of non-empty documentation files in it.
///
/// An empty `docs/` directory used to be worth 25 of the category's 100 points
/// on the strength of existing.
pub(super) fn score_docs_dir(dir: &Path, max: f64) -> DocPart {
    const PER_FILE: f64 = 5.0;
    const DOC_EXTENSIONS: [&str; 5] = ["md", "rst", "txt", "adoc", "org"];

    let files = walkdir::WalkDir::new(dir)
        .max_depth(4)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| DOC_EXTENSIONS.contains(&x.to_lowercase().as_str()))
        })
        .filter(|e| e.metadata().map(|m| m.len() > 0).unwrap_or(false))
        .count();

    let why = if !dir.is_dir() {
        "missing".to_string()
    } else {
        format!("{files} non-empty file(s)")
    };

    DocPart {
        label: "docs/",
        earned: (files as f64 * PER_FILE).min(max),
        max,
        why,
    }
}

/// A markdown heading anywhere in the text.
pub(super) fn has_heading(text: &str) -> bool {
    text.lines().any(|l| l.trim_start().starts_with('#'))
}

/// A README earns full marks when it is navigable (two or more headings) and
/// shows the reader how to use the thing (a fenced code block).
pub(super) fn readme_has_structure(text: &str) -> bool {
    let headings = text
        .lines()
        .filter(|l| l.trim_start().starts_with('#'))
        .count();
    let has_example = text.matches("```").count() >= 2;
    headings >= 2 && has_example
}

/// A CHANGELOG heading naming a version, e.g. `## [1.2.3]` or `## v1.2.3`.
pub(super) fn has_version_entry(text: &str) -> bool {
    text.lines()
        .filter(|l| l.trim_start().starts_with('#'))
        .any(looks_like_version)
}

/// `digit . digit` somewhere in the line — a version number without a regex.
fn looks_like_version(line: &str) -> bool {
    let bytes = line.as_bytes();
    bytes
        .windows(3)
        .any(|w| w[0].is_ascii_digit() && w[1] == b'.' && w[2].is_ascii_digit())
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
