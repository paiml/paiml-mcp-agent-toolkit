#![cfg_attr(coverage_nightly, coverage(off))]
//! Clippy integration, lint parsing, and diagnostic processing

use super::metrics::{build_lint_hotspot_result, calculate_defect_density};
use super::types::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Stdio;

// --- Extracted function bodies ---
include!("clippy_parsing.rs");
include!("clippy_file_analysis.rs");

/// Run clippy and analyze the JSON output.
///
/// Returns `Ok(None)` when clippy ran to completion and found nothing — a
/// measured clean result, distinct from "could not measure", which is an `Err`.
///
/// # Errors
///
/// Returns an error if the lint measurement could not be performed.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn run_clippy_analysis(
    project_path: &Path,
    clippy_flags: &str,
) -> Result<Option<LintHotspotResult>> {
    ensure_cargo_project(project_path)?;

    let flags: Vec<&str> = clippy_flags.split_whitespace().collect();
    let output = execute_clippy_command(project_path, &flags).await?;

    // Must come BEFORE any result is built: a run that never linted anything
    // must not be rendered as a clean project (#679).
    check_clippy_output(&output)?;

    let mut file_metrics = parse_clippy_json_output(&output)?;

    let workspace_root = find_workspace_root(project_path)?;
    calculate_sloc_for_files(&mut file_metrics, project_path, workspace_root.as_ref()).await?;

    build_lint_hotspot_result(file_metrics)
}

/// Fail fast when there is nothing for cargo to lint.
///
/// #679: `analyze lint-hotspot -p <dir with no Cargo.toml>` used to print
/// "project is clean" with rc=0 — including for an EMPTY directory. A directory
/// that cannot be linted has not been measured, so it gets an error instead.
fn ensure_cargo_project(project_path: &Path) -> Result<()> {
    if !project_path.exists() {
        anyhow::bail!("path does not exist: {}", project_path.display());
    }
    let mut current = Some(project_path);
    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists() {
            return Ok(());
        }
        current = dir.parent();
    }
    anyhow::bail!(
        "no Cargo.toml at or above {} — `analyze lint-hotspot` measures a Rust \
         crate with `cargo clippy` and cannot report on this path",
        project_path.display()
    )
}

/// Directories a cargo span path may be relative to, most specific first.
///
/// cargo does NOT emit span paths relative to the directory it was spawned in —
/// it emits them relative to the WORKSPACE ROOT. With
/// `-p <repo>/src --file cli/handlers/x.rs`, cargo reports `src/cli/handlers/x.rs`,
/// which resolved against `<repo>/src` gives `<repo>/src/src/cli/handlers/x.rs`
/// and matches nothing: the file was reported with 0 violations when the
/// project scan found 9 in it. Both candidates are tried, and each is compared
/// for path IDENTITY, so offering two bases cannot make an unrelated file match.
fn span_base_dirs(project_path: &Path) -> Result<Vec<PathBuf>> {
    let mut bases = Vec::new();
    if let Some(root) = find_workspace_root(project_path)? {
        bases.push(std::fs::canonicalize(&root).unwrap_or(root));
    }
    let here = std::fs::canonicalize(project_path).unwrap_or_else(|_| project_path.to_path_buf());
    if !bases.contains(&here) {
        bases.push(here);
    }
    Ok(bases)
}

/// Run clippy on a single file and analyze the JSON output
///
/// # Errors
///
/// Returns an error if the operation fails
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) async fn run_clippy_analysis_single_file(
    project_path: &Path,
    file_path: &Path,
    clippy_flags: &str,
) -> Result<LintHotspotResult> {
    ensure_cargo_project(project_path)?;

    // Resolve the target BEFORE spawning cargo so a typo'd `--file` fails with
    // "no such file" instead of being reported as a file with zero violations.
    let abs_file_path = resolve_absolute_path(project_path, file_path);
    if !abs_file_path.exists() {
        anyhow::bail!("--file path does not exist: {}", abs_file_path.display());
    }
    let abs_file_path = std::fs::canonicalize(&abs_file_path).unwrap_or(abs_file_path);
    let bases = span_base_dirs(project_path)?;

    let output = run_clippy_command(project_path, clippy_flags).await?;
    check_clippy_output(&output)?;

    let (file_violations, all_violations, severity_dist) =
        parse_clippy_output(&output.stdout, &abs_file_path, file_path, &bases)?;

    // #679: this used to be `.unwrap_or(100)`. An unreadable file produced a
    // FABRICATED SLOC of 100 and a defect density computed against it.
    let sloc = count_source_lines(project_path, file_path)
        .await
        .with_context(|| format!("could not read {} to count SLOC", abs_file_path.display()))?;

    create_single_file_result(
        file_path,
        file_violations,
        all_violations,
        severity_dist,
        sloc,
    )
}

#[cfg(test)]
mod lint_hotspot_measurement_tests {
    //! Regression tests for #679 — `analyze lint-hotspot` reported a FALSE
    //! CLEAN BILL OF HEALTH for every input. Each test here fails on the
    //! pre-fix code; the observed wrong value is named in the assertion.
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};

    /// One real `cargo clippy --message-format=json` record, verbatim from a
    /// fixture (`clippy::len_zero`). Note the populated `text` array — this is
    /// the shape that used to be undecodable.
    const REAL_DIAGNOSTIC: &str = r#"{"reason":"compiler-message","package_id":"path+file:///tmp/dirty#0.1.0","manifest_path":"/tmp/dirty/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"dirty","src_path":"/tmp/dirty/src/lib.rs","edition":"2021","doc":true,"doctest":true,"test":true},"message":{"rendered":"warning: length comparison to zero","$message_type":"diagnostic","children":[],"code":{"code":"clippy::len_zero","explanation":null},"level":"warning","message":"length comparison to zero","spans":[{"byte_end":420,"byte_start":408,"column_end":19,"column_start":5,"expansion":null,"file_name":"src/lib.rs","is_primary":true,"label":null,"line_end":19,"line_start":19,"suggested_replacement":"v.is_empty()","suggestion_applicability":"MachineApplicable","text":[{"highlight_end":19,"highlight_start":5,"text":"    v.len() == 0"}]}]}}"#;

    const BUILD_FINISHED: &str = r#"{"reason":"build-finished","success":true}"#;

    fn output_with(stdout: &str, code: i32) -> Output {
        Output {
            status: ExitStatus::from_raw(code << 8),
            stdout: stdout.as_bytes().to_vec(),
            stderr: Vec::new(),
        }
    }

    // ── #679 root cause 1: malformed cargo argv (missing `--`) ──────────────

    #[test]
    fn test_clippy_argv_puts_lint_flags_after_the_double_dash() {
        // PRE-FIX the project path produced
        //   ["clippy","--message-format=json","-W","warnings",...]
        // which cargo rejects with "unexpected argument '-W' found", exits 1,
        // emits nothing, and the caller rendered that as "project is clean".
        let argv = build_clippy_argv(&["-W", "warnings", "-W", "clippy::pedantic"]);
        let sep = argv
            .iter()
            .position(|a| a == "--")
            .expect("lint flags must be separated from cargo's own flags by `--`");
        let first_lint_flag = argv
            .iter()
            .position(|a| a == "-W")
            .expect("lint flag missing from argv");
        assert!(
            sep < first_lint_flag,
            "every -W flag must follow `--`; got {argv:?}"
        );
        assert_eq!(argv[0], "clippy");
        assert!(argv.contains(&"--message-format=json".to_string()));
    }

    #[test]
    fn test_clippy_argv_without_flags_has_no_dangling_separator() {
        let argv = build_clippy_argv(&[]);
        assert!(!argv.contains(&"--".to_string()), "got {argv:?}");
    }

    // ── #679 root cause 2: text-carrying diagnostics were undecodable ───────

    #[test]
    fn test_diagnostic_with_source_text_is_decoded() {
        // PRE-FIX: DiagnosticSpan wanted a `text` array of structs with fields
        // `_text`/`_highlight_start`/`_highlight_end`, which cargo never emits,
        // so this record failed to deserialize and was dropped in silence.
        let msg = decode_clippy_line(REAL_DIAGNOSTIC)
            .expect("a real cargo record must decode")
            .expect("not a blank line");
        let diagnostic = msg.message.expect("compiler-message carries a message");
        assert_eq!(diagnostic.level, "warning");
        assert_eq!(diagnostic.code.as_ref().unwrap().code, "clippy::len_zero");
        assert_eq!(diagnostic.spans[0].file_name, "src/lib.rs");
    }

    #[test]
    fn test_parse_counts_a_text_carrying_diagnostic() {
        // PRE-FIX this returned an EMPTY map: 0 violations for a stream that
        // plainly contains one.
        let stream = format!("{REAL_DIAGNOSTIC}\n{BUILD_FINISHED}\n");
        let metrics = parse_clippy_json_output(&output_with(&stream, 0)).unwrap();
        assert_eq!(metrics.len(), 1, "expected src/lib.rs in {metrics:?}");
        let m = metrics.get(&PathBuf::from("src/lib.rs")).unwrap();
        assert_eq!(m.severity_counts.warning, 1);
        assert_eq!(m.detailed_violations.len(), 1);
        assert_eq!(m.detailed_violations[0].lint_name, "clippy::len_zero");
    }

    #[test]
    fn test_undecodable_record_is_an_error_not_a_silent_drop() {
        // Silently skipping records is how the under-count hid for releases.
        let err = decode_clippy_line(r#"{"reason":"compiler-message","message":{"level":1}}"#)
            .expect_err("a malformed diagnostic must not be skipped");
        assert!(err.to_string().contains("under-report"), "{err}");
    }

    #[test]
    fn test_blank_lines_are_skipped_not_errors() {
        assert!(decode_clippy_line("").unwrap().is_none());
        assert!(decode_clippy_line("   ").unwrap().is_none());
    }

    // ── #679 root cause 3: a failed run was rendered as "clean" ─────────────

    #[test]
    fn test_check_clippy_output_errors_when_cargo_never_built() {
        // Exactly the observed failure: cargo rejected the argv, exit 1, empty
        // stdout. PRE-FIX this returned Ok(()) and the run reported 0 lint
        // hotspots with rc=0.
        let err = check_clippy_output(&output_with("", 1))
            .expect_err("a run with no build-finished record cannot be reported");
        assert!(err.to_string().contains("did NOT run"), "{err}");
    }

    #[test]
    fn test_check_clippy_output_accepts_a_finished_build() {
        let stream = format!("{REAL_DIAGNOSTIC}\n{BUILD_FINISHED}\n");
        assert!(check_clippy_output(&output_with(&stream, 101)).is_ok());
    }

    #[test]
    fn test_check_clippy_output_accepts_clean_finished_build() {
        assert!(check_clippy_output(&output_with(BUILD_FINISHED, 0)).is_ok());
    }

    #[test]
    fn test_ensure_cargo_project_rejects_a_dir_without_a_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let err = ensure_cargo_project(dir.path())
            .expect_err("an empty directory has no lint result to report");
        assert!(err.to_string().contains("no Cargo.toml"), "{err}");
    }

    #[test]
    fn test_ensure_cargo_project_rejects_a_missing_path() {
        let err = ensure_cargo_project(Path::new("/definitely/not/here/zz9")).unwrap_err();
        assert!(err.to_string().contains("does not exist"), "{err}");
    }

    #[test]
    fn test_ensure_cargo_project_accepts_a_crate() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        assert!(ensure_cargo_project(dir.path()).is_ok());
    }

    // ── `--all-targets` double counting ─────────────────────────────────────

    #[test]
    fn test_identical_findings_from_two_targets_count_once() {
        // `--all-targets` compiles src/lib.rs for both the lib and the test
        // target, so cargo emits this record twice. Counting both reported 40
        // violations for a fixture that has 20.
        let stream = format!("{REAL_DIAGNOSTIC}\n{REAL_DIAGNOSTIC}\n{BUILD_FINISHED}\n");
        let metrics = parse_clippy_json_output(&output_with(&stream, 0)).unwrap();
        let m = metrics.get(&PathBuf::from("src/lib.rs")).unwrap();
        assert_eq!(
            m.severity_counts.warning, 1,
            "duplicate target copy counted"
        );
        assert_eq!(m.detailed_violations.len(), 1);
    }

    // ── single-file path: absolute --file matched nothing ───────────────────

    #[test]
    fn test_single_file_matches_when_user_passes_an_absolute_path() {
        // PRE-FIX: cargo emits `src/lib.rs`; the user's absolute
        // `/tmp/dirty/src/lib.rs` matched none of the comparisons, so the file
        // was reported with total_violations 0.
        let base = Path::new("/tmp/dirty");
        let abs = Path::new("/tmp/dirty/src/lib.rs");
        // This is the pre-fix comparison, pinned as the bug it was: cargo's raw
        // relative span name against the user's absolute --file never matched.
        assert!(
            !is_target_file("src/lib.rs", abs, abs),
            "pre-fix behaviour changed; this test no longer pins the defect"
        );
        let resolved = resolve_diagnostic_path("src/lib.rs", base);
        assert!(
            is_target_file(&resolved, abs, abs),
            "resolved {resolved:?} should match {abs:?}"
        );
    }

    #[test]
    fn test_single_file_still_matches_a_relative_path() {
        let base = Path::new("/tmp/dirty");
        let rel = Path::new("src/lib.rs");
        let abs = Path::new("/tmp/dirty/src/lib.rs");
        let resolved = resolve_diagnostic_path("src/lib.rs", base);
        assert!(is_target_file(&resolved, abs, rel));
    }

    #[test]
    fn test_single_file_rejects_a_different_file() {
        let base = Path::new("/tmp/dirty");
        let abs = Path::new("/tmp/dirty/src/lib.rs");
        let resolved = resolve_diagnostic_path("src/other.rs", base);
        assert!(!is_target_file(&resolved, abs, Path::new("src/lib.rs")));
    }

    // ── #698: `--file <bare name>` matched files in other directories ───────

    #[test]
    fn test_bare_filename_does_not_claim_a_file_in_another_directory() {
        // PRE-FIX `is_target_file` ended with `diagnostic_path.ends_with(file_path)`,
        // and `Path::ends_with` compares trailing COMPONENTS. Observed on a
        // two-binary fixture (clean root `main.rs`, dirty `src/main.rs`):
        // `--file main.rs` reported total_violations 22 / sloc 3 /
        // defect_density 7.33 and exited 1, with every violation labelled
        // `"file": "main.rs"` but carrying `src/main.rs` line numbers.
        let base = Path::new("/tmp/fx2");
        let abs = Path::new("/tmp/fx2/main.rs");
        let rel = Path::new("main.rs");
        let resolved = resolve_diagnostic_path("src/main.rs", base);
        assert!(
            !is_target_file(&resolved, abs, rel),
            "a diagnostic in {resolved:?} must not be attributed to {abs:?}"
        );
    }

    #[test]
    fn test_workspace_member_lib_does_not_claim_the_root_lib() {
        // Same defect, the shape that bites every cargo workspace:
        // `--file src/lib.rs` swallowed `crates/<member>/src/lib.rs` too, so
        // one file's report carried several crates' violations.
        let base = Path::new("/repo");
        let abs = Path::new("/repo/src/lib.rs");
        let rel = Path::new("src/lib.rs");
        let resolved = resolve_diagnostic_path("crates/dashboard/src/lib.rs", base);
        assert!(
            !is_target_file(&resolved, abs, rel),
            "a diagnostic in {resolved:?} must not be attributed to {abs:?}"
        );
    }

    #[test]
    fn test_bare_filename_still_matches_its_own_file() {
        // The tightening must not cost the legitimate match.
        let base = Path::new("/tmp/fx2");
        let abs = Path::new("/tmp/fx2/main.rs");
        let rel = Path::new("main.rs");
        let resolved = resolve_diagnostic_path("main.rs", base);
        assert!(is_target_file(&resolved, abs, rel));
    }

    // ── #698 follow-on: spans are relative to the WORKSPACE ROOT, not to -p ──

    #[test]
    fn test_span_matches_when_project_path_is_below_the_workspace_root() {
        // Measured on this repo: `-p <repo>/src --file cli/handlers/.../
        // platform_routes.rs` reported total_violations 0 while the project
        // scan reported 9 in that same file, because cargo emits
        // `src/cli/handlers/.../platform_routes.rs` (workspace-root relative)
        // and the only base tried was `<repo>/src`, giving `<repo>/src/src/...`.
        // The loose `ends_with` used to paper over this for RELATIVE --file
        // values only; with an absolute --file it was wrong even before #698.
        // A unique dir per run: a fixed /tmp name collides when two `cargo
        // test` processes overlap.
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let sub = root.join("src");
        let target = sub.join("thing.rs");
        std::fs::create_dir_all(&sub).expect("mkdir fixture");
        std::fs::write(&target, "// fixture\n").expect("write fixture");

        let abs = std::fs::canonicalize(&target).expect("canonicalize target");
        let bases = vec![
            std::fs::canonicalize(root).expect("canonicalize root"),
            std::fs::canonicalize(&sub).expect("canonicalize sub"),
        ];

        assert!(
            span_matches_target("src/thing.rs", &bases, &abs, &abs),
            "a workspace-root-relative span must resolve to {abs:?}"
        );
        // Only the workspace-root base can match; the project-path base alone
        // is what produced the 0-violation report.
        assert!(
            !span_matches_target("src/thing.rs", &bases[1..], &abs, &abs),
            "this pins the wrong-base behaviour that caused the under-report"
        );
        // And identity is still required: another file is still rejected.
        assert!(!span_matches_target("src/other.rs", &bases, &abs, &abs));
    }

    #[test]
    fn test_span_base_dirs_lists_workspace_root_before_project_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let sub = root.join("crates").join("member");
        std::fs::create_dir_all(&sub).expect("mkdir fixture");
        std::fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n")
            .expect("write workspace manifest");
        std::fs::write(sub.join("Cargo.toml"), "[package]\nname = \"member\"\n")
            .expect("write member manifest");

        let bases = span_base_dirs(&sub).expect("bases");
        assert_eq!(bases.len(), 2, "got {bases:?}");
        assert_eq!(bases[0], std::fs::canonicalize(root).expect("canon root"));
        assert_eq!(bases[1], std::fs::canonicalize(&sub).expect("canon sub"));

        // A crate that is its own workspace root yields exactly one base, not
        // the same directory twice.
        let solo = span_base_dirs(root).expect("bases");
        assert_eq!(solo.len(), 1, "got {solo:?}");
    }

    // ── determinism ─────────────────────────────────────────────────────────

    #[test]
    fn test_parse_is_byte_identical_across_repeated_runs() {
        // DETERMINISM: >= 5 iterations, same input, compare serialized output.
        let stream = format!(
            "{}\n{}\n{}\n",
            REAL_DIAGNOSTIC,
            REAL_DIAGNOSTIC.replace("src/lib.rs", "src/other.rs"),
            BUILD_FINISHED
        );
        let mut renders = Vec::new();
        for _ in 0..8 {
            let mut metrics = parse_clippy_json_output(&output_with(&stream, 0)).unwrap();
            for m in metrics.values_mut() {
                m.sloc = 100;
            }
            let result = build_lint_hotspot_result(metrics)
                .unwrap()
                .expect("two findings must produce a hotspot");
            renders.push(serde_json::to_string(&result).unwrap());
        }
        if let Some(i) = (1..renders.len()).find(|&i| renders[i] != renders[i - 1]) {
            panic!(
                "lint-hotspot rendering is not deterministic (run {} != run {}):\n{}\n---\n{}",
                i - 1,
                i,
                renders[i - 1],
                renders[i]
            );
        }
    }

    // ── output must not depend on the process's cwd ──────────────────────────

    #[test]
    fn test_sloc_path_resolution_ignores_the_process_cwd() {
        // PRE-FIX `resolve_file_path` began with `if file_path.exists()`, which
        // resolves cargo's relative `src/lib.rs` against the CALLER's directory.
        // Observed: analysing a 23-line fixture from inside the pmat repo
        // measured pmat's own 414-line src/lib.rs.
        // `cargo test` runs with the crate root as cwd, so the decoy is pmat's
        // own src/lib.rs — no set_current_dir (and no cross-test race) needed.
        assert!(
            Path::new("src/lib.rs").exists(),
            "test needs a cwd-relative decoy at src/lib.rs to be meaningful"
        );

        let project = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(project.path().join("src")).unwrap();
        std::fs::write(project.path().join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(project.path().join("src/lib.rs"), "// real\n").unwrap();

        let resolved = resolve_file_path(
            Path::new("src/lib.rs"),
            project.path(),
            None,
            find_manifest_dir(project.path()).as_deref(),
        );

        assert_eq!(
            std::fs::read_to_string(&resolved).unwrap(),
            "// real\n",
            "resolved {resolved:?} — must come from the analysed project, not the cwd"
        );
    }

    #[test]
    fn test_no_violations_is_a_measured_none_not_an_error() {
        // PRE-FIX this was `Err("No lint violations found in any Rust files")`,
        // which callers detected by SUBSTRING and turned into empty stdout.
        assert!(build_lint_hotspot_result(HashMap::new()).unwrap().is_none());
    }
}
