/// Source files the gate could have examined, counted the way the analyzers walk.
///
/// Deliberately the same `ignore` walk the rest of the tool uses (gitignore
/// honoured, hidden skipped) so this number cannot disagree with what the checks
/// actually saw. It counts FILES WITH AN EXTENSION rather than a language list:
/// the question here is "was there anything to look at", and answering it from a
/// list of supported languages would add another copy of a set this repository
/// already keeps in several places that disagree.
fn count_examined_sources(project_path: &std::path::Path) -> usize {
    ignore::WalkBuilder::new(project_path)
        .hidden(true)
        .git_ignore(true)
        .build()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .filter(|e| e.path().extension().is_some())
        .count()
}

/// The project config files this gate reads its own thresholds out of.
///
/// `.pmat-gates.toml` supplies the entropy threshold, the entropy enable
/// switch, `max_violations` and the exclude globs; `.pmat-metrics.toml`
/// supplies `[exclude]` and the legacy `entropy_min_diversity`; `pmat.toml`
/// supplies the `[quality]` fallbacks.
const GATE_CONFIG_FILES: &[&str] = &[".pmat-gates.toml", ".pmat-metrics.toml", "pmat.toml"];

/// Config files that EXIST and do not parse, paired with the parser's message.
///
/// #1019: every reader of these files is written `read_to_string(..).ok()?`,
/// `content.parse().ok()?` or `Err(_) => <defaults>`. So a `.pmat-gates.toml`
/// with a typo in a section header — `[entropy` — drops the project silently
/// back to the built-in defaults and the gate reports PASSED. Measured on
/// 3.32.0: a fixture carrying `min_pattern_diversity = 0.99`, a threshold that
/// tree cannot meet, passed with `violations: []` because the file above it
/// failed to parse and nothing said so.
///
/// A configured limit that had no effect must not read as a limit that was met.
fn unparsable_gate_configs(project_path: &Path) -> Vec<(String, String)> {
    let mut broken = Vec::new();
    for name in GATE_CONFIG_FILES {
        let path = project_path.join(name);
        // Absent is fine — that is a project with no config, not a broken one.
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Err(e) = content.parse::<toml::Table>() {
            broken.push(((*name).to_string(), e.to_string()));
        }
    }
    broken
}

/// Handles project-wide quality gate checks
#[allow(clippy::too_many_arguments)]
async fn handle_project_quality_gate(
    project_path: PathBuf,
    format: QualityGateOutputFormat,
    exit_on_violation: bool,
    checks: Vec<QualityCheckType>,
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    include_provability: bool,
    output: Option<PathBuf>,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;
    let mut violations = Vec::new();
    let mut results = QualityGateResults::default();

    // Run selected checks
    let checks_start = if perf { Some(Instant::now()) } else { None };

    run_project_checks(
        &project_path,
        &checks,
        max_dead_code,
        min_entropy,
        max_complexity_p99,
        &mut violations,
        &mut results,
        perf,
    )
    .await?;

    // Apply [exclude] paths from .pmat-metrics.toml to ALL violations (#196, #197),
    // through the one function that owns the rule — the MCP gate applies the
    // same one to the same findings.
    let removed = apply_gate_exclude_paths(&project_path, &mut violations);
    if removed > 0 {
        crate::status_eprintln!("  📁 Excluded {removed} violations from excluded paths");
        results.recalculate_from(&violations);
    }

    // Add provability if requested
    if include_provability {
        let prov_start = if perf { Some(Instant::now()) } else { None };
        let provability_score = calculate_provability_score(&project_path).await?;
        results.provability_score = Some(provability_score);

        if let Some(start) = prov_start {
            crate::status_eprintln!(
                "  ⏱️  Provability analysis: {:.3}s",
                start.elapsed().as_secs_f64()
            );
        }
    }

    if let Some(start) = checks_start {
        let duration = start.elapsed();
        crate::status_eprintln!(
            "\n⏱️  All checks completed in: {:.3}s",
            duration.as_secs_f64()
        );
    }

    // Calculate overall pass/fail — THE rule, from the one place that owns it.
    // This was `violations.is_empty()`, i.e. any finding of any severity failed
    // the gate, while the MCP `quality_gate` tool over the SAME producer
    // (`check_satd`) ignored `severity:"info"`. One `// TODO` in one file was
    // `passed:false` here and `passed:true` there, with byte-identical findings.
    // A gate that examined NOTHING is not a gate that passed.
    //
    // Measured before this: `quality-gate --checks complexity` over an empty
    // directory and over a clean two-function project produced byte-identical
    // JSON, byte-identical stderr, and exit 0 from both. `violations_pass` over
    // an empty list is `true` whether the list is empty because the code is
    // clean or because no code was read.
    //
    // The precedent is already in this repository: an unmeasured COVERAGE gate
    // blocks rather than passing. This applies the same rule to the population.
    results.files_examined = count_examined_sources(&project_path);
    results.checks_run = checks.iter().map(|c| format!("{c:?}")).collect();
    if results.files_examined == 0 {
        violations.push(QualityViolation {
            check_type: "population".to_string(),
            severity: "error".to_string(),
            file: project_path.display().to_string(),
            line: None,
            message: format!(
                "no source files were examined under {}, so no check could have found \
                 anything — this is an unmeasured gate, not a clean one",
                project_path.display()
            ),
            details: None,
        });
    }
    // #1019: a config file the gate reads and cannot parse is not a config file
    // it ignored — it is a set of thresholds the run silently replaced with its
    // own defaults. Blocking, for the same reason the population check above is:
    // the gate that ran is not the gate the project asked for.
    for (file, error) in unparsable_gate_configs(&project_path) {
        violations.push(QualityViolation {
            check_type: "config".to_string(),
            severity: "error".to_string(),
            file: file.clone(),
            line: None,
            message: format!(
                "{file} exists but is not valid TOML ({error}), so every threshold in \
                 it was silently replaced by pmat's built-in defaults — the gate that \
                 ran is not the gate this project configured"
            ),
            details: None,
        });
    }

    results.passed = violations_pass(&violations);
    results.total_violations = violations.len();
    results.blocking_violations = blocking_violation_count(&violations);
    // `results.violations` shipped as a permanently-empty array while
    // `results.total_violations` beside it said 3.
    results.set_violation_lines(&violations);

    // Persist violations to SQLite for `pmat sql` queryability
    persist_violations_to_sqlite(&project_path, &violations);

    // Persist per-function provability scores to specialized table (#231)
    persist_provability_to_sqlite(&project_path).await;

    // Format and output results
    output_project_results(&results, &violations, format, output).await?;

    // Print final status (chatter: the verdict is also in the report on stdout
    // and in the exit status, so --quiet suppresses this line only)
    print_quality_gate_final_status(&results, &violations);

    // Handle exit status
    handle_quality_gate_exit_status(exit_on_violation, results.passed);

    Ok(())
}

/// Runs project-wide quality checks
#[allow(clippy::too_many_arguments)]
async fn run_project_checks(
    project_path: &Path,
    checks: &[QualityCheckType],
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    // If checks contains All, just run that single check which will run all checks
    if checks.contains(&QualityCheckType::All) {
        run_single_project_check(
            &QualityCheckType::All,
            project_path,
            max_dead_code,
            min_entropy,
            max_complexity_p99,
            violations,
            results,
            perf,
        )
        .await?;
    } else {
        // Otherwise run each specified check
        run_individual_project_checks(
            checks,
            project_path,
            max_dead_code,
            min_entropy,
            max_complexity_p99,
            violations,
            results,
            perf,
        )
        .await?;
    }
    Ok(())
}

/// Run individual quality checks with optional performance timing
#[allow(clippy::too_many_arguments)]
async fn run_individual_project_checks(
    checks: &[QualityCheckType],
    project_path: &Path,
    max_dead_code: f64,
    min_entropy: Option<f64>,
    max_complexity_p99: u32,
    violations: &mut Vec<QualityViolation>,
    results: &mut QualityGateResults,
    perf: bool,
) -> Result<()> {
    use std::time::Instant;

    for check in checks {
        let check_start = if perf { Some(Instant::now()) } else { None };

        run_single_project_check(
            check,
            project_path,
            max_dead_code,
            min_entropy,
            max_complexity_p99,
            violations,
            results,
            perf,
        )
        .await?;

        if let Some(start) = check_start {
            print_check_performance(check, start.elapsed().as_secs_f64());
        }
    }
    Ok(())
}

/// Print performance timing for a quality check
fn print_check_performance(check: &QualityCheckType, elapsed_secs: f64) {
    let check_name = get_check_display_name(check);
    crate::status_eprintln!("    ⏱️  {check_name} check: {elapsed_secs:.3}s");
}

/// Get display name for a quality check type
fn get_check_display_name(check: &QualityCheckType) -> &'static str {
    match check {
        QualityCheckType::Complexity => "Complexity",
        QualityCheckType::DeadCode => "Dead code",
        QualityCheckType::Satd => "SATD",
        QualityCheckType::Security => "Security",
        QualityCheckType::Entropy => "Entropy",
        QualityCheckType::Duplicates => "Duplicates",
        QualityCheckType::Coverage => "Coverage",
        QualityCheckType::Sections => "Sections",
        QualityCheckType::Provability => "Provability",
        QualityCheckType::All => "All",
    }
}

#[cfg(test)]
mod population_tests {
    use super::count_examined_sources;

    /// An empty tree has no population, and that is the fact the gate turns
    /// into a verdict.
    ///
    /// Before this, `quality-gate --checks complexity` over an empty directory
    /// and over a clean two-function project produced byte-identical JSON,
    /// byte-identical stderr, and exit 0 from both. `violations_pass` over an
    /// empty violation list is `true` whether the list is empty because the
    /// code is clean or because no code was read.
    #[test]
    fn an_empty_tree_examines_nothing() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        assert_eq!(count_examined_sources(dir.path()), 0);
    }

    /// The counter-test, and the more important half. A tree WITH sources must
    /// report a non-zero population, or the fix converts every clean run into a
    /// failure and the gate becomes one people disable.
    #[test]
    fn a_populated_tree_is_counted() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("lib.rs"), "pub fn f() {}\n").expect("write");
        std::fs::write(dir.path().join("notes.md"), "# hi\n").expect("write");
        assert_eq!(count_examined_sources(dir.path()), 2);
    }

    /// Gitignored files are not population. The walk must agree with the one
    /// the checks use, or the denominator would count files no check could see.
    ///
    /// `git init` is load-bearing: `ignore::WalkBuilder` only applies
    /// `.gitignore` inside a repository, so without it this test passes for the
    /// wrong reason — it would count 2 and prove nothing about the rule.
    #[test]
    fn gitignored_files_are_not_population() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(["init", "-q"])
                .status()
                .expect("git must be runnable")
                .success(),
            "git init"
        );
        std::fs::write(dir.path().join(".gitignore"), "skipme.rs\n").expect("write");
        std::fs::write(dir.path().join("skipme.rs"), "pub fn s() {}\n").expect("write");
        std::fs::write(dir.path().join("keep.rs"), "pub fn k() {}\n").expect("write");
        assert_eq!(
            count_examined_sources(dir.path()),
            1,
            "only keep.rs is population; .gitignore itself has no extension"
        );
    }
}

#[cfg(test)]
mod gate_config_parse_tests {
    use super::unparsable_gate_configs;

    /// #1019: a `.pmat-gates.toml` with a typo'd section header is dropped on
    /// the floor by every reader (`content.parse().ok()?`, `Err(_) => default`)
    /// and the gate reports PASSED at the built-in thresholds.
    ///
    /// Measured on 3.32.0 before this: a fixture whose `.pmat-gates.toml` said
    /// `min_pattern_diversity = 0.99` — unmeetable by that tree — printed
    /// `"violations": []` and `Quality gate PASSED`, because the header above
    /// it was `[entropy` and the parse error went nowhere.
    #[test]
    fn a_config_that_does_not_parse_is_reported() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(
            dir.path().join(".pmat-gates.toml"),
            "[entropy\nmin_pattern_diversity = 0.99\n",
        )
        .expect("write");

        let broken = unparsable_gate_configs(dir.path());
        assert_eq!(
            broken.len(),
            1,
            "the malformed config must be reported, not silently replaced by \
             defaults: {broken:?}"
        );
        assert_eq!(broken[0].0, ".pmat-gates.toml");
        assert!(
            !broken[0].1.is_empty(),
            "the parser's reason must be carried to the user: {broken:?}"
        );
    }

    /// Counter-test one: a project with NO config is not a project with a
    /// broken one. Without this, the fix fails every repo that never wrote a
    /// `.pmat-gates.toml` — a gate nobody can pass is as useless as one nobody
    /// can fail.
    #[test]
    fn an_absent_config_is_not_a_finding() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        assert!(unparsable_gate_configs(dir.path()).is_empty());
    }

    /// Counter-test two: a well-formed config is silent, including one whose
    /// keys pmat does not read. Unknown-key detection is a separate, larger
    /// question (#1019 acceptance criterion 1); this check is only about a file
    /// that could not be parsed at all, and it must not stray into the other.
    #[test]
    fn a_well_formed_config_is_silent_even_with_unread_keys() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(
            dir.path().join(".pmat-gates.toml"),
            "[entropy]\nmin_pattern_diversity = 0.3\n[nonsense]\nwhat = 1\n",
        )
        .expect("write");
        std::fs::write(dir.path().join("pmat.toml"), "[quality]\n").expect("write");
        assert!(
            unparsable_gate_configs(dir.path()).is_empty(),
            "a parseable file is not a finding here"
        );
    }

    /// All three files the gate reads are checked, not just the first.
    #[test]
    fn every_config_file_the_gate_reads_is_checked() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join(".pmat-metrics.toml"), "not = = toml\n").expect("write");
        let broken = unparsable_gate_configs(dir.path());
        assert_eq!(broken.len(), 1, "{broken:?}");
        assert_eq!(broken[0].0, ".pmat-metrics.toml");
    }
}
