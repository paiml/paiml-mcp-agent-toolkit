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
    execute_quality_check_template(
        check_dead_code(project_path, max_dead_code),
        |count| results.dead_code_violations = count,
        violations,
    )
    .await
}

/// Helper for SATD check execution
///
/// Not routed through `execute_quality_check_template` like its neighbours,
/// because this check reports a second thing the template has no slot for: the
/// population it declined to read. Dropping that on the floor is what let
/// `satd_violations: 1` stand beside a file, unread for being over 500 KB, that
/// held a marker. See [`check_satd_with_scope`].
async fn execute_satd_check(
    project_path: &Path,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
) -> Result<()> {
    let (found, not_read) = check_satd_with_scope(project_path).await?;
    results.satd_violations = found.len();
    results.files_not_read.insert("satd".to_string(), not_read);
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
        |count| results.duplicate_violations = count,
        violations,
    )
    .await
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
/// a full `SkipCounts` and `check_satd` used to discard it, so the gate reported
/// `satd_violations: N` with nothing to say what N was measured over. Over the
/// fixture below the gate reads `src/lib.rs`, declines `examples/demo.rs` for
/// being out of scope and declines the 600 KB `src/big.rs` for size — and the
/// file it declined for size holds a marker, so the finding count it prints is
/// strictly lower than `pmat analyze satd` prints over the same tree, for a
/// reason the output never mentioned.
#[cfg(test)]
mod satd_gate_discloses_what_it_did_not_read_tests {
    use super::*;

    /// `src/lib.rs` marker; `examples/demo.rs` marker (out of scope);
    /// `src/big.rs` marker inside 600 KB of filler (over the 500 KB threshold).
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).expect("mkdir src");
        std::fs::create_dir_all(root.join("examples")).expect("mkdir examples");
        std::fs::write(
            root.join("src/lib.rs"),
            "// TODO: marker in src\npub fn a() {}\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            root.join("examples/demo.rs"),
            "// TODO: marker in examples\n",
        )
        .expect("write demo.rs");
        let mut big = String::from("// TODO: marker in an oversized file\n");
        while big.len() < 600_000 {
            big.push_str("// filler line\n");
        }
        std::fs::write(root.join("src/big.rs"), big).expect("write big.rs");
        dir
    }

    async fn run_satd_gate(root: &Path) -> QualityGateResults {
        let mut violations = Vec::new();
        let mut results = QualityGateResults::default();
        execute_satd_check(root, &mut violations, &mut results)
            .await
            .expect("the satd check reports");
        results
    }

    #[tokio::test]
    async fn the_files_the_satd_gate_refused_to_read_are_counted_and_named() {
        let dir = fixture();
        let results = run_satd_gate(dir.path()).await;

        let skipped = results
            .files_not_read
            .get("satd")
            .expect("the satd check declares its scope even when nothing was skipped");

        // The marker in src/lib.rs is the only one the gate can see.
        assert_eq!(
            results.satd_violations, 1,
            "one readable file held a marker"
        );

        // …and the two it could NOT see are stated, rather than silently absent.
        assert_eq!(
            skipped.too_large, 1,
            "the 600 KB file was declined for size and must be disclosed: {skipped:?}"
        );
        assert_eq!(
            skipped.out_of_scope, 1,
            "examples/ was declined by policy and must be disclosed: {skipped:?}"
        );
        assert!(
            skipped.total() >= 2,
            "the denominator is the point of the field: {skipped:?}"
        );
    }

    /// Counter-test bounding the over-correction: disclosure must not become a
    /// standing complaint. A tree where every candidate WAS read reports an
    /// empty scope note, not a fabricated one — otherwise the new line is noise
    /// and readers learn to skip it, which is how the original defect survives.
    #[tokio::test]
    async fn a_tree_the_gate_read_in_full_discloses_nothing_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "// TODO: marker\npub fn a() {}\n",
        )
        .expect("write lib.rs");

        let results = run_satd_gate(dir.path()).await;
        let skipped = results.files_not_read.get("satd").expect("scope declared");
        assert_eq!(results.satd_violations, 1);
        assert_eq!(
            skipped.total(),
            0,
            "nothing was declined, so nothing may be claimed as declined: {skipped:?}"
        );
        assert!(
            skipped.note().is_none(),
            "an empty scope prints no note: {skipped:?}"
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
            crate::services::satd_detector::SkipCounts {
                too_large: 1,
                out_of_scope: 1,
                ..Default::default()
            },
        );
        let rendered =
            format_quality_gate_output(&results, &[], crate::cli::QualityGateOutputFormat::Human)
                .expect("human report renders");
        assert!(
            rendered.contains("not read"),
            "the report must name what was not read:\n{rendered}"
        );
        assert!(rendered.contains("too large"), "and why:\n{rendered}");
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
