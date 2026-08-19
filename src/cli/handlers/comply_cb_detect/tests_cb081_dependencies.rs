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
    assert!(violations.is_empty(), "Empty hooks dir should return empty");
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
    assert!(violations.is_empty(), "Sample hooks should be ignored");
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
