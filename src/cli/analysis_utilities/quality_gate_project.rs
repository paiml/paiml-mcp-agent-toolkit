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

/// Sections of `pmat.toml` that no reader in pmat will ever consult, paired
/// with the nearest section that IS read.
///
/// #1019 acceptance criterion 1, the half that was deferred. The guard above
/// asks only "does the file parse as TOML?", treating "parses" as a proxy for
/// "will be applied". It is not one. Measured on 3.34.0 in
/// `paiml/interactive.paiml.com`, whose `pmat.toml` says
///
/// ```toml
/// [quality_gate]
/// max_cyclomatic_complexity = 15
/// max_cognitive_complexity = 20
/// ```
///
/// `pmat quality-gate` printed `Complexity thresholds: cyclomatic 30, cognitive
/// 25 (from built-in defaults)` and 51 violations. `[quality_gate]` is not a
/// section pmat has ever read — those thresholds live in `[quality]`, as
/// `max_complexity` / `max_cognitive_complexity` — and because the file parses,
/// `unparsable_gate_configs` is silent about it. The repo believed it had
/// zero-tolerance enforcement at 15/20 and was being measured at 30/25, with
/// one parenthetical on stderr as the only hint.
///
/// This is the same failure the guard above exists to stop, one level up: a
/// configured limit that had no effect, reading as a limit that was met. So it
/// carries the same verdict — blocking, not advisory.
///
/// The accepted set is taken FROM THE SCHEMA (`PmatConfig`), never from a list
/// maintained beside it: two hand-maintained lists with nothing tying them
/// together is exactly how a validator comes to disagree with the reader it is
/// supposed to police. Add a field to `PmatConfig` and this set grows on its
/// own — `every_schema_section_is_accepted` below fails if it ever does not.
///
/// Scope is `pmat.toml` deliberately. `.pmat-gates.toml` and
/// `.pmat-metrics.toml` have no declared type to derive an accepted set from,
/// so the same check over them could only be a hand-maintained list — the thing
/// this function refuses to be. Giving those two files a schema is the
/// prerequisite, and a separate change.
///
/// Scope is SECTIONS, not keys, for the same reason. `[quality]` legitimately
/// carries keys that are not `QualityConfig` fields — this repository's own
/// `pmat.toml` sets `min_pattern_diversity`, `max_pattern_repetition` and
/// `max_entropy_violations` there, consumed by the ad-hoc readers in
/// `quality_gate_config.rs` — so the accepted KEY set cannot be derived from
/// the schema alone, and writing it out by hand would put a second list beside
/// the readers with nothing tying them together. The consequence is measured
/// and real: `[quality] max_cyclomatic_complexity = 15` (right section, wrong
/// key) still resolves to the built-in 30 in silence. Closing that needs the
/// ad-hoc readers to declare the keys they consume first.
fn inapplicable_pmat_toml_sections(project_path: &Path) -> Vec<(String, Option<String>)> {
    let known = schema_pmat_toml_sections();

    // Absent is a project with no config, not a broken one; unparsable belongs
    // to `unparsable_gate_configs`, which already blocks on it — reporting it
    // here too would mean guessing at sections we could not read.
    let Ok(content) = std::fs::read_to_string(project_path.join("pmat.toml")) else {
        return Vec::new();
    };
    let Ok(table) = content.parse::<toml::Table>() else {
        return Vec::new();
    };

    table
        .keys()
        .filter(|section| !known.contains(section.as_str()))
        .map(|section| (section.clone(), nearest_known_section(section, &known)))
        .collect()
}

/// The top-level sections `pmat.toml` may contain — the SAME derived set
/// `pmat config --validate` uses, from `configuration_service`.
fn schema_pmat_toml_sections() -> std::collections::BTreeSet<String> {
    crate::services::configuration_service::schema_pmat_toml_sections()
}

/// One list, shared with `pmat config --validate`: both the gate and the
/// validator call `crate::services::configuration_service::nearest_known_section`,
/// so a section the gate blocks on is a section the validator names, with the
/// same suggestion (CRUX-03 leg 3d).
fn nearest_known_section(
    unknown: &str,
    known: &std::collections::BTreeSet<String>,
) -> Option<String> {
    crate::services::configuration_service::nearest_known_section(unknown, known)
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
    thresholds: QualityThresholds,
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
        thresholds,
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
    // …and a file that DOES parse but puts its settings where nothing reads them
    // is the same defect one level up: thresholds the project wrote, replaced by
    // pmat's defaults, with no signal. Blocking for the same reason.
    for (section, nearest) in inapplicable_pmat_toml_sections(&project_path) {
        let hint = nearest.map_or_else(String::new, |n| format!(" (did you mean `[{n}]`?)"));
        violations.push(QualityViolation {
            check_type: "config".to_string(),
            severity: "error".to_string(),
            file: "pmat.toml".to_string(),
            line: None,
            message: format!(
                "pmat.toml declares `[{section}]`, which no part of pmat reads{hint}, \
                 so every setting under it had no effect and the run used pmat's \
                 built-in defaults instead — the gate that ran is not the gate this \
                 project configured"
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
    thresholds: QualityThresholds,
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
            thresholds,
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
            thresholds,
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
    thresholds: QualityThresholds,
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
            thresholds,
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
        QualityCheckType::FileSize => "File size",
        QualityCheckType::Churn => "Churn",
        QualityCheckType::Lint => "Lint",
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

#[cfg(test)]
mod inapplicable_config_tests {
    //! #1019 acceptance criterion 1: a `pmat.toml` that parses but whose
    //! settings land where nothing reads them.
    use super::inapplicable_pmat_toml_sections;

    fn sections(contents: &str) -> Vec<(String, Option<String>)> {
        let dir = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(dir.path().join("pmat.toml"), contents).expect("write");
        let mut found = inapplicable_pmat_toml_sections(dir.path());
        found.sort();
        found
    }

    /// THE reported defect, verbatim from `paiml/interactive.paiml.com`.
    ///
    /// Before the fix this returns `[]` and the gate runs at 30/25 while the
    /// file asks for 15/20, saying so only in a parenthetical on stderr.
    #[test]
    fn the_interactive_paiml_com_defect_is_reported() {
        let found = sections(
            "[quality_gate]\n\
             max_cyclomatic_complexity = 15\n\
             max_cognitive_complexity = 20\n\
             max_satd_comments = 0\n",
        );
        assert_eq!(
            found.len(),
            1,
            "a section pmat never reads must not be silently ignored: {found:?}"
        );
        assert_eq!(found[0].0, "quality_gate");
        assert_eq!(
            found[0].1.as_deref(),
            Some("quality"),
            "the message has to name the section that WOULD work, or the user \
             is told only that they are wrong: {found:?}"
        );
    }

    /// Counter-test one, and the load-bearing half: pmat's OWN schema must
    /// round-trip clean. Serialise the built-in default config to a
    /// `pmat.toml` and every section in it must be accepted.
    ///
    /// This is what stops the accepted set from being a second, drifting copy
    /// of the schema: add a field to `PmatConfig` and forget this function, and
    /// this test goes red rather than the field going silently unreadable.
    #[test]
    fn every_schema_section_is_accepted() {
        use crate::services::configuration_service::ConfigurationService;
        let cfg = ConfigurationService::default_config();
        let rendered =
            toml::to_string(&toml::Value::try_from(&cfg).expect("schema serialises to toml"))
                .expect("schema renders");
        let found = sections(&rendered);
        assert!(
            found.is_empty(),
            "pmat's own default config must not be reported as inapplicable — \
             the accepted set has drifted from the schema: {found:?}"
        );
    }

    /// Counter-test two: no config is not a broken config.
    #[test]
    fn an_absent_pmat_toml_is_not_a_finding() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        assert!(inapplicable_pmat_toml_sections(dir.path()).is_empty());
    }

    /// Counter-test three: a file that does not parse belongs to
    /// `unparsable_gate_configs`, which already blocks on it. Reporting it
    /// twice, once with a message that guesses at sections it could not read,
    /// would be noise.
    #[test]
    fn an_unparsable_pmat_toml_is_left_to_the_parse_guard() {
        assert!(sections("[quality\nmax_complexity = 5\n").is_empty());
    }

    /// Counter-test four: a correctly-written config is silent. Without this,
    /// the fix fails every repo that configured pmat properly, and a gate
    /// nobody can pass is a gate everybody bypasses.
    #[test]
    fn a_correctly_written_config_is_silent() {
        assert!(sections("[quality]\nmax_complexity = 15\nmax_cognitive_complexity = 20\n")
            .is_empty());
    }

    /// A nested table is part of its top-level section, not a section of its
    /// own: `[roadmap.git]` is read, and `[python.quality]` is reported once as
    /// `python` rather than twice.
    #[test]
    fn nested_tables_are_attributed_to_their_top_level_section() {
        let found = sections(
            "[roadmap.git]\ncreate_branches = false\n\
             [python]\nversion = \"3.12\"\n\
             [python.quality]\nallow_satd = false\n",
        );
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "python");
    }

    /// Every unknown section is reported, not just the first — a file with
    /// nineteen inert sections must say nineteen things, once each.
    #[test]
    fn every_unknown_section_is_reported_once() {
        let found = sections("[alpha]\na = 1\n[quality]\nmax_complexity = 5\n[beta]\nb = 2\n");
        assert_eq!(
            found.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "beta"],
            "{found:?}"
        );
    }
}
