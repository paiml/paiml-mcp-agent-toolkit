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
}
