// PMAT Scorer integration tests (Category F: PMAT Compliance, 5 points).
// #1232: these were `#[ignore]`d TDD-RED placeholders ending in `panic!` that never ran. They now execute
// against the shipped PmatScorer. Partial-config / low-threshold scoring is NOT implemented by the scorer
// (any non-empty valid TOML scores F1 2.5 + F2 2.5); those two placeholders asserted behaviour that does not
// exist and were removed rather than weakened — threshold scoring is a feature, tracked separately.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod pmat_scorer_tests {
    use crate::services::repo_score::models::ScoreStatus;
    use crate::services::repo_score::scorers::{PmatScorer, Scorer, ScorerConfig};
    use crate::tests::repo_score::test_utils::*;

    async fn run(
        repo_path: &std::path::Path,
    ) -> crate::services::repo_score::models::CategoryScore {
        PmatScorer::new()
            .score(repo_path, &ScorerConfig::default())
            .await
            .expect("scorer must degrade gracefully, never error")
    }

    #[tokio::test]
    async fn test_pmat_no_config() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);

        let result = run(repo_path).await;

        // F1 = 0 (absent), F2 = 2.5 (nothing to violate)
        assert_eq!(result.score, 2.5);
        assert_eq!(result.max_score, 5.0);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains(".pmat-gates.toml not found")));
    }

    #[tokio::test]
    async fn test_pmat_perfect_config() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);
        create_pmat_gates(repo_path, PERFECT_PMAT_GATES);

        let result = run(repo_path).await;

        assert_eq!(result.score, 5.0);
        assert_eq!(result.max_score, 5.0);
        assert_eq!(result.percentage, 100.0);
        assert_eq!(result.status, ScoreStatus::Pass);
    }

    #[tokio::test]
    async fn test_pmat_quality_toml_alternative() {
        // #1232 second-order defect: pmat-quality.toml was documented as an alternative but never recognised.
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);
        std::fs::write(repo_path.join("pmat-quality.toml"), PERFECT_PMAT_GATES).unwrap();

        let result = run(repo_path).await;

        assert_eq!(result.score, 5.0);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("pmat-quality.toml present and valid")));
    }

    #[tokio::test]
    async fn test_pmat_gates_toml_wins_over_quality_toml() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);
        create_pmat_gates(repo_path, PERFECT_PMAT_GATES);
        std::fs::write(repo_path.join("pmat-quality.toml"), "").unwrap();

        let result = run(repo_path).await;

        assert_eq!(result.score, 5.0, "precedence: .pmat-gates.toml first");
    }

    #[tokio::test]
    async fn test_pmat_subcategories() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);
        create_pmat_gates(repo_path, PERFECT_PMAT_GATES);

        let result = run(repo_path).await;

        let f1 = result.subcategories.iter().find(|s| s.id == "F1").unwrap();
        assert_eq!(f1.name, "PMAT Configuration Present");
        assert_eq!((f1.score, f1.max_score), (2.5, 2.5));
        let f2 = result.subcategories.iter().find(|s| s.id == "F2").unwrap();
        assert_eq!(f2.name, "No PMAT Violations");
        assert_eq!((f2.score, f2.max_score), (2.5, 2.5));
    }

    #[tokio::test]
    async fn test_pmat_config_validates_toml_syntax() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);
        std::fs::write(
            repo_path.join(".pmat-gates.toml"),
            "[gates\nrun_clippy = true\n",
        )
        .unwrap();

        let result = run(repo_path).await;

        // F1 partial credit 0.5, F2 0.0 (cannot check violations)
        assert_eq!(result.score, 0.5);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("invalid TOML")));
    }
}
