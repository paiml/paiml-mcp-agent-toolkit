    // Wow Factor Tests (G4) - Additional edge cases

    #[tokio::test]
    async fn test_wow_factor_asciinema() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n[![Demo](https://asciinema.org/a/123456.svg)](https://asciinema.org/a/123456)"
        ).expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect asciinema demo");
    }

    #[tokio::test]
    async fn test_wow_factor_video_tag() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n<video src=\"demo.mp4\" controls></video>",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect video element");
    }

    #[tokio::test]
    async fn test_wow_factor_playground_links() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n[Try it on Replit](https://replit.com/@user/project)",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 0.75, "Should detect Replit playground");
    }

    #[tokio::test]
    async fn test_wow_factor_codesandbox() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n[Try on CodeSandbox](https://codesandbox.io/s/example)",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 0.75, "Should detect CodeSandbox");
    }

    #[tokio::test]
    async fn test_wow_factor_rust_playground() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n[Run on Rust Playground](https://play.rust-lang.org/?...)",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 0.75, "Should detect Rust Playground");
    }

    #[tokio::test]
    async fn test_wow_factor_try_it_online() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n## Try it online\n\nVisit our demo site...",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 0.75, "Should detect 'try it online'");
    }

    #[tokio::test]
    async fn test_wow_factor_excessive_badges_info() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            r#"# Project

![Badge1](http://badge1.svg)
![Badge2](http://badge2.svg)
![Badge3](http://badge3.svg)
![Badge4](http://badge4.svg)
![Badge5](http://badge5.svg)
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        // Should have info about excessive badges
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("excessive") || f.message.contains("consider reducing")));
    }

    #[tokio::test]
    async fn test_wow_factor_ascii_art() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            r#"# Project

<img src="logo.svg" alt="Logo" width="200">
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 0.25, "Should detect logo image");
    }

    #[tokio::test]
    async fn test_wow_factor_web_demo_paths() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create demo/index.html
        fs::create_dir_all(repo_path.join("demo")).expect("Mkdir failed");
        fs::write(repo_path.join("demo/index.html"), "<html></html>").expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 0.75, "Should detect web demo");
    }

    #[tokio::test]
    async fn test_wow_factor_docs_index_html() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("docs")).expect("Mkdir failed");
        fs::write(repo_path.join("docs/index.html"), "<html></html>").expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 0.75, "Should detect docs web demo");
    }

    #[tokio::test]
    async fn test_wow_factor_no_readme_info_message() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // No README at all
        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result
            .findings
            .iter()
            .any(|f| f.severity == Severity::Info && f.message.contains("demo GIF/video")));
    }

    #[tokio::test]
    async fn test_wow_factor_capped_at_max() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create all bonuses that exceed 2.0
        fs::create_dir_all(repo_path.join("docs")).expect("Mkdir failed");
        fs::write(repo_path.join("docs/index.html"), "<html></html>").expect("Write failed");
        fs::write(
            repo_path.join("README.md"),
            r#"# Project

![Badge1](b1.svg)
![Badge2](b2.svg)

<img src="logo.svg" alt="Logo" width="200">

![Demo](demo.gif)

[Try on Replit](https://replit.com/@user/project)
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_wow_factor(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score <= 2.0, "Score should be capped at 2.0");
        assert_eq!(result.max_score, 2.0);
    }

    // Find Demo Files Tests

    #[tokio::test]
    async fn test_find_demo_files_multiple_dirs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create files in multiple demo directories
        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(repo_path.join("examples/ex.rs"), "fn main() {}").expect("Write failed");

        fs::create_dir_all(repo_path.join("demos")).expect("Mkdir failed");
        fs::write(repo_path.join("demos/demo.py"), "print('hello')").expect("Write failed");

        let scorer = DemoScorer::new();
        let files = scorer.find_demo_files(repo_path).await;

        assert!(
            files.len() >= 2,
            "Should find files from multiple demo dirs"
        );
    }

    #[tokio::test]
    async fn test_find_demo_files_various_extensions() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(repo_path.join("examples/ex.rs"), "fn main() {}").expect("Write failed");
        fs::write(repo_path.join("examples/ex.py"), "print('hi')").expect("Write failed");
        fs::write(repo_path.join("examples/ex.js"), "console.log('hi')").expect("Write failed");
        fs::write(repo_path.join("examples/ex.ts"), "console.log('hi')").expect("Write failed");
        fs::write(repo_path.join("examples/ex.go"), "package main").expect("Write failed");
        fs::write(repo_path.join("examples/ex.rb"), "puts 'hi'").expect("Write failed");
        fs::write(repo_path.join("examples/ex.sh"), "echo hi").expect("Write failed");

        let scorer = DemoScorer::new();
        let files = scorer.find_demo_files(repo_path).await;

        assert_eq!(files.len(), 7, "Should find all 7 demo files");
    }

    #[tokio::test]
    async fn test_find_demo_files_root_demo_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create demo files in root
        fs::write(repo_path.join("demo.rs"), "fn main() {}").expect("Write failed");
        fs::write(repo_path.join("demo.py"), "print('hi')").expect("Write failed");
        fs::write(repo_path.join("example.rs"), "fn main() {}").expect("Write failed");
        fs::write(repo_path.join("example.py"), "print('hi')").expect("Write failed");

        let scorer = DemoScorer::new();
        let files = scorer.find_demo_files(repo_path).await;

        assert_eq!(files.len(), 4, "Should find root demo files");
    }

    // Count Files Tests

    #[tokio::test]
    async fn test_count_code_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(repo_path.join("src/main.rs"), "fn main() {}").expect("Write failed");
        fs::write(repo_path.join("src/lib.py"), "pass").expect("Write failed");
        fs::write(repo_path.join("src/app.js"), "console.log()").expect("Write failed");
        fs::write(repo_path.join("src/index.ts"), "export {}").expect("Write failed");
        fs::write(repo_path.join("src/main.go"), "package main").expect("Write failed");
        fs::write(repo_path.join("src/app.rb"), "puts 'hi'").expect("Write failed");
        fs::write(repo_path.join("src/App.java"), "class App {}").expect("Write failed");
        fs::write(repo_path.join("src/main.c"), "int main() {}").expect("Write failed");
        fs::write(repo_path.join("src/main.cpp"), "int main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let count = scorer.count_code_files(repo_path).await;

        assert_eq!(count, 9, "Should count all code files");
    }

    #[tokio::test]
    async fn test_count_files_skips_hidden_dirs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join(".hidden")).expect("Mkdir failed");
        fs::write(repo_path.join(".hidden/file.md"), "# Hidden").expect("Write failed");
        fs::write(repo_path.join("visible.md"), "# Visible").expect("Write failed");

        let scorer = DemoScorer::new();
        let count = scorer.count_files_by_extension(repo_path, "md").await;

        assert_eq!(count, 1, "Should skip hidden directories");
    }

    // Full Integration Tests

    #[tokio::test]
    async fn test_demo_scorer_dynamic_max_score() {
        let temp_dir = TempDir::with_prefix("my-cookbook").expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        let config = ScorerConfig::default();
        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("Scoring failed");

        // For cookbook, G2 is N/A so max_score should be 7 (10 - 3)
        assert!(
            result.max_score < 10.0,
            "Cookbook should have reduced max score due to N/A G2"
        );
    }

    #[tokio::test]
    async fn test_demo_scorer_includes_archetype_finding() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        let config = ScorerConfig::default();
        let result = scorer
            .score(repo_path, &config)
            .await
            .expect("Scoring failed");

        // Should have a finding about detected archetype
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("archetype")));
    }
