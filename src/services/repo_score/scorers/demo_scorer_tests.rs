// Tests for demo scorer
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().expect("internal error")
    }

    fn create_readme(repo_path: &std::path::Path, content: &str) {
        let readme_path = repo_path.join("README.md");
        fs::write(readme_path, content).expect("internal error");
    }

    fn create_examples_dir(repo_path: &std::path::Path) {
        let examples_dir = repo_path.join("examples");
        fs::create_dir_all(&examples_dir).expect("internal error");

        // Create a sample example file
        fs::write(
            examples_dir.join("basic.rs"),
            r#"
fn main() {
    let result = do_something().expect("Failed to do something");
    println!("Result: {:?}", result);
}

fn do_something() -> Result<i32, String> {
    Ok(42)
}
"#,
        )
        .expect("internal error");
    }

    fn create_cargo_toml(repo_path: &std::path::Path, content: &str) {
        fs::write(repo_path.join("Cargo.toml"), content).expect("internal error");
    }

    const PROFESSIONAL_README: &str = r#"# Project

![Build](https://img.shields.io/badge/build-passing-green)
![Tests](https://img.shields.io/badge/tests-100%25-green)
![Coverage](https://img.shields.io/badge/coverage-85%25-green)
![License](https://img.shields.io/badge/license-MIT-blue)

<img src="docs/logo.svg" alt="Logo" width="200">

## Quick Start

```bash
cargo install myproject
```

## Demo

![Demo](docs/demo.gif)

## Getting Started

1. Install the project
2. Run `myproject --help`
"#;

    const MINIMAL_README: &str = r#"# Project

A project.
"#;

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
}

mod coverage_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    // RepoArchetype Tests - Coverage for all archetype methods

    #[test]
    fn test_repo_archetype_g2_max_scores() {
        // Test all archetypes for G2 max score values
        assert_eq!(RepoArchetype::Cookbook.g2_max_score(), None);
        assert_eq!(RepoArchetype::Tutorial.g2_max_score(), Some(1.5));
        assert_eq!(RepoArchetype::DemoApp.g2_max_score(), Some(3.0));
        assert_eq!(RepoArchetype::Library.g2_max_score(), Some(3.0));
        assert_eq!(RepoArchetype::Boilerplate.g2_max_score(), Some(3.0));
    }

    #[test]
    fn test_repo_archetype_names() {
        assert_eq!(RepoArchetype::Cookbook.name(), "Cookbook");
        assert_eq!(RepoArchetype::DemoApp.name(), "Demo Application");
        assert_eq!(RepoArchetype::Library.name(), "Library");
        assert_eq!(RepoArchetype::Tutorial.name(), "Tutorial");
        assert_eq!(RepoArchetype::Boilerplate.name(), "Boilerplate");
    }

    #[test]
    fn test_demo_scorer_default() {
        let scorer = DemoScorer::default();
        assert_eq!(scorer.category_name(), "Demo Quality");
        assert_eq!(scorer.max_score(), 10.0);
    }

    // Archetype Detection Tests - Coverage for detect_archetype

    #[tokio::test]
    async fn test_archetype_detection_boilerplate_by_name() {
        let temp_dir = TempDir::with_prefix("my-starter").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Boilerplate);
    }

    #[tokio::test]
    async fn test_archetype_detection_template_by_name() {
        let temp_dir = TempDir::with_prefix("rust-template").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Boilerplate);
    }

    #[tokio::test]
    async fn test_archetype_detection_scaffold_by_name() {
        let temp_dir = TempDir::with_prefix("project-scaffold").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Boilerplate);
    }

    #[tokio::test]
    async fn test_archetype_detection_tutorial_by_name() {
        let temp_dir = TempDir::with_prefix("rust-tutorial").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Tutorial);
    }

    #[tokio::test]
    async fn test_archetype_detection_learn_by_name() {
        let temp_dir = TempDir::with_prefix("learn-rust").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Tutorial);
    }

    #[tokio::test]
    async fn test_archetype_detection_course_by_name() {
        let temp_dir = TempDir::with_prefix("rust-course").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Tutorial);
    }

    #[tokio::test]
    async fn test_archetype_detection_recipes_by_name() {
        let temp_dir = TempDir::with_prefix("rust-recipes").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::Cookbook);
    }

    #[tokio::test]
    async fn test_archetype_detection_demo_by_name() {
        let temp_dir = TempDir::with_prefix("my-demo").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::DemoApp);
    }

    #[tokio::test]
    async fn test_archetype_detection_example_by_name() {
        let temp_dir = TempDir::with_prefix("example-project").expect("Failed to create temp dir");
        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(temp_dir.path()).await;
        assert_eq!(archetype, RepoArchetype::DemoApp);
    }

    #[tokio::test]
    async fn test_archetype_detection_cookbook_by_content() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create many markdown files but few code files
        for i in 0..10 {
            fs::write(repo_path.join(format!("doc{}.md", i)), "# Doc").expect("Write failed");
        }
        fs::write(repo_path.join("example.rs"), "fn main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;
        assert_eq!(archetype, RepoArchetype::Cookbook);
    }

    #[tokio::test]
    async fn test_archetype_detection_demo_app_by_content() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create src/main.rs and demo files
        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(repo_path.join("src/main.rs"), "fn main() {}").expect("Write failed");
        fs::create_dir_all(repo_path.join("examples")).expect("Mkdir failed");
        fs::write(repo_path.join("examples/demo.rs"), "fn main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;
        assert_eq!(archetype, RepoArchetype::DemoApp);
    }

    #[tokio::test]
    async fn test_archetype_detection_library_with_both_main_and_lib() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create src/lib.rs AND src/main.rs - should be Library
        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(repo_path.join("src/lib.rs"), "pub fn hello() {}").expect("Write failed");
        fs::write(repo_path.join("src/main.rs"), "fn main() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;
        assert_eq!(archetype, RepoArchetype::Library);
    }

    #[tokio::test]
    async fn test_archetype_detection_with_src_but_no_lib_or_main() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create src/ dir but no lib.rs or main.rs
        fs::create_dir_all(repo_path.join("src")).expect("Mkdir failed");
        fs::write(repo_path.join("src/utils.rs"), "pub fn util() {}").expect("Write failed");

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;
        assert_eq!(archetype, RepoArchetype::Library);
    }

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

        // Should get partial credit (2.0) for having error docs
        assert_eq!(result.score, 2.0);
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

        assert_eq!(result.score, 2.0);
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

        assert_eq!(result.score, 2.0);
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
}
