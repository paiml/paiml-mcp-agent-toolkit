//! AD-05: the three threshold checks, each with the control that must pass.
//!
//! Every leg here is the pair the spec asks for — a planted breach that FAILS
//! and a fixture just under the line that PASSES — because a check that fires on
//! everything is as useless as one that fires on nothing.

use super::*;

/// A file of exactly `lines` lines, newline-terminated (so `wc -l` agrees).
fn write_lines(path: &Path, lines: usize) {
    let mut body = String::new();
    for i in 0..lines {
        body.push_str(&format!("pub const V{i}: u32 = {i};\n"));
    }
    std::fs::write(path, body).expect("write fixture");
}

fn fixture_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    dir
}

// ── file-size ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_file_over_the_cap_is_a_violation_naming_its_length() {
    let dir = fixture_dir();
    write_lines(&dir.path().join("src/big.rs"), 501);

    let violations = check_file_size(dir.path(), 500).await.expect("reports");

    assert_eq!(violations.len(), 1, "one file, one finding: {violations:?}");
    let v = &violations[0];
    assert_eq!(v.check_type, "file-size");
    assert_eq!(v.severity, "error", "a breached threshold blocks");
    assert!(v.file.contains("big.rs"), "the file is named: {v:?}");
    assert!(
        v.message.contains("501") && v.message.contains("500"),
        "the count AND the threshold are stated, so the rule is visible: {}",
        v.message
    );
}

/// CONTROL: one line under the cap must pass. Without it, "flags a 501-line
/// file" and "flags every file" are the same evidence.
#[tokio::test]
async fn a_file_under_the_cap_is_not_a_violation() {
    let dir = fixture_dir();
    write_lines(&dir.path().join("src/big.rs"), 499);

    let violations = check_file_size(dir.path(), 500).await.expect("reports");
    assert!(violations.is_empty(), "499 <= 500: {violations:?}");
}

/// The boundary itself: `> max`, not `>=`. A 500-line file at a 500-line cap is
/// AT the limit, not over it.
#[tokio::test]
async fn a_file_exactly_at_the_cap_passes() {
    let dir = fixture_dir();
    write_lines(&dir.path().join("src/exact.rs"), 500);

    let violations = check_file_size(dir.path(), 500).await.expect("reports");
    assert!(violations.is_empty(), "500 is not over 500: {violations:?}");
}

/// The threshold is a parameter, not a constant: the same tree fails at 400 and
/// passes at 500.
#[tokio::test]
async fn the_same_tree_changes_verdict_with_the_threshold() {
    let dir = fixture_dir();
    write_lines(&dir.path().join("src/mid.rs"), 499);

    assert!(check_file_size(dir.path(), 500)
        .await
        .expect("reports")
        .is_empty());
    assert_eq!(
        check_file_size(dir.path(), 400)
            .await
            .expect("reports")
            .len(),
        1,
        "the flag has to be able to change the answer"
    );
}

// ── churn ────────────────────────────────────────────────────────────────────

/// `git log --format=%H --name-only` for two commits, the second touching two
/// files. Parsing is tested without a subprocess so this leg cannot go quiet
/// when git is unavailable.
#[test]
fn the_churn_log_parser_counts_commits_per_path() {
    let log = "\
1111111111111111111111111111111111111111

src/a.rs

2222222222222222222222222222222222222222

src/a.rs
src/b.rs
";
    let counts = parse_churn_log(log);
    assert_eq!(counts.get("src/a.rs"), Some(&2));
    assert_eq!(counts.get("src/b.rs"), Some(&1));
    assert_eq!(counts.len(), 2, "no commit hash was counted as a path");
}

/// A rename shows both sides inside ONE commit; the question is how many
/// commits touched the file, so it still weighs once.
#[test]
fn a_path_named_twice_in_one_commit_counts_once() {
    let log = "\
1111111111111111111111111111111111111111

src/a.rs
src/a.rs
";
    assert_eq!(parse_churn_log(log).get("src/a.rs"), Some(&1));
}

fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .expect("git must be available: the churn check reads a repository");
    assert!(
        status.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&status.stderr)
    );
}

/// A repository whose `src/hot.rs` is touched by `commits` commits, all today.
fn churn_repo(commits: usize) -> tempfile::TempDir {
    let dir = fixture_dir();
    // `--template=` keeps a user's global hook/template directory out of the
    // fixture; `core.hooksPath=/dev/null` keeps this machine's hooks out of it.
    git(dir.path(), &["init", "-q", "--template="]);
    git(dir.path(), &["config", "core.hooksPath", "/dev/null"]);
    for i in 0..commits {
        std::fs::write(
            dir.path().join("src/hot.rs"),
            format!("pub const REV: u32 = {i};\n"),
        )
        .expect("write fixture");
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-q", "-m", &format!("rev {i}")]);
    }
    dir
}

#[tokio::test]
async fn a_file_over_the_churn_cap_is_a_violation_naming_its_commit_count() {
    let dir = churn_repo(7);

    let violations = check_churn(dir.path(), 5).await.expect("reports");

    assert_eq!(violations.len(), 1, "one hot file: {violations:?}");
    let v = &violations[0];
    assert_eq!(v.check_type, "churn");
    assert_eq!(v.severity, "error");
    assert!(v.file.contains("hot.rs"), "{v:?}");
    assert!(
        v.message.contains('7') && v.message.contains('5'),
        "the count AND the threshold are stated: {}",
        v.message
    );
}

/// CONTROL: the same repository, a cap above its churn, passes.
#[tokio::test]
async fn the_same_repository_passes_a_higher_churn_cap() {
    let dir = churn_repo(7);
    let violations = check_churn(dir.path(), 10).await.expect("reports");
    assert!(violations.is_empty(), "7 <= 10: {violations:?}");
}

/// A directory with no history is DISCLOSED, never counted as quiet.
///
/// "no commits" and "no repository" produce the same empty finding list, and
/// only one of them is a statement about the tree.
#[tokio::test]
async fn a_directory_that_is_not_a_repository_is_disclosed_not_passed() {
    let dir = fixture_dir();
    write_lines(&dir.path().join("src/lib.rs"), 3);

    let violations = check_churn(dir.path(), 5).await.expect("reports");

    assert_eq!(violations.len(), 1, "the disclosure row: {violations:?}");
    let row = &violations[0];
    assert_eq!(row.check_type, "scope", "not a churn finding");
    assert_eq!(
        row.severity, ADVISORY_SEVERITY,
        "a limit of this run must not fail a user's gate"
    );
    assert!(
        row.message.contains("NOT measured"),
        "the row must say which of the two zeros this is: {}",
        row.message
    );
    assert_eq!(
        blocking_violation_count(&violations),
        0,
        "advisory, so the verdict is unmoved: {violations:?}"
    );
}

// ── lint ─────────────────────────────────────────────────────────────────────

/// The lint gate must ask clippy the SAME question `pmat verify` asks.
///
/// Value-level, not a comment: two constants that merely look alike is the
/// defect this repository keeps finding, so the check reads verify's own.
#[test]
fn the_lint_check_asks_verifys_clippy_question() {
    assert_eq!(crate::cli::verify::CLIPPY_TARGETS, "--all-targets");
    assert_eq!(
        crate::cli::verify::CLIPPY_LINTS,
        &["-D", "warnings", "-A", "unused-variables"]
    );
    // …and the check passes those constants rather than its own copy.
    let source = include_str!("quality_checks_part1_thresholds.rs");
    assert!(
        source.contains("crate::cli::verify::CLIPPY_TARGETS")
            && source.contains("crate::cli::verify::CLIPPY_LINTS"),
        "the lint check must reuse verify's flags, not restate them"
    );
    assert!(
        !source.contains("\"-D\", \"warnings\""),
        "a second copy of the lint flags is how the two surfaces come to disagree"
    );
}

/// A tree with nothing for clippy to read is DISCLOSED, not passed. No
/// subprocess: the check returns before spawning one.
#[tokio::test]
async fn a_tree_without_a_cargo_toml_is_disclosed_not_passed() {
    let dir = fixture_dir();
    write_lines(&dir.path().join("src/lib.rs"), 3);

    let violations = check_lint(dir.path()).await.expect("reports");

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].check_type, "scope");
    assert_eq!(violations[0].severity, ADVISORY_SEVERITY);
    assert!(
        violations[0].message.contains("no Cargo.toml"),
        "and says why: {}",
        violations[0].message
    );
    assert_eq!(blocking_violation_count(&violations), 0);
}

#[test]
fn the_first_clippy_diagnostic_carries_its_file_and_line() {
    let stream = concat!(
        r#"{"reason":"compiler-artifact","package_id":"fx"}"#,
        "\n",
        r#"{"reason":"compiler-message","message":{"code":{"code":"clippy::needless_return"},"#,
        r#""level":"error","message":"unneeded `return` statement","spans":[{"is_primary":true,"#,
        r#""file_name":"src/lib.rs","line_start":3}]}}"#,
        "\n",
        r#"{"reason":"compiler-message","message":{"code":null,"level":"error","#,
        r#""message":"aborting due to 1 previous error","spans":[]}}"#,
        "\n",
    );
    let (file, line, detail) = first_clippy_diagnostic(stream).expect("a diagnostic is found");
    assert_eq!(file, "src/lib.rs");
    assert_eq!(line, Some(3));
    assert!(detail.contains("needless_return"), "{detail}");
    assert!(
        detail.contains("unneeded `return`"),
        "the rule AND its message: {detail}"
    );
}

/// "aborting due to N previous errors" is a COUNT, not a cause, and must never
/// become the reported finding.
#[test]
fn the_abort_summary_is_never_the_reported_diagnostic() {
    let stream = concat!(
        r#"{"reason":"compiler-message","message":{"code":null,"level":"error","#,
        r#""message":"aborting due to 2 previous errors","spans":[]}}"#,
        "\n",
    );
    assert!(first_clippy_diagnostic(stream).is_none());
}

/// A minimal crate in `dir`, whose `src/lib.rs` is `body`.
fn cargo_fixture(dir: &Path, body: &str) {
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"fx\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    std::fs::write(dir.join("src/lib.rs"), body).expect("write lib.rs");
}

/// The real thing, both ways round: one clippy warning fails, and the same
/// crate with the warning fixed passes.
///
/// Not `#[ignore]`d. It compiles a three-line, dependency-free crate twice, and
/// an ignored test is a silently unmeasured one — this is the only leg that
/// proves the check actually runs clippy rather than merely composing its
/// arguments.
#[tokio::test]
async fn one_clippy_warning_fails_and_the_fixed_crate_passes() {
    let dir = fixture_dir();
    cargo_fixture(
        dir.path(),
        "pub fn one() -> u32 {\n    let x = 1;\n    return x;\n}\n",
    );

    let violations = check_lint(dir.path()).await.expect("reports");
    assert_eq!(
        violations.len(),
        1,
        "one finding, not one per line: {violations:?}"
    );
    assert_eq!(violations[0].check_type, "lint");
    assert_eq!(violations[0].severity, "error");
    assert!(
        violations[0].message.contains("needless_return"),
        "the finding carries clippy's own diagnostic: {}",
        violations[0].message
    );

    // CONTROL: fix the warning and the same check must go quiet. Without this,
    // "fails a warning" is indistinguishable from "always fails".
    cargo_fixture(dir.path(), "pub fn one() -> u32 {\n    1\n}\n");
    let clean = check_lint(dir.path()).await.expect("reports");
    assert!(clean.is_empty(), "a clean crate lints clean: {clean:?}");
}

// ── threshold resolution ─────────────────────────────────────────────────────

#[test]
fn the_shipped_defaults_are_the_config_schemas_defaults() {
    let t = QualityThresholds::default();
    assert_eq!(t.max_file_lines, 500, "the number `pmat work` already uses");
    assert_eq!(t.max_churn_commits_90d, 20);
    let quality = crate::services::configuration_service::QualityConfig::default();
    assert_eq!(t.max_file_lines, quality.max_file_lines);
    assert_eq!(t.max_churn_commits_90d, quality.max_churn_commits_90d);
}

/// Both keys must be SCHEMA keys of `[quality]`.
///
/// `pmat config --validate` and the gate's own `inapplicable_pmat_toml_sections`
/// derive what a project may write from `PmatConfig`; a threshold key missing
/// from that derivation is reported to the user as read by nothing, which is
/// exactly the "configured limit that had no effect" defect those checks exist
/// to catch.
#[test]
fn both_threshold_keys_are_part_of_the_declared_pmat_toml_schema() {
    let schema = crate::services::configuration_service::schema_pmat_toml_keys();
    let quality = schema
        .get("quality")
        .expect("pmat.toml declares a [quality] section");
    assert!(
        quality.contains("max_file_lines"),
        "a project writing max_file_lines must not be told nothing reads it: {quality:?}"
    );
    assert!(
        quality.contains("max_churn_commits_90d"),
        "…and the same for the churn cap: {quality:?}"
    );
}

#[test]
fn pmat_toml_overrides_the_defaults_and_the_cli_overrides_pmat_toml() {
    let dir = fixture_dir();
    std::fs::write(
        dir.path().join("pmat.toml"),
        "[quality]\nmax_file_lines = 120\nmax_churn_commits_90d = 3\n",
    )
    .expect("write pmat.toml");

    let from_config = QualityThresholds::resolve(dir.path(), None, None);
    assert_eq!(from_config.max_file_lines, 120);
    assert_eq!(from_config.max_churn_commits_90d, 3);

    // #683's rule: a number the user typed must not be silently replaced.
    let from_cli = QualityThresholds::resolve(dir.path(), Some(400), Some(5));
    assert_eq!(from_cli.max_file_lines, 400);
    assert_eq!(from_cli.max_churn_commits_90d, 5);
}

/// The CLI flag is `--max-churn-commits`, the schema key carries the window.
/// Both spellings resolve, so a project that wrote the flag's name into
/// `pmat.toml` is not silently running at 20.
#[test]
fn the_churn_key_accepts_the_flags_spelling_as_an_alias() {
    let dir = fixture_dir();
    std::fs::write(
        dir.path().join("pmat.toml"),
        "[quality]\nmax_churn_commits = 7\n",
    )
    .expect("write pmat.toml");
    assert_eq!(
        QualityThresholds::resolve(dir.path(), None, None).max_churn_commits_90d,
        7
    );
}

/// A `pmat.toml` that does not parse falls back to the defaults here rather
/// than erroring: `handle_project_quality_gate` already BLOCKS on that file
/// (`unparsable_gate_configs`), so this reader must not double-report it.
#[test]
fn an_unparsable_config_falls_back_without_a_second_complaint() {
    let dir = fixture_dir();
    std::fs::write(dir.path().join("pmat.toml"), "[quality\nbroken = ").expect("write pmat.toml");
    assert_eq!(
        QualityThresholds::resolve(dir.path(), None, None),
        QualityThresholds::default()
    );
}

// ── the flags reach the gate, on BOTH dispatch routes ────────────────────────

/// `pmat quality-gate` has TWO dispatch routes, and both files say in their own
/// comments that wiring one leaves the other silent. These pin the wiring at
/// each route.
///
/// Source-level, for the reason `test_both_report_dispatchers_route_through_execute_report_command`
/// gives about its own route pin: there is no observable difference at the CLI
/// between "forwarded the threshold" and "dropped it and used the default" —
/// the defect IS that the tested path is not the one that runs. The behaviour
/// itself is pinned by the clap and resolution tests above and by
/// `scripts/quality-gate-thresholds-audit.sh` end to end.
#[test]
fn both_dispatch_routes_resolve_and_forward_the_thresholds() {
    for (name, src) in [
        (
            "command_dispatcher_scoring.rs",
            include_str!("../command_dispatcher/command_dispatcher_scoring.rs"),
        ),
        (
            "dispatch_ext_scoring.rs",
            include_str!("../command_structure/executor/dispatch_ext_scoring.rs"),
        ),
    ] {
        assert!(
            src.contains("max_file_lines") && src.contains("max_churn_commits"),
            "{name} must destructure both new flags off Commands::QualityGate"
        );
        assert!(
            src.contains("QualityThresholds::resolve("),
            "{name} must RESOLVE the thresholds (CLI over pmat.toml over default), \
             not pass the raw flags or a default"
        );
        assert!(
            src.contains("thresholds,"),
            "{name} must forward the resolved thresholds to the gate"
        );
        assert!(
            !src.contains("analysis_utilities::handle_quality_gate("),
            "{name} still calls the defaulted 11-argument entry point, so the \
             thresholds it resolved never reach the checks"
        );
    }
}

/// Clap accepts both flags and carries their values — the parser is asked, not
/// a constant that merely looks like its defaults.
#[test]
fn clap_parses_both_threshold_flags_and_defaults_them_to_unset() {
    // `try_parse_from` needs the 8MB stack (`on_big_stack`); a bare
    // `cargo test --lib` otherwise aborts the whole binary.
    let (bare, typed) = crate::cli::commands::on_big_stack(|| {
        use clap::Parser;
        let read = |argv: &[&str]| {
            let cli = crate::cli::Cli::try_parse_from(argv).expect("argv must parse");
            let crate::cli::Commands::QualityGate {
                max_file_lines,
                max_churn_commits,
                ..
            } = cli.command
            else {
                unreachable!("{argv:?} must parse as QualityGate");
            };
            (max_file_lines, max_churn_commits)
        };
        (
            read(&["pmat", "quality-gate"]),
            read(&[
                "pmat",
                "quality-gate",
                "--max-file-lines",
                "400",
                "--max-churn-commits",
                "5",
            ]),
        )
    });

    assert_eq!(
        bare,
        (None, None),
        "unset must be distinguishable from a typed value, or pmat.toml can \
         never outrank the flag's default (#683)"
    );
    assert_eq!(typed, (Some(400), Some(5)));

    // …and the values a user typed are the ones the checks are given.
    let dir = fixture_dir();
    let resolved = QualityThresholds::resolve(dir.path(), typed.0, typed.1);
    assert_eq!(resolved.max_file_lines, 400);
    assert_eq!(resolved.max_churn_commits_90d, 5);
}

/// `--checks file-size,churn,lint` must parse to the three new variants, and
/// `--checks all` must run file-size and churn without running lint.
#[test]
fn the_three_new_checks_are_selectable_and_two_of_them_are_in_the_suite() {
    let checks = crate::cli::commands::on_big_stack(|| {
        use clap::Parser;
        let cli = crate::cli::Cli::try_parse_from([
            "pmat",
            "quality-gate",
            "--checks",
            "file-size,churn,lint",
        ])
        .expect("argv must parse");
        let crate::cli::Commands::QualityGate { checks, .. } = cli.command else {
            unreachable!("must parse as QualityGate");
        };
        checks
    });
    assert_eq!(
        checks,
        vec![
            QualityCheckType::FileSize,
            QualityCheckType::Churn,
            QualityCheckType::Lint
        ]
    );

    let suite = QualityCheckType::default_checks();
    assert!(suite.contains(&QualityCheckType::FileSize));
    assert!(suite.contains(&QualityCheckType::Churn));
    // Deliberate: clippy costs a full compile of the analysed tree, so `lint`
    // is opt-in. `default_checks()` is what the MCP suite ADVERTISES as run, so
    // it must not name a check `run_all_project_checks` does not run.
    assert!(
        !suite.contains(&QualityCheckType::Lint),
        "lint is opt-in; if that changes, run_all_project_checks must run it too"
    );
    let all_checks_source = include_str!("quality_gate_part2a.rs");
    assert!(
        all_checks_source.contains("check_file_size(project_path")
            && all_checks_source.contains("check_churn(project_path"),
        "the two suite members must actually run in run_all_project_checks"
    );
    assert!(
        !all_checks_source.contains("check_lint("),
        "…and the opt-in one must not"
    );
}

/// Caught by the AD-04 quorum on the AD-05 PR: the MCP suite passed
/// `QualityThresholds::default()` while the CLI resolved pmat.toml, so a project's
/// `[quality] max_file_lines` applied over one transport and not the other. The
/// suite must resolve the same way the CLI does.
#[test]
fn the_mcp_suite_resolves_thresholds_from_pmat_toml_like_the_cli() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let d = tmp.path();
    std::fs::write(
        d.join("pmat.toml"),
        "[quality]\nmax_file_lines = 10\nmax_churn_commits_90d = 3\n",
    )
    .expect("write");
    let resolved = QualityThresholds::resolve(d, None, None);
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let mut violations = Vec::new();
    let mut results = crate::cli::analysis_utilities::QualityGateResults::default();
    // 20 lines > 10: the resolved thresholds must produce the violation the defaults would not
    std::fs::create_dir_all(d.join("src")).expect("mkdir");
    std::fs::write(d.join("src/big.rs"), "// x\n".repeat(20)).expect("write");
    rt.block_on(super::run_all_project_checks(
        d,
        0.1,
        None,
        20,
        &mut violations,
        &mut results,
        false,
        resolved,
    ))
    .expect("checks");
    assert_eq!(
        resolved.max_file_lines, 10,
        "pmat.toml must win over the shipped default"
    );
    assert!(
        results.file_size_violations >= 1,
        "a 20-line file must fail max_file_lines = 10 through the resolved thresholds: {results:?}"
    );
    let mut v2 = Vec::new();
    let mut r2 = crate::cli::analysis_utilities::QualityGateResults::default();
    rt.block_on(super::run_all_project_checks(
        d,
        0.1,
        None,
        20,
        &mut v2,
        &mut r2,
        false,
        QualityThresholds::default(),
    ))
    .expect("checks");
    assert_eq!(
        r2.file_size_violations, 0,
        "the control: the shipped default (500) passes the same file"
    );
}
