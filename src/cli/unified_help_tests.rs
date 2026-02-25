// Tests for unified help: NLP, graph, service, and levenshtein

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new("2.0.0");

        // Analyze command with subcommands
        let complexity_sub = CommandMetadata::builder("complexity")
            .short_description("Analyze code complexity")
            .long_description("Calculate cyclomatic complexity for all functions")
            .tags(["quality", "metrics", "complexity"])
            .build();

        let satd_sub = CommandMetadata::builder("satd")
            .short_description("Find technical debt")
            .long_description("Detect TODO/FIXME/HACK comments indicating technical debt")
            .tags(["quality", "debt", "satd"])
            .build();

        registry.register(
            CommandMetadata::builder("analyze")
                .short_description("Analyze code metrics")
                .long_description("Run various code analysis tools on your project")
                .aliases(["a", "an"])
                .subcommand(complexity_sub)
                .subcommand(satd_sub)
                .category("analysis")
                .tags(["quality", "metrics"])
                .related("context")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("context")
                .short_description("Generate project context")
                .long_description("Generate AI-friendly project context using AST analysis")
                .aliases(["ctx"])
                .category("generation")
                .tags(["generation", "ast", "context"])
                .related("analyze")
                .build(),
        );

        registry.register(
            CommandMetadata::builder("quality-gate")
                .short_description("Run quality gates")
                .long_description("Check code quality against configured thresholds")
                .aliases(["qg", "gate"])
                .category("quality")
                .tags(["quality", "ci", "gate"])
                .build(),
        );

        registry
    }

    mod nlp_tests {
        use super::*;

        #[test]
        fn test_nlp_processor_creation() {
            let nlp = HelpNlpProcessor::new();
            let tokens = nlp.preprocess("analyze code complexity");
            assert!(!tokens.is_empty());
        }

        #[test]
        fn test_preprocess_removes_stop_words() {
            let nlp = HelpNlpProcessor::new();
            let tokens = nlp.preprocess("run the pmat command");
            // "run", "the", "pmat", "command" should be filtered
            assert!(tokens.is_empty() || !tokens.contains(&"the".to_string()));
        }

        #[test]
        fn test_term_frequency() {
            let nlp = HelpNlpProcessor::new();
            let tf = nlp.term_frequency("code code code analysis");
            // "code" appears three times, "analysis" once
            let code_freq = tf.get("code").unwrap_or(&0.0);
            let analysis_freq = tf.get("analysi").unwrap_or(&0.0);
            assert!(
                code_freq > analysis_freq,
                "code freq {} should be > analysis freq {}",
                code_freq,
                analysis_freq
            );
        }

        #[test]
        fn test_bm25_score() {
            let nlp = HelpNlpProcessor::new();
            let score1 = nlp.bm25_score("complexity", "analyze complexity metrics", 1.2, 0.75);
            let score2 = nlp.bm25_score("complexity", "generate context ast", 1.2, 0.75);
            // Query "complexity" should match better with first doc
            assert!(score1 > score2);
        }
    }

    mod graph_tests {
        use super::*;

        #[test]
        fn test_graph_creation() {
            let graph = CommandGraph::new();
            assert!(graph.command_to_node.is_empty());
        }

        #[test]
        fn test_build_from_registry() {
            let registry = sample_registry();
            let mut graph = CommandGraph::new();
            graph.build_from_registry(&registry);

            // Should have all commands indexed
            assert!(graph.command_to_node.contains_key("analyze"));
            assert!(graph.command_to_node.contains_key("context"));
            assert!(graph.command_to_node.contains_key("analyze complexity"));
        }

        #[test]
        fn test_pagerank_importance() {
            let registry = sample_registry();
            let mut graph = CommandGraph::new();
            graph.build_from_registry(&registry);

            // All commands should have non-zero importance
            let importance = graph.importance("analyze");
            assert!(importance >= 0.0);
        }

        #[test]
        fn test_top_k_important() {
            let registry = sample_registry();
            let mut graph = CommandGraph::new();
            graph.build_from_registry(&registry);

            let top = graph.top_k_important(3);
            assert!(top.len() <= 3);
        }
    }

    mod unified_help_tests {
        use super::*;

        #[test]
        fn test_unified_help_creation() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);
            assert!(!help.command_docs.is_empty());
        }

        #[test]
        fn test_lookup_exact_match() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let response = help.lookup("analyze");
            assert!(matches!(response, HelpResponse::Exact(_)));
        }

        #[test]
        fn test_lookup_fuzzy_match() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            // Typo: "analize" instead of "analyze"
            let response = help.lookup("analize");
            assert!(matches!(response, HelpResponse::DidYouMean { .. }));
        }

        #[test]
        fn test_lookup_semantic_search() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            // Query that doesn't match any command name
            let response = help.lookup("how to check code quality");
            match response {
                HelpResponse::SearchResults { results, .. } => {
                    assert!(!results.is_empty());
                }
                _ => panic!("Expected SearchResults"),
            }
        }

        #[test]
        fn test_search_returns_relevant_results() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let results = help.search("complexity", 3);
            assert!(!results.is_empty());
            // First result should be related to complexity
            assert!(
                results[0].command.contains("complexity")
                    || results[0].description.to_lowercase().contains("complex")
            );
        }

        #[test]
        fn test_search_ranking() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let results = help.search("technical debt", 5);
            // satd command should rank highly for "technical debt"
            let satd_result = results.iter().find(|r| r.command.contains("satd"));
            assert!(satd_result.is_some());
        }

        #[test]
        fn test_get_important_commands() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let important = help.get_important_commands(5);
            assert!(!important.is_empty());
        }

        #[test]
        fn test_get_by_tag() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let quality_cmds = help.get_by_tag("quality");
            assert!(!quality_cmds.is_empty());
        }

        #[test]
        fn test_get_by_category() {
            let registry = sample_registry();
            let help = UnifiedHelpService::new(registry);

            let analysis_cmds = help.get_by_category("analysis");
            assert_eq!(analysis_cmds.len(), 1);
            assert_eq!(analysis_cmds[0].name, "analyze");
        }
    }

    mod levenshtein_tests {
        use super::*;

        #[test]
        fn test_levenshtein_identical() {
            assert_eq!(levenshtein("test", "test"), 0);
        }

        #[test]
        fn test_levenshtein_one_char() {
            assert_eq!(levenshtein("test", "fest"), 1); // One substitution
            assert_eq!(levenshtein("test", "tests"), 1); // One insertion
            assert_eq!(levenshtein("test", "tes"), 1); // One deletion
        }

        #[test]
        fn test_levenshtein_empty() {
            assert_eq!(levenshtein("", "test"), 4);
            assert_eq!(levenshtein("test", ""), 4);
            assert_eq!(levenshtein("", ""), 0);
        }
    }
}
