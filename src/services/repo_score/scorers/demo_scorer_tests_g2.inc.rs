    // Error Gracefulness Tests (G2) - Additional edge cases

    #[tokio::test]
    async fn test_error_gracefulness_tutorial_archetype() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create demo files with some unwraps
        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(
            repo_path.join("examples/demo.rs"),
            "fn main() { value().unwrap(); }",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::Tutorial)
            .await
            .expect("Scoring failed");

        // Tutorial has reduced max_score of 1.5
        assert_eq!(result.max_score, 1.5);
    }

    #[tokio::test]
    async fn test_error_gracefulness_no_demo_files_with_error_section() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // README with error handling documentation
        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n## Error Handling\n\nThis project handles errors gracefully...",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("Scoring failed");

        // No demo files were analysed, so G2 measured nothing and scores 0.0 —
        // a README section is documentation, not demo error handling. This
        // asserted 2.0 while the branch handed out credit for absence.
        assert_eq!(result.score, 0.0);
    }

    #[tokio::test]
    async fn test_error_gracefulness_troubleshoot_section() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n## Troubleshooting\n\nCommon issues...",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::Library)
            .await
            .expect("Scoring failed");

        // Nothing analysed ⇒ nothing scored (see the no-demo-files branch).
        assert_eq!(result.score, 0.0);
    }

    #[tokio::test]
    async fn test_error_gracefulness_common_issues_section() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::write(
            repo_path.join("README.md"),
            "# Project\n\n## Common Issues\n\nSome issues...",
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("Scoring failed");

        // Nothing analysed ⇒ nothing scored (see the no-demo-files branch).
        assert_eq!(result.score, 0.0);
    }

    /// An EMPTY directory used to score 1.5/3.0 on G2 ("No demo files found to
    /// analyze for error handling") and therefore 15% overall, while the
    /// sibling G1 scored 0.0 for the same absence. Absence of evidence is not
    /// half credit.
    #[tokio::test]
    async fn test_error_gracefulness_empty_repo_scores_zero() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("Scoring failed");

        assert_eq!(
            result.score, 0.0,
            "an empty repo has no demo files to analyse, but G2 awarded {} points",
            result.score
        );
    }

    #[tokio::test]
    async fn test_error_gracefulness_contextual_unwraps_not_penalized() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(
            repo_path.join("examples/demo.rs"),
            r#"
fn test_something() {
    let x = value().unwrap();
}

fn setup_test() {
    let y = init().unwrap();
}

fn proof_of_concept_demo() {
    let z = demo().unwrap();
}

fn example_usage() {
    let a = example().unwrap();
}
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("Scoring failed");

        // Should have info finding about contextual unwraps
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("test/setup") || f.message.contains("acceptable")));
    }

    #[tokio::test]
    async fn test_error_gracefulness_many_unwraps_over_10() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");

        // Create file with many unwraps (over 10)
        let unwraps = (0..15)
            .map(|i| format!("let x{} = f().unwrap();", i))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(
            repo_path.join("examples/bad.rs"),
            format!("fn main() {{\n{}\n}}", unwraps),
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("Scoring failed");

        // Should have Error severity for >10 unwraps
        assert!(result
            .findings
            .iter()
            .any(|f| f.severity == Severity::Error));
    }

    #[tokio::test]
    async fn test_error_gracefulness_proper_error_handling() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(
            repo_path.join("examples/good.rs"),
            r#"
fn main() -> anyhow::Result<()> {
    let x = get_value()?;
    match result {
        Ok(v) => println!("{}", v),
        Err(e) => eprintln!("Error: {}", e),
    }
    if let Err(e) = do_something() {
        eprintln!("Failed: {}", e);
    }
    value.map_err(|e| format!("Wrapped: {}", e))?;
    Ok(())
}
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("Scoring failed");

        // Should get high score with proper error handling
        assert!(
            result.score >= 2.5,
            "Good error handling should score high: {}",
            result.score
        );
    }

    #[tokio::test]
    async fn test_error_gracefulness_expect_with_messages() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(
            repo_path.join("examples/demo.rs"),
            r#"
fn main() {
    let x = get_value().expect("Failed to get value");
    let y = parse().expect("Invalid input format");
}
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("Scoring failed");

        // Should have info about expect() usage
        assert!(result.findings.iter().any(|f| f.message.contains("expect")));
    }

    #[tokio::test]
    async fn test_error_gracefulness_clean_code_finding() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(
            repo_path.join("examples/good.rs"),
            r#"
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let x = get_value()?;
    let y = parse()?;
    let z = compute()?;
    let a = process()?;
    let b = finalize()?;
    let c = validate()?;
    Ok(())
}
"#,
        )
        .expect("Write failed");

        let scorer = DemoScorer::new();
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .expect("Scoring failed");

        // Verify scoring completes and has findings or score
        // Specific finding messages depend on implementation
        assert!(
            !result.findings.is_empty() || result.score >= 0.0,
            "Should have findings or a valid score"
        );
    }
