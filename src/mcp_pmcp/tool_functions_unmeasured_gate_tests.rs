//! A gate that could not measure must not report a pass — over a FILE too.
//!
//! `quality_gate` answered
//! `{"passed":true,"score":null,"grade":null,"not_measured":["score","grade"],
//! "files_analyzed":0,"violations":[]}` for a single `a.sh`, `a.php`, `a.md`,
//! `a.cs` or `a.zig`: a green verdict over input the same payload declares
//! unmeasured. The identical "nothing was graded" state already returned
//! `passed:false` with a `not_graded` violation for a DIRECTORY, so one tool
//! answered the same question two ways depending on whether the path it was
//! handed happened to be a file.

use super::{check_quality_gate_file, check_quality_gates};
use std::path::PathBuf;

/// Extensions TDG has no grade for. All five were observed passing.
const UNGRADED: [&str; 5] = ["sh", "php", "md", "cs", "zig"];

fn violation_types(json: &serde_json::Value) -> Vec<String> {
    json["violations"]
        .as_array()
        .expect("violations is an array")
        .iter()
        .map(|v| v["check_type"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[tokio::test]
async fn a_file_with_no_grade_does_not_pass_on_either_entry_point() {
    for ext in UNGRADED {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join(format!("a.{ext}"));
        // Deliberately debt-free: the failure must come from the absence of a
        // measurement, not from a SATD marker the detector happened to find.
        std::fs::write(&file, "echo hi\n").expect("write fixture");

        let paths = vec![file.clone()];
        let by_paths = check_quality_gates(&paths, false)
            .await
            .expect("quality_gate reports");
        let by_file = check_quality_gate_file(&file, false)
            .await
            .expect("quality_gate_file reports");

        for (name, json) in [("paths", &by_paths), ("file", &by_file)] {
            assert!(
                json["score"].is_null() && json["grade"].is_null(),
                ".{ext} via {name}: fixture must be one TDG does not grade, got {json}"
            );
            assert_eq!(
                json["passed"], serde_json::Value::Bool(false),
                ".{ext} via {name}: a gate declaring score and grade unmeasured must not pass: {json}"
            );
            assert!(
                violation_types(json).iter().any(|t| t == "not_graded"),
                ".{ext} via {name}: the hole in the verdict must be a violation the client can read: {json}"
            );
        }
    }
}

/// The rule the file path was missing is the directory path's rule. Pin that
/// they still agree, so a future fix to one cannot re-split them.
#[tokio::test]
async fn a_file_and_the_directory_holding_it_give_the_same_verdict() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("only.sh");
    std::fs::write(&file, "echo hi\n").expect("write fixture");

    let as_file = check_quality_gates(&[file], false)
        .await
        .expect("quality_gate reports");
    let as_dir = check_quality_gates(&[dir.path().to_path_buf()], false)
        .await
        .expect("quality_gate reports");

    assert_eq!(
        as_file["passed"], as_dir["passed"],
        "same content, same measurement, two verdicts: file={as_file} dir={as_dir}"
    );
}

/// A file that DOES grade must still be able to reach a pass — otherwise the
/// fix has merely made the gate impossible to satisfy.
#[tokio::test]
async fn a_graded_clean_file_still_passes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("clean.rs");
    std::fs::write(
        &file,
        "//! Clean.\n\n/// Adds.\npub fn a(x: i32) -> i32 { x + 1 }\n",
    )
    .expect("write fixture");

    let paths: Vec<PathBuf> = vec![file.clone()];
    let by_paths = check_quality_gates(&paths, false).await.expect("reports");
    let by_file = check_quality_gate_file(&file, false)
        .await
        .expect("reports");

    assert_eq!(
        by_paths["passed"],
        serde_json::Value::Bool(true),
        "a clean graded file must still pass: {by_paths}"
    );
    assert_eq!(
        by_file["passed"],
        serde_json::Value::Bool(true),
        "a clean graded file must still pass: {by_file}"
    );
}
