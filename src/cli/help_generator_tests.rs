// help_generator_tests.rs — included by help_generator.rs
// Contains all unit tests for the help generator module.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::registry::{CommandMetadata, ExampleMetadata, FlagMetadata};

    fn sample_registry() -> CommandRegistry {
        let mut registry = CommandRegistry::new("2.0.0");

        // Add global flags
        registry.register_global_flag(FlagMetadata {
            name: "verbose".to_string(),
            short: Some('v'),
            long: Some("verbose".to_string()),
            description: "Enable verbose output".to_string(),
            default: None,
        });

        // Add analyze command with subcommands
        let complexity_sub = CommandMetadata::builder("complexity")
            .short_description("Analyze code complexity")
            .long_description("Calculate cyclomatic complexity for all functions")
            .aliases(["cx"])
            .argument(crate::cli::registry::ArgumentMetadata {
                name: "project-path".to_string(),
                short: Some('p'),
                long: Some("project-path".to_string()),
                description: "Path to project".to_string(),
                required: false,
                default: Some(".".to_string()),
                value_type: ValueType::Path,
                ..Default::default()
            })
            .example(ExampleMetadata {
                description: "Analyze current directory".to_string(),
                command: "pmat analyze complexity".to_string(),
                ..Default::default()
            })
            .category("analysis")
            .build();

        registry.register(
            CommandMetadata::builder("analyze")
                .short_description("Analyze code metrics")
                .long_description("Run various code analysis tools")
                .aliases(["a", "an"])
                .subcommand(complexity_sub)
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
                .tags(["generation", "ast"])
                .related("analyze")
                .build(),
        );

        registry
    }

    #[test]
    fn test_help_generator_creation() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        assert!(gen.width > 0);
    }

    #[test]
    fn test_generate_overview() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let overview = gen.generate_overview();

        assert!(overview.contains("pmat 2.0.0"));
        assert!(overview.contains("USAGE:"));
        assert!(overview.contains("COMMANDS:"));
        assert!(overview.contains("analyze"));
        assert!(overview.contains("context"));
    }

    #[test]
    fn test_generate_command_help() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("analyze");

        assert!(help.contains("analyze"));
        assert!(help.contains("USAGE:"));
        assert!(help.contains("SUBCOMMANDS:"));
        assert!(help.contains("complexity"));
    }

    #[test]
    fn test_generate_subcommand_help() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("analyze complexity");

        assert!(help.contains("complexity"));
        assert!(help.contains("cyclomatic complexity"));
        assert!(help.contains("OPTIONS:"));
        assert!(help.contains("--project-path"));
        assert!(help.contains("EXAMPLES:"));
    }

    #[test]
    fn test_generate_command_not_found() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("nonexistent");

        assert!(help.contains("error:"));
        assert!(help.contains("unrecognized command"));
        assert!(help.contains("Did you mean:"));
    }

    #[test]
    fn test_find_by_alias() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("ctx");

        assert!(help.contains("context"));
        assert!(help.contains("Generate project context"));
    }

    #[test]
    fn test_levenshtein_basic() {
        assert_eq!(levenshtein("analyze", "analyze"), 0);
        assert_eq!(levenshtein("analyze", "analize"), 1);
        assert_eq!(levenshtein("analyze", ""), 7);
        assert_eq!(levenshtein("", "test"), 4);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_format_usage_simple() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);

        let cmd = CommandMetadata::builder("test")
            .argument(crate::cli::registry::ArgumentMetadata {
                name: "file".to_string(),
                positional: true,
                required: true,
                ..Default::default()
            })
            .build();

        let usage = gen.format_usage(&cmd);
        assert!(usage.contains("<FILE>"));
    }

    #[test]
    fn test_format_usage_with_options() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);

        let cmd = CommandMetadata::builder("test")
            .argument(crate::cli::registry::ArgumentMetadata {
                name: "verbose".to_string(),
                short: Some('v'),
                long: Some("verbose".to_string()),
                positional: false,
                ..Default::default()
            })
            .build();

        let usage = gen.format_usage(&cmd);
        assert!(usage.contains("[OPTIONS]"));
    }

    #[test]
    fn test_overview_contains_global_flags() {
        let registry = sample_registry();
        let gen = HelpGenerator::new(registry);
        let overview = gen.generate_overview();

        assert!(overview.contains("OPTIONS:"));
        assert!(overview.contains("-v, --verbose"));
    }

    // ── HelpGenerator builder methods ──

    #[test]
    fn test_with_color_overrides_default() {
        let gen = HelpGenerator::new(CommandRegistry::new("1.0.0"))
            .with_color(false)
            .with_color(true);
        // No public getter, but observable via print_help (which dispatches to
        // print_colored when color=true). Just verify chaining compiles + runs.
        assert!(gen.print_help(None).is_ok());
    }

    #[test]
    fn test_with_width_overrides_default() {
        let gen = HelpGenerator::new(CommandRegistry::new("1.0.0")).with_width(120);
        // width is private; verify via overview generation succeeding.
        let _ = gen.generate_overview();
    }

    // ── print_help dispatcher arms ──

    #[test]
    fn test_print_help_with_path_invokes_command_help() {
        let gen = HelpGenerator::new(sample_registry()).with_color(false);
        // Some(path) → generate(path).
        gen.print_help(Some("analyze")).unwrap();
    }

    #[test]
    fn test_print_help_no_path_invokes_overview() {
        let gen = HelpGenerator::new(sample_registry()).with_color(false);
        // None → generate_overview().
        gen.print_help(None).unwrap();
    }

    #[test]
    fn test_print_help_color_true_uses_print_colored() {
        let gen = HelpGenerator::new(sample_registry()).with_color(true);
        // color=true exercises the print_colored branch which writes ANSI codes.
        gen.print_help(None).unwrap();
    }

    // ── format_flag ──

    #[test]
    fn test_format_flag_short_only_arm() {
        let gen = HelpGenerator::new(CommandRegistry::new("1.0.0"));
        let flag = FlagMetadata {
            name: "f".to_string(),
            short: Some('f'),
            long: None,
            description: "fast".to_string(),
            default: None,
        };
        let line = gen.format_flag(&flag);
        assert!(line.contains("-f"));
        assert!(line.contains("fast"));
    }

    #[test]
    fn test_format_flag_long_only_arm() {
        let gen = HelpGenerator::new(CommandRegistry::new("1.0.0"));
        let flag = FlagMetadata {
            name: "verbose".to_string(),
            short: None,
            long: Some("verbose".to_string()),
            description: "v".to_string(),
            default: None,
        };
        let line = gen.format_flag(&flag);
        assert!(line.contains("--verbose"));
    }

    #[test]
    fn test_format_flag_no_short_no_long_falls_back_to_name() {
        let gen = HelpGenerator::new(CommandRegistry::new("1.0.0"));
        let flag = FlagMetadata {
            name: "raw".to_string(),
            short: None,
            long: None,
            description: "d".to_string(),
            default: None,
        };
        let line = gen.format_flag(&flag);
        assert!(line.contains("raw"));
    }

    #[test]
    fn test_format_flag_with_default_appends_default_marker() {
        let gen = HelpGenerator::new(CommandRegistry::new("1.0.0"));
        let flag = FlagMetadata {
            name: "n".to_string(),
            short: Some('n'),
            long: Some("name".to_string()),
            description: "Name".to_string(),
            default: Some("anonymous".to_string()),
        };
        let line = gen.format_flag(&flag);
        assert!(line.contains("[default: anonymous]"));
    }

    // ── find_similar_commands ──

    #[test]
    fn test_find_similar_commands_ranks_close_matches() {
        let gen = HelpGenerator::new(sample_registry());
        // "analyz" → close to "analyze".
        let similar = gen.find_similar_commands("analyz", 5);
        let names: Vec<&str> = similar.iter().map(|(s, _)| s.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("analyze")));
    }

    #[test]
    fn test_find_similar_commands_filters_dissimilar_when_distance_exceeds_query_len() {
        let gen = HelpGenerator::new(sample_registry());
        // "z" has length 1; nothing should pass the `score <= query.len()` filter.
        let similar = gen.find_similar_commands("z", 5);
        // Either empty (filtered out) or only entries with distance ≤ 1.
        for (_, score) in &similar {
            assert!(*score <= 1);
        }
    }

    // ── format_command_help: deprecated branch + execution_time::Slow ──

    #[test]
    fn test_format_command_help_deprecated_arm_emits_deprecation_warning() {
        let mut registry = CommandRegistry::new("1.0.0");
        let dep = crate::cli::registry::DeprecationInfo {
            since_version: "0.5.0".to_string(),
            removal_version: None,
            replacement: Some("new-command".to_string()),
            reason: "legacy".to_string(),
        };
        registry.register(
            CommandMetadata::builder("old")
                .short_description("Old cmd")
                .deprecated(dep)
                .build(),
        );
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("old");
        assert!(help.contains("DEPRECATED: legacy"));
        assert!(help.contains("Use 'new-command' instead"));
    }

    #[test]
    fn test_format_command_help_slow_execution_appends_note() {
        let mut registry = CommandRegistry::new("1.0.0");
        registry.register(
            CommandMetadata::builder("slowcmd")
                .short_description("Slow")
                .execution_time(ExecutionTime::Slow)
                .build(),
        );
        let gen = HelpGenerator::new(registry);
        let help = gen.generate("slowcmd");
        assert!(help.contains("This command may take several seconds"));
    }

    // ── format_command_not_found: with + without suggestions ──

    #[test]
    fn test_format_command_not_found_emits_error_message() {
        let gen = HelpGenerator::new(sample_registry());
        let help = gen.generate("nonexistent");
        assert!(help.contains("error: unrecognized command 'nonexistent'"));
        assert!(help.contains("'pmat --help'"));
    }

    #[test]
    fn test_format_command_not_found_with_close_match_suggests() {
        let gen = HelpGenerator::new(sample_registry());
        // "analyz" close to "analyze".
        let help = gen.generate("analyz");
        assert!(help.contains("Did you mean:"));
    }
}
