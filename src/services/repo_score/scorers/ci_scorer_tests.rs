// CiScorer tests
// Included from ci_scorer.rs - do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().expect("internal error")
    }

    fn create_workflow(repo_path: &Path, name: &str, content: &str) {
        let workflows_dir = repo_path.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("internal error");
        let workflow_path = workflows_dir.join(name);
        fs::write(workflow_path, content).expect("internal error");
    }

    const PERFECT_WORKFLOW: &str = r#"
name: CI

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: cargo test
      - name: Run clippy
        run: cargo clippy -- -D warnings
"#;

    const MINIMAL_WORKFLOW: &str = r#"
name: Build

on: push

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Hello"
"#;

    #[tokio::test]
    async fn test_ci_scorer_no_workflows() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        assert_eq!(result.score, 0.0);
        assert_eq!(result.status, ScoreStatus::Fail);
    }

    #[tokio::test]
    async fn test_ci_scorer_perfect_workflow() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // E1: 2 (1 workflow), E2: 4 (valid + test + lint), E3: 0 (no advanced) = 6
        // With the improved workflow, should get some E3 points too
        assert!(result.score >= 5.0 && result.score <= 8.0);
    }

    #[tokio::test]
    async fn test_ci_scorer_multiple_workflows() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);
        create_workflow(repo_path, "release.yml", MINIMAL_WORKFLOW);
        create_workflow(repo_path, "lint.yml", MINIMAL_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // E1: 6 (3 workflows), E2: 6 (maxed out), E3: 0 (no advanced) = 12
        assert!(result.score >= 10.0 && result.score <= 14.0);
    }

    #[tokio::test]
    async fn test_ci_scorer_minimal_workflow() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "build.yml", MINIMAL_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // E1: 2 (1 workflow), E2: 2 (valid only), E3: 0 = 4
        assert!(result.score >= 3.0 && result.score <= 5.0);
    }

    #[tokio::test]
    async fn test_ci_workflows_present_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        let e1 = result
            .subcategories
            .iter()
            .find(|s| s.id == "E1")
            .expect("internal error");
        assert_eq!(e1.name, "CI Workflows Present");
        assert_eq!(e1.max_score, 6.0);
        assert!(e1.score >= 1.0 && e1.score <= 3.0); // 1 workflow = 2 pts
    }

    #[tokio::test]
    async fn test_ci_workflows_configured_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        let e2 = result
            .subcategories
            .iter()
            .find(|s| s.id == "E2")
            .expect("internal error");
        assert_eq!(e2.name, "Workflows Configured Properly");
        assert_eq!(e2.max_score, 6.0);
        assert!(e2.score >= 3.0 && e2.score <= 5.0); // valid + test + lint
    }

    #[tokio::test]
    async fn test_ci_advanced_features_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_workflow(repo_path, "ci.yml", PERFECT_WORKFLOW);

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        let e3 = result
            .subcategories
            .iter()
            .find(|s| s.id == "E3")
            .expect("internal error");
        assert_eq!(e3.name, "Advanced CI Features");
        assert_eq!(e3.max_score, 8.0);
        // Should have findings telling us what's missing
        assert!(!e3.findings.is_empty());
    }

    #[tokio::test]
    async fn test_ci_empty_workflows_dir() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create directory but no files
        let workflows_dir = repo_path.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("internal error");

        let scorer = CiScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // E1: 1 (dir exists but empty), E2: 0, E3: 0 = 1
        assert!(result.score >= 0.0 && result.score <= 2.0);
    }

    #[tokio::test]
    async fn test_ci_category_name() {
        let scorer = CiScorer::new();
        assert_eq!(scorer.category_name(), "Continuous Integration");
        assert_eq!(scorer.max_score(), 20.0);
    }
}
