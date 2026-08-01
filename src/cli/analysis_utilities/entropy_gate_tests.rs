//! Regression tests for the entropy quality-gate contract (#683).
//!
//! On the shipped artifact `quality-gate --checks entropy --min-entropy 0.0`
//! ("require zero diversity", which cannot be unmet) reported `Status: FAILED`,
//! exactly as `--min-entropy 0.99` did, and no emitted violation named any
//! threshold: `[warning] entropy: ApiCall pattern repeated 10 times (saves 302
//! lines)`.
//!
//! NOTE: `tests_core_part1.rs` also contains entropy tests, but that file is not
//! included by any module and therefore never compiles or runs — these live here
//! so they actually execute.

#[cfg(test)]
mod entropy_gate_tests {
    use super::super::check_entropy;
    use tempfile::TempDir;

    /// A project with enough structural repetition to produce real violations.
    fn repetitive_project(file_count: usize) -> TempDir {
        let temp_dir = TempDir::new().expect("tempdir");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("create src");
        for f in 0..file_count {
            let mut body =
                String::from("pub fn dispatch(v: i32) -> i32 {\n    if v == 0 {\n        0\n");
            for i in 1..8 {
                body.push_str(&format!("    }} else if v == {i} {{\n        {i}\n"));
            }
            body.push_str("    } else {\n        -1\n    }\n}\n");
            std::fs::write(src_dir.join(format!("m{f}.rs")), body).expect("write");
        }
        temp_dir
    }

    /// #683: a zero requirement can never be unmet, so the check must pass.
    #[tokio::test]
    async fn test_zero_threshold_never_reports_violations() {
        let project = repetitive_project(6);

        let violations = check_entropy(project.path(), 0.0).await.expect("check");

        assert!(
            violations.is_empty(),
            "min_entropy 0.0 requires zero diversity and can never be unmet, \
             got {} violations, first: {:?}",
            violations.len(),
            violations.first().map(|v| v.message.clone())
        );
    }

    /// #683: the threshold must actually gate, so a demanding threshold on the
    /// same project must produce violations that a zero threshold does not.
    #[tokio::test]
    async fn test_threshold_changes_the_outcome() {
        let project = repetitive_project(6);

        let at_zero = check_entropy(project.path(), 0.0).await.expect("check");
        let at_high = check_entropy(project.path(), 0.99).await.expect("check");

        assert!(at_zero.is_empty());
        assert!(
            !at_high.is_empty(),
            "a 99% diversity requirement must not be silently satisfied"
        );
    }

    /// #683: every violation names the threshold that was applied and the value
    /// it was compared against.
    #[tokio::test]
    async fn test_violations_name_the_applied_threshold() {
        let project = repetitive_project(6);

        let violations = check_entropy(project.path(), 0.99).await.expect("check");

        assert!(!violations.is_empty());
        for violation in &violations {
            assert!(
                violation.message.contains("--min-entropy 0.99"),
                "violation must name the applied threshold, got: {}",
                violation.message
            );
            assert!(
                violation.message.contains("required 99.0%"),
                "violation must state what was required, got: {}",
                violation.message
            );
            assert!(
                violation.message.contains("pattern diversity"),
                "violation must state what was measured, got: {}",
                violation.message
            );
        }
    }

    /// NONDETERMINISM: `quality-gate --checks entropy` reported '- Entropy: 8',
    /// then '6', then '7' across three runs on one path. Five runs, one answer.
    #[tokio::test]
    async fn test_entropy_check_is_deterministic_across_five_runs() {
        let project = repetitive_project(10);

        let mut runs = Vec::new();
        for _ in 0..5 {
            let violations = check_entropy(project.path(), 0.99).await.expect("check");
            runs.push(
                violations
                    .iter()
                    .map(|v| (v.severity.clone(), v.file.clone(), v.message.clone()))
                    .collect::<Vec<_>>(),
            );
        }

        for (i, run) in runs.iter().enumerate().skip(1) {
            assert_eq!(
                runs[0].len(),
                run.len(),
                "run {i} reported {} violations, run 0 reported {}",
                run.len(),
                runs[0].len()
            );
            assert_eq!(
                &runs[0], run,
                "run {i} disagreed with run 0 on identical input"
            );
        }
    }
}
