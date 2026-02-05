// TDD: Pre-commit Scorer Tests
// Tests Category B: Pre-commit Hooks and Linting (20 points)
// All tests should FAIL until PrecommitScorer is implemented

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod precommit_scorer_tests {
    use crate::tests::repo_score::test_utils::*;

    #[tokio::test]
    #[ignore] // RED: Will fail until PrecommitScorer is implemented
    async fn test_precommit_no_hooks() {
        // ARRANGE
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);
        // Note: NOT creating .git/hooks/pre-commit

        // use crate::services::repo_score::scorers::{PrecommitScorer, Scorer, ScorerConfig};
        // let scorer = PrecommitScorer::new();
        // let config = ScorerConfig {
        //     verbose: false,
        //     timeout_seconds: 60,
        //     skip_slow_checks: false,
        // };

        // ACT
        // let result = scorer.score(repo_path, &config).await.unwrap();

        // ASSERT
        // No hooks = 0 points
        // assert_eq!(result.score, 0.0);
        // assert_eq!(result.max_score, 20.0);
        // assert_eq!(result.status, ScoreStatus::Fail);
        // assert!(result.findings.iter().any(|f| f.message.contains("No pre-commit hook")));

        panic!("PrecommitScorer not implemented yet");
    }

    #[tokio::test]
    #[ignore] // RED: Will fail until PrecommitScorer is implemented
    async fn test_precommit_perfect_setup() {
        // ARRANGE
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);
        create_precommit_hook(repo_path, PERFECT_PRECOMMIT_HOOK);

        // use crate::services::repo_score::scorers::{PrecommitScorer, Scorer, ScorerConfig};
        // let scorer = PrecommitScorer::new();
        // let config = ScorerConfig {
        //     verbose: false,
        //     timeout_seconds: 60,
        //     skip_slow_checks: false,
        // };

        // ACT
        // let result = scorer.score(repo_path, &config).await.unwrap();

        // ASSERT
        // Perfect setup = 20 points
        // assert_eq!(result.score, 20.0);
        // assert_eq!(result.max_score, 20.0);
        // assert_eq!(result.percentage, 100.0);
        // assert_eq!(result.status, ScoreStatus::Pass);

        panic!("PrecommitScorer not implemented yet");
    }

    #[tokio::test]
    #[ignore] // RED: Will fail until PrecommitScorer is implemented
    async fn test_precommit_hook_exists_but_not_executable() {
        // ARRANGE
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);

        // Create hook but don't make it executable
        let hooks_dir = repo_path.join(".git/hooks");
        std::fs::create_dir_all(&hooks_dir).unwrap();
        std::fs::write(hooks_dir.join("pre-commit"), PERFECT_PRECOMMIT_HOOK).unwrap();
        // Intentionally NOT setting executable permission

        // use crate::services::repo_score::scorers::{PrecommitScorer, Scorer, ScorerConfig};
        // let scorer = PrecommitScorer::new();
        // let config = ScorerConfig {
        //     verbose: false,
        //     timeout_seconds: 60,
        //     skip_slow_checks: false,
        // };

        // ACT
        // let result = scorer.score(repo_path, &config).await.unwrap();

        // ASSERT
        // Hook exists but not executable = partial points
        // assert!(result.score < 20.0);
        // assert!(result.findings.iter().any(|f| f.message.contains("not executable")));

        panic!("PrecommitScorer not implemented yet");
    }

    #[tokio::test]
    #[ignore] // RED: Will fail until PrecommitScorer is implemented
    async fn test_precommit_with_pre_commit_config_yaml() {
        // ARRANGE
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);

        // Create .pre-commit-config.yaml instead of manual hook
        let precommit_config = r#"
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
"#;
        std::fs::write(repo_path.join(".pre-commit-config.yaml"), precommit_config).unwrap();

        // use crate::services::repo_score::scorers::{PrecommitScorer, Scorer, ScorerConfig};
        // let scorer = PrecommitScorer::new();
        // let config = ScorerConfig {
        //     verbose: false,
        //     timeout_seconds: 60,
        //     skip_slow_checks: false,
        // };

        // ACT
        // let result = scorer.score(repo_path, &config).await.unwrap();

        // ASSERT
        // .pre-commit-config.yaml should score well (B1: 9-10 points)
        // assert!(result.score >= 15.0); // Some points for config, may lose points if not installed
        // assert!(result.findings.iter().any(|f| f.message.contains(".pre-commit-config.yaml")));

        panic!("PrecommitScorer not implemented yet");
    }

    #[tokio::test]
    #[ignore] // RED: Will fail until PrecommitScorer is implemented
    async fn test_precommit_execution_timeout() {
        // ARRANGE
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);

        // Create a slow hook that sleeps for 35 seconds (over 30s threshold)
        let slow_hook = r#"#!/usr/bin/env bash
echo "Starting slow hook..."
sleep 35
echo "Done!"
"#;
        create_precommit_hook(repo_path, slow_hook);

        // use crate::services::repo_score::scorers::{PrecommitScorer, Scorer, ScorerConfig};
        // let scorer = PrecommitScorer::new();
        // let config = ScorerConfig {
        //     verbose: false,
        //     timeout_seconds: 60,
        //     skip_slow_checks: false,
        // };

        // ACT
        // let result = scorer.score(repo_path, &config).await.unwrap();

        // ASSERT
        // Hook takes >30s = lose performance points (B2: 0/10)
        // Should still get some points for having a hook (B1: partial)
        // assert!(result.score < 15.0);
        // assert!(result.findings.iter().any(|f| f.message.contains("slow") || f.message.contains(">30")));

        panic!("PrecommitScorer not implemented yet");
    }

    #[tokio::test]
    #[ignore] // RED: Will fail until PrecommitScorer is implemented
    async fn test_precommit_with_skip_slow_checks() {
        // ARRANGE
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);

        let slow_hook = r#"#!/usr/bin/env bash
sleep 35
"#;
        create_precommit_hook(repo_path, slow_hook);

        // use crate::services::repo_score::scorers::{PrecommitScorer, Scorer, ScorerConfig};
        // let scorer = PrecommitScorer::new();
        // let config = ScorerConfig {
        //     verbose: false,
        //     timeout_seconds: 60,
        //     skip_slow_checks: true, // Skip performance check
        // };

        // ACT
        // let result = scorer.score(repo_path, &config).await.unwrap();

        // ASSERT
        // With skip_slow_checks, should not actually run the hook
        // Should give partial credit for existence
        // assert!(result.score >= 10.0);

        panic!("PrecommitScorer not implemented yet");
    }

    #[tokio::test]
    #[ignore] // RED: Will fail until PrecommitScorer is implemented
    async fn test_precommit_subcategories() {
        // Test B1 and B2 subcategories separately
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);
        create_precommit_hook(repo_path, PERFECT_PRECOMMIT_HOOK);

        // use crate::services::repo_score::scorers::{PrecommitScorer, Scorer, ScorerConfig};
        // let scorer = PrecommitScorer::new();
        // let config = ScorerConfig {
        //     verbose: false,
        //     timeout_seconds: 60,
        //     skip_slow_checks: false,
        // };

        // let result = scorer.score(repo_path, &config).await.unwrap();

        // Find subcategories
        // let b1 = result.subcategories.iter().find(|s| s.id == "B1").unwrap();
        // let b2 = result.subcategories.iter().find(|s| s.id == "B2").unwrap();

        // assert_eq!(b1.name, "Best Practices");
        // assert_eq!(b1.max_score, 10.0);
        // assert_eq!(b2.name, "Performance & Effectiveness");
        // assert_eq!(b2.max_score, 10.0);

        panic!("PrecommitScorer not implemented yet");
    }

    #[tokio::test]
    #[ignore] // RED: Will fail until PrecommitScorer is implemented
    async fn test_precommit_both_config_and_hook() {
        // Test when both .pre-commit-config.yaml AND .git/hooks/pre-commit exist
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        init_git_repo(repo_path);

        // Create both
        let precommit_config = r#"
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
"#;
        std::fs::write(repo_path.join(".pre-commit-config.yaml"), precommit_config).unwrap();
        create_precommit_hook(repo_path, PERFECT_PRECOMMIT_HOOK);

        // use crate::services::repo_score::scorers::{PrecommitScorer, Scorer, ScorerConfig};
        // let scorer = PrecommitScorer::new();
        // let config = ScorerConfig {
        //     verbose: false,
        //     timeout_seconds: 60,
        //     skip_slow_checks: false,
        // };

        // ACT
        // let result = scorer.score(repo_path, &config).await.unwrap();

        // ASSERT
        // Having both should give maximum points
        // assert_eq!(result.score, 20.0);
        // assert!(result.findings.iter().any(|f| f.message.contains("config") || f.message.contains("hook")));

        panic!("PrecommitScorer not implemented yet");
    }
}
