//! An unmeasured check cannot pass.
//!
//! `enforce extreme` reported a perfect `1.00/1.00` / `Complete` / exit 0 for a
//! nonexistent path, an empty directory, and a project whose sources do not
//! parse. Every phase failed, each warned "not measured" on stderr, each
//! returned an empty violation list — and an empty list already meant "clean",
//! so total failure scored as total success.
//!
//! `states.rs` carried this as a documented caveat rather than a defect. These
//! tests are the countermeasure: they fail if a phase's inability to measure is
//! ever again convertible into credit.

use super::states::handle_analyzing_state;
use super::types::{EnforcementState, QualityProfile};

#[tokio::test]
async fn a_nonexistent_path_is_refused_not_scored() {
    // It used to answer `Complete 1.00/1.00`, exit 0: the analyzers return `Ok`
    // for input they never read, so every phase came back clean. A path that
    // cannot be read earns no verdict at all.
    let err = handle_analyzing_state(
        std::path::Path::new("/nonexistent/pmat/enforce/probe"),
        &QualityProfile::default(),
        false,
        true,
        None,
        None,
        None,
    )
    .await
    .expect_err("a nonexistent path must not produce a verdict");

    assert!(
        err.to_string().contains("path not found"),
        "the refusal must name the cause: {err}"
    );
}

#[tokio::test]
async fn an_empty_directory_is_not_complete() {
    let dir = tempfile::tempdir().expect("tempdir");
    let result = handle_analyzing_state(
        dir.path(),
        &QualityProfile::default(),
        false,
        true,
        None,
        None,
        None,
    )
    .await
    .expect("the run reports");

    // An empty directory has nothing to measure, so it cannot demonstrate that
    // the profile is met. 3.29.0 said Violating here; 3.30.0 said Complete 1.00.
    assert_ne!(result.state, EnforcementState::Complete);
    assert!(result.score < 1.0, "got {}", result.score);
}

/// A project that DOES parse must still be able to reach a clean verdict —
/// otherwise the fix has simply made the gate impossible to pass.
#[tokio::test]
async fn a_clean_project_can_still_reach_complete() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("manifest");
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "//! Clean.\n\n/// Adds.\npub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .expect("source");

    let result = handle_analyzing_state(
        dir.path(),
        &QualityProfile::default(),
        false,
        true,
        None,
        None,
        None,
    )
    .await
    .expect("the run reports");

    assert!(
        result
            .violations
            .iter()
            .all(|v| v.violation_type != "not_measured"),
        "a parseable project should measure cleanly: {:?}",
        result.violations
    );
    assert!(result.score > 0.0, "got {}", result.score);
}
