#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_creation() {
        let scorer = CodeQualityScorer::new();
        assert_eq!(scorer.name(), "Code Quality");
        assert_eq!(scorer.max_points(), 26.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = CodeQualityScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_default_trait() {
        let scorer = CodeQualityScorer::default();
        assert_eq!(scorer.name(), "Code Quality");
        assert_eq!(scorer.max_points(), 26.0);
    }

    #[test]
    fn test_invalid_project_no_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let scorer = CodeQualityScorer::new();

        let result = scorer.score_with_mode(temp_dir.path(), ScoringMode::Fast);
        assert!(result.is_err());
        match result {
            Err(ScorerError::InvalidProject(msg)) => {
                assert!(msg.contains("No Cargo.toml found"));
            }
            _ => panic!("Expected InvalidProject error"),
        }
    }

    #[test]
    fn test_complexity_no_src_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();

        // No source is NOT "no complexity issues": nothing was measured. This
        // used to return a full 3.0 for a project with no code at all.
        assert_eq!(scorer.measure_cyclomatic(temp_dir.path(), None), None);
    }

    #[test]
    fn test_complexity_simple_function_scores_full_points() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "fn main() {\n    println!(\"hello\");\n}",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let profile = scorer
            .measure_cyclomatic(temp_dir.path(), None)
            .expect("one measurable function");

        assert_eq!(profile.functions, 1);
        assert_eq!(profile.over_error, 0);
        assert_eq!(score_from_cyclomatic(profile), 3.0);
    }

    /// A wide line is not complexity. The old check counted lines indented past
    /// column 40 and charged a point for each handful of them, so formatting
    /// moved the score.
    #[test]
    fn test_deep_indentation_alone_does_not_cost_points() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        // 25 lines indented 45 columns, cyclomatic complexity 1.
        let deep_code = format!(
            "fn main() {{\n{}}}\n",
            "                                             nested();\n".repeat(25)
        );
        fs::write(temp_dir.path().join("src/lib.rs"), deep_code).unwrap();

        let scorer = CodeQualityScorer::new();
        let profile = scorer
            .measure_cyclomatic(temp_dir.path(), None)
            .expect("one measurable function");

        assert_eq!(profile.max, 1, "no decision points in the fixture");
        assert_eq!(
            score_from_cyclomatic(profile),
            3.0,
            "indentation must not be scored as complexity"
        );
    }

    /// #937: the reported case. A crate whose only function has cyclomatic
    /// complexity 241 scored the Complexity check 3.0/3.0 (and Code Quality
    /// 14.0/14.0) because none of its lines are deeply indented, while the
    /// same binary's `quality-gate --checks complexity` blocked it.
    #[test]
    fn test_high_cyclomatic_function_scores_zero() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let mut monster = String::from("pub fn monster(a: i32, b: i32, c: i32, d: i32) -> i32 {\n    let mut r = 0;\n");
        for i in 0..60 {
            monster.push_str(&format!(
                "    if a > {i} && b < {i} || c == {i} {{ r += {i}; }} else if d != {i} {{ r -= {i}; }}\n"
            ));
        }
        monster.push_str("    r\n}\n");
        fs::write(temp_dir.path().join("src/lib.rs"), monster).unwrap();

        let scorer = CodeQualityScorer::new();
        let profile = scorer
            .measure_cyclomatic(temp_dir.path(), None)
            .expect("one measurable function");

        assert!(
            profile.max > 200,
            "expected the measured cyclomatic complexity, got {}",
            profile.max
        );
        assert_eq!(profile.over_error, 1);
        assert_eq!(score_from_cyclomatic(profile), 0.0);

        // ...and the category it feeds cannot report 100% for that crate.
        let category = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .expect("fast score");
        assert!(
            category.percentage() < 100.0,
            "Code Quality was {}/{} for a cyclomatic-241 crate",
            category.earned,
            category.max
        );
    }

    /// The ranking must not be inverted: a complex crate cannot outscore a
    /// trivial one on the Complexity check.
    #[test]
    fn test_complex_crate_scores_below_simple_crate() {
        let make = |name: &str, body: &str| {
            let dir = TempDir::new().unwrap();
            fs::create_dir_all(dir.path().join("src")).unwrap();
            fs::write(
                dir.path().join("Cargo.toml"),
                format!("[package]\nname = \"{name}\""),
            )
            .unwrap();
            fs::write(dir.path().join("src/lib.rs"), body).unwrap();
            dir
        };

        let mut hot_body = String::from("pub fn monster(a: i32, b: i32) -> i32 {\n    let mut r = 0;\n");
        for i in 0..60 {
            hot_body.push_str(&format!("    if a > {i} && b < {i} {{ r += {i}; }}\n"));
        }
        hot_body.push_str("    r\n}\n");
        let hot = make("hot", &hot_body);

        // One function, complexity 1, one line indented 44 columns.
        let deep = make(
            "deep",
            "pub fn simple() -> i32 {\n                                            1\n}\n",
        );

        let scorer = CodeQualityScorer::new();
        let hot_score = scorer
            .score_with_mode(hot.path(), ScoringMode::Fast)
            .unwrap();
        let deep_score = scorer
            .score_with_mode(deep.path(), ScoringMode::Fast)
            .unwrap();

        assert!(
            hot_score.percentage() < deep_score.percentage(),
            "cyclomatic-heavy crate scored {}% vs {}% for the trivial one",
            hot_score.percentage(),
            deep_score.percentage()
        );
    }

    #[test]
    fn test_unsafe_no_src_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), None).unwrap();

        // No code = no unsafe = full points
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_unsafe_no_unsafe_blocks() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "fn safe_code() { println!(\"safe\"); }",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), None).unwrap();

        // No unsafe = full points
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_unsafe_documented_blocks() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
// SAFETY: This is safe because reasons
unsafe {
    do_unsafe_thing();
}
"#,
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), None).unwrap();

        // 100% documented = full points
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_unsafe_undocumented_blocks() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        // SAFETY: String literal test fixture -- not an actual unsafe block in this file.
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
// No SAFETY comment here
fn foo() {
    unsafe {
        do_thing();
    }
}
"#,
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), None).unwrap();

        // 0% documented = 1.0 points
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_dead_code_no_src_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_dead_code(temp_dir.path(), None).unwrap();

        // No code = full points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_dead_code_no_allow_attributes() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            "fn used_function() { println!(\"used\"); }",
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_dead_code(temp_dir.path(), None).unwrap();

        // No dead code = full points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_dead_code_few_allow_attributes() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            format!("\nfn unused1() {{}}\n#[allow({})]\nfn unused2() {{}}", "dead_code"),
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_dead_code(temp_dir.path(), None).unwrap();

        // 1-3 dead code = 1.0 points
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_dead_code_many_allow_attributes() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            format!("\nfn unused1() {{}}\n#[allow({dc})]\nfn unused2() {{}}\n#[allow({dc})]\nfn unused3() {{}}\n#[allow({dc})]\nfn unused4() {{}}\n#[allow({dc})]\nfn unused5() {{}}", dc = "dead_code"),
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_dead_code(temp_dir.path(), None).unwrap();

        // >3 dead code = 0.0 points
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_score_fast_mode() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();

        // Fast mode runs complexity(3) + unsafe(9) + dead_code(2) = 14 points.
        //
        // This used to assert `earned >= 18.0` and `max == 26.0` — the comment it
        // replaced even named the reason ("mutation(4, skipped) + build(2,
        // skipped)"): fast mode was awarding heuristic partial credit for two
        // checks it never ran, and counting their points in the denominator. A
        // check that did not run can contribute to neither side of the ratio, so
        // both numbers drop by 12.
        assert_eq!(result.max, 14.0, "skipped checks must leave the denominator");
        assert!(
            result.earned <= result.max,
            "earned {} exceeds max {}",
            result.earned,
            result.max
        );
    }

    #[test]
    fn test_score_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        // Create cache
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "fn main() {}".to_string(),
        );

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_with_cache(temp_dir.path(), ScoringMode::Fast, Some(&cache))
            .unwrap();

        // Same correction as test_score_with_mode_fast above: this used to
        // assert `earned >= 18.0` and `max == 26.0`, which only held because
        // fast mode invented partial credit for mutation testing (8pts) and
        // build time (4pts) — two checks it never runs. Fast mode scores the
        // three checks it can run: complexity(3) + unsafe(9) + dead_code(2).
        assert_eq!(result.max, 14.0, "skipped checks must leave the denominator");
        assert!(
            result.earned <= result.max,
            "earned {} exceeds max {}",
            result.earned,
            result.max
        );

        // The cache is a read path for the same source, not a scoring input:
        // it must not change the result.
        let uncached = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Fast)
            .unwrap();
        assert_eq!(result.earned, uncached.earned);
        assert_eq!(result.max, uncached.max);
    }

    #[test]
    fn test_recommendations_clean_code() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        let scorer = CodeQualityScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should always include mutation testing recommendation
        assert!(recommendations.iter().any(|r| r.contains("cargo-mutants")));
    }

    #[test]
    fn test_recommendations_with_issues() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();

        // Create code with undocumented unsafe and dead code
        // SAFETY: String literal test fixture -- not an actual unsafe block in this file.
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"

fn unused() {}

fn foo() {
    unsafe {
        do_thing();
    }
}
"#,
        )
        .unwrap();

        let scorer = CodeQualityScorer::new();
        let recommendations = scorer.recommendations(temp_dir.path());

        // Should have SAFETY comment recommendation
        assert!(recommendations.iter().any(|r| r.contains("SAFETY")));
    }

    #[test]
    fn test_complexity_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        // Create cache with a trivial function
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "fn main() {\n    println!(\"hello\");\n}".to_string(),
        );

        let scorer = CodeQualityScorer::new();
        let profile = scorer
            .measure_cyclomatic(temp_dir.path(), Some(&cache))
            .expect("one measurable function");

        assert_eq!(score_from_cyclomatic(profile), 3.0);
    }

    #[test]
    fn test_unsafe_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        // Create cache with documented unsafe
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "// SAFETY: documented\nunsafe {\n    thing();\n}".to_string(),
        );

        let scorer = CodeQualityScorer::new();
        let result = scorer.score_unsafe(temp_dir.path(), Some(&cache)).unwrap();

        // 100% documented = full points
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_dead_code_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        // Create cache with no dead code
        let mut cache = FileCache::new();
        cache.insert(
            temp_dir.path().join("src/lib.rs"),
            "fn used_function() {}".to_string(),
        );

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_dead_code(temp_dir.path(), Some(&cache))
            .unwrap();

        // No dead code = full points
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_scoring_mode_quick() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(temp_dir.path().join("src/lib.rs"), "fn main() {}").unwrap();

        let scorer = CodeQualityScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .unwrap();

        // Quick mode should still produce valid scores
        assert!(result.earned >= 0.0);
        assert!(result.earned <= result.max);
    }
}
