    // Visual Stability Tests (G3) - Additional edge cases

    #[tokio::test]
    async fn test_visual_stability_package_json_libs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("package.json"),
            r#"{"dependencies": {"chalk": "^5.0.0", "ora": "^6.0.0"}}"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_visual_stability(repo_path)
            .await
            .expect("Scoring failed");

        assert!(
            result.score >= 0.5,
            "Should detect JS rich output libraries"
        );
    }

    #[tokio::test]
    async fn test_visual_stability_pyproject_libs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("pyproject.toml"),
            r#"[project]
dependencies = [
    "rich",
    "tqdm",
]
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_visual_stability(repo_path)
            .await
            .expect("Scoring failed");

        assert!(
            result.score >= 0.5,
            "Should detect Python rich output libraries"
        );
    }

    #[tokio::test]
    async fn test_visual_stability_verified_usage() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create Cargo.toml with indicatif
        fs::write(
            repo_path.join("Cargo.toml"),
            r#"[dependencies]
indicatif = "0.17"
"#,
        )
        .expect("Write failed");

        // Create src/ with actual usage
        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(
            repo_path.join("src/main.rs"),
            r#"
use indicatif::ProgressBar;

fn main() {
    let pb = ProgressBar::new(100);
    for _ in 0..100 {
        pb.inc(1);
    }
}
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_visual_stability(repo_path)
            .await
            .expect("Scoring failed");

        // Should get full credit for verified usage
        assert!(result.score >= 1.0, "Should verify indicatif usage");
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("verified")));
    }

    #[tokio::test]
    async fn test_visual_stability_structured_output_patterns() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(
            repo_path.join("examples/demo.rs"),
            r#"
fn main() {
    eprintln!("Error occurred");
    let json = serde_json::to_string_pretty(&data).unwrap();
    let formatted = format!("{}: {}", key, value);
    table.add_row(row);
    let pb = ProgressBar::new(100);
}
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_visual_stability(repo_path)
            .await
            .expect("Scoring failed");

        assert!(
            result.score >= 1.0,
            "Should detect structured output patterns"
        );
    }

    #[tokio::test]
    async fn test_visual_stability_no_libs_warning() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Just a basic project with no rich output libs
        fs::write(
            repo_path.join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.0"
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_visual_stability(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("Consider adding rich terminal output")));
    }

    #[tokio::test]
    async fn test_visual_stability_colored_crate() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("Cargo.toml"),
            r#"[dependencies]
colored = "2.0"
"#,
        )
        .expect("Write failed");

        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(
            repo_path.join("src/main.rs"),
            r#"
use colored::Colorize;
fn main() {
    println!("{}", "Hello".red().bold());
    println!("{}", "World".green());
}
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_visual_stability(repo_path)
            .await
            .expect("Scoring failed");

        assert!(result.score >= 1.0, "Should detect verified colored usage");
    }
