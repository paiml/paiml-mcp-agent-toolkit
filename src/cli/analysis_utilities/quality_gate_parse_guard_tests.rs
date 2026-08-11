//! `quality-gate --file` must refuse a file that does not parse.
//!
//! In its own file rather than at the end of `quality_gate_single_file.rs`,
//! which is `include!`d into `mod.rs` ahead of other fragments: a test module
//! there puts items after a test module in the expanded source
//! (`clippy::items_after_test_module`, denied by `ci / lint`).

use std::io::Write;

fn write_temp(name: &str, body: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("pmat-qg-guard-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p).expect("create");
    f.write_all(body.as_bytes()).expect("write");
    p
}

/// The defect: the CLI reported "Quality Gate: PASSED / Total Violations: 0"
/// for `def f(:` while the MCP `quality_gate` tool refused the same file.
#[test]
fn unparseable_python_is_refused_not_passed() {
    let p = write_temp("bad.py", "def f(:\n  ???\n");
    let err = crate::tdg::ensure_parseable(&p).expect_err("must refuse");
    let msg = err.to_string();
    assert!(msg.contains("not parseable as Python"), "{msg}");
    assert!(msg.contains("did not parse"), "{msg}");
}

#[test]
fn unparseable_rust_is_refused() {
    let p = write_temp("bad.rs", "fn main( { let x = ;;;\n");
    let err = crate::tdg::ensure_parseable(&p).expect_err("must refuse");
    assert!(err.to_string().contains("not parseable as Rust"));
}

#[test]
fn valid_source_passes_the_guard() {
    let p = write_temp("good.py", "def f(x):\n    return x\n");
    crate::tdg::ensure_parseable(&p).expect("valid python must pass");
    let r = write_temp("good.rs", "pub fn f(x: i32) -> i32 { x }\n");
    crate::tdg::ensure_parseable(&r).expect("valid rust must pass");
}

/// The guard is a *parse* gate, not a "source files only" gate — `quality-gate`
/// legitimately scans prose for SATD markers, so a language this build has no
/// grammar for must pass through rather than be refused.
#[test]
fn a_language_without_a_grammar_is_not_refused() {
    let p = write_temp("notes.md", "# TODO: this is not code\n");
    crate::tdg::ensure_parseable(&p).expect("markdown must not be refused");
}

#[test]
fn an_unreadable_path_is_left_for_the_caller_to_report() {
    let p = std::env::temp_dir().join("pmat-qg-guard-does-not-exist.rs");
    crate::tdg::ensure_parseable(&p).expect("missing file is not this guard's error");
}
