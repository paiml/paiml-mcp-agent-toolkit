// CB-081 Dependency Count + CB-400/401/402 bashrs integration tests
// Included from tests.rs via include!() - shares parent module scope

#[test]
fn test_cb081_detects_excessive_direct_deps() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml with many dependencies (>50)
    let mut deps =
        String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
    for i in 0..60 {
        deps.push_str(&format!("dep{} = \"1.0\"\n", i));
    }
    fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();
    fs::write(
        temp.path().join("Cargo.lock"),
        "[[package]]\nname = \"test\"",
    )
    .unwrap();

    let report = detect_cb081_dependency_count(temp.path());
    assert_eq!(report.direct_count, 60);
    assert_eq!(report.score, 0); // >50 direct = score 0
    assert!(!report.violations.is_empty());
    assert_eq!(report.violations[0].pattern_id, "CB-081-A");
}

#[test]
fn test_cb081_moderate_deps() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml with moderate dependencies (30-40)
    let mut deps =
        String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
    for i in 0..35 {
        deps.push_str(&format!("dep{} = \"1.0\"\n", i));
    }
    fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();

    // Create Cargo.lock with 180 packages (between 150-200)
    let mut lock = String::new();
    for _ in 0..180 {
        lock.push_str("[[package]]\nname = \"pkg\"\n");
    }
    fs::write(temp.path().join("Cargo.lock"), &lock).unwrap();

    let report = detect_cb081_dependency_count(temp.path());
    assert_eq!(report.direct_count, 35);
    assert_eq!(report.transitive_count, 180);
    assert_eq!(report.score, 3); // 30-40 direct, 150-200 transitive = 3
}

#[test]
fn test_cb081_low_deps_excellent() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml with few dependencies (<=20)
    let mut deps =
        String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
    for i in 0..15 {
        deps.push_str(&format!("dep{} = \"1.0\"\n", i));
    }
    fs::write(temp.path().join("Cargo.toml"), &deps).unwrap();

    // Create Cargo.lock with few packages (<=100)
    let mut lock = String::new();
    for _ in 0..80 {
        lock.push_str("[[package]]\nname = \"pkg\"\n");
    }
    fs::write(temp.path().join("Cargo.lock"), &lock).unwrap();

    let report = detect_cb081_dependency_count(temp.path());
    assert_eq!(report.direct_count, 15);
    assert_eq!(report.transitive_count, 80);
    assert_eq!(report.score, 5); // <=20 direct, <=100 transitive = 5
    assert!(report.violations.is_empty());
}

#[test]
fn test_cb081_excludes_dev_dependencies() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml with few regular deps but many dev-deps
    let deps = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
anyhow = "1.0"

[dev-dependencies]
criterion = "0.5"
tempfile = "3.0"
proptest = "1.0"
quickcheck = "1.0"
tokio-test = "0.4"
"#;
    fs::write(temp.path().join("Cargo.toml"), deps).unwrap();
    fs::write(
        temp.path().join("Cargo.lock"),
        "[[package]]\nname = \"test\"",
    )
    .unwrap();

    let report = detect_cb081_dependency_count(temp.path());
    // Only counts [dependencies], not [dev-dependencies]
    assert_eq!(report.direct_count, 2);
}

#[test]
fn test_cb081_no_cargo_toml() {
    let temp = TempDir::new().unwrap();
    // No Cargo.toml

    let report = detect_cb081_dependency_count(temp.path());
    assert_eq!(report.direct_count, 0);
    assert_eq!(report.transitive_count, 0);
}

/// Every path under `root`, relative and sorted — a fingerprint of the tree.
fn tree_fingerprint(root: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.clone());
            }
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.display().to_string());
            }
        }
    }
    out.sort();
    out
}

/// #939: an auditor must not write to the tree it audits.
///
/// CB-081 wrote `.pmat/deps-cache.json` and `.pmat/metrics/dependencies.json`
/// into the project as a side effect of scoring it, so a second `comply check`
/// on an untouched repo saw a different repo: `CB-1332: Cache Staleness` went
/// Skip -> Pass and the pass count moved 25 -> 26 with no edit in between. On a
/// project with no `.pmat/` at all, being scored created one.
#[test]
fn cb081_does_not_write_into_the_audited_project() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1.0\"\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("Cargo.lock"),
        "[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let before = tree_fingerprint(temp.path());
    let lock_before = fs::read(temp.path().join("Cargo.lock")).unwrap();
    let first = detect_cb081_dependency_count(temp.path());
    let after = tree_fingerprint(temp.path());

    assert_eq!(
        before, after,
        "scoring a project must leave it byte-for-byte alone"
    );
    // `cargo tree` without --locked resolves and rewrites the lockfile: this
    // fixture's 45-byte Cargo.lock came back at 1,790 bytes, fetched from the
    // network, purely because the project was scored.
    assert_eq!(
        fs::read(temp.path().join("Cargo.lock")).unwrap(),
        lock_before,
        "CB-081 rewrote the audited project's Cargo.lock"
    );
    assert!(
        !temp.path().join(".pmat").exists(),
        "CB-081 created a .pmat/ directory in the audited project"
    );

    // …and the second run answers the same, from the out-of-project cache.
    let second = detect_cb081_dependency_count(temp.path());
    assert_eq!(first.direct_count, second.direct_count);
    assert_eq!(first.transitive_count, second.transitive_count);
    assert_eq!(tree_fingerprint(temp.path()), before);
}

// =========================================================================
// CB-400/401/402 bashrs integration tests
// =========================================================================

#[test]
fn test_cb400_no_git_hooks_dir() {
    let temp = TempDir::new().unwrap();
    // No .git/hooks directory
    let violations = detect_cb400_git_hooks_quality(temp.path());
    assert!(violations.is_empty(), "No hooks dir should return empty");
}

#[test]
fn test_cb400_empty_git_hooks_dir() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".git/hooks")).unwrap();
    // Empty hooks dir - no hook files
    let violations = detect_cb400_git_hooks_quality(temp.path());
    assert!(
        violations.iter().all(|v| v.severity == Severity::Info),
        "Empty hooks dir must produce no scored violation"
    );
    // ...but it must not be SILENT about having linted nothing (#1020).
    let scope = violations
        .iter()
        .find(|v| v.pattern_id == "CB-400-SCOPE")
        .expect("an empty hooks dir must be disclosed, not rendered as a pass");
    assert!(
        scope.description.contains("no hook was linted"),
        "the scope row must say nothing was linted: {}",
        scope.description
    );
}

#[test]
fn test_cb400_sample_hooks_ignored() {
    let temp = TempDir::new().unwrap();
    let hooks_dir = temp.path().join(".git/hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    // Sample hooks should be ignored
    fs::write(
        hooks_dir.join("pre-commit.sample"),
        "#!/bin/bash\necho test",
    )
    .unwrap();
    let violations = detect_cb400_git_hooks_quality(temp.path());
    assert!(
        violations.iter().all(|v| v.severity == Severity::Info),
        "Sample hooks should be ignored"
    );
}

#[test]
fn test_cb401_no_makefile() {
    let temp = TempDir::new().unwrap();
    // No Makefile
    let violations = detect_cb401_makefile_quality(temp.path());
    assert!(violations.is_empty(), "No Makefile should return empty");
}

#[test]
fn test_cb402_no_shell_scripts() {
    let temp = TempDir::new().unwrap();
    // No shell scripts
    let violations = detect_cb402_shell_script_quality(temp.path());
    assert!(
        violations.is_empty(),
        "No shell scripts should return empty"
    );
}

/// In a git repository the scan is the set of TRACKED shell scripts, so a file
/// git has never heard of is not the project's problem.
///
/// The regression this pins: the scan used to be an unfiltered `walkdir` at
/// depth 4, and in this repository it spent its whole 20-file budget inside
/// `.claude/worktrees/` — ephemeral agent copies of the tree holding 2,780
/// scripts plus vendored installers — then reported 40 violations in files the
/// project does not own while announcing that 140 real scripts went unexamined.
///
/// Asserts BOTH directions. A scan that simply returned nothing would satisfy
/// "the untracked file is absent" on its own, so the tracked file must still be
/// found; that is what makes this a test rather than a tautology.
#[test]
fn cb402_scans_tracked_scripts_and_ignores_untracked_ones() {
    let temp = TempDir::new().expect("a temp dir is required for this test to mean anything");
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(temp.path())
            .args(args)
            .output()
            .expect("git must be runnable for this test to mean anything")
    };
    assert!(git(&["init", "-q"]).status.success(), "git init");

    // Both scripts carry the same defect; only their tracked-ness differs.
    let bad = "#!/bin/bash\nrm $UNQUOTED\n";
    fs::write(temp.path().join("tracked.sh"), bad).expect("write tracked.sh");
    fs::write(temp.path().join("untracked.sh"), bad).expect("write untracked.sh");
    assert!(git(&["add", "tracked.sh"]).status.success(), "git add");

    let violations = detect_cb402_shell_script_quality(temp.path());
    let files: Vec<&str> = violations.iter().map(|v| v.file.as_str()).collect();

    // If bashrs is not installed every file yields CB-402-UNMEASURED, which is
    // still a per-file verdict and still distinguishes the two.
    assert!(
        files.iter().any(|f| f.contains("tracked.sh")),
        "the tracked script must be examined; got {files:?}"
    );
    assert!(
        !files.iter().any(|f| f.contains("untracked.sh")),
        "an untracked script is not the repository's code; got {files:?}"
    );
}

/// A worktree that tracks no shell script must not read as "the shell scripts
/// here pass". `git ls-files` exits 0 and prints nothing in that case, so the
/// success-only fallback guard cannot distinguish it from a clean scan.
///
/// Both directions: the disclosure must fire when scripts exist untracked, and
/// must NOT fire when there are genuinely no scripts — otherwise the check would
/// simply complain about every project without shell scripts.
#[test]
fn cb402_discloses_a_worktree_where_no_script_is_tracked() {
    let temp = TempDir::new().expect("temp dir");
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(temp.path())
            .args(args)
            .output()
            .expect("git must be runnable")
    };
    assert!(git(&["init", "-q"]).status.success(), "git init");

    // Counter-test first: an empty worktree has no scripts and no complaint.
    let clean = detect_cb402_shell_script_quality(temp.path());
    assert!(
        !clean
            .iter()
            .any(|v| v.pattern_id == "CB-402-UNTRACKED-ONLY"),
        "a project with no shell scripts must not be told its scripts went unexamined: {:?}",
        clean.iter().map(|v| &v.pattern_id).collect::<Vec<_>>()
    );

    // Now an untracked script: git tracks nothing, so nothing was examined.
    fs::write(temp.path().join("build.sh"), "#!/bin/bash\nrm $X\n").expect("write build.sh");
    let disclosed = detect_cb402_shell_script_quality(temp.path());
    assert!(
        disclosed
            .iter()
            .any(|v| v.pattern_id == "CB-402-UNTRACKED-ONLY"),
        "a scan that reached zero of one script must say so: {:?}",
        disclosed.iter().map(|v| &v.pattern_id).collect::<Vec<_>>()
    );
}

#[test]
fn test_cb402_target_dir_excluded() {
    let temp = TempDir::new().unwrap();
    // Shell script in target/ should be ignored
    let target_dir = temp.path().join("target");
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(target_dir.join("test.sh"), "#!/bin/bash\necho test").unwrap();
    let violations = detect_cb402_shell_script_quality(temp.path());
    assert!(
        violations.is_empty(),
        "Scripts in target/ should be ignored"
    );
}

#[test]
fn test_parse_bashrs_json_array() {
    let json = r#"[{"code":"SC2086","message":"Double quote","line":5,"severity":"warning"}]"#;
    let result = parse_bashrs_json_output(json).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, "SC2086");
    assert_eq!(result[0].line, 5);
}

#[test]
fn test_parse_bashrs_json_object() {
    let json =
        r#"{"diagnostics":[{"code":"SC2046","message":"Quote this","line":3,"severity":"error"}]}"#;
    let result = parse_bashrs_json_output(json).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, "SC2046");
    assert_eq!(result[0].severity, "error");
}

/// Invalid output is an ERROR. This test asserted the opposite —
/// `assert!(result.is_empty(), "Invalid JSON should return empty")` — and that
/// assertion is precisely the defect: it pinned "I could not read the linter"
/// to the same value as "the linter found nothing", which is what let
/// `CB-400: Shell & Makefile Quality` report Pass on a tree with 217 bashrs
/// errors. The test was guarding the bug.
#[test]
fn test_parse_bashrs_json_invalid_is_an_error() {
    let err = parse_bashrs_json_output("not valid json")
        .expect_err("invalid JSON must not be reported as zero violations");
    assert!(err.contains("not JSON"), "{err}");
}

#[test]
fn test_parse_bashrs_json_empty_array() {
    let json = "[]";
    let result = parse_bashrs_json_output(json).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_bashrs_json_multiple_issues() {
    let json = r#"[
        {"code":"SC2086","message":"Double quote","line":5,"severity":"warning"},
        {"code":"SC2046","message":"Quote this","line":10,"severity":"error"},
        {"code":"SC2116","message":"Useless echo","line":15,"severity":"info"}
    ]"#;
    let result = parse_bashrs_json_output(json).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].code, "SC2086");
    assert_eq!(result[1].code, "SC2046");
    assert_eq!(result[2].code, "SC2116");
}

/// bashrs 6.66.2 prefixes its stdout with an ANSI-coloured log line before the
/// JSON payload. The parser used to fail on that and `return Ok(Vec::new())`
/// under the comment "graceful degradation", so every script came back clean and
/// `CB-400: Shell & Makefile Quality` reported
/// `Pass: bashrs: All shell scripts and Makefiles pass quality checks`
/// for `infra`, where bashrs itself exits 2 with hundreds of diagnostics.
///
/// After the fix the same input yields the real diagnostics: CB-400 on infra
/// went from Pass to `217 errors, 476 warnings`.
#[test]
fn bashrs_log_preamble_does_not_hide_the_diagnostics() {
    let raw = "\u{1b}[2m2026-08-16T11:38:43.919522Z\u{1b}[0m \u{1b}[32m INFO\u{1b}[0m \
               \u{1b}[2mbashrs::cli::commands::lint_cmds\u{1b}[0m\u{1b}[2m:\u{1b}[0m \
               Linting ./scripts/x.sh\n\
               {\"file\":\"./scripts/x.sh\",\"diagnostics\":[\
               {\"code\":\"SC2086\",\"message\":\"Double quote\",\"line\":5,\"severity\":\"error\"}]}\n";
    let result = parse_bashrs_json_output(raw).expect("payload after a log preamble is parseable");
    assert_eq!(
        result.len(),
        1,
        "the log preamble swallowed the diagnostics"
    );
    assert_eq!(result[0].code, "SC2086");
}

/// The anchor must be a LINE that opens the payload, not the first `{`/`[` byte.
/// Every ANSI escape contains a literal `[` (`ESC[2m`), so a byte search lands
/// inside the colour code — the first version of this fix did exactly that and
/// still reported "output was not JSON" against real bashrs output.
#[test]
fn ansi_escapes_in_the_preamble_do_not_capture_the_json_anchor() {
    let raw = "\u{1b}[2mINFO\u{1b}[0m Linting x\n[{\"code\":\"SC1014\",\"message\":\"m\",\"line\":1,\"severity\":\"error\"}]";
    let result = parse_bashrs_json_output(raw).expect("array payload is parseable");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].code, "SC1014");
}

/// Output that carries no JSON at all is an ERROR, not a clean bill of health.
///
/// This is the load-bearing half. A tool that could not be read tells you
/// nothing about the code; reporting "no violations" turns "I did not measure"
/// into "there is nothing to find", which is the defect that made CB-400
/// unfailable.
#[test]
fn unparseable_output_is_an_error_not_an_empty_result() {
    let err = parse_bashrs_json_output("bashrs: command not found\n")
        .expect_err("unmeasured must not read as clean");
    assert!(
        err.contains("not JSON"),
        "the error must say what went wrong: {err}"
    );
}

// =========================================================================
// CB-400 attribution: a hook pmat wrote is pmat's, not the repository's
// (#1049, #1020)
// =========================================================================

/// Shell that bashrs has real findings in, so a "no violations" result cannot
/// be an artefact of clean input. Deliberately awful.
const DIRTY_SHELL: &str = "#!/bin/bash\nX=`ls`\nfor f in $X; do rm $f; done\ncd $1\n";

fn write_hook(dir: &std::path::Path, name: &str, body: &str) {
    let hooks = dir.join(".git/hooks");
    fs::create_dir_all(&hooks).expect("create hooks dir");
    fs::write(hooks.join(name), body).expect("write hook");
}

fn non_info(v: &[CbPatternViolation]) -> Vec<&CbPatternViolation> {
    v.iter().filter(|x| x.severity != Severity::Info).collect()
}

/// The defect in #1049 / #1020: `pmat hooks install` writes the hook, marks it
/// DO NOT EDIT, git never tracks it, and then CB-400 reports its shell quality
/// as the audited repository's. No commit in that repository can change the
/// file, so the finding is unactionable by the only person who sees it.
#[test]
fn cb400_does_not_bill_the_repo_for_a_hook_pmat_generated() {
    let temp = TempDir::new().expect("tempdir");
    write_hook(
        temp.path(),
        "pre-commit",
        &format!(
            "#!/bin/bash\n# Generated pre-commit hook (auto-managed by PMAT)\n\
             # DO NOT EDIT: This file is automatically generated\n{}",
            DIRTY_SHELL.trim_start_matches("#!/bin/bash\n")
        ),
    );
    let violations = detect_cb400_git_hooks_quality(temp.path());
    assert!(
        non_info(&violations).is_empty(),
        "pmat's own generated hook must not produce scored violations, got: {:?}",
        non_info(&violations)
    );
}

/// The exclusion must be DISCLOSED, never silent: an empty result and a
/// suppressed result must not look the same.
#[test]
fn cb400_discloses_the_pmat_generated_hook_it_did_not_score() {
    let temp = TempDir::new().expect("tempdir");
    write_hook(
        temp.path(),
        "pre-commit",
        &format!(
            "#!/bin/bash\n# Generated pre-commit hook (auto-managed by PMAT)\n{}",
            DIRTY_SHELL.trim_start_matches("#!/bin/bash\n")
        ),
    );
    let violations = detect_cb400_git_hooks_quality(temp.path());
    let disclosure = violations
        .iter()
        .find(|v| v.pattern_id == "CB-400-PMAT-GENERATED")
        .expect("a suppressed hook must be disclosed, not silently dropped");
    assert_eq!(disclosure.severity, Severity::Info);
    assert!(
        disclosure.file.contains("pre-commit"),
        "the disclosure must name the file it skipped: {disclosure:?}"
    );
}

/// COUNTER-TEST. The lazy over-correction is "stop linting .git/hooks", which
/// would make CB-400 incapable of failing. A hook the repository itself wrote
/// is still the repository's, and must still be scored.
#[test]
fn cb400_still_scores_a_hook_the_repo_wrote_itself() {
    let temp = TempDir::new().expect("tempdir");
    write_hook(temp.path(), "pre-commit", DIRTY_SHELL);
    let violations = detect_cb400_git_hooks_quality(temp.path());
    assert!(
        !non_info(&violations).is_empty(),
        "a hand-written hook must still be scored; CB-400 must stay able to fail"
    );
}

/// COUNTER-TEST for the predicate itself. A hand-written hook that merely
/// *invokes* pmat is not a hook pmat generated. Classifying it as one would
/// silently exempt every repository that adopted pmat in its own hooks.
#[test]
fn a_hand_written_hook_that_calls_pmat_is_not_a_pmat_generated_hook() {
    assert!(
        !hook_is_pmat_generated(
            "#!/bin/bash\n# our team's pre-commit gate\nset -e\npmat verify --stage clippy\n"
        ),
        "calling pmat is not the same as being written by pmat"
    );
    assert!(
        !hook_is_pmat_generated("#!/bin/bash\n# DO NOT EDIT: generated by our codegen\nexit 0\n"),
        "a non-pmat generator's marker must not exempt the file"
    );
}

/// The exclusion above is only sound while every hook pmat writes SAYS pmat
/// wrote it. `GitHookManager::install_hooks` shipped a pre-commit and a
/// commit-msg hook with no provenance line at all, so CB-400 could not tell
/// them from the repository's own work and billed them to the repository.
#[test]
fn every_hook_pmat_installs_declares_that_pmat_wrote_it() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join(".git/hooks")).expect("hooks dir");
    crate::quality::git_hooks::GitHookManager::new(temp.path())
        .install_hooks()
        .expect("install hooks");

    for name in ["pre-commit", "commit-msg", "pre-push"] {
        let body = fs::read_to_string(temp.path().join(".git/hooks").join(name))
            .unwrap_or_else(|e| format!("<{name} was not installed or is unreadable: {e}>"));
        assert!(
            hook_is_pmat_generated(&body),
            "the {name} hook pmat writes must declare its provenance, else CB-400 \
             charges the repository for pmat's output. Header was:\n{}",
            body.lines().take(4).collect::<Vec<_>>().join("\n")
        );
    }
}

/// The `pmat hooks install` template (a different generator from the one above)
/// must carry the same provenance.
#[test]
fn the_hooks_install_pre_commit_template_declares_its_provenance() {
    let cmd = crate::cli::handlers::hooks_command_handlers::HooksCommand::new(
        std::path::PathBuf::from("/tmp"),
        std::path::PathBuf::from("/tmp"),
    );
    assert!(
        hook_is_pmat_generated(&cmd.generate_hook_header()),
        "`pmat hooks install`'s header must declare pmat as its author"
    );
}

/// #1020, criterion 4: an empty result must be distinguishable from an
/// unscanned tree. CB-400 examines exactly four hook names; a repository whose
/// gate lives in `prepare-commit-msg` was told nothing at all.
#[test]
fn cb400_states_which_hooks_it_actually_linted() {
    let temp = TempDir::new().expect("tempdir");
    write_hook(temp.path(), "pre-commit", DIRTY_SHELL);
    write_hook(temp.path(), "prepare-commit-msg", DIRTY_SHELL);
    let violations = detect_cb400_git_hooks_quality(temp.path());
    let scope = violations
        .iter()
        .find(|v| v.pattern_id == "CB-400-SCOPE")
        .expect("CB-400 must state its scope");
    assert!(
        scope.description.contains("pre-commit"),
        "scope must name what was linted: {}",
        scope.description
    );
    assert!(
        !scope.description.contains("prepare-commit-msg"),
        "prepare-commit-msg is NOT linted by CB-400; the scope row must not imply \
         it was: {}",
        scope.description
    );
}

/// The `--tdg-enforcement` templates are the exact files #1049 was filed
/// against (forjar's `.git/hooks/pre-commit`, pmat 3.32.0). They already
/// declared their provenance; this pins that, because the exclusion in
/// `lint_single_hook` is only as good as the marker it keys on.
#[test]
fn the_tdg_enforcement_hook_templates_declare_their_provenance() {
    for (name, body) in [
        (
            "pre-commit-tdg.sh",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/templates/hooks/pre-commit-tdg.sh"
            )),
        ),
        (
            "post-commit-tdg.sh",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/templates/hooks/post-commit-tdg.sh"
            )),
        ),
    ] {
        assert!(
            hook_is_pmat_generated(body),
            "{name} must declare pmat as its author, or CB-400 charges the audited \
             repository for pmat's output (#1049). Header was:\n{}",
            body.lines().take(5).collect::<Vec<_>>().join("\n")
        );
    }
}
