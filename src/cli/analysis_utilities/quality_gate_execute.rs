/// Runs a single project-wide check
#[allow(clippy::too_many_arguments)]
/// Toyota Way: Data-Driven Design - eliminated 41→≤8 complexity
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn run_single_project_check(
    check: &QualityCheckType,
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    match check {
        QualityCheckType::All => {
            run_all_project_checks(
                project_path,
                max_dead_code,
                min_entropy,
                max_complexity_p99,
                violations,
                results,
                perf,
            )
            .await
        }
        _ => {
            execute_specific_quality_check(
                check,
                project_path,
                max_dead_code,
                min_entropy,
                max_complexity_p99,
                violations,
                results,
            )
            .await
        }
    }
}

/// Toyota Way: Extract Method - handle specific quality checks (complexity ≤5)
/// Toyota Way: Template Method pattern - reduced complexity from 23→≤3
async fn execute_specific_quality_check(
    check: &QualityCheckType,
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    use QualityCheckType::{
        All, Complexity, Coverage, DeadCode, Duplicates, Entropy, Provability, Satd, Sections,
        Security,
    };

    match check {
        Complexity => {
            execute_complexity_check(project_path, max_complexity_p99, violations, results).await
        }
        DeadCode => execute_dead_code_check(project_path, max_dead_code, violations, results).await,
        Satd => execute_satd_check(project_path, violations, results).await,
        Entropy => execute_entropy_check(project_path, min_entropy, violations, results).await,
        Security => execute_security_check(project_path, violations, results).await,
        Duplicates => execute_duplicates_check(project_path, violations, results).await,
        Coverage => execute_coverage_check(project_path, violations, results).await,
        Sections => execute_sections_check(project_path, violations, results).await,
        Provability => execute_provability_check(project_path, violations, results).await,
        All => unreachable!("All case handled in parent function"),
    }
}

/// Toyota Way: Template Method - extracts common quality check pattern
async fn execute_quality_check_template<Fut, S>(
    check_future: Fut,
    set_result: S,
    violations: &mut Vec<QualityViolation>,
) -> Result<()>
where
    Fut: std::future::Future<Output = Result<Vec<QualityViolation>>>,
    S: FnOnce(usize),
{
    let violations_found = check_future.await?;
    set_result(violations_found.len());
    violations.extend(violations_found);
    Ok(())
}

/// Helper for complexity check execution
async fn execute_complexity_check(
    project_path: &Path,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_complexity(project_path, max_complexity_p99),
        |count| results.complexity_violations = count,
        violations,
    )
    .await
}

/// Helper for dead code check execution
async fn execute_dead_code_check(
    project_path: &Path,
    max_dead_code: f64,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    run_dead_code_check(project_path, max_dead_code, violations, results).await
}

/// The dead-code check with its outcome DISCLOSED: findings into
/// `violations`, and the run that could not measure (or had nothing to
/// measure) into `results.not_measured` / `results.not_applicable`. Both gate
/// paths — `--checks <selection>` here and `--checks all` in
/// `run_all_project_checks` — go through this one function so the two cannot
/// disclose differently (CRUX-02, #1153).
///
/// # Errors
/// Only the contract macro's own; analyzer failures are disclosed, not raised.
pub(crate) async fn run_dead_code_check(
    project_path: &Path,
    max_dead_code: f64,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    let outcome = check_dead_code_outcome(project_path, max_dead_code).await?;
    results.dead_code_violations = outcome.violations.len();
    violations.extend(outcome.violations);
    results.not_measured.extend(outcome.not_measured);
    results.not_applicable.extend(outcome.not_applicable);
    Ok(())
}

/// What the gate's duplicate check does NOT measure, said in the payload.
///
/// `check_duplicates` hashes whole files; `analyze duplicates` finds
/// block-level clones (21.67 % on this tree while the gate reported 0). Until
/// that detector is wired behind the gate — a separate item, because it costs
/// 8× the wall clock and ~180× the CPU of the whole gate — every run says so
/// here, beside the `identical_files` count it did measure.
pub(crate) fn duplicates_block_level_disclosure(project_path: &Path) -> UnmeasuredCheck {
    UnmeasuredCheck {
        check: "duplicates".to_string(),
        path: project_path.display().to_string(),
        reason: "block-level duplicate detection is not wired into this gate: `identical_files` \
                 counts whole files with byte-identical content only; run `pmat analyze \
                 duplicates` for clone blocks (separate item)"
            .to_string(),
    }
}

/// Helper for SATD check execution
///
/// Not routed through `execute_quality_check_template` like its neighbours,
/// because this check reports a second thing the template has no slot for: the
/// population it measured over. Dropping that on the floor is what let
/// `satd_violations: 1` stand beside a file, unread for being over 500 KB, that
/// held a marker. See [`check_satd_with_scope`].
async fn execute_satd_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    let (found, census) = check_satd_with_scope(project_path).await?;
    results.satd_violations = found.len();
    results.files_not_read.insert("satd".to_string(), census);
    violations.extend(found);
    Ok(())
}

/// Helper for entropy check execution (loads config from .pmat-gates.toml, #220)
async fn execute_entropy_check(
    project_path: &Path,
    min_entropy: Option<f64>,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    let gate_config = load_entropy_gate_config(project_path);
    if !gate_config.enabled {
        crate::status_eprintln!("  ⏭️  Entropy check disabled via .pmat-gates.toml");
        return Ok(());
    }
    let threshold = load_entropy_threshold(project_path, min_entropy);
    let mut exclude_paths = load_entropy_exclude_paths(project_path);
    // Merge per-check excludes from [entropy] section (#220)
    for pattern in &gate_config.exclude {
        if !exclude_paths.contains(pattern) {
            exclude_paths.push(pattern.clone());
        }
    }
    let mut entropy_violations =
        check_entropy_with_excludes(project_path, threshold, &exclude_paths).await?;
    // Apply max_violations threshold (#220)
    if let Some(max) = gate_config.max_violations {
        if entropy_violations.len() <= max {
            entropy_violations.clear();
        }
    }
    results.entropy_violations = entropy_violations.len();
    violations.extend(entropy_violations);
    Ok(())
}

/// Helper for security check execution
async fn execute_security_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    // Like the SATD check above, and unlike its other neighbours, this one
    // reports the population it covered as well as what it found. The scan
    // reads ONE directory: on any ordinary Rust project it never opens `src/`,
    // so `security_violations: 0` was a statement about a dozen files rendered
    // as a statement about the repository. See `check_security_with_scope`.
    let (found, scope) = check_security_with_scope(project_path).await?;
    results.security_violations = found.len();
    violations.extend(found);
    violations.push(security_scope_disclosure(project_path, scope));
    Ok(())
}

/// Helper for duplicates check execution
async fn execute_duplicates_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_duplicates(project_path),
        |count| results.identical_files = count,
        violations,
    )
    .await?;
    results.not_measured.push(duplicates_block_level_disclosure(project_path));
    Ok(())
}

/// Helper for coverage check execution
async fn execute_coverage_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    // `--checks coverage` and `--checks all` run the same composition, so the
    // one surface cannot report a gap the other silently counts as zero.
    execute_quality_check_template(
        run_coverage_check(project_path),
        |count| results.coverage_violations = count,
        violations,
    )
    .await
}

/// `results.coverage_violations` had two meanings and one value.
///
/// These run the surface a user runs — `pmat quality-gate --checks coverage` is
/// exactly [`execute_coverage_check`] — rather than the helper underneath it, so
/// they fail if the composition is ever unwired again.
#[cfg(test)]
mod coverage_is_measured_or_disclosed_tests {
    use super::*;

    async fn coverage_violations_for(project_path: &Path) -> (usize, Vec<QualityViolation>) {
        let mut violations = Vec::new();
        let mut results = QualityGateResults::default();
        execute_coverage_check(project_path, &mut violations, &mut results)
            .await
            .expect("the coverage check reports");
        (results.coverage_violations, violations)
    }

    fn write_metrics(dir: &Path, body: &str) {
        std::fs::create_dir_all(dir.join(".pmat-metrics")).expect("mkdir .pmat-metrics");
        std::fs::write(dir.join(".pmat-metrics/coverage.json"), body).expect("write coverage.json");
    }

    #[tokio::test]
    async fn an_absent_coverage_report_is_disclosed_not_counted_as_zero() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (count, violations) = coverage_violations_for(dir.path()).await;

        // It used to be 0 here — the same 0 a project measured at 100% gets.
        assert_eq!(
            count, 1,
            "an unmeasured check has not passed: {violations:?}"
        );
        let disclosure = &violations[0];
        assert_eq!(disclosure.check_type, "coverage");
        assert_eq!(disclosure.severity, "error", "an unmeasured check blocks");
        assert!(
            disclosure.message.contains("NOT measured"),
            "the row must say which of the two zeros this is: {}",
            disclosure.message
        );
    }

    #[tokio::test]
    async fn a_measured_project_above_the_floor_reports_no_violation() {
        // The disclosure must not become a second constant: a project that DID
        // measure, and passed, still reports 0.
        let dir = tempfile::tempdir().expect("tempdir");
        write_metrics(dir.path(), "{\"coverage\": 95.0}");
        let (count, violations) = coverage_violations_for(dir.path()).await;
        assert_eq!(count, 0, "95% clears the 80% floor: {violations:?}");
    }

    #[tokio::test]
    async fn a_measured_project_below_the_floor_reports_the_measurement() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_metrics(dir.path(), "{\"coverage\": 42.5}");
        let (count, violations) = coverage_violations_for(dir.path()).await;
        assert_eq!(count, 1);
        assert!(
            violations[0].message.contains("42.5"),
            "a real breach names the measured value, not the gap: {}",
            violations[0].message
        );
        assert!(
            !violations[0].message.contains("NOT measured"),
            "and is not the disclosure: {}",
            violations[0].message
        );
    }
}

/// Helper for sections check execution
async fn execute_sections_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    execute_quality_check_template(
        check_sections(project_path),
        |count| results.section_violations = count,
        violations,
    )
    .await
}

/// The SATD gate must state the population it measured over.
///
/// #1035's root cause, one level above the detector: `analyze_project` computes
/// a full census and `check_satd` used to discard it, so the gate reported
/// `satd_violations: N` with nothing to say what N was measured over.
///
/// The fixture below is the one the issue's audits describe, and on the pre-fix
/// build it produced this — four markers planted, one reported:
///
/// ```text
///   satd_violations: 1        files_examined: 7
///   files_not_read: { tests: 1, out_of_scope: 3, too_large: 1 }
/// ```
///
/// `examples/hello.rs` (two markers) was in `out_of_scope`, lumped with the
/// vendored and generated files; `src/big.rs` (one marker) was in `too_large`
/// for being over 512,000 bytes, announced on stderr only. Neither number could
/// be checked, because 1 analysed + 5 not read = 6 and the only other figure in
/// the payload was 7, counted over a different population.
#[cfg(test)]
mod satd_gate_discloses_what_it_did_not_read_tests {
    use super::*;

    /// `src/lib.rs` (a marker), `examples/hello.rs` (two markers — shipped code,
    /// analysed since #1035), `vendor/dep.rs` and `src/schema.generated.rs`
    /// (markers that must STAY excluded), `tests/harness.rs` (a marker, excluded
    /// without `--include-tests`), and `src/huge.rs`, past `MAX_FILE_BYTES`.
    ///
    /// The oversized file is created sparse — its length is a metadata fact and
    /// the walk declines it without ever reading a byte — so the fixture costs
    /// no disk and no I/O.
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        for sub in ["src", "examples", "vendor", "tests"] {
            std::fs::create_dir_all(root.join(sub)).expect("mkdir");
        }
        std::fs::write(
            root.join("src/lib.rs"),
            "// TODO: marker in src\npub fn a() {}\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            root.join("examples/hello.rs"),
            "// TODO: marker one in examples\n// FIXME: marker two in examples\n",
        )
        .expect("write hello.rs");
        std::fs::write(root.join("vendor/dep.rs"), "// TODO: vendored marker\n")
            .expect("write dep.rs");
        std::fs::write(
            root.join("src/schema.generated.rs"),
            "// TODO: generated marker\n",
        )
        .expect("write schema.generated.rs");
        std::fs::write(root.join("tests/harness.rs"), "// TODO: test marker\n")
            .expect("write harness.rs");
        let huge = std::fs::File::create(root.join("src/huge.rs")).expect("create huge.rs");
        huge.set_len(crate::services::satd_detector::MAX_FILE_BYTES + 1)
            .expect("size huge.rs");
        dir
    }

    async fn run_satd_gate(root: &Path) -> (QualityGateResults, Vec<QualityViolation>) {
        let mut violations = Vec::new();
        let mut results = QualityGateResults::default();
        execute_satd_check(root, &mut violations, &mut results)
            .await
            .expect("the satd check reports");
        // `results.violations` is the RENDERED line list, filled later by
        // `set_violation_lines`; the findings themselves are the vec the check
        // extends.
        results.set_violation_lines(&violations);
        (results, violations)
    }

    /// RED before the fix: `satd_violations` was 1 and the two markers in
    /// `examples/` were counted as out of scope.
    #[tokio::test]
    async fn markers_in_examples_are_debt_and_are_reported() {
        let dir = fixture();
        let (results, _) = run_satd_gate(dir.path()).await;

        assert_eq!(
            results.satd_violations, 3,
            "one marker in src/ and two in examples/, which is shipped code: {:?}",
            results.violations
        );
        assert!(
            results
                .violations
                .iter()
                .any(|v| v.contains("hello.rs") && v.contains("marker one")),
            "the example's debt must be named, not merely counted: {:?}",
            results.violations
        );
    }

    /// COUNTER-TEST: the fix must not turn into "report everything as debt".
    /// Code this project cannot fix in place stays excluded — and stays
    /// COUNTED, which is the half that makes the exclusion legible.
    #[tokio::test]
    async fn vendored_and_generated_code_stays_excluded_and_stays_counted() {
        let dir = fixture();
        let (results, _) = run_satd_gate(dir.path()).await;

        assert!(
            !results.violations.iter().any(|v| v.contains("vendor/")),
            "a vendored dependency is not this project's debt: {:?}",
            results.violations
        );
        assert!(
            !results.violations.iter().any(|v| v.contains(".generated")),
            "generated output is not hand-written debt: {:?}",
            results.violations
        );

        let census = results
            .files_not_read
            .get("satd")
            .expect("the satd check declares the population it measured over");
        assert_eq!(
            census.not_read.out_of_scope, 2,
            "vendored + generated, declined and disclosed: {census:?}"
        );
        assert_eq!(
            census.not_read.tests, 1,
            "tests/harness.rs was found and declined: {census:?}"
        );
    }

    /// RED before the fix: the oversized file's skip reached stderr only, and
    /// `--format json` discards stderr.
    #[tokio::test]
    async fn the_oversized_file_is_named_in_the_machine_readable_census() {
        let dir = fixture();
        let (results, _) = run_satd_gate(dir.path()).await;
        let census = results.files_not_read.get("satd").expect("scope declared");

        assert_eq!(
            census.not_read.too_large, 1,
            "the file past the limit was declined and must be disclosed: {census:?}"
        );
        let oversized = census
            .oversized
            .first()
            .expect("a count alone cannot answer WHICH file was not looked at");
        assert!(oversized.path.contains("huge.rs"), "{oversized:?}");
        assert!(
            oversized.bytes > oversized.limit_bytes,
            "the size and the limit are both stated, so the rule is visible: {oversized:?}"
        );
        assert_eq!(
            oversized.limit_bytes,
            crate::services::satd_detector::MAX_FILE_BYTES
        );
    }

    /// The whole point of #1035: the buckets must add up. A census that does not
    /// partition is the same defect in a new place.
    #[tokio::test]
    async fn the_census_partitions_the_files_it_walked() {
        let dir = fixture();
        let (results, _) = run_satd_gate(dir.path()).await;
        let census = results.files_not_read.get("satd").expect("scope declared");

        assert_eq!(census.discovered, 6, "six .rs files were walked: {census:?}");
        assert_eq!(census.analyzed, 2, "src/lib.rs and examples/hello.rs");
        assert_eq!(census.not_read.total(), 4, "{census:?}");
        assert!(
            census.partitions(),
            "analysed + not read must equal walked: {census:?}"
        );
        assert_eq!(census.unaccounted(), 0, "{census:?}");
    }

    /// COUNTER-TEST: a clean tree must still report zero findings over a
    /// NON-ZERO denominator. Without this, "everything was skipped" and "nothing
    /// was found" go back to being the same output — the defect, not the fix.
    #[tokio::test]
    async fn a_clean_tree_reports_zero_findings_over_a_nonzero_denominator() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        std::fs::create_dir_all(dir.path().join("examples")).expect("mkdir examples");
        std::fs::write(dir.path().join("src/lib.rs"), "pub fn a() -> u32 { 1 }\n")
            .expect("write lib.rs");
        std::fs::write(dir.path().join("examples/hello.rs"), "fn main() {}\n")
            .expect("write hello.rs");

        let (results, _) = run_satd_gate(dir.path()).await;
        let census = results.files_not_read.get("satd").expect("scope declared");

        assert_eq!(results.satd_violations, 0, "the tree really is clean");
        assert_eq!(
            census.not_read.total(),
            0,
            "nothing was declined, so nothing may be claimed as declined: {census:?}"
        );
        assert_eq!(
            census.analyzed, 2,
            "and the zero above was measured over two files, not over nothing: {census:?}"
        );
        let note = census
            .note()
            .expect("a clean tree still has a population to state");
        assert!(note.contains("analysed 2 of 2"), "{note}");
        assert!(
            !note.contains("not read"),
            "an empty bucket is noise, not disclosure: {note}"
        );
    }

    /// The human report is the surface most readers actually see, so the
    /// disclosure has to survive rendering — it lived only in the JSON once.
    #[test]
    fn the_human_report_renders_the_scope_note() {
        let mut results = QualityGateResults {
            satd_violations: 1,
            ..Default::default()
        };
        results.files_not_read.insert(
            "satd".to_string(),
            crate::services::satd_detector::FileCensus {
                discovered: 4,
                analyzed: 2,
                not_read: crate::services::satd_detector::SkipCounts {
                    too_large: 1,
                    out_of_scope: 1,
                    ..Default::default()
                },
                oversized: vec![crate::services::satd_detector::OversizedFile {
                    path: "src/huge.rs".to_string(),
                    bytes: crate::services::satd_detector::MAX_FILE_BYTES + 1,
                    limit_bytes: crate::services::satd_detector::MAX_FILE_BYTES,
                }],
            },
        );
        let rendered =
            format_quality_gate_output(&results, &[], crate::cli::QualityGateOutputFormat::Human)
                .expect("human report renders");
        assert!(
            rendered.contains("analysed 2 of 4"),
            "the report must state the denominator:\n{rendered}"
        );
        assert!(
            rendered.contains("not read"),
            "the report must name what was not read:\n{rendered}"
        );
        assert!(rendered.contains("too large"), "and why:\n{rendered}");
        assert!(
            rendered.contains("src/huge.rs"),
            "and which file, for the size skip that once reached stderr only:\n{rendered}"
        );
    }
}

/// The security gate must state that it read one directory.
///
/// `check_security` walks `read_dir(project_path)` and never recurses, so on any
/// project whose code lives in `src/` it opens no source file at all and reports
/// `security_violations: 0`. Two byte-identical `let password = "hunter2";`
/// lines, one at the root and one in `src/`, produced exactly one finding and no
/// statement anywhere that the second file was never opened.
#[cfg(test)]
mod security_gate_states_its_reach_tests {
    use super::*;

    fn two_identical_secrets() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        let leak = "fn a() { let password = \"hunter2\"; }\n";
        std::fs::write(dir.path().join("root_leak.rs"), leak).expect("write root_leak.rs");
        std::fs::write(dir.path().join("src/deep_leak.rs"), leak).expect("write deep_leak.rs");
        dir
    }

    async fn run_security_gate(root: &Path) -> (QualityGateResults, Vec<QualityViolation>) {
        let mut violations = Vec::new();
        let mut results = QualityGateResults::default();
        execute_security_check(root, &mut violations, &mut results)
            .await
            .expect("the security check reports");
        (results, violations)
    }

    #[tokio::test]
    async fn the_narrow_reach_is_stated_beside_the_finding_count() {
        let dir = two_identical_secrets();
        let (results, violations) = run_security_gate(dir.path()).await;

        // Unchanged: the scan still finds only the root copy.
        assert_eq!(
            results.security_violations, 1,
            "the src/ copy is not reached: {violations:?}"
        );

        let scope = violations
            .iter()
            .find(|v| v.check_type == "scope")
            .expect("the reach of the scan is stated, not left to be inferred");
        assert!(
            scope.message.contains("did NOT descend"),
            "the row must say the scan stopped at the root: {}",
            scope.message
        );
        assert!(
            scope.message.contains("1 source file"),
            "and how many files that was: {}",
            scope.message
        );
    }

    /// Counter-test bounding the over-correction: the disclosure must not fail
    /// anyone's gate. It describes a limit of this tool, not a defect in the
    /// tree being scanned, so it is advisory and the verdict is unmoved.
    #[tokio::test]
    async fn the_disclosure_is_advisory_and_does_not_decide_the_verdict() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("clean.rs"), "fn a() {}\n").expect("write clean.rs");

        let (results, violations) = run_security_gate(dir.path()).await;
        assert_eq!(results.security_violations, 0, "nothing to find");

        let scope = violations
            .iter()
            .find(|v| v.check_type == "scope")
            .expect("stated even when the scan found nothing — that is the case it exists for");
        assert_eq!(scope.severity, ADVISORY_SEVERITY);
        assert_eq!(
            blocking_violation_count(&violations),
            0,
            "a disclosure about our own reach must never block a user's gate: {violations:?}"
        );
    }
}
