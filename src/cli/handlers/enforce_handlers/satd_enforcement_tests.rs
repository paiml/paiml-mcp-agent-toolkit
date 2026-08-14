//! `enforce` must count SATD it finds.
//!
//! `run_satd_analysis` used to return `Vec::new()` on every path — it ran the
//! printing handler for its side effect and discarded the result — so
//! `pmat enforce extreme` printed "Found 40 SATD items" and then reported
//! `State: Complete / Score: 1.00/1.00 / Violations: 0`.

use super::analysis::run_satd_analysis;
use super::types::QualityProfile;

fn write_crate(dir: &std::path::Path, body: &str) {
    std::fs::create_dir_all(dir.join("src")).expect("mkdir");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write manifest");
    std::fs::write(dir.join("src/lib.rs"), body).expect("write source");
}

#[tokio::test]
async fn satd_markers_become_violations() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_crate(
        dir.path(),
        "// FIXME: broken\n// HACK: workaround\npub fn f() -> i32 { 1 }\n",
    );

    let violations = run_satd_analysis(dir.path(), &QualityProfile::default(), None)
        .await
        .expect("analysis runs")
        .violations;

    assert!(
        !violations.is_empty(),
        "an enforcer that finds SATD must report it"
    );
    assert!(violations.iter().all(|v| v.violation_type == "satd"));
    // Each violation must be locatable, not just a count.
    assert!(violations
        .iter()
        .all(|v| v.location.contains("lib.rs") && v.location.matches(':').count() >= 2));
}

#[tokio::test]
async fn clean_code_produces_no_violations() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_crate(
        dir.path(),
        "//! Clean.\n\n/// Adds.\npub fn f() -> i32 { 1 }\n",
    );

    let violations = run_satd_analysis(dir.path(), &QualityProfile::default(), None)
        .await
        .expect("analysis runs")
        .violations;

    assert!(violations.is_empty(), "got {violations:?}");
}

/// An unreadable path must not be reported as clean.
#[tokio::test]
async fn an_unanalysable_path_errors_rather_than_reporting_zero_debt() {
    let missing = std::path::Path::new("/nonexistent/pmat/satd/probe");
    let outcome = run_satd_analysis(missing, &QualityProfile::default(), None)
        .await
        .expect("the phase reports rather than aborting the run");

    // The distinction that matters: no violations, but NOT measured — so the
    // verdict cannot count this as a clean phase.
    assert!(outcome.violations.is_empty());
    assert!(
        !outcome.is_measured(),
        "a path that cannot be analysed must report as unmeasured, not clean"
    );
}

/// `--file` means THIS file.
///
/// `AnalysisScope::walk_root` hands directory phases the file's parent module
/// dir, and SATD was treated as a directory phase — so a sibling's `// TODO`
/// was attributed to the named file. `enforce extreme --file good.rs --ci-mode`
/// exited 1 reporting one violation whose own location was `bad.rs`: a clean
/// file failing CI on code it does not contain.
#[tokio::test]
async fn file_scope_ignores_a_dirty_sibling() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).expect("mkdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    let good = src.join("good.rs");
    std::fs::write(&good, "//! clean\n/// g\npub fn g(a: i32) -> i32 { a }\n").expect("good");
    std::fs::write(
        src.join("bad.rs"),
        "//! dirty\n// TODO: fix me later\n/// h\npub fn h(a: i32) -> i32 { a }\n",
    )
    .expect("bad");

    let clean = run_satd_analysis(&src, &QualityProfile::default(), Some(&good))
        .await
        .expect("analysis runs");
    assert!(
        clean.violations.is_empty(),
        "the named file is clean; a sibling's TODO is not its violation: {:?}",
        clean.violations
    );
    assert!(clean.is_measured(), "the file was read, so this phase ran");

    // ...and the sibling's own debt is still found when IT is the named file,
    // so the scoping did not simply stop looking.
    let dirty = run_satd_analysis(&src, &QualityProfile::default(), Some(&src.join("bad.rs")))
        .await
        .expect("analysis runs");
    assert!(!dirty.violations.is_empty());
    assert!(
        dirty
            .violations
            .iter()
            .all(|v| v.location.contains("bad.rs")),
        "every violation must name the file it came from: {:?}",
        dirty.violations
    );
}

/// A `--file` target that cannot be read is unmeasured, not clean.
#[tokio::test]
async fn an_unreadable_file_target_is_unmeasured() {
    let missing = std::path::Path::new("/nonexistent/pmat/satd/file.rs");
    let outcome = run_satd_analysis(
        missing.parent().expect("parent"),
        &QualityProfile::default(),
        Some(missing),
    )
    .await
    .expect("the phase reports rather than aborting");

    assert!(outcome.violations.is_empty());
    assert!(!outcome.is_measured());
}
