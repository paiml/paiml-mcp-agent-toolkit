//! `analyze dead-code` pointed at a DIRECTORY INSIDE a crate.
//!
//! Fixing "a library's public API is not dead" put the whole target-shape
//! decision behind a `Cargo.toml` looked for in the directory the caller named.
//! A subdirectory holds no manifest, so the decision came out "binary-only
//! crate" for every subdirectory of every library, and the consequences ran all
//! the way to the answer:
//!
//! ```text
//!   analyze dead-code --path <crate>            dead_functions:1  dead_classes:1
//!   analyze dead-code --path <crate>/src/inner  dead_functions:0  dead_classes:0  exit 0
//!     library_target: {"verdict":"not-a-library",
//!       "detail":"cargo: Cargo.toml declares no [lib] and there is no src/lib.rs …"}
//! ```
//!
//! …about a crate whose `src/lib.rs` was two directories up. `--lib` was
//! dropped from the cargo invocation, `cargo check --bins` on a lib-only crate
//! matched no target, rustc compiled NOTHING, and a zero measured over nothing
//! was published at exit 0.
//!
//! The bin-crate half failed the opposite way. There cargo did compile
//! something — the whole crate, since `--bins` matched — so the report of a
//! subdirectory listed files from everywhere else in the crate.
//!
//! rustc cannot type-check less than a crate. So the crate is found by walking
//! up, the crate is what cargo compiles, and the findings are restricted to the
//! requested subtree afterwards.

use super::{cargo_library_target, run_dead_code_analysis_with_filters, DeadCodeAnalysisFilters};
use std::path::Path;
use tempfile::TempDir;

fn filters() -> DeadCodeAnalysisFilters {
    DeadCodeAnalysisFilters {
        include_unreachable: false,
        include_tests: false,
        min_dead_lines: 0,
        top_files: None,
        include: Vec::new(),
        exclude: Vec::new(),
        max_depth: 10,
    }
}

/// Every finding, as `(file, item)` pairs re-expressed against a common root so
/// two runs over different roots can be compared at all.
///
/// `prefix` is where the analysed path sits inside the crate: the empty string
/// for a whole-crate run, `src/inner` for a run over that subdirectory.
async fn findings(path: &Path, prefix: &str) -> Vec<(String, String)> {
    let outcome =
        run_dead_code_analysis_with_filters(path, filters(), std::time::Duration::from_secs(600))
            .await
            .expect("the analysis runs");

    let mut rows: Vec<(String, String)> = outcome
        .report
        .files
        .iter()
        .flat_map(|file| {
            let full = if prefix.is_empty() {
                file.path.clone()
            } else {
                format!("{prefix}/{}", file.path)
            };
            file.items
                .iter()
                .map(move |item| (full.clone(), item.name.clone()))
        })
        .collect();
    rows.sort();
    rows
}

/// A library crate whose only dead code lives in a subdirectory.
fn lib_crate_with_dead_code_in_a_subdirectory() -> TempDir {
    let tmp = tempfile::Builder::new()
        .prefix("dcsub")
        .tempdir()
        .expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("src/inner")).expect("mkdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"dcsub\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write manifest");
    std::fs::write(
        tmp.path().join("src/lib.rs"),
        "pub mod inner;\n\npub fn public_api() -> i32 {\n    42\n}\n",
    )
    .expect("write lib.rs");
    std::fs::write(
        tmp.path().join("src/inner/mod.rs"),
        "fn dead_private_helper() -> i32 {\n    7\n}\n\n\
         struct NeverConstructed {\n    field: i32,\n}\n",
    )
    .expect("write inner/mod.rs");
    crate::services::cargo_dead_code_analyzer::write_fixture_lockfile(tmp.path());
    tmp
}

/// A binary crate with dead code in a subdirectory AND dead code outside it.
fn bin_crate_with_dead_code_inside_and_outside_a_subdirectory() -> TempDir {
    let tmp = tempfile::Builder::new()
        .prefix("dcbin")
        .tempdir()
        .expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("src/inner")).expect("mkdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"dcbin\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write manifest");
    std::fs::write(
        tmp.path().join("src/main.rs"),
        "mod inner;\nmod other;\n\nfn main() {\n    println!(\"{}\", inner::used());\n}\n",
    )
    .expect("write main.rs");
    std::fs::write(
        tmp.path().join("src/inner/mod.rs"),
        "pub fn used() -> i32 {\n    1\n}\n\n\
         fn dead_private_helper() -> i32 {\n    7\n}\n\n\
         struct NeverConstructed {\n    field: i32,\n}\n",
    )
    .expect("write inner/mod.rs");
    std::fs::write(
        tmp.path().join("src/other.rs"),
        "fn outside_subtree_dead() -> i32 {\n    9\n}\n",
    )
    .expect("write other.rs");
    crate::services::cargo_dead_code_analyzer::write_fixture_lockfile(tmp.path());
    tmp
}

// ── the subtree must report what the crate reports, restricted ──────────────

/// LIBRARY CRATE. The subdirectory reported nothing at all: `--lib` was never
/// passed, so nothing was compiled and nothing could be found.
#[tokio::test]
async fn a_lib_crates_subdirectory_reports_the_dead_code_the_crate_reports() {
    let tmp = lib_crate_with_dead_code_in_a_subdirectory();

    let whole_crate = findings(tmp.path(), "").await;
    let subtree = findings(&tmp.path().join("src/inner"), "src/inner").await;

    // The crate's own findings, restricted to the subtree — which here is all
    // of them, because that is where the dead code is.
    let restricted: Vec<(String, String)> = whole_crate
        .iter()
        .filter(|(file, _)| file.starts_with("src/inner"))
        .cloned()
        .collect();

    assert!(
        !restricted.is_empty(),
        "the fixture must have dead code in the subtree for this to prove anything; \
         the whole-crate run found {whole_crate:?}"
    );
    assert_eq!(
        subtree, restricted,
        "the subtree must report what the crate reports about it. The crate found \
         {whole_crate:?}; the subtree found {subtree:?}"
    );
    assert!(
        subtree
            .iter()
            .any(|(_, name)| name == "dead_private_helper"),
        "the dead private function is invisible from the subdirectory: {subtree:?}"
    );
    assert!(
        subtree.iter().any(|(_, name)| name == "NeverConstructed"),
        "the never-constructed struct is invisible from the subdirectory: {subtree:?}"
    );
}

/// BINARY CRATE, the mirror-image failure: cargo did compile, so the
/// subdirectory's report listed `src/other.rs` — a file the request excluded —
/// under a summary claiming one file analysed.
#[tokio::test]
async fn a_bin_crates_subdirectory_reports_its_own_dead_code_and_no_one_elses() {
    let tmp = bin_crate_with_dead_code_inside_and_outside_a_subdirectory();

    let whole_crate = findings(tmp.path(), "").await;
    let subtree = findings(&tmp.path().join("src/inner"), "src/inner").await;

    let restricted: Vec<(String, String)> = whole_crate
        .iter()
        .filter(|(file, _)| file.starts_with("src/inner"))
        .cloned()
        .collect();

    assert!(
        whole_crate
            .iter()
            .any(|(_, name)| name == "outside_subtree_dead"),
        "the fixture must have dead code OUTSIDE the subtree for this to prove \
         anything: {whole_crate:?}"
    );
    assert!(!restricted.is_empty(), "{whole_crate:?}");
    assert_eq!(
        subtree, restricted,
        "the subtree must report what the crate reports about it, and nothing else. \
         The crate found {whole_crate:?}; the subtree found {subtree:?}"
    );
    assert!(
        !subtree
            .iter()
            .any(|(_, name)| name == "outside_subtree_dead"),
        "a request for src/inner reported a finding in src/other.rs: {subtree:?}"
    );
}

// ── no crate at all: refuse, do not publish a zero ──────────────────────────

/// A loose `.rs` tree with no manifest above it. rustc has nothing to compile,
/// so there is no measurement — and an unmeasured project must not be reported
/// as a clean one.
#[tokio::test]
async fn a_tree_with_no_enclosing_crate_is_refused_not_reported_as_clean() {
    let tmp = tempfile::Builder::new()
        .prefix("dcloose")
        .tempdir()
        .expect("tempdir");
    std::fs::write(
        tmp.path().join("stray.rs"),
        "fn dead_helper() -> i32 {\n    3\n}\n",
    )
    .expect("write stray.rs");

    let outcome = run_dead_code_analysis_with_filters(
        tmp.path(),
        filters(),
        std::time::Duration::from_secs(600),
    )
    .await;
    let error = match outcome {
        Ok(run) => panic!(
            "a tree with no crate cannot be measured, so it must not be reported; got \
             a report claiming {} file(s) with dead code",
            run.report.summary.files_with_dead_code
        ),
        Err(e) => e.to_string(),
    };

    assert!(
        error.contains("no dead-code measurement was taken"),
        "the refusal must say that nothing was measured, not merely relay a cargo \
         failure: {error}"
    );
    assert!(
        error.contains("This is not a clean result"),
        "the refusal must say what the absence of a result is NOT: {error}"
    );
    assert!(
        error.contains(&tmp.path().display().to_string()),
        "the refusal must name the path it could find no crate for: {error}"
    );
}

// ── COUNTER-TESTS: whole-crate analysis is unchanged ────────────────────────

/// Must pass BEFORE and AFTER. A "fix" that refuses subdirectories, or that
/// filters everything away, fails here.
#[tokio::test]
async fn the_whole_lib_crate_still_reports_exactly_what_it_reported_before() {
    let tmp = lib_crate_with_dead_code_in_a_subdirectory();

    let outcome = run_dead_code_analysis_with_filters(
        tmp.path(),
        filters(),
        std::time::Duration::from_secs(600),
    )
    .await
    .expect("the analysis runs");

    assert_eq!(outcome.report.summary.dead_functions, 1);
    assert_eq!(outcome.report.summary.dead_classes, 1);
    assert_eq!(outcome.report.summary.files_with_dead_code, 1);
    assert_eq!(
        outcome
            .report
            .files
            .iter()
            .map(|f| f.path.clone())
            .collect::<Vec<_>>(),
        vec!["src/inner/mod.rs".to_string()]
    );
    assert_eq!(
        outcome
            .report
            .library_target
            .as_ref()
            .map(|t| t.verdict.as_str()),
        Some("library")
    );
}

/// The same control for the binary crate, where the whole-crate report is the
/// one that was already correct.
#[tokio::test]
async fn the_whole_bin_crate_still_reports_exactly_what_it_reported_before() {
    let tmp = bin_crate_with_dead_code_inside_and_outside_a_subdirectory();

    let outcome = run_dead_code_analysis_with_filters(
        tmp.path(),
        filters(),
        std::time::Duration::from_secs(600),
    )
    .await
    .expect("the analysis runs");

    assert_eq!(outcome.report.summary.dead_functions, 2);
    assert_eq!(outcome.report.summary.dead_classes, 1);
    assert_eq!(outcome.report.summary.files_with_dead_code, 2);
    assert_eq!(
        outcome
            .report
            .library_target
            .as_ref()
            .map(|t| t.verdict.as_str()),
        Some("not-a-library"),
        "a crate with no [lib] and no src/lib.rs is a DECIDED verdict"
    );
}

// ── the verdict itself ─────────────────────────────────────────────────────

/// A real directory of this repo. The published `detail` asserted that this
/// crate "declares no [lib] and there is no src/lib.rs"; `src/lib.rs` is right
/// there, and the claim was checkable and false.
#[test]
fn a_subdirectory_of_this_repo_is_reported_against_this_repos_crate() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(
        repo.join("src/lib.rs").is_file(),
        "this repo has a library target"
    );

    let verdict = cargo_library_target(&repo.join("src/services/satd_detector"));

    assert_eq!(
        verdict.verdict, "library",
        "a subdirectory of a library crate is inside a library: {verdict:?}"
    );
    assert!(
        !verdict.detail.contains("declares no [lib]"),
        "the detail asserts something the enclosing Cargo.toml contradicts: {verdict:?}"
    );
    assert!(
        verdict.detail.contains(&repo.display().to_string()),
        "the detail must name the crate the verdict is about, so a reader can check \
         it: {verdict:?}"
    );
}

/// COUNTER-TEST for the verdict, and the one that keeps the rule from becoming
/// "everything is a library": a bin-only crate is still not one. Passes before
/// and after.
#[test]
fn a_bin_only_crate_is_still_not_a_library() {
    let tmp = bin_crate_with_dead_code_inside_and_outside_a_subdirectory();
    assert_eq!(cargo_library_target(tmp.path()).verdict, "not-a-library");
    assert_eq!(
        cargo_library_target(&tmp.path().join("src/inner")).verdict,
        "not-a-library",
        "a subdirectory of a bin crate is inside a bin crate"
    );
}

/// …and a path with no crate above it gets no verdict at all. "not-a-library"
/// is a decision; the absence of a manifest is the absence of one.
#[test]
fn a_path_with_no_enclosing_crate_has_an_undetermined_verdict() {
    let tmp = tempfile::Builder::new()
        .prefix("dcnov")
        .tempdir()
        .expect("tempdir");
    let verdict = cargo_library_target(tmp.path());
    assert_eq!(verdict.verdict, "undetermined", "{verdict:?}");
    assert!(
        verdict.detail.contains("no Cargo.toml"),
        "the reason must name what was missing: {verdict:?}"
    );
}

/// `--path` naming a single FILE. Scope and naming are different questions and
/// a file answers them differently: it is in scope only if it IS that file, but
/// a row named relative to itself is the empty string — a row that names no
/// file, next to a `total_lines: 0` that reads as a measurement.
#[tokio::test]
async fn a_single_file_request_names_the_file_it_reports() {
    let tmp = lib_crate_with_dead_code_in_a_subdirectory();

    let outcome = run_dead_code_analysis_with_filters(
        &tmp.path().join("src/inner/mod.rs"),
        filters(),
        std::time::Duration::from_secs(600),
    )
    .await
    .expect("the analysis runs");

    assert_eq!(outcome.report.files.len(), 1, "{:?}", outcome.report.files);
    let row = &outcome.report.files[0];
    assert_eq!(
        row.path, "mod.rs",
        "a row that names no file cannot be acted on: {row:?}"
    );
    assert!(
        row.total_lines > 0,
        "the file was reported but its length was not measured: {row:?}"
    );
    assert_eq!(outcome.report.summary.dead_functions, 1);
    assert_eq!(outcome.report.summary.dead_classes, 1);
}

/// …and a sibling file in the same directory is NOT in scope, which is what
/// naming rows against the parent directory could quietly have admitted.
#[tokio::test]
async fn a_single_file_request_excludes_its_siblings() {
    let tmp = bin_crate_with_dead_code_inside_and_outside_a_subdirectory();
    std::fs::write(
        tmp.path().join("src/inner/sibling.rs"),
        "pub fn sibling_dead() -> i32 {\n    5\n}\n",
    )
    .expect("write sibling.rs");
    std::fs::write(
        tmp.path().join("src/inner/mod.rs"),
        "pub mod sibling;\n\npub fn used() -> i32 {\n    1\n}\n\n\
         fn dead_private_helper() -> i32 {\n    7\n}\n",
    )
    .expect("rewrite inner/mod.rs");

    let outcome = run_dead_code_analysis_with_filters(
        &tmp.path().join("src/inner/mod.rs"),
        filters(),
        std::time::Duration::from_secs(600),
    )
    .await
    .expect("the analysis runs");

    let names: Vec<String> = outcome
        .report
        .files
        .iter()
        .flat_map(|f| f.items.iter().map(|i| i.name.clone()))
        .collect();
    assert!(
        names.contains(&"dead_private_helper".to_string()),
        "the requested file's own dead code must be reported: {names:?}"
    );
    assert!(
        !names.contains(&"sibling_dead".to_string()),
        "a sibling in the same directory is not the requested file: {names:?}"
    );
}
