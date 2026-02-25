    #[tokio::test]
    async fn test_demo_scorer_professional_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, PROFESSIONAL_README);
        create_examples_dir(repo_path);
        create_cargo_toml(
            repo_path,
            r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
indicatif = "0.17"
colored = "2.0"
"#,
        );

        let scorer = DemoScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // Should get a good score with professional setup
        assert!(
            result.score >= 6.0,
            "Professional repo should score >= 6.0, got {}",
            result.score
        );
        assert_eq!(result.subcategories.len(), 4);
    }

    #[tokio::test]
    async fn test_demo_scorer_minimal_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, MINIMAL_README);

        let scorer = DemoScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // Minimal repo should get lower score
        assert!(
            result.score < 5.0,
            "Minimal repo should score < 5.0, got {}",
            result.score
        );
    }

    #[tokio::test]
    async fn test_time_to_interaction_with_examples() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_examples_dir(repo_path);
        create_readme(
            repo_path,
            "# Project\n\n## Quick Start\n\n```bash\ncargo run\n```",
        );

        let scorer = DemoScorer::new();
        let result = scorer
            .score_time_to_interaction(repo_path)
            .await
            .expect("internal error");

        assert!(
            result.score >= 2.0,
            "Should score >= 2.0 with examples and quick-start"
        );
        assert_eq!(result.id, "G1");
    }

    #[tokio::test]
    async fn test_error_gracefulness_with_unwraps() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        let examples_dir = repo_path.join("examples");
        fs::create_dir_all(&examples_dir).expect("internal error");

        // Create file with many unwraps
        fs::write(
            examples_dir.join("bad.rs"),
            r#"
fn main() {
    let x = get_value().unwrap();
    let y = parse().unwrap();
    let z = read().unwrap();
    let a = write().unwrap();
    let b = compute().unwrap();
    let c = process().unwrap();
    panic!("Something went wrong");
}
"#,
        )
        .expect("internal error");

        let scorer = DemoScorer::new();
        // Use DemoApp archetype for standard error gracefulness scoring
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("internal error");

        // Should be penalized for raw unwraps and panic
        assert!(result.score < 3.0, "Should lose points for raw unwraps");
        assert!(
            result.findings.iter().any(|f| f.message.contains("unwrap")),
            "Should warn about unwraps"
        );
    }

    #[tokio::test]
    async fn test_error_gracefulness_cookbook_na() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        // Cookbook archetype should have N/A for G2
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::Cookbook)
            .await
            .expect("internal error");

        // G2 should be N/A for cookbooks (max_score = 0)
        assert_eq!(
            result.max_score, 0.0,
            "Cookbook G2 max_score should be 0 (N/A)"
        );
        assert_eq!(result.score, 0.0, "Cookbook G2 score should be 0 (N/A)");
        assert!(
            result.name.contains("N/A"),
            "Cookbook G2 should indicate N/A"
        );
    }

    #[tokio::test]
    async fn test_visual_stability_with_rich_libs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_cargo_toml(
            repo_path,
            r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
indicatif = "0.17"
"#,
        );

        let scorer = DemoScorer::new();
        let result = scorer
            .score_visual_stability(repo_path)
            .await
            .expect("internal error");

        // Should at least get partial credit for having the library in manifest
        assert!(result.score >= 0.5, "Should detect rich output library");
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.message.contains("indicatif")),
            "Should mention the library"
        );
    }

    #[tokio::test]
    async fn test_archetype_detection_cookbook() {
        let temp_dir = TempDir::with_prefix("my-cookbook").expect("internal error");
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;

        assert_eq!(
            archetype,
            RepoArchetype::Cookbook,
            "Should detect cookbook by name"
        );
    }

    #[tokio::test]
    async fn test_archetype_detection_library() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        let src_dir = repo_path.join("src");
        fs::create_dir_all(&src_dir).expect("internal error");
        fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").expect("internal error");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;

        assert_eq!(
            archetype,
            RepoArchetype::Library,
            "Should detect library by src/lib.rs"
        );
    }

    #[tokio::test]
    async fn test_wow_factor_with_demo_gif() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(
            repo_path,
            r#"# Project

![Build](https://img.shields.io/badge/build-passing-green)
![Tests](https://img.shields.io/badge/tests-100%25-green)
![Coverage](https://img.shields.io/badge/coverage-85%25-green)
![License](https://img.shields.io/badge/license-MIT-blue)

## Demo

![Demo](docs/demo.gif)
"#,
        );

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("internal error");

        assert!(result.score >= 1.0, "Should detect demo GIF");
        assert!(
            result.findings.iter().any(|f| f.message.contains("GIF")),
            "Should mention demo GIF"
        );
    }

    #[tokio::test]
    async fn test_category_name_and_max_score() {
        let scorer = DemoScorer::new();
        assert_eq!(scorer.category_name(), "Demo Quality");
        assert_eq!(scorer.max_score(), 10.0);
    }

    #[tokio::test]
    async fn test_empty_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        let config = ScorerConfig::default();

        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("internal error");

        // Empty repo should still return a valid score
        assert!(result.score >= 0.0);
        assert_eq!(result.max_score, 10.0);
    }
