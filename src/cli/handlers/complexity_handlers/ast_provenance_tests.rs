//! Issue #1068 (#1050 P6): a complexity number that came from a regex must not
//! be shaped identically to one that came from the AST.
//!
//! `analyze complexity` prefers `syn` for Rust and silently falls back to the
//! heuristic counter when the parse fails. The only disclosure was a
//! `Warning: AST analysis failed for …, using heuristic fallback` line on
//! STDERR — invisible to `--format json`, to `--output FILE`, and to every
//! machine consumer. On a full bashrs run ten files were counted by regex and
//! none of them were distinguishable in the document.
//!
//! These tests are serial because the provenance ledger is process-global for
//! the duration of one armed run.

use super::*;
use serial_test::serial;
use std::path::Path;

/// A Rust file whose `pub mod` brace is never closed: `syn` refuses it, the
/// heuristic counter still finds every `fn`.
fn unparseable_rust(n: usize) -> String {
    let mut src = String::from("pub mod inner {\n");
    for i in 0..n {
        src.push_str(&format!(
            "pub fn f{i}(x:i32)->i32{{ if x>0 {{x}} else {{-x}} }}\n"
        ));
    }
    src
}

async fn json_for(dir: &Path) -> serde_json::Value {
    let out = dir.join("report.json");
    handle_analyze_complexity(
        dir.to_path_buf(),
        None,
        vec![],
        None,
        ComplexityOutputFormat::Json,
        Some(out.clone()),
        None,
        None,
        vec![],
        false,
        10,
        false,
        300,
    )
    .await
    .expect("complexity analysis must succeed over a readable tree");
    serde_json::from_str(&std::fs::read_to_string(&out).expect("report written"))
        .expect("report is JSON")
}

/// RED: the document says nothing about the fallback.
#[tokio::test]
#[serial]
async fn json_marks_the_file_whose_ast_parse_failed() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::create_dir_all(dir.path().join("src")).expect("src");
    std::fs::write(dir.path().join("src/lib.rs"), unparseable_rust(40)).expect("lib.rs");

    let doc = json_for(dir.path()).await;

    let file = doc["files"]
        .as_array()
        .expect("files array")
        .iter()
        .find(|f| f["path"].as_str().is_some_and(|p| p.ends_with("lib.rs")))
        .expect("lib.rs is in the listing");

    assert_eq!(
        file["analysis"], "heuristic_fallback",
        "a regex-derived count must say so in the document, not only on stderr: {file}"
    );
}

/// The counts partition the analysed population — a count with a denominator.
#[tokio::test]
#[serial]
async fn provenance_counts_are_stated_against_files_analyzed() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::create_dir_all(dir.path().join("src")).expect("src");
    std::fs::write(dir.path().join("src/lib.rs"), unparseable_rust(3)).expect("lib.rs");
    std::fs::write(
        dir.path().join("src/ok.rs"),
        "pub fn g(x:i32)->i32{ if x>0 {x} else {-x} }\n",
    )
    .expect("ok.rs");

    let doc = json_for(dir.path()).await;
    let prov = &doc["analysis_provenance"];

    assert_eq!(prov["heuristic_fallback"], 1, "doc: {doc}");
    assert_eq!(prov["ast"], 1, "doc: {doc}");
    assert_eq!(
        prov["files_analyzed"], doc["files_analyzed"],
        "the provenance denominator must be the analysed population itself"
    );
    let sum = prov["ast"].as_u64().expect("ast count")
        + prov["heuristic"].as_u64().expect("heuristic count")
        + prov["heuristic_include_fragment"]
            .as_u64()
            .expect("include-fragment count")
        + prov["heuristic_fallback"].as_u64().expect("fallback count")
        + prov["unrecorded"].as_u64().expect("unrecorded count");
    assert_eq!(
        sum,
        doc["files_analyzed"].as_u64().expect("files_analyzed"),
        "the buckets must add up to the population they describe: {doc}"
    );
    assert_eq!(
        prov["unrecorded"], 0,
        "every reported file was recorded: {doc}"
    );
}

/// COUNTER-TEST bounding the over-correction: a tree that parses cleanly must
/// NOT be labelled as having fallen back, and a non-Rust file — which never had
/// an AST analyzer in this build — is `heuristic`, not `heuristic_fallback`.
/// Reporting a normal path as a degradation is the same defect pointing the
/// other way.
#[tokio::test]
#[serial]
async fn a_clean_parse_is_not_reported_as_a_fallback() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::create_dir_all(dir.path().join("src")).expect("src");
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "pub fn g(x:i32)->i32{ if x>0 {x} else {-x} }\n",
    )
    .expect("lib.rs");
    std::fs::write(
        dir.path().join("src/app.py"),
        "def h(x):\n    if x > 0:\n        return x\n    return -x\n",
    )
    .expect("app.py");

    let doc = json_for(dir.path()).await;
    let prov = &doc["analysis_provenance"];

    assert_eq!(prov["heuristic_fallback"], 0, "doc: {doc}");
    assert_eq!(prov["ast"], 1, "the Rust file parsed: {doc}");
    assert_eq!(
        prov["heuristic"], 1,
        "Python has no AST complexity analyzer in this build; that is the \
         normal path and must not be reported as a degradation: {doc}"
    );

    for f in doc["files"].as_array().expect("files array") {
        let path = f["path"].as_str().unwrap_or_default();
        let want = if path.ends_with(".py") {
            "heuristic"
        } else {
            "ast"
        };
        assert_eq!(f["analysis"], want, "wrong provenance for {path}: {f}");
    }
}

/// COUNTER-TEST 2. `include!()` fragments are Rust that pmat HAS an AST
/// analyzer for and deliberately does not apply. They are the single largest
/// non-AST bucket in this repo (470 of 830 `.rs` files under
/// `src/cli/handlers`), so folding them into `heuristic_fallback` would invent
/// hundreds of failures, and folding them into `heuristic` would claim Rust has
/// no AST analyzer. Neither is true and both are worse than the silence they
/// would replace.
#[tokio::test]
#[serial]
async fn a_deliberately_skipped_include_fragment_is_not_a_failed_parse() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::create_dir_all(dir.path().join("src")).expect("src");
    std::fs::write(
        dir.path().join("src/thing_tests.rs"),
        "pub fn g(x:i32)->i32{ if x>0 {x} else {-x} }\n",
    )
    .expect("thing_tests.rs");

    let doc = json_for(dir.path()).await;
    let prov = &doc["analysis_provenance"];

    assert_eq!(
        prov["heuristic_include_fragment"], 1,
        "the fragment has its own bucket: {doc}"
    );
    assert_eq!(
        prov["heuristic_fallback"], 0,
        "a deliberate skip is not a failed parse: {doc}"
    );
    assert_eq!(prov["ast"], 0, "doc: {doc}");

    let file = doc["files"]
        .as_array()
        .expect("files array")
        .iter()
        .find(|f| {
            f["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("thing_tests.rs"))
        })
        .expect("the fragment is listed");
    assert_eq!(file["analysis"], "heuristic_include_fragment", "{file}");
}

/// A run scoped to ONE file must not publish the whole working directory as
/// its denominator.
///
/// Same family as issue #1065, pointing the other way. `--file X` leaves
/// `project_path` at its default `.`, and the census guard tested
/// `!project_path.is_dir()` — which is false for `.` — so the census walked the
/// entire current directory. `analyze complexity --file one.rs --format json`
/// run inside this repo reported
///
/// ```json
/// {"files_analyzed": 1, "files_discovered": 5363,
///  "files_not_analyzed": {"total": 5363, "supported_but_unmeasured": {"rs": 4426, …}}}
/// ```
///
/// — 5,363 files "discovered" and 4,426 Rust files declared unmeasured by a run
/// that was never asked to measure them. The comment on `files_discovered`
/// already states the intended rule ("single-file and explicit-file modes have
/// no population to compare against"); only the guard was wrong.
#[tokio::test]
#[serial]
async fn a_single_file_run_does_not_borrow_the_working_directory_as_its_denominator() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::create_dir_all(dir.path().join("src")).expect("src");
    let target = dir.path().join("src/lib.rs");
    std::fs::write(&target, "pub fn g(x:i32)->i32{ if x>0 {x} else {-x} }\n").expect("lib.rs");
    // Files the run was NOT asked about, in the directory it is not scoped to.
    for i in 0..7 {
        std::fs::write(
            dir.path().join(format!("src/other{i}.rs")),
            "pub fn h(){}\n",
        )
        .expect("other");
    }

    let out = dir.path().join("report.json");
    handle_analyze_complexity(
        // The default `project_path` the CLI supplies alongside `--file`.
        dir.path().to_path_buf(),
        Some(target.clone()),
        vec![],
        None,
        ComplexityOutputFormat::Json,
        Some(out.clone()),
        None,
        None,
        vec![],
        false,
        10,
        false,
        300,
    )
    .await
    .expect("single-file analysis must succeed");
    let doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).expect("report written"))
            .expect("report is JSON");

    assert_eq!(doc["files_analyzed"], 1, "doc: {doc}");
    assert_eq!(
        doc["files_discovered"], 1,
        "a one-file run discovered one file; the other seven were never in \
         its scope: {doc}"
    );
    assert!(
        doc["files_not_analyzed"].is_null(),
        "no population was compared, so there is no skip breakdown — a \
         fabricated one is the absence-rendered-as-measurement this field \
         exists to stop: {doc}"
    );
}

/// COUNTER-TEST for the above: silencing the census for a scoped run must not
/// silence it for a PROJECT run. The easiest way to "fix" a wrong denominator
/// is to stop publishing one, which would revert issue #1065 — the whole point
/// of which is that `files_discovered` must be the walk's count and not the
/// analysed count.
#[tokio::test]
#[serial]
async fn a_project_run_still_publishes_the_walks_denominator() {
    let dir = tempfile::TempDir::new().expect("temp dir");
    std::fs::create_dir_all(dir.path().join("src")).expect("src");
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "pub fn g(x:i32)->i32{ if x>0 {x} else {-x} }\n",
    )
    .expect("lib.rs");
    // A file the walk sees and has no complexity analyzer for.
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").expect("toml");

    let doc = json_for(dir.path()).await;

    assert_eq!(doc["files_analyzed"], 1, "doc: {doc}");
    assert_eq!(
        doc["files_discovered"], 2,
        "the walk's count, not the analysed count: {doc}"
    );
    assert_eq!(doc["files_not_analyzed"]["total"], 1, "doc: {doc}");
    assert_eq!(
        doc["files_not_analyzed"]["no_complexity_analyzer"]["toml"], 1,
        "the skip keeps its reason: {doc}"
    );
}
