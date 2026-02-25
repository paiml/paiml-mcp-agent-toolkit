    // Time-to-Interaction Tests (G1) - Additional edge cases

    #[tokio::test]
    async fn test_time_to_interaction_demos_dir() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Use "demos" instead of "examples"
        fs::create_dir_all(repo_path.join("demos")).expect("Mkdir failed");
        fs::write(repo_path.join("demos/basic.rs"), "fn main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect demos/ directory");
        assert!(result.findings.iter().any(|f| f.message.contains("demos")));
    }

    #[tokio::test]
    async fn test_time_to_interaction_samples_dir() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Use "samples" directory
        fs::create_dir_all(repo_path.join("samples")).expect("Mkdir failed");
        fs::write(repo_path.join("samples/sample.rs"), "fn main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect samples/ directory");
    }

    #[tokio::test]
    async fn test_time_to_interaction_getting_started_section() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n## Getting Started\n\nFollow these steps...",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect Getting Started section");
    }

    #[tokio::test]
    async fn test_time_to_interaction_try_it_now_section() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n## Try It Now\n\n```bash\nnpx my-tool\n```",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect Try It Now section");
    }

    #[tokio::test]
    async fn test_time_to_interaction_tldr_section() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n## TLDR\n\n```bash\ncargo install my-tool\n```",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(
            result.score >= 2.0,
            "Should detect TLDR section with one-liner"
        );
    }

    #[tokio::test]
    async fn test_time_to_interaction_5_minute_guide() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n## 5-Minute Guide\n\nGet started quickly...",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect 5-minute guide");
    }

    #[tokio::test]
    async fn test_time_to_interaction_one_liner_commands() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n```bash\npip install myproject\n```",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect pip install one-liner");
    }

    #[tokio::test]
    async fn test_time_to_interaction_npm_install() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n```bash\nnpm install my-package\n```",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect npm install one-liner");
    }

    #[tokio::test]
    async fn test_time_to_interaction_npx_command() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n```sh\nnpx create-my-app\n```",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect npx one-liner");
    }

    #[tokio::test]
    async fn test_time_to_interaction_no_examples_warning() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Only a minimal README with no quick-start
        fs::write(
            repo_path.join("README.md"),
            "# Project\n\nA simple project.",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("No examples/")));
    }

    #[tokio::test]
    async fn test_time_to_interaction_capped_at_max() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create all possible bonuses
        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(repo_path.join("examples/demo.rs"), "fn main() {}").expect("Write failed");
        fs::write(
            repo_path.join("README.md"),
            r#"# Project

## Quick Start

```bash
cargo install myproject
```

## Getting Started

More content...
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score <= 3.0, "Score should be capped at 3.0");
        assert_eq!(result.max_score, 3.0);
    }
