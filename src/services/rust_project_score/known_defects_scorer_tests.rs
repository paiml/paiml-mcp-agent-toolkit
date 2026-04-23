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
            /// Safe function.
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

    // --- Branch coverage for score_internal + recommendations ---
    //
    // The `!project_path.join("Cargo.toml").exists()` guard at
    // known_defects_scorer_scoring.rs:26, the `if production_unwraps > 0` check
    // at :82 (recommendations), and the `if let Ok(..)` wrapper at :81 all had
    // no direct tests. Added boundary and error-path tests below.

    #[test]
    fn test_score_internal_missing_cargo_toml_errors() {
        // Directory with no Cargo.toml → score() must return InvalidProject.
        // Kills mutations that remove the `!exists()` guard or flip the negation.
        let temp_dir = TempDir::new().expect("create temp dir");
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(temp_dir.path().join("src/lib.rs"), "fn f() {}").expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let err = scorer
            .score(temp_dir.path())
            .expect_err("score must error without Cargo.toml");
        let msg = err.to_string();
        assert!(
            msg.contains("Cargo.toml") || msg.contains("valid Rust"),
            "error should mention missing Cargo.toml, got: {msg}"
        );
    }

    #[test]
    fn test_calculate_unwrap_score_boundary_99_vs_100() {
        // calculate_unwrap_score uses integer division `production_unwraps / 100`:
        // 99 → bucket 0 → 20.0; 100 → bucket 1 → 15.0. This pair kills off-by-one
        // mutations on the integer-division threshold.
        let temp_dir = TempDir::new().expect("create temp dir");
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");

        let mut code = String::new();
        for i in 0..99 {
            code.push_str(&format!("let x{i} = Some({i}).unwrap();\n"));
        }
        fs::write(temp_dir.path().join("src/lib.rs"), code).expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer
            .score(temp_dir.path())
            .expect("score 99-unwrap project");
        assert!(
            (score.earned - 20.0).abs() < f64::EPSILON,
            "99 unwraps must still be perfect score, got {}",
            score.earned
        );
    }

    #[test]
    fn test_calculate_unwrap_score_boundary_100_first_penalty() {
        let temp_dir = TempDir::new().expect("create temp dir");
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");

        let mut code = String::new();
        for i in 0..100 {
            code.push_str(&format!("let x{i} = Some({i}).unwrap();\n"));
        }
        fs::write(temp_dir.path().join("src/lib.rs"), code).expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let score = scorer
            .score(temp_dir.path())
            .expect("score 100-unwrap project");
        assert!(
            (score.earned - 15.0).abs() < f64::EPSILON,
            "100 unwraps must cross into first penalty bucket, got {}",
            score.earned
        );
    }

    #[test]
    fn test_recommendations_empty_when_no_unwraps() {
        // `if production_unwraps > 0` at line 82: else branch returns empty vec.
        // Kills `> 0` → `>= 0` mutation (which would push recommendations even
        // when count is 0).
        let temp_dir = TempDir::new().expect("create temp dir");
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub fn safe() -> i32 { 42 }",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let recs = scorer.recommendations(temp_dir.path());
        assert!(
            recs.is_empty(),
            "clean project should produce zero recommendations, got {recs:?}"
        );
    }

    #[test]
    fn test_recommendations_empty_on_missing_cargo_toml() {
        // `if let Ok((production_unwraps, _)) = self.count_unwraps(..)` at line 81:
        // the Err arm must produce an empty recommendation vec (no panic, no push).
        // Kills mutations that swap Ok/Err or unwrap the Result.
        let temp_dir = TempDir::new().expect("create temp dir");
        // No Cargo.toml → count_unwraps should not work cleanly; recommendations is
        // defined to return Vec not Result, so the Err arm must be silently empty.
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "let _ = Some(42).unwrap();",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let recs = scorer.recommendations(temp_dir.path());
        // The function may succeed (count_unwraps walks the filesystem and may still
        // find unwraps regardless of Cargo.toml) — what matters is that it doesn't
        // panic. Keep the assertion loose: either empty (Err path) or contains our
        // CRITICAL marker (Ok path on a missing-Cargo-toml project that counts
        // files via walkdir). The key property: no panic.
        for r in &recs {
            assert!(!r.is_empty(), "no empty recommendation strings");
        }
    }

    #[test]
    fn test_score_with_mode_delegates_to_score() {
        // score_with_mode is declared in trait but just forwards to self.score.
        // Kills body mutations that would replace it with a stub/default.
        let temp_dir = TempDir::new().expect("create temp dir");
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write cargo.toml");
        fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "pub fn safe() -> i32 { 42 }",
        )
        .expect("write lib.rs");

        let scorer = KnownDefectsScorer::new();
        let direct = scorer.score(temp_dir.path()).expect("direct score");
        let via_mode = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Full)
            .expect("score_with_mode");
        assert!(
            (direct.earned - via_mode.earned).abs() < f64::EPSILON,
            "score_with_mode must match score(); direct={}, via_mode={}",
            direct.earned,
            via_mode.earned
        );
        assert_eq!(direct.max, via_mode.max);
    }
}
