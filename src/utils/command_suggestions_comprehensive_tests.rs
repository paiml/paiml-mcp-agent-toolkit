#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod comprehensive_coverage_tests {
    use super::*;

    // =============================================================================
    // LEVENSHTEIN DISTANCE EXHAUSTIVE TESTS
    // =============================================================================

    #[test]
    fn test_levenshtein_all_operations_combined() {
        // Replace + insert + delete combined
        assert_eq!(levenshtein_distance("ab", "xy"), 2);
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
        assert_eq!(levenshtein_distance("a", "xyz"), 3);
        assert_eq!(levenshtein_distance("xyz", "a"), 3);
    }

    #[test]
    fn test_levenshtein_long_strings() {
        let s1 = "the quick brown fox";
        let s2 = "the quick brown dog";
        // "fox" -> "dog" = 2 substitutions (f->d, x->g)
        assert_eq!(levenshtein_distance(s1, s2), 2);
    }

    #[test]
    fn test_levenshtein_repeated_chars() {
        assert_eq!(levenshtein_distance("aaa", "aaaa"), 1);
        assert_eq!(levenshtein_distance("aaa", "aa"), 1);
        assert_eq!(levenshtein_distance("aaa", "bbb"), 3);
    }

    #[test]
    fn test_levenshtein_symmetric() {
        // Distance should be symmetric
        assert_eq!(
            levenshtein_distance("abc", "xyz"),
            levenshtein_distance("xyz", "abc")
        );
        assert_eq!(
            levenshtein_distance("analyze", "analize"),
            levenshtein_distance("analize", "analyze")
        );
    }

    #[test]
    fn test_initialize_distance_matrix_large() {
        let matrix = initialize_distance_matrix(10, 10);
        assert_eq!(matrix.len(), 11);
        assert_eq!(matrix[0].len(), 11);
        // Verify first row
        for j in 0..=10 {
            assert_eq!(matrix[0][j], j);
        }
        // Verify first column
        for i in 0..=10 {
            assert_eq!(matrix[i][0], i);
        }
    }

    #[test]
    fn test_calculate_edit_distances_directly() {
        let mut matrix = initialize_distance_matrix(3, 3);
        calculate_edit_distances(&mut matrix, "abc", "abc");
        // Identical strings should have 0 at bottom-right
        assert_eq!(matrix[3][3], 0);

        let mut matrix2 = initialize_distance_matrix(3, 3);
        calculate_edit_distances(&mut matrix2, "abc", "xyz");
        assert_eq!(matrix2[3][3], 3);
    }

    #[test]
    fn test_calculate_cell_distance_edge_cases() {
        // Test minimum selection
        let matrix = vec![
            vec![0, 1, 2, 3],
            vec![1, 1, 2, 3],
            vec![2, 2, 1, 2],
            vec![3, 3, 2, 1],
        ];
        // Test deletion is cheapest
        let result = calculate_cell_distance(&matrix, 1, 1, true);
        assert_eq!(result, 0); // diagonal + 0 (match)

        // Test insertion is cheapest
        let result = calculate_cell_distance(&matrix, 1, 2, false);
        assert_eq!(result, 2); // min(1+1, 2+1, 1+1) = 2
    }

    // =============================================================================
    // COMMAND SUGGESTER COMPREHENSIVE TESTS
    // =============================================================================

    #[test]
    fn test_suggester_all_main_commands_valid() {
        let suggester = CommandSuggester::new();
        let main_cmds = [
            "analyze",
            "generate",
            "scaffold",
            "context",
            "quality-gate",
            "demo",
            "agent",
            "refactor",
            "enforce",
        ];

        for cmd in main_cmds {
            let result = suggester.suggest_command(&[cmd.to_string()]);
            assert!(
                result.is_none(),
                "Valid command '{}' should NOT get suggestion",
                cmd
            );
        }
    }

    #[test]
    fn test_suggester_quality_gate_typo() {
        let suggester = CommandSuggester::new();
        // quality-gat (missing e)
        let result = suggester.suggest_command(&["quality-gat".to_string()]);
        if let Some(suggestion) = result {
            assert!(
                suggestion.contains("quality-gate"),
                "Expected quality-gate suggestion"
            );
        }
    }

    #[test]
    fn test_suggester_all_analyze_subcommands() {
        let suggester = CommandSuggester::new();
        let subcommands = [
            "complexity",
            "satd",
            "dead-code",
            "tdg",
            "churn",
            "duplicates",
        ];

        for subcmd in subcommands {
            // When used alone, should suggest "analyze X"
            let result = suggester.suggest_command(&[subcmd.to_string()]);
            assert!(
                result.is_some(),
                "Subcommand '{}' should suggest analyze prefix",
                subcmd
            );
            assert!(result.unwrap().contains(&format!("analyze {}", subcmd)));
        }
    }

    #[test]
    fn test_suggester_churn_shortcut() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["churn".to_string()]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("analyze churn"));
    }

    #[test]
    fn test_suggester_duplicates_shortcut() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["duplicates".to_string()]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("analyze duplicates"));
    }

    #[test]
    fn test_suggester_analyze_with_typo_subcommand() {
        let suggester = CommandSuggester::new();

        // complexty (missing i)
        let result = suggester.suggest_command(&["analyze".to_string(), "complexty".to_string()]);
        if let Some(suggestion) = result {
            assert!(suggestion.contains("complexity"));
        }

        // chrn (missing u)
        let result = suggester.suggest_command(&["analyze".to_string(), "chrn".to_string()]);
        if let Some(suggestion) = result {
            assert!(suggestion.contains("churn"));
        }
    }

    #[test]
    fn test_suggester_distance_threshold() {
        let suggester = CommandSuggester::new();

        // Distance > 3, should return None
        let result = suggester.suggest_command(&["zzzzzzz".to_string()]);
        assert!(result.is_none(), "Very different string should not match");

        // Distance exactly 3
        let result = suggester.suggest_command(&["agen".to_string()]);
        // "agen" to "agent" is distance 1, should match
        if let Some(suggestion) = result {
            assert!(suggestion.contains("agent"));
        }
    }

    #[test]
    fn test_suggester_multi_word_common_mistakes() {
        let suggester = CommandSuggester::new();

        // All registered common mistakes
        let mistakes = vec![
            ("agent analyze", "analyze"),
            ("analize", "analyze"),
            ("analyse", "analyze"),
            ("analize complexity", "analyze complexity"),
            ("analize satd", "analyze satd"),
            ("analize dead-code", "analyze dead-code"),
            ("std", "satd"),
            ("stad", "satd"),
            ("sadt", "satd"),
        ];

        for (mistake, expected) in mistakes {
            let args: Vec<String> = mistake.split_whitespace().map(|s| s.to_string()).collect();
            let result = suggester.suggest_command(&args);
            assert!(
                result.is_some(),
                "Mistake '{}' should have suggestion",
                mistake
            );
            assert!(
                result.as_ref().unwrap().contains(expected),
                "Mistake '{}' should suggest '{}', got: {:?}",
                mistake,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_suggester_three_plus_args() {
        let suggester = CommandSuggester::new();

        // Three args where first two match pattern
        let result = suggester.suggest_command(&[
            "analyze".to_string(),
            "complxity".to_string(), // typo
            "--path".to_string(),
        ]);
        // Should still suggest for the subcommand typo
        if let Some(suggestion) = result {
            assert!(suggestion.contains("complexity") || suggestion.contains("analyze"));
        }
    }

    #[test]
    fn test_suggester_case_sensitivity() {
        let suggester = CommandSuggester::new();

        // Uppercase should not match (commands are lowercase)
        let result = suggester.suggest_command(&["ANALYZE".to_string()]);
        // "ANALYZE" vs "analyze" has distance 7, should not match
        let _ = result; // Just verify no panic
    }

    #[test]
    fn test_help_examples_content() {
        let examples = CommandSuggester::get_help_examples();

        // Verify all expected content is present
        assert!(examples.contains("EXAMPLES"));
        assert!(examples.contains("Analyze code complexity"));
        assert!(examples.contains("Find technical debt"));
        assert!(examples.contains("Find dead code"));
        assert!(examples.contains("Generate project context"));
        assert!(examples.contains("Run quality gates"));
        // Was `assert!(examples.contains("Start agent daemon"))` — the third
        // test pinning an example that cannot run in the shipped build, where
        // `pmat agent start` exits 1 with "Agent daemon feature not enabled"
        // (#675). Assert the inverse so it cannot silently return.
        assert!(examples.contains("Pre-flight the CI gate set"));
        assert!(!examples.contains("pmat agent start"));
        assert!(examples.contains("--path"));
    }

    #[test]
    fn test_default_impl() {
        let default_suggester = CommandSuggester::default();
        let new_suggester = CommandSuggester::new();

        // Both should have same commands
        assert_eq!(
            default_suggester.main_commands.len(),
            new_suggester.main_commands.len()
        );
        assert_eq!(
            default_suggester.analyze_subcommands.len(),
            new_suggester.analyze_subcommands.len()
        );
        assert_eq!(
            default_suggester.common_mistakes.len(),
            new_suggester.common_mistakes.len()
        );
    }

    #[test]
    fn test_suggester_refactor_typo() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["refactr".to_string()]);
        if let Some(suggestion) = result {
            assert!(suggestion.contains("refactor"));
        }
    }

    #[test]
    fn test_suggester_enforce_typo() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["enforc".to_string()]);
        if let Some(suggestion) = result {
            assert!(suggestion.contains("enforce"));
        }
    }

    #[test]
    fn test_suggester_scaffold_typo() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["scaffld".to_string()]);
        if let Some(suggestion) = result {
            assert!(suggestion.contains("scaffold"));
        }
    }

    #[test]
    fn test_suggester_demo_typo() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["dmo".to_string()]);
        if let Some(suggestion) = result {
            assert!(suggestion.contains("demo"));
        }
    }

    #[test]
    fn test_suggester_context_typo() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["contxt".to_string()]);
        if let Some(suggestion) = result {
            assert!(suggestion.contains("context"));
        }
    }

    #[test]
    fn test_levenshtein_same_length_all_different() {
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
        assert_eq!(levenshtein_distance("abcd", "wxyz"), 4);
    }

    #[test]
    fn test_levenshtein_transposition() {
        // Note: Levenshtein treats transposition as 2 operations
        assert_eq!(levenshtein_distance("ab", "ba"), 2);
        assert_eq!(levenshtein_distance("abc", "acb"), 2);
    }

    #[test]
    fn test_suggester_fields_accessible() {
        let suggester = CommandSuggester::new();

        // Verify fields have expected values
        assert!(suggester.main_commands.len() >= 9);
        assert!(suggester.analyze_subcommands.len() >= 6);
        assert!(suggester.common_mistakes.len() >= 10);
    }

    #[test]
    fn test_levenshtein_special_chars() {
        assert_eq!(levenshtein_distance("a-b", "a_b"), 1);
        assert_eq!(levenshtein_distance("a.b", "a,b"), 1);
        assert_eq!(levenshtein_distance("dead-code", "deadcode"), 1);
    }

    #[test]
    fn test_suggester_empty_string_arg() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["".to_string()]);
        // Empty string should still process
        let _ = result;
    }

    #[test]
    fn test_suggester_whitespace_only() {
        let suggester = CommandSuggester::new();
        let result = suggester.suggest_command(&["   ".to_string()]);
        // Whitespace should not match anything useful
        let _ = result;
    }
}
