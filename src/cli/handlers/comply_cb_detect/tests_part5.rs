#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;

// ============================================================================
// CB-529: .pmat/ Tracked in Git
// ============================================================================
mod cb529_tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init", "--initial-branch=main"])
            .current_dir(dir)
            .output()
            .expect("git init");
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .expect("git config email");
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .expect("git config name");
    }

    fn git_add_commit(dir: &Path, files: &[&str]) {
        for f in files {
            Command::new("git")
                .args(["add", f])
                .current_dir(dir)
                .output()
                .expect("git add");
        }
        Command::new("git")
            .args(["commit", "-m", "init", "--allow-empty"])
            .current_dir(dir)
            .output()
            .expect("git commit");
    }

    #[test]
    fn test_cb529_detects_root_pmat_files() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        init_git_repo(root);

        fs::create_dir_all(root.join(".pmat")).unwrap();
        fs::write(root.join(".pmat/context.db"), "fake").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        git_add_commit(root, &[".pmat/context.db", "Cargo.toml"]);

        let violations = detect_cb529_pmat_tracked_in_git(root);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-529");
        assert!(violations[0].file.contains(".pmat/context.db"));
        assert_eq!(violations[0].severity, Severity::Error);
    }

    #[test]
    fn test_cb529_detects_subcrate_pmat_files() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        init_git_repo(root);

        fs::create_dir_all(root.join("crates/foo/.pmat")).unwrap();
        fs::write(root.join("crates/foo/.pmat/context.idx"), "data").unwrap();
        fs::write(root.join("crates/foo/.pmat/dead-code-cache.json"), "{}").unwrap();
        git_add_commit(
            root,
            &[
                "crates/foo/.pmat/context.idx",
                "crates/foo/.pmat/dead-code-cache.json",
            ],
        );

        let violations = detect_cb529_pmat_tracked_in_git(root);
        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|v| v.pattern_id == "CB-529"));
        assert!(violations.iter().all(|v| v.file.contains("/.pmat/")));
    }

    #[test]
    fn test_cb529_passes_clean_repo() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        init_git_repo(root);

        fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        fs::write(root.join("README.md"), "# Test\n").unwrap();
        git_add_commit(root, &["Cargo.toml", "README.md"]);

        let violations = detect_cb529_pmat_tracked_in_git(root);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb529_ignores_pmat_in_filenames() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        init_git_repo(root);

        // A file named "some.pmat_config" is NOT a .pmat/ directory artifact
        fs::write(root.join("some.pmat_config"), "data").unwrap();
        git_add_commit(root, &["some.pmat_config"]);

        let violations = detect_cb529_pmat_tracked_in_git(root);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb529_not_git_repo() {
        let temp = TempDir::new().unwrap();
        let violations = detect_cb529_pmat_tracked_in_git(temp.path());
        assert!(violations.is_empty());
    }
}

// ============================================================================
// CB-528: Division-by-length Without Empty Guard
// ============================================================================
mod cb528_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb528_detects_unguarded_division_by_len() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("stats.rs"),
            r#"
fn mean(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}
"#,
        )
        .unwrap();

        let violations = detect_cb528_division_by_length(temp.path());
        assert_eq!(
            violations.len(),
            1,
            "should detect unguarded division by len"
        );
        assert!(violations[0].description.contains("Division by .len()"));
        assert_eq!(violations[0].pattern_id, "CB-528");
    }

    #[test]
    fn test_cb528_allows_guarded_division_with_is_empty() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("stats.rs"),
            r#"
fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}
"#,
        )
        .unwrap();

        let violations = detect_cb528_division_by_length(temp.path());
        assert!(
            violations.is_empty(),
            "should not flag when is_empty guard present"
        );
    }

    #[test]
    fn test_cb528_allows_guarded_division_with_max_1() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("stats.rs"),
            r#"
fn diversity(items: &[String], total: &[String]) -> f32 {
    items.len() as f32 / total.len().max(1) as f32
}
"#,
        )
        .unwrap();

        let violations = detect_cb528_division_by_length(temp.path());
        assert!(
            violations.is_empty(),
            "should not flag when .max(1) guard present"
        );
    }

    #[test]
    fn test_cb528_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
fn production() {}

#[cfg(test)]
mod tests {
    fn test_mean() {
        let sum = 10.0;
        let result = sum / vec![1,2,3].len() as f64;
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb528_division_by_length(temp.path());
        assert!(violations.is_empty(), "should not flag test code");
    }

    #[test]
    fn test_cb528_allows_len_gt_zero_guard() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("stats.rs"),
            r#"
fn mean(values: &[f64]) -> f64 {
    if values.len() > 0 {
        values.iter().sum::<f64>() / values.len() as f64
    } else {
        0.0
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb528_division_by_length(temp.path());
        assert!(
            violations.is_empty(),
            "should not flag when len > 0 guard present"
        );
    }
}

// ============================================================================
// CB-530: Log Without Clamp Guard
// ============================================================================
mod cb530_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb530_detects_bare_ln() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("loss.rs"),
            r#"
fn cross_entropy(probs: &[f32], labels: &[usize]) -> f32 {
    let mut total = 0.0;
    for &label in labels {
        total -= probs[label].ln();
    }
    total
}
"#,
        )
        .unwrap();

        let violations = detect_cb530_log_without_clamp(temp.path());
        assert_eq!(
            violations.len(),
            1,
            "should detect bare .ln() without guard"
        );
        assert!(violations[0].description.contains("ln()"));
        assert_eq!(violations[0].pattern_id, "CB-530");
    }

    #[test]
    fn test_cb530_allows_max_guarded_ln() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("loss.rs"),
            r#"
fn cross_entropy(probs: &[f32], labels: &[usize]) -> f32 {
    let mut total = 0.0;
    for &label in labels {
        total -= probs[label].max(1e-10).ln();
    }
    total
}
"#,
        )
        .unwrap();

        let violations = detect_cb530_log_without_clamp(temp.path());
        assert!(
            violations.is_empty(),
            "should not flag when .max(eps) guard present"
        );
    }

    #[test]
    fn test_cb530_allows_clamp_guarded_log() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("info.rs"),
            r#"
fn entropy(p: f64) -> f64 {
    let safe_p = p.clamp(1e-15, 1.0);
    -safe_p * safe_p.ln()
}
"#,
        )
        .unwrap();

        let violations = detect_cb530_log_without_clamp(temp.path());
        // The clamp is on a preceding line — should be caught by context check
        // Actually .ln() is on safe_p which was clamped, but our detector checks same line
        // This tests that we look at preceding lines for guards
        assert!(
            violations.is_empty(),
            "should not flag when .clamp() guard on preceding line"
        );
    }

    #[test]
    fn test_cb530_allows_positive_literal_log() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("math.rs"),
            r#"
fn log_base_2() -> f64 {
    let ln2 = 2.0_f64.ln();
    ln2
}
"#,
        )
        .unwrap();

        let violations = detect_cb530_log_without_clamp(temp.path());
        assert!(
            violations.is_empty(),
            "should not flag positive literal .ln()"
        );
    }

    #[test]
    fn test_cb530_detects_log2() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("bits.rs"),
            r#"
fn bit_entropy(freq: f32) -> f32 {
    -freq * freq.log2()
}
"#,
        )
        .unwrap();

        let violations = detect_cb530_log_without_clamp(temp.path());
        assert_eq!(violations.len(), 1, "should detect bare .log2()");
        assert!(violations[0].description.contains("log2()"));
    }

    #[test]
    fn test_cb530_skips_test_code() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("lib.rs"),
            r#"
fn production() {}

#[cfg(test)]
mod tests {
    fn test_log() {
        let x = 0.5_f32.ln();
    }
}
"#,
        )
        .unwrap();

        let violations = detect_cb530_log_without_clamp(temp.path());
        assert!(violations.is_empty(), "should not flag test code");
    }

    #[test]
    fn test_cb530_allows_log_of_one_plus_x() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("mel.rs"),
            r#"
fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}
"#,
        )
        .unwrap();

        let violations = detect_cb530_log_without_clamp(temp.path());
        assert!(
            violations.is_empty(),
            "should not flag log of (1.0 + x) pattern"
        );
    }
}
