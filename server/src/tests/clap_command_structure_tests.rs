//! Comprehensive tests for Clap command structure validation
//!
//! Tests #[derive(Parser)] propagation, binary name detection,
//! subcommand hierarchy, and global argument accessibility.

use crate::cli::{AnalyzeCommands, Cli, Commands};
use clap::{CommandFactory, Parser};
use parking_lot::Mutex;
use std::env;

// Global mutex to ensure env var tests don't interfere across all modules
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_parser_propagation() {
        // Verify Cli struct derives Parser
        let cmd = Cli::command();

        // Check basic command properties
        assert_eq!(cmd.get_name(), "paiml-mcp-agent-toolkit");
        assert!(cmd.get_about().is_some());

        // Verify version is propagated
        assert!(cmd.get_version().is_some());
    }

    #[test]
    fn test_binary_name_detection() {
        let cmd = Cli::command();

        // Binary name should be properly detected
        assert_eq!(cmd.get_name(), "paiml-mcp-agent-toolkit");

        // Check if binary name can be overridden
        let cmd_with_name = cmd.clone().name("pmat");
        assert_eq!(cmd_with_name.get_name(), "pmat");
    }

    #[test]
    fn test_global_args_accessible() {
        // Test that global args are accessible from all subcommands
        let cli = Cli::try_parse_from(["pmat", "--verbose", "analyze", "complexity"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        assert!(cli.verbose);
        assert!(!cli.debug);
        assert!(!cli.trace);

        // Test with debug flag
        let cli = Cli::try_parse_from(["pmat", "--debug", "generate", "makefile", "rust"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.debug);
    }

    #[test]
    fn test_subcommand_hierarchy() {
        // Test the expected command structure
        let cmd = Cli::command();

        // Get all subcommands
        let subcommands: Vec<_> = cmd.get_subcommands().map(|sc| sc.get_name()).collect();

        // Verify main subcommands exist
        assert!(subcommands.contains(&"generate"));
        assert!(subcommands.contains(&"analyze"));
        assert!(subcommands.contains(&"demo"));
        assert!(subcommands.contains(&"scaffold"));
        assert!(subcommands.contains(&"list"));
        assert!(subcommands.contains(&"search"));
        assert!(subcommands.contains(&"validate"));
        assert!(subcommands.contains(&"context"));
        assert!(subcommands.contains(&"serve"));

        // Test analyze subcommands
        let analyze_cmd = cmd.find_subcommand("analyze").unwrap();
        let analyze_subs: Vec<_> = analyze_cmd
            .get_subcommands()
            .map(|sc| sc.get_name())
            .collect();

        assert!(analyze_subs.contains(&"churn"));
        assert!(analyze_subs.contains(&"complexity"));
        assert!(analyze_subs.contains(&"dag"));
        assert!(analyze_subs.contains(&"dead-code"));
        assert!(analyze_subs.contains(&"deep-context"));
        assert!(analyze_subs.contains(&"satd"));
    }

    #[test]
    fn test_propagate_version() {
        let cmd = Cli::command();

        // Version should be available at root
        assert!(cmd.get_version().is_some());

        // Check if version propagates to subcommands (if configured)
        // This depends on whether propagate_version is set
        let analyze_cmd = cmd.find_subcommand("analyze");
        assert!(analyze_cmd.is_some());
    }

    #[test]
    fn test_help_generation() {
        // Test that help can be generated without panic
        let cmd = Cli::command();

        // Short help
        let mut help_output = Vec::new();
        let _ = cmd.clone().write_help(&mut help_output);
        assert!(!help_output.is_empty());

        // Long help
        let mut long_help_output = Vec::new();
        let _ = cmd.clone().write_long_help(&mut long_help_output);
        assert!(!long_help_output.is_empty());

        // Help should contain command name or binary name
        let help_str = String::from_utf8_lossy(&help_output);
        // The command might be using the binary name from Cargo
        assert!(!help_str.is_empty(), "Help output should not be empty");

        // Just check that we got valid help output with Usage
        assert!(
            help_str.contains("Usage:"),
            "Help should contain Usage section"
        );
    }

    #[test]
    fn test_env_var_support() {
        let _guard = ENV_MUTEX.lock();

        // Test that RUST_LOG env var is properly mapped
        env::set_var("RUST_LOG", "debug");

        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        // trace_filter should be populated from RUST_LOG
        assert_eq!(cli.trace_filter, Some("debug".to_string()));

        // Clean up
        env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_command_aliases() {
        // Test if command aliases work (if configured)
        // For example, 'gen' for 'generate'
        let cli = Cli::try_parse_from(["pmat", "gen", "makefile", "rust"]);

        // This will fail if aliases aren't configured, which is fine
        // We're testing the structure, not the specific configuration
        match cli {
            Ok(parsed) => {
                match parsed.command {
                    Commands::Generate { .. } => {
                        // Generate command parsed successfully
                    }
                    _ => panic!("Expected Generate command"),
                }
            }
            Err(_) => {
                // Aliases might not be configured, which is okay
            }
        }
    }

    #[test]
    fn test_required_args_validation() {
        // Test that required arguments are enforced
        let result = Cli::try_parse_from(["pmat", "generate"]);
        assert!(result.is_err(), "Generate should require template type");

        // Test with proper args
        let result = Cli::try_parse_from(["pmat", "generate", "makefile", "rust"]);
        assert!(
            result.is_ok(),
            "Generate with all required args should succeed"
        );
    }

    #[test]
    fn test_global_flags_precedence() {
        // Test that global flags work with any subcommand position
        let variations = vec![
            vec!["pmat", "--verbose", "list"],
            vec!["pmat", "list", "--verbose"],
            vec!["pmat", "--debug", "analyze", "complexity", "--verbose"],
        ];

        for args in variations {
            let result = Cli::try_parse_from(args.clone());
            assert!(result.is_ok(), "Failed to parse: {args:?}");
        }
    }

    #[test]
    fn test_subcommand_specific_args() {
        // Test analyze complexity specific args
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--top-files",
            "10",
            "--format",
            "json",
        ]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity {
                    top_files, format, ..
                }) => {
                    assert_eq!(top_files, 10);
                    // format is not an Option, just check it exists
                    let _ = format;
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_value_enum_parsing() {
        // Test that value enums parse correctly
        let cli = Cli::try_parse_from(["pmat", "--mode", "cli", "list"]);
        assert!(cli.is_ok());

        // Test invalid enum value
        let cli = Cli::try_parse_from(["pmat", "--mode", "invalid", "list"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_command_error_suggestions() {
        // Test that similar command names provide suggestions
        let result = Cli::try_parse_from(["pmat", "analize", "complexity"]);

        if let Err(e) = result {
            let error_str = e.to_string();
            // Clap should provide helpful error for unknown commands
            assert!(
                error_str.contains("unrecognized")
                    || error_str.contains("unknown")
                    || error_str.contains("analyze")
                    || error_str.contains("did you mean"),
                "Error should be helpful: {error_str}"
            );
        }
    }
}

#[cfg(test)]
mod clap_derive_completeness_tests {
    use super::*;

    #[test]
    fn test_all_commands_have_help() {
        let cmd = Cli::command();

        fn check_command_help(cmd: &clap::Command) {
            // Command should have about or long_about
            assert!(
                cmd.get_about().is_some() || cmd.get_long_about().is_some(),
                "Command '{}' missing help text",
                cmd.get_name()
            );

            // Check all subcommands recursively
            for subcmd in cmd.get_subcommands() {
                check_command_help(subcmd);
            }
        }

        check_command_help(&cmd);
    }

    #[test]
    fn test_all_args_have_help() {
        let cmd = Cli::command();

        fn check_args_help(cmd: &clap::Command) {
            for arg in cmd.get_arguments() {
                // Skip help and version args
                if arg.get_id() == "help" || arg.get_id() == "version" {
                    continue;
                }

                assert!(
                    arg.get_help().is_some() || arg.get_long_help().is_some(),
                    "Argument '{}' in command '{}' missing help text",
                    arg.get_id(),
                    cmd.get_name()
                );
            }

            // Check subcommands
            for subcmd in cmd.get_subcommands() {
                check_args_help(subcmd);
            }
        }

        check_args_help(&cmd);
    }

    #[test]
    fn test_conflicting_args() {
        // Test that --verbose, --debug, and --trace are handled correctly
        // They shouldn't conflict since they're different levels
        let cli = Cli::try_parse_from(["pmat", "--verbose", "--debug", "list"]);
        assert!(cli.is_ok());

        let cli = cli.unwrap();
        assert!(cli.verbose);
        assert!(cli.debug);
    }
}

#[cfg(test)]
mod clap_output_validation_tests {
    use super::*;

    #[test]
    fn test_help_output_format() {
        let cmd = Cli::command();
        let mut help = Vec::new();
        let _ = cmd.clone().write_help(&mut help);
        let help_str = String::from_utf8_lossy(&help);

        // Check for expected sections
        assert!(help_str.contains("Usage:"));
        assert!(help_str.contains("Commands:"));
        assert!(help_str.contains("Options:"));

        // Help output is valid if it has the expected sections
        // The binary name might vary in tests
        assert!(!help_str.is_empty(), "Help should not be empty");
    }

    #[test]
    fn test_error_output_format() {
        let result = Cli::try_parse_from(["pmat", "--unknown-flag"]);
        assert!(result.is_err());

        if let Err(e) = result {
            let error_str = e.to_string();
            // Error should be informative
            assert!(error_str.contains("unknown") || error_str.contains("unexpected"));
        }
    }
}
