// Tests for KnownDefectsScorer
// included from known_defects_scorer.rs - shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = KnownDefectsScorer::new();
        assert_eq!(scorer.name(), "Known Defects");
        assert_eq!(scorer.max_points(), 20.0);
    }

    #[test]
    fn test_perfect_score_no_unwraps() {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create Cargo.toml
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        // Create src directory with clean code
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
            pub fn safe_function() -> Result<i32, String> {
                Ok(42)
            }
            "#,
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 20.0, "Perfect score with no unwraps");
    }

    #[test]
    fn test_unwrap_penalty_production_code() {
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");

        // Production code with 150 unwraps (should lose 5 points)
        let mut code = String::new();
        for i in 0..150 {
            code.push_str(&format!("let x{} = Some(42).unwrap();\n", i));
        }

        fs::write(temp_dir.path().join("src/lib.rs"), code).expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 15.0, "150 unwraps = -5 points");
    }

    #[test]
    fn test_test_code_exemption() {
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        // Tests directory with unwraps (should not count)
        fs::create_dir_all(temp_dir.path().join("tests")).expect("create tests");
        fs::write(
            temp_dir.path().join("tests/integration.rs"),
            "fn test() { Some(42).unwrap(); Some(42).unwrap(); }",
        )
        .expect("write test");

        // Production code - clean
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub fn safe() -> i32 { 42 }",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 20.0, "Test unwraps don't count against score");
    }

    #[test]
    fn test_src_tests_exemption() {
        // RED test for /src/tests/ pattern (currently fails - false positive bug)
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        // src/tests/ directory with unwraps (should not count - common pattern in pmat)
        fs::create_dir_all(temp_dir.path().join("src/tests")).expect("create src/tests");
        fs::write(
            temp_dir.path().join("src/tests/unit_tests.rs"),
            "fn test() { Some(42).unwrap(); Some(42).unwrap(); Some(42).unwrap(); }",
        )
        .expect("write test");

        // Production code - clean
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub fn safe() -> i32 { 42 }",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 20.0, "src/tests/ unwraps should not count");
    }

    #[test]
    fn test_maximum_penalty() {
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");

        // 500 unwraps (should max out penalty at 0 points)
        let mut code = String::new();
        for i in 0..500 {
            code.push_str(&format!("let x{} = Some(42).unwrap();\n", i));
        }

        fs::write(temp_dir.path().join("src/lib.rs"), code).expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 0.0, "Maximum penalty capped at 0");
    }

    #[test]
    fn test_examples_dir_exemption() {
        // Fixes #234: examples/ directory should not count as production code
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        // Examples with unwraps (should not count)
        fs::create_dir_all(temp_dir.path().join("examples")).expect("create examples");
        fs::write(
            temp_dir.path().join("examples/demo.rs"),
            "fn main() { let x = Some(42).unwrap(); let y = None::<i32>.unwrap(); }",
        )
        .expect("write example");

        // Production code - clean
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub fn safe() -> i32 { 42 }",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(
            score.earned, 20.0,
            "Example unwraps don't count against score"
        );
    }

    #[test]
    fn test_book_dir_exemption() {
        // Fixes #234: book/ directory should not count as production code
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        // Book content with unwraps (should not count)
        fs::create_dir_all(temp_dir.path().join("book/ch01")).expect("create book");
        fs::write(
            temp_dir.path().join("book/ch01/snippet.rs"),
            "fn main() { Some(42).unwrap(); Some(42).unwrap(); Some(42).unwrap(); }",
        )
        .expect("write book snippet");

        // Production code - clean
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub fn safe() -> i32 { 42 }",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer.score(temp_dir.path()).expect("score project");

        assert_eq!(score.earned, 20.0, "Book unwraps don't count against score");
    }

    #[test]
    fn test_recommendations_generated() {
        let temp_dir = TempDir::new().expect("create temp dir");

        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");

        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "let x = Some(42).unwrap();",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        assert!(
            !recommendations.is_empty(),
            "Should generate recommendations"
        );
        assert!(
            recommendations[0].contains("CRITICAL"),
            "Should be marked critical"
        );
        assert!(
            recommendations[0].contains("Cloudflare"),
            "Should reference incident"
        );
    }
}
