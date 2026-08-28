// THE set of checks that carries the name "quality gate", in ONE place, for
// every surface that runs them.
//
// The MCP `quality_gate` tool advertised "complexity, SATD, dead code, lint,
// docs, etc." and ran TDG + SATD. Over a one-file fixture with two planted
// markers, `pmat quality-gate --checks all` reported {satd: 2, coverage: 1} and
// the MCP tool reported {satd: 2} — with `not_measured: []` beside it. The
// coverage row is itself a DISCLOSURE row ("Code coverage was NOT measured …"),
// so the surface that dropped seven of the nine checks is the one that claimed
// to have left nothing out; and this file's neighbours state the rule that field
// exists for: "`not_measured` is what a reader consults to learn what a verdict
// does NOT cover, so an empty list is a positive claim of full coverage".
//
// The fix is not a second list of checks in the MCP file. `GateScope`,
// `satd_violations_for_file` and the "THIRD copy of the detector-severity
// mapping" are all recorded in `src/mcp_pmcp/tool_functions/quality_tools.rs` as
// what happens when one rule gets two implementations, and a duplicated check
// list would be the next entry. [`run_gate_suite`] calls the SAME functions
// `pmat quality-gate` calls — `run_all_project_checks` for a directory,
// `run_all_single_file_checks` for a file — so the two surfaces cannot report
// different findings for the same path without the CLI reporting them too.
//
// The other half of the rule: a check this suite did NOT run is NAMED
// ([`GateSuite::not_run`]), never skipped in silence. A check that did not run
// has not passed — the same reasoning that made an absent coverage report a
// violation instead of a zero (see `run_coverage_check`).

/// `--max-dead-code`'s default, so the MCP gate and the CLI gate cannot fail a
/// project at two different thresholds under one name.
///
/// Pinned to clap's `default_value` by
/// `the_suite_thresholds_are_the_cli_defaults` below: two numbers under one name
/// is the same defect as two implementations under one name.
pub const GATE_DEFAULT_MAX_DEAD_CODE: f64 = 15.0;

/// `--max-complexity-p99`'s default. Pinned to clap the same way.
pub const GATE_DEFAULT_MAX_COMPLEXITY_P99: u32 = 50;

/// Why a single file cannot answer a project-wide check.
///
/// One sentence, from one place: the five checks below read the whole tree, a
/// coverage report or a README, and `pmat quality-gate --file` does not run them
/// either — it prints "Skipping … not applicable to single file" to a terminal
/// nobody parses. Named here so an MCP client learns it from the payload.
fn file_scope_reason(check: QualityCheckType) -> String {
    format!(
        "{check} is a project-wide check (it reads the whole tree, a coverage report or a README) \
         and this path is a single file; `pmat quality-gate --file` does not run it either"
    )
}

/// One advertised check that a given path did NOT measure, and why.
#[derive(Debug, Clone)]
pub struct UnrunCheck {
    /// The check's name, spelled the way `--checks` spells it.
    pub check: String,
    /// The path it was not run for. Per-path, never global: a sibling path that
    /// DID run the check does not fill this hole, which is the same rule
    /// `GateScope::measure` applies to ungraded files.
    pub path: PathBuf,
    /// Why it did not run.
    pub reason: String,
}

/// What the gate suite measured for one or more paths, and what it did not.
#[derive(Debug, Default)]
pub struct GateSuite {
    /// Every finding the checks that DID run produced.
    pub violations: Vec<QualityViolation>,
    /// The checks that ran, deduplicated, in `--checks` spelling.
    pub ran: Vec<String>,
    /// Every advertised check that did not run, per path.
    pub not_run: Vec<UnrunCheck>,
}

impl GateSuite {
    /// Absorb another path's outcome.
    ///
    /// `ran` is a union and `not_run` is a list: a check that ran for one path
    /// and not for another appears in BOTH, because "coverage ran" and "coverage
    /// did not run for `a.rs`" are both true and a reader needs the second one.
    fn merge(&mut self, other: Self) {
        self.violations.extend(other.violations);
        for check in other.ran {
            if !self.ran.contains(&check) {
                self.ran.push(check);
            }
        }
        self.not_run.extend(other.not_run);
    }

    /// The names of the checks that did not run, deduplicated and sorted.
    ///
    /// This is what goes in `not_measured`, and it errs towards disclosure: a
    /// check missing for ANY path in the call is named, even if another path
    /// measured it. The per-path detail is in [`GateSuite::not_run`].
    #[must_use]
    pub fn not_run_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.not_run.iter().map(|u| u.check.clone()).collect();
        names.sort();
        names.dedup();
        names
    }

    /// Two paths in one call can name the same file (`[dir, dir/a.rs]`, or the
    /// same path twice), and every check would then report its findings twice.
    /// The same rule `GateScope::measure` applies to graded files: the same
    /// finding on the same line of the same file weighs once.
    fn dedup_violations(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.violations.retain(|v| {
            seen.insert((
                v.check_type.clone(),
                v.severity.clone(),
                v.file.clone(),
                v.line,
                v.message.clone(),
            ))
        });
    }
}

/// Run the `--checks all` suite over every path the caller named.
///
/// # Errors
/// Propagates a check's own failure, exactly as `pmat quality-gate` does.
pub async fn run_gate_suite_over(paths: &[PathBuf]) -> Result<GateSuite> {
    let mut merged = GateSuite::default();
    for path in paths {
        merged.merge(run_gate_suite(path).await?);
    }
    merged.dedup_violations();
    Ok(merged)
}

/// Run the `--checks all` suite over one path.
///
/// # Errors
/// Propagates a check's own failure, exactly as `pmat quality-gate` does.
pub async fn run_gate_suite(path: &Path) -> Result<GateSuite> {
    if path.is_dir() {
        run_project_suite(path).await
    } else {
        run_file_suite(path).await
    }
}

/// The nine checks `run_all_project_checks` runs, in the order it runs them.
///
/// `QualityCheckType::default_checks()` IS that list — one list, so a tenth
/// check added to the gate cannot be advertised as run without being run.
fn project_suite_checks() -> Vec<QualityCheckType> {
    QualityCheckType::default_checks()
}

/// The four checks `run_all_single_file_checks` runs.
fn file_suite_checks() -> [QualityCheckType; 4] {
    [
        QualityCheckType::Complexity,
        QualityCheckType::DeadCode,
        QualityCheckType::Satd,
        QualityCheckType::Security,
    ]
}

async fn run_project_suite(project_path: &Path) -> Result<GateSuite> {
    let mut violations = Vec::new();
    let mut results = QualityGateResults::default();

    // THE implementation `pmat quality-gate --checks all` runs, called rather
    // than copied.
    run_all_project_checks(
        project_path,
        GATE_DEFAULT_MAX_DEAD_CODE,
        None,
        GATE_DEFAULT_MAX_COMPLEXITY_P99,
        &mut violations,
        &mut results,
        false,
    )
    .await?;

    // …including the `[exclude]` rule the CLI applies to the findings
    // afterwards, which the MCP gate applied to nothing: a path a project had
    // excluded was a violation over MCP and clean over the CLI.
    apply_gate_exclude_paths(project_path, &mut violations);

    let mut ran: Vec<String> = project_suite_checks()
        .iter()
        .map(ToString::to_string)
        .collect();
    let mut not_run = Vec::new();
    let mut unmeasured = |check: QualityCheckType, reason: String| {
        let name = check.to_string();
        ran.retain(|c| c != &name);
        not_run.push(UnrunCheck {
            check: name,
            path: project_path.to_path_buf(),
            reason,
        });
    };

    // `.pmat-gates.toml` can switch entropy off (#220), and `run_all_project_checks`
    // then returns without running it — a hole the payload has to disclose rather
    // than count as a clean entropy result.
    if !load_entropy_gate_config(project_path).enabled {
        unmeasured(
            QualityCheckType::Entropy,
            "disabled by [entropy] enabled = false in .pmat-gates.toml".to_string(),
        );
    }

    // Two checks read an artifact someone else produced, and return an empty
    // finding list when it is absent — the same empty list a perfect project
    // gets. The gate already reports the coverage gap as a violation ("this gate
    // does not cover coverage"); `ran` means MEASURED, so neither may be counted
    // as a check this verdict covers. Both questions are asked of the functions
    // that own them (`read_coverage_from_cache`, `sections_source`), never
    // re-derived here.
    if read_coverage_from_cache(project_path).is_none() {
        unmeasured(
            QualityCheckType::Coverage,
            "no coverage report at .pmat/coverage-cache.json or .pmat-metrics/coverage.json, \
             so this gate does not cover coverage"
                .to_string(),
        );
    }
    if sections_source(project_path).is_none() {
        unmeasured(
            QualityCheckType::Sections,
            "no README.md, so there is nothing for the documentation-sections check to read"
                .to_string(),
        );
    }

    Ok(GateSuite {
        violations,
        ran,
        not_run,
    })
}

async fn run_file_suite(file_path: &Path) -> Result<GateSuite> {
    // The five project-wide checks a single file cannot answer, named before
    // anything else so that every early return below still discloses them.
    let mut not_run: Vec<UnrunCheck> = project_suite_checks()
        .into_iter()
        .filter(|check| !file_suite_checks().contains(check))
        .map(|check| UnrunCheck {
            check: check.to_string(),
            path: file_path.to_path_buf(),
            reason: file_scope_reason(check),
        })
        .collect();

    // A file that does not parse gets no verdict from any of the four — this is
    // the guard `handle_single_file_quality_gate` runs before the same four
    // checks, and without it `check_single_file_complexity` would abort the whole
    // MCP call for one bad file among N.
    if let Err(parse_error) = crate::tdg::ensure_parseable(file_path) {
        for check in file_suite_checks() {
            not_run.push(UnrunCheck {
                check: check.to_string(),
                path: file_path.to_path_buf(),
                reason: parse_error.to_string(),
            });
        }
        return Ok(GateSuite {
            violations: Vec::new(),
            ran: Vec::new(),
            not_run,
        });
    }

    let project_path = file_path.parent().unwrap_or_else(|| Path::new("."));
    let mut violations = Vec::new();
    let mut results = QualityGateResults::default();

    // THE implementation `pmat quality-gate --file` runs.
    run_all_single_file_checks(
        project_path,
        file_path,
        GATE_DEFAULT_MAX_COMPLEXITY_P99,
        &mut violations,
        &mut results,
    )
    .await?;

    Ok(GateSuite {
        violations,
        ran: file_suite_checks()
            .iter()
            .map(ToString::to_string)
            .collect(),
        not_run,
    })
}

/// Drop the findings under `[exclude]` paths from `.pmat-metrics.toml`
/// (#196, #197), and say how many were dropped.
///
/// ONE implementation: `handle_project_quality_gate` applied this rule to the
/// CLI's findings and nothing applied it to the MCP tool's, so an excluded path
/// was clean on one surface and a violation on the other.
pub fn apply_gate_exclude_paths(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
) -> usize {
    let exclude_paths = load_entropy_exclude_paths(project_path);
    if exclude_paths.is_empty() {
        return 0;
    }
    let before = violations.len();
    filter_violations_by_exclude(violations, &exclude_paths);
    before - violations.len()
}

/// Give a fixture the two artifacts the gate reads but does not produce.
///
/// A project with no coverage report and no README is not "clean": the gate
/// reports the coverage gap as a blocking violation and leaves both checks
/// unmeasured. Fixtures that mean "clean for every check this gate runs" say so
/// here, rather than by weakening the assertion that noticed.
#[cfg(test)]
pub(crate) fn write_gate_artifacts(project_path: &Path, coverage_percent: f64) {
    std::fs::create_dir_all(project_path.join(".pmat-metrics")).expect("mkdir .pmat-metrics");
    std::fs::write(
        project_path.join(".pmat-metrics/coverage.json"),
        format!("{{\"coverage\": {coverage_percent}}}"),
    )
    .expect("write coverage.json");
    std::fs::write(
        project_path.join("README.md"),
        "# Fixture\n\n## Installation\n\n## Usage\n\n## Contributing\n\n## License\n",
    )
    .expect("write README.md");
}

/// The suite is only "the same checks" if it runs at the same thresholds and
/// over the same list of checks as the CLI it borrows its name from.
#[cfg(test)]
mod quality_gate_suite_tests {
    use super::*;

    /// The MCP gate's thresholds are clap's defaults, read from clap.
    ///
    /// A constant that merely *looks* like the CLI default is the same defect as
    /// a second implementation that merely looks like the first, so this asks
    /// the parser rather than the source comment.
    #[test]
    fn the_suite_thresholds_are_the_cli_defaults() {
        // `try_parse_from` needs the 8MB stack (see `on_big_stack`); a bare
        // `cargo test --lib` otherwise aborts the whole binary.
        let (max_dead_code, max_complexity_p99, min_entropy) =
            crate::cli::commands::on_big_stack(|| {
                use clap::Parser;
                let cli =
                    crate::cli::Cli::try_parse_from(["pmat", "quality-gate"]).expect("parses");
                let crate::cli::Commands::QualityGate {
                    max_dead_code,
                    max_complexity_p99,
                    min_entropy,
                    ..
                } = cli.command
                else {
                    panic!("`pmat quality-gate` must parse as QualityGate");
                };
                (max_dead_code, max_complexity_p99, min_entropy)
            });
        assert_eq!(
            max_dead_code, GATE_DEFAULT_MAX_DEAD_CODE,
            "the MCP gate must fail dead code at the threshold the CLI fails it at"
        );
        assert_eq!(
            max_complexity_p99, GATE_DEFAULT_MAX_COMPLEXITY_P99,
            "the MCP gate must fail complexity at the threshold the CLI fails it at"
        );
        assert_eq!(
            min_entropy, None,
            "the suite passes None so project config decides, as the CLI does"
        );
    }

    /// Every check the suite claims to have run must be one the CLI knows, and
    /// the file subset must be a subset of the project list.
    #[test]
    fn the_file_suite_is_a_subset_of_the_project_suite() {
        let project = project_suite_checks();
        for check in file_suite_checks() {
            assert!(
                project.contains(&check),
                "{check} is run for a file but is not part of the project suite"
            );
        }
        assert_eq!(
            project.len() - file_suite_checks().len(),
            5,
            "five project-wide checks a file cannot answer; if that changed, \
             `file_scope_reason` and the disclosure list changed with it"
        );
    }

    #[tokio::test]
    async fn a_file_names_the_project_wide_checks_it_did_not_run() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, "/// Adds.\npub fn add(a: i32) -> i32 { a + 1 }\n")
            .expect("write fixture");

        let suite = run_gate_suite(&file).await.expect("the suite reports");

        assert!(
            suite.violations.is_empty(),
            "a clean file has no findings: {:?}",
            suite.violations
        );
        for check in [
            "coverage",
            "duplicates",
            "entropy",
            "provability",
            "sections",
        ] {
            assert!(
                suite.not_run_names().iter().any(|n| n == check),
                "{check} did not run for a single file and must say so: {:?}",
                suite.not_run_names()
            );
        }
        assert!(
            suite.ran.iter().any(|c| c == "satd"),
            "the four file checks did run: {:?}",
            suite.ran
        );
    }

    /// An unparseable file must not abort the call, and must not silently look
    /// like a file the four checks cleared.
    #[tokio::test]
    async fn an_unparseable_file_reports_every_check_as_unrun() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("broken.rs");
        std::fs::write(&file, "fn main( { let x = ;;;\n").expect("write fixture");

        let suite = run_gate_suite(&file).await.expect("no hard error");

        assert!(suite.ran.is_empty(), "nothing ran: {:?}", suite.ran);
        assert_eq!(
            suite.not_run_names().len(),
            project_suite_checks().len(),
            "every advertised check is unrun for a file that did not parse: {:?}",
            suite.not_run_names()
        );
    }

    /// A check that read no artifact measured nothing, and says so.
    ///
    /// `check_coverage` and `check_sections` both answer "no violations" for a
    /// project that has no coverage report and no README — the same answer a
    /// fully covered, fully documented project gets. `ran` means MEASURED.
    #[tokio::test]
    async fn a_check_with_nothing_to_read_is_not_reported_as_run() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("lib.rs"),
            "/// Adds.\npub fn a() -> i32 { 1 }\n",
        )
        .expect("write fixture");

        let bare = run_gate_suite(dir.path()).await.expect("the suite reports");
        assert_eq!(
            bare.not_run_names(),
            vec!["coverage".to_string(), "sections".to_string()],
            "neither artifact exists, so neither check measured anything: {:?}",
            bare.not_run
        );
        assert!(
            bare.violations.iter().any(|v| v.check_type == "coverage"),
            "…and the gap is a finding as well as a disclosure: {:?}",
            bare.violations
        );

        write_gate_artifacts(dir.path(), 95.0);
        let measured = run_gate_suite(dir.path()).await.expect("the suite reports");
        assert!(
            measured.not_run_names().is_empty(),
            "with both artifacts present the whole suite measured: {:?}",
            measured.not_run
        );
        assert!(
            !measured
                .violations
                .iter()
                .any(|v| v.check_type == "coverage"),
            "…and 95% clears the floor, so the gap row is gone rather than \
             replaced by a second constant: {:?}",
            measured.violations
        );
    }

    /// The same file named twice must not double its findings.
    #[tokio::test]
    async fn the_same_path_twice_weighs_once() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("debt.rs");
        std::fs::write(
            &file,
            "/// Adds.\npub fn add(a: i32) -> i32 {\n    // FIXME: overflow\n    a + 1\n}\n",
        )
        .expect("write fixture");

        let once = run_gate_suite_over(std::slice::from_ref(&file))
            .await
            .expect("the suite reports");
        let twice = run_gate_suite_over(&[file.clone(), file])
            .await
            .expect("the suite reports");

        assert_eq!(
            once.violations.len(),
            1,
            "one FIXME, one finding: {:?}",
            once.violations
        );
        assert_eq!(
            once.violations.len(),
            twice.violations.len(),
            "naming a path twice does not double its debt: {:?}",
            twice.violations
        );
    }
}
