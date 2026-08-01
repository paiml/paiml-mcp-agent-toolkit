//! Regression tests for `analyze tdg -n/--top-files` and for the
//! nondeterministic key order of the TDG project JSON.
//!
//! Observed defects (round 3):
//!   * `-n 5`, `-n 10` and `-n 100000` produced identical output, and `-n 3` on
//!     a 1593-file tree still emitted all 1593 entries: the flag was parsed
//!     into a discarded binding.
//!   * 6 runs of `analyze tdg --format json` on an unchanged tree gave 6
//!     different `grade_distribution` key orders (values stable, sum 495).

use super::grade::Grade;
use super::language_simple::Language;
use super::project_score::ProjectScore;
use super::score::TdgScore;
use std::path::PathBuf;

/// Build `n` scored files with descending totals so "worst" is unambiguous.
fn scores(n: usize) -> Vec<TdgScore> {
    (0..n)
        .map(|i| {
            let total = 100.0 - i as f32;
            TdgScore {
                total,
                grade: Grade::from_score(total),
                language: Language::Rust,
                file_path: Some(PathBuf::from(format!("src/f{i:03}.rs"))),
                ..TdgScore::default()
            }
        })
        .collect()
}

#[test]
fn test_top_files_truncates_to_the_worst_n() {
    let mut project = ProjectScore::aggregate(scores(50));
    project.limit_to_worst_files(5);

    assert_eq!(project.files.len(), 5, "--top-files 5 must list 5 files");
    assert_eq!(project.files_reported, 5);
    assert!(project.files_truncated);
    // Worst first: totals 51..=55 are the five lowest of 51..=100.
    let totals: Vec<f32> = project.files.iter().map(|f| f.total).collect();
    assert_eq!(totals, vec![51.0, 52.0, 53.0, 54.0, 55.0]);
}

#[test]
fn test_different_limits_give_different_lists() {
    // The exact symptom: -n 5 / -n 10 / -n 100000 were indistinguishable.
    let lens: Vec<usize> = [5usize, 10, 100_000]
        .iter()
        .map(|limit| {
            let mut project = ProjectScore::aggregate(scores(50));
            project.limit_to_worst_files(*limit);
            project.files.len()
        })
        .collect();
    assert_eq!(lens, vec![5, 10, 50]);
}

#[test]
fn test_zero_means_all_files() {
    let mut project = ProjectScore::aggregate(scores(7));
    project.limit_to_worst_files(0);
    assert_eq!(project.files.len(), 7);
    assert!(!project.files_truncated);
    assert_eq!(project.files_reported, 7);
}

#[test]
fn test_truncation_never_rewrites_the_project_aggregates() {
    let full = ProjectScore::aggregate(scores(50));
    let mut truncated = ProjectScore::aggregate(scores(50));
    truncated.limit_to_worst_files(3);

    // A cap on the list must not become a cap on the totals.
    assert_eq!(truncated.total_files, full.total_files);
    assert_eq!(truncated.total_files, 50);
    assert_eq!(truncated.average_score, full.average_score);
    assert_eq!(truncated.grade_distribution, full.grade_distribution);
    assert_eq!(truncated.language_distribution, full.language_distribution);
    assert_eq!(
        truncated.grade_distribution.values().sum::<usize>(),
        truncated.total_files,
        "grade distribution must still cover every analysed file"
    );
}

#[test]
fn test_truncated_json_names_both_numbers() {
    let mut project = ProjectScore::aggregate(scores(20));
    project.limit_to_worst_files(4);
    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&project).expect("serialize"))
            .expect("parse back");

    assert_eq!(json["total_files"], 20);
    assert_eq!(json["files_reported"], 4);
    assert_eq!(json["files_truncated"], true);
    assert_eq!(
        json["files"].as_array().expect("files array").len(),
        json["files_reported"].as_u64().expect("count") as usize,
        "the count must agree with the list it heads"
    );
}

#[test]
fn test_project_json_key_order_is_byte_identical_across_runs() {
    // >= 5 iterations: the HashMap-backed distributions used to reorder on
    // every run for identical input.
    let mut renders = Vec::new();
    for _ in 0..8 {
        let mut project = ProjectScore::aggregate(scores(40));
        project.limit_to_worst_files(10);
        renders.push(serde_json::to_string(&project).expect("serialize"));
    }
    let first = &renders[0];
    for (i, render) in renders.iter().enumerate() {
        assert_eq!(first, render, "run {i} differed from run 0");
    }
    // And the order is the documented best-to-worst grade order.
    let grades: Vec<String> = ProjectScore::aggregate(scores(40))
        .grade_distribution
        .keys()
        .map(std::string::ToString::to_string)
        .collect();
    let mut sorted = grades.clone();
    sorted.sort_by_key(|g| {
        ["A+", "A", "A-", "B+", "B", "B-", "C+", "C", "C-", "D", "F"]
            .iter()
            .position(|k| k == g)
            .unwrap_or(usize::MAX)
    });
    assert_eq!(grades, sorted);
}

#[test]
fn test_file_order_is_deterministic_regardless_of_input_order() {
    let forward = {
        let mut p = ProjectScore::aggregate(scores(30));
        p.limit_to_worst_files(0);
        serde_json::to_string(&p.files).expect("serialize")
    };
    let reversed = {
        let mut files = scores(30);
        files.reverse();
        let mut p = ProjectScore::aggregate(files);
        p.limit_to_worst_files(0);
        serde_json::to_string(&p.files).expect("serialize")
    };
    assert_eq!(
        forward, reversed,
        "file list order must not depend on filesystem walk order"
    );
}
