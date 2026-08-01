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

    /// Rows produced by the diversity threshold, identified by the annotation
    /// only that row carries.
    fn diversity_rows(
        violations: &[crate::cli::analysis_utilities::QualityViolation],
    ) -> Vec<&crate::cli::analysis_utilities::QualityViolation> {
        violations
            .iter()
            .filter(|v| v.message.contains("Low pattern diversity"))
            .collect()
    }

    /// #683: a zero diversity requirement can never be unmet, so it must not
    /// raise a diversity violation.
    ///
    /// UPDATED (round 3): this used to assert the whole result was empty, which
    /// pinned a worse defect — `--min-entropy` acted as a master switch over
    /// findings it has nothing to do with. On the real tree `--min-entropy 0.3`
    /// against a measured 62.6% diversity suppressed six High-severity concrete
    /// repetition violations and certified "Total violations: 0" for code
    /// `analyze entropy` flags 14 times. The flag gates the diversity finding;
    /// repetition findings answer to `max_pattern_repetition`.
    #[tokio::test]
    async fn test_zero_threshold_raises_no_diversity_violation() {
        let project = repetitive_project(6);

        let violations = check_entropy(project.path(), 0.0).await.expect("check");

        assert!(
            diversity_rows(&violations).is_empty(),
            "min_entropy 0.0 requires zero diversity and can never be unmet, \
             got: {:?}",
            violations.iter().map(|v| &v.message).collect::<Vec<_>>()
        );
    }

    /// A concrete repetition finding is not suppressed by a lenient diversity
    /// threshold: the same repetition rows appear at 0.0 and at 0.99.
    #[tokio::test]
    async fn test_repetition_findings_survive_a_lenient_diversity_threshold() {
        let project = repetitive_project(6);

        let at_zero = check_entropy(project.path(), 0.0).await.expect("check");
        let at_high = check_entropy(project.path(), 0.99).await.expect("check");

        let reps_at_zero: Vec<_> = at_zero
            .iter()
            .filter(|v| !v.message.contains("Low pattern diversity"))
            .map(|v| v.message.clone())
            .collect();
        let reps_at_high: Vec<_> = at_high
            .iter()
            .filter(|v| !v.message.contains("Low pattern diversity"))
            .map(|v| v.message.clone())
            .collect();

        assert!(
            !reps_at_zero.is_empty(),
            "a tree with 8-arm if/else chains repeated across 6 files has \
             repetition findings regardless of the diversity threshold"
        );
        assert_eq!(
            reps_at_zero, reps_at_high,
            "the diversity threshold must not change repetition findings"
        );
    }

    /// #683: the threshold must actually gate, so a demanding threshold on the
    /// same project must produce a violation that a zero threshold does not.
    #[tokio::test]
    async fn test_threshold_changes_the_outcome() {
        let project = repetitive_project(6);

        let at_zero = check_entropy(project.path(), 0.0).await.expect("check");
        let at_high = check_entropy(project.path(), 0.99).await.expect("check");

        assert!(diversity_rows(&at_zero).is_empty());
        assert_eq!(
            diversity_rows(&at_high).len(),
            1,
            "a 99% diversity requirement must not be silently satisfied"
        );
        assert!(at_high.len() > at_zero.len());
    }

    /// #683: the diversity row names the threshold that was applied and the value
    /// it was compared against.
    ///
    /// UPDATED (round 3): the loop used to require this annotation on *every*
    /// violation, which pinned the defect of stamping "[pattern diversity 62.6% <
    /// required 30%]" onto an `ApiCall pattern repeated 17 times` row that the
    /// diversity threshold did not produce.
    #[tokio::test]
    async fn test_diversity_violation_names_the_applied_threshold() {
        let project = repetitive_project(6);

        let violations = check_entropy(project.path(), 0.99).await.expect("check");
        let rows = diversity_rows(&violations);

        assert_eq!(rows.len(), 1);
        for violation in rows {
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

    /// The flag echo must name the value the user typed. `--min-entropy 0.999`
    /// rendered as "required 99.9% (--min-entropy 1.00)": one sentence naming two
    /// different numbers, the second of which was never passed.
    #[tokio::test]
    async fn test_flag_echo_is_not_rounded_to_two_decimals() {
        let project = repetitive_project(6);

        let violations = check_entropy(project.path(), 0.999).await.expect("check");
        let rows = diversity_rows(&violations);

        assert_eq!(rows.len(), 1);
        let message = &rows[0].message;
        assert!(
            message.contains("--min-entropy 0.999"),
            "flag echo must be the value passed, got: {message}"
        );
        assert!(
            !message.contains("--min-entropy 1"),
            "flag echo must not round 0.999 up to 1, got: {message}"
        );
    }

    /// `analyze entropy` and `quality-gate --checks entropy` must look at the
    /// same files, or they report different values for the same metric on the
    /// same tree (939 vs 1328 files; 62.6% vs 63.9% diversity).
    #[test]
    fn test_gate_and_analyze_share_one_exclusion_list() {
        use crate::entropy::EntropyConfig;

        let analyze = crate::cli::handlers::analysis_handlers::create_entropy_config(
            crate::cli::EntropySeverity::Medium,
            false,
        );
        let shared = EntropyConfig::analysis_excludes(false);

        assert_eq!(
            analyze.exclude_paths, shared,
            "analyze entropy must use the shared exclusion list"
        );
        assert!(
            shared.iter().any(|p| p == "**/*test*.rs"),
            "test-named files are excluded by both entry points"
        );
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
