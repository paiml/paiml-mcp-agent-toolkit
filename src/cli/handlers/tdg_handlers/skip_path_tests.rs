//! Regression tests for GH #716: `pmat tdg <file>` graded (or refused to
//! grade) a file according to how the CALLER SPELLED ITS PATH.
//!
//! `should_skip_path` substring-matched the user-supplied string for
//! `"/tests/"`, so the identical bytes came back two ways from one binary:
//! `pmat tdg /abs/tcorp/tests/m00.rs` printed `Skipping test file: …` and no
//! score, while `cd tcorp/tests && pmat tdg m00.rs` printed
//! `{"score":{"total":92.57…,"grade":"A"}}`. The skip also announced itself
//! with a bare `println!`, so `--format json` emitted a sentence and exited 0.

use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

fn config_for(path: PathBuf, format: TdgOutputFormat) -> TdgCommandConfig {
    TdgCommandConfig {
        path,
        command: None,
        format,
        config: None,
        quiet: false,
        include_components: false,
        min_grade: None,
        output: None,
        with_git_context: false,
        explain: false,
        threshold: 10,
        baseline: None,
        viz: false,
        viz_theme: "default".to_string(),
    }
}

/// A tempdir holding `tests/m00.rs`, returned with the file's absolute path.
fn tests_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let tests_dir = dir.path().join("tests");
    std::fs::create_dir_all(&tests_dir).expect("mkdir tests");
    let file = tests_dir.join("m00.rs");
    std::fs::write(&file, "#[test]\nfn t0() { assert!(true); }\n").expect("write");
    (dir, file)
}

/// The reported reproduction: absolute spelling vs. spelling relative to the
/// `tests` directory itself. Both name the same inode, so both must get the
/// same verdict. On the old substring check the relative form had no
/// `"/tests/"` in it and was scored.
#[test]
#[serial_test::serial]
fn skip_verdict_does_not_depend_on_the_callers_cwd() {
    let (dir, file) = tests_fixture();

    let absolute = should_skip_path(&config_for(file.clone(), TdgOutputFormat::Json));

    let previous = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(file.parent().expect("parent")).expect("chdir");
    let relative = should_skip_path(&config_for(PathBuf::from("m00.rs"), TdgOutputFormat::Json));
    std::env::set_current_dir(previous).expect("restore cwd");

    assert_eq!(
        absolute, relative,
        "same file, two spellings, two verdicts — the grade depended on the caller's cwd"
    );
    assert!(absolute, "a file under tests/ is a test file either way");
    drop(dir);
}

/// Path spelling is not file identity: a symlinked spelling contains no
/// `"/tests/"` substring at all, yet resolves into `tests/`.
#[test]
fn skip_verdict_follows_the_resolved_path_not_the_spelling() {
    let (dir, file) = tests_fixture();
    let link = dir.path().join("aliased");
    #[cfg(unix)]
    std::os::unix::fs::symlink(file.parent().expect("parent"), &link).expect("symlink");
    #[cfg(not(unix))]
    return;

    let aliased = link.join("m00.rs");
    assert!(
        !aliased.to_string_lossy().contains("/tests/"),
        "fixture must not contain the substring the old check looked for"
    );
    assert!(
        should_skip_path(&config_for(aliased, TdgOutputFormat::Json)),
        "the file is under tests/ however it is spelled"
    );
}

/// A file merely NAMED `tests.rs` is source, not a test directory — the old
/// `/tests/` substring required a directory component and the component match
/// must keep that.
#[test]
fn a_file_named_tests_rs_is_not_skipped() {
    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("tests.rs");
    std::fs::write(&file, "pub fn f() {}\n").expect("write");
    assert!(!should_skip_path(&config_for(file, TdgOutputFormat::Json)));
}

/// `--format json` must produce JSON even when nothing was scored. The skip
/// used to `println!("Skipping test file: …")`, which is not parseable and
/// carries no "we did not measure this" fact at all.
#[test]
fn skipped_json_is_json_and_says_nothing_was_measured() {
    let (_dir, file) = tests_fixture();
    let rendered =
        skipped_output(&config_for(file.clone(), TdgOutputFormat::Json)).expect("render");

    let value: serde_json::Value =
        serde_json::from_str(&rendered).unwrap_or_else(|e| panic!("not JSON: {e}\n{rendered}"));
    assert_eq!(value["analyzed"], serde_json::json!(false));
    assert!(value["score"].is_null(), "no score may be invented");
    assert!(value["grade"].is_null(), "no grade may be invented");
    assert_eq!(
        value["not_measured"],
        serde_json::json!(["score", "grade"]),
        "the unmeasured fields must be named"
    );
}

/// `--format sarif` must produce SARIF, not the JSON object above and not a
/// sentence (the shape issue #669 already had to fix once).
#[test]
fn skipped_sarif_is_a_valid_empty_sarif_run() {
    let (_dir, file) = tests_fixture();
    let rendered = skipped_output(&config_for(file, TdgOutputFormat::Sarif)).expect("render");

    let value: serde_json::Value =
        serde_json::from_str(&rendered).unwrap_or_else(|e| panic!("not JSON: {e}\n{rendered}"));
    assert_eq!(value["version"], serde_json::json!("2.1.0"));
    assert_eq!(
        value["runs"][0]["results"],
        serde_json::json!([]),
        "a skipped file produces no results"
    );
    assert_eq!(
        value["runs"][0]["properties"]["analyzed"],
        serde_json::json!(false)
    );
}
