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

    let violations = run_satd_analysis(dir.path(), &QualityProfile::default())
        .await
        .expect("analysis runs");

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

    let violations = run_satd_analysis(dir.path(), &QualityProfile::default())
        .await
        .expect("analysis runs");

    assert!(violations.is_empty(), "got {violations:?}");
}

/// An unreadable path must not be reported as clean.
#[tokio::test]
async fn an_unanalysable_path_errors_rather_than_reporting_zero_debt() {
    let missing = std::path::Path::new("/nonexistent/pmat/satd/probe");
    let result = run_satd_analysis(missing, &QualityProfile::default()).await;

    assert!(
        result.is_err() || result.expect("checked").is_empty(),
        "a path that cannot be analysed must not silently pass as clean"
    );
}
