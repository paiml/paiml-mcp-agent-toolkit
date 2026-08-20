#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for target selection.
//!
//! `select_top_targets` used to hard-code a limit of 10 files regardless of
//! configuration, so `max_targets` is exercised here with values on both sides
//! of that former constant.

use super::*;

fn scores(n: usize) -> std::collections::HashMap<PathBuf, f64> {
    (0..n)
        .map(|i| (PathBuf::from(format!("src/f{:02}.rs", i)), i as f64))
        .collect()
}

fn service_with_max_targets(max_targets: usize) -> CoverageImprovementService {
    CoverageImprovementService::new(CoverageImprovementConfig {
        max_targets,
        ..CoverageImprovementConfig::default()
    })
}

#[test]
fn test_max_targets_below_former_hardcoded_ten_is_honored() {
    let targets = service_with_max_targets(3).select_top_targets(scores(15));

    assert_eq!(
        targets.len(),
        3,
        "max_targets=3 must return exactly 3 files"
    );
    // Highest score first: f14 (14.0), f13, f12.
    assert_eq!(
        targets,
        vec![
            PathBuf::from("src/f14.rs"),
            PathBuf::from("src/f13.rs"),
            PathBuf::from("src/f12.rs"),
        ]
    );
}

#[test]
fn test_max_targets_above_former_hardcoded_ten_is_honored() {
    let targets = service_with_max_targets(12).select_top_targets(scores(15));

    assert_eq!(
        targets.len(),
        12,
        "max_targets=12 must return 12 files, not the former hard-coded 10"
    );
}

#[test]
fn test_max_targets_zero_means_no_limit() {
    let targets = service_with_max_targets(0).select_top_targets(scores(15));

    assert_eq!(targets.len(), 15, "max_targets=0 must return every file");
}

#[test]
fn test_default_max_targets_is_ten() {
    let targets = CoverageImprovementService::new(CoverageImprovementConfig::default())
        .select_top_targets(scores(15));

    assert_eq!(
        targets.len(),
        10,
        "default preserves the previous behaviour of 10 targets"
    );
}

#[test]
fn test_fewer_files_than_max_targets_returns_all() {
    let targets = service_with_max_targets(10).select_top_targets(scores(4));

    assert_eq!(targets.len(), 4);
}

#[test]
fn test_ties_broken_by_path_for_determinism() {
    let mut tied = std::collections::HashMap::new();
    for name in ["src/c.rs", "src/a.rs", "src/b.rs"] {
        tied.insert(PathBuf::from(name), 1.0);
    }

    let first = service_with_max_targets(2).select_top_targets(tied.clone());
    let second = service_with_max_targets(2).select_top_targets(tied);

    assert_eq!(first, second, "equal scores must produce a stable order");
    assert_eq!(
        first,
        vec![PathBuf::from("src/a.rs"), PathBuf::from("src/b.rs")]
    );
}
