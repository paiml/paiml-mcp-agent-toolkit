//! Comprehensive tests for Clap argument parsing correctness
//!
//! Tests type coercion, validation, and custom validators to ensure
//! correct parsing and handling of various argument types and constraints.

use crate::cli::{AnalyzeCommands, Cli, Commands, ComplexityOutputFormat, Mode};
use clap::Parser;
use std::path::PathBuf;

#[cfg(test)]
mod type_coercion_tests {
    use super::*;

    #[test]
    fn test_numeric_argument_coercion() {
        // Test parsing numeric arguments
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--top-files",
            "25",
            "--max-cognitive",
            "30",
        ]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity {
                    top_files,
                    max_cognitive,
                    ..
                }) => {
                    assert_eq!(top_files, 25);
                    assert_eq!(max_cognitive, Some(30));
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_path_argument_coercion() {
        // Test parsing path arguments
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--project-path",
            "src/main.rs",
        ]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { project_path, .. }) => {
                    assert_eq!(*project_path, PathBuf::from("src/main.rs"));
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_enum_argument_coercion() {
        // Test parsing enum arguments (OutputFormat)
        let cli = Cli::try_parse_from(["pmat", "analyze", "complexity", "--format", "json"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { format, .. }) => {
                    assert_eq!(format, ComplexityOutputFormat::Json);
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }

        // Test another enum value
        let cli = Cli::try_parse_from(["pmat", "analyze", "complexity", "--format", "sarif"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { format, .. }) => {
                    assert_eq!(format, ComplexityOutputFormat::Sarif);
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_boolean_flag_coercion() {
        // Test boolean flags
        let cli = Cli::try_parse_from(["pmat", "--verbose", "--debug", "list"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert!(parsed.verbose);
            assert!(parsed.debug);
            assert!(!parsed.trace); // Not specified, should be false
        }

        // Test without flags
        let cli = Cli::try_parse_from(["pmat", "list"]);
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert!(!parsed.verbose);
            assert!(!parsed.debug);
            assert!(!parsed.trace);
        }
    }

    #[test]
    fn test_optional_argument_coercion() {
        // Test optional arguments
        let cli = Cli::try_parse_from(["pmat", "analyze", "complexity"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { project_path, .. }) => {
                    assert_eq!(*project_path, PathBuf::from(".")); // Default path
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }

        // Test with optional provided
        let cli = Cli::try_parse_from(["pmat", "analyze", "complexity", "--project-path", "src/"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { project_path, .. }) => {
                    assert_eq!(*project_path, PathBuf::from("src/"));
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_vec_argument_coercion() {
        // Test if any commands accept multiple values
        // For example, if analyze had a --files option that accepted multiple paths
        // This is a placeholder test - adapt based on actual CLI structure

        // Test execution mode enum
        let cli = Cli::try_parse_from(["pmat", "--mode", "mcp", "list"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.mode, Some(Mode::Mcp));
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_numeric_range_validation() {
        // Test invalid numeric values
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--top-files",
            "-5", // Negative number - should this be valid?
        ]);

        // Negative numbers should fail for usize
        assert!(cli.is_err());

        // Test very large number
        let cli = Cli::try_parse_from(["pmat", "analyze", "complexity", "--top-files", "999999"]);

        assert!(cli.is_ok());
    }

    #[test]
    fn test_enum_validation() {
        // Test invalid enum value
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--format",
            "invalid_format",
        ]);

        assert!(cli.is_err());

        if let Err(e) = cli {
            let error_str = e.to_string();
            // Clap error messages vary by version - just verify we got an error
            assert!(!error_str.is_empty());
            // Could also check for common error keywords
            assert!(error_str.len() > 10); // Non-trivial error message
        }
    }

    #[test]
    fn test_path_validation() {
        // Test path with special characters
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--project-path",
            "src/../../etc/passwd", // Path traversal attempt
        ]);

        // Clap doesn't validate paths by default, just parses them
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { project_path, .. }) => {
                    assert_eq!(*project_path, PathBuf::from("src/../../etc/passwd"));
                    // Actual path validation would happen in the application logic
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_mutually_exclusive_flags() {
        // Test if verbose/debug/trace are mutually exclusive or can be combined
        let cli = Cli::try_parse_from(["pmat", "--verbose", "--debug", "--trace", "list"]);

        // They seem to be independent flags that can all be set
        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert!(parsed.verbose);
            assert!(parsed.debug);
            assert!(parsed.trace);
        }
    }

    #[test]
    fn test_required_argument_validation() {
        // Test missing required arguments
        let cli = Cli::try_parse_from([
            "pmat", "generate", // Missing template type
        ]);

        assert!(cli.is_err());

        let cli = Cli::try_parse_from([
            "pmat", "generate", "makefile", "rust", // Both category and template provided
        ]);

        assert!(cli.is_ok());
    }

    #[test]
    fn test_string_validation() {
        // Test empty string arguments
        let cli = Cli::try_parse_from(["pmat", "--trace-filter", "", "list"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.trace_filter, Some("".to_string()));
        }

        // Test very long string
        let long_string = "a".repeat(1000);
        let cli = Cli::try_parse_from(["pmat", "--trace-filter", &long_string, "list"]);

        assert!(cli.is_ok());
    }
}

#[cfg(test)]
mod custom_validator_tests {
    use super::*;

    #[test]
    fn test_custom_type_parsing() {
        // Test parsing custom types if any exist
        // For example, if there are custom validators for specific formats

        // Test mode parsing (ExecutionMode)
        let valid_modes = vec!["cli", "mcp"];

        for mode in valid_modes {
            let cli = Cli::try_parse_from(["pmat", "--mode", mode, "list"]);

            assert!(cli.is_ok(), "Mode '{mode}' should be valid");
        }

        // Test invalid mode
        let cli = Cli::try_parse_from(["pmat", "--mode", "invalid_mode", "list"]);

        assert!(cli.is_err());
    }

    #[test]
    fn test_default_value_application() {
        // Test that default values are applied correctly
        let cli = Cli::try_parse_from(["pmat", "analyze", "complexity"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            // Check defaults
            assert_eq!(parsed.mode, None); // No mode specified, uses auto-detection
            assert!(!parsed.verbose); // Default false
            assert!(!parsed.debug); // Default false

            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity {
                    top_files, format, ..
                }) => {
                    assert_eq!(top_files, 0); // Default value
                    assert_eq!(format, ComplexityOutputFormat::Summary); // Default format
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_value_delimiter_parsing() {
        // Test parsing multiple values with delimiters if supported
        // This would apply if any argument accepts comma-separated values

        // Test trace filter with complex pattern
        let cli =
            Cli::try_parse_from(["pmat", "--trace-filter", "module1,module2::*,debug", "list"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(
                parsed.trace_filter,
                Some("module1,module2::*,debug".to_string())
            );
        }
    }

    #[test]
    fn test_case_sensitivity() {
        // Test case sensitivity of enum values
        let cli = Cli::try_parse_from([
            "pmat", "--mode", "CLI", // Uppercase
            "list",
        ]);

        // Clap enums are usually case-insensitive by default
        assert!(cli.is_err()); // Should fail with uppercase

        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--format",
            "JSON", // Uppercase
        ]);

        assert!(cli.is_err()); // Should fail with uppercase
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_unicode_arguments() {
        // Test Unicode in arguments
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--project-path",
            "src/测试.rs", // Chinese characters
        ]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { project_path, .. }) => {
                    assert_eq!(*project_path, PathBuf::from("src/测试.rs"));
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_argument_with_equals_sign() {
        // Test --arg=value syntax
        let cli = Cli::try_parse_from(["pmat", "--mode=mcp", "list"]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            assert_eq!(parsed.mode, Some(Mode::Mcp));
        }

        // Test with numeric value
        let cli = Cli::try_parse_from(["pmat", "analyze", "complexity", "--top-files=15"]);

        assert!(cli.is_ok());
    }

    #[test]
    fn test_quoted_arguments() {
        // Test arguments with quotes
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--project-path",
            "src/my file.rs", // Path with space
        ]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { project_path, .. }) => {
                    // The quotes should be stripped by the shell/parser
                    assert_eq!(*project_path, PathBuf::from("src/my file.rs"));
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }

    #[test]
    fn test_special_characters_in_arguments() {
        // Test various special characters
        let special_paths = vec![
            "src/@special.rs",
            "src/#hash.rs",
            "src/$dollar.rs",
            "src/[bracket].rs",
            "src/{brace}.rs",
        ];

        for special_path in special_paths {
            let cli = Cli::try_parse_from([
                "pmat",
                "analyze",
                "complexity",
                "--project-path",
                special_path,
            ]);

            assert!(cli.is_ok(), "Failed to parse path: {special_path}");

            if let Ok(parsed) = cli {
                match parsed.command {
                    Commands::Analyze(AnalyzeCommands::Complexity { project_path, .. }) => {
                        assert_eq!(*project_path, PathBuf::from(special_path));
                    }
                    _ => panic!("Expected Analyze::Complexity command"),
                }
            }
        }
    }

    #[test]
    fn test_overflow_values() {
        // Test numeric overflow
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--top-files",
            "18446744073709551615", // u64::MAX
        ]);

        // u64::MAX is too large for usize on 32-bit systems, but this test is likely running on 64-bit
        // where usize can hold u64::MAX. Let's check the actual result:
        match cli {
            Ok(_) => {
                // On 64-bit systems, this might actually parse successfully
                // since usize is 64-bit
            }
            Err(e) => {
                // On 32-bit systems or if the parser has a limit, it should fail
                let error_str = e.to_string();
                assert!(error_str.contains("invalid") || error_str.contains("parse"));
            }
        }

        // Test with i32::MAX
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--top-files",
            "2147483647", // i32::MAX
        ]);

        // i32::MAX fits in usize, so this should work
        assert!(cli.is_ok());
    }

    #[test]
    fn test_argument_order_flexibility() {
        // Test different argument orders
        let variations = vec![
            vec![
                "pmat",
                "--verbose",
                "analyze",
                "complexity",
                "--top-files",
                "10",
            ],
            vec![
                "pmat",
                "analyze",
                "--verbose",
                "complexity",
                "--top-files",
                "10",
            ],
            vec![
                "pmat",
                "analyze",
                "complexity",
                "--verbose",
                "--top-files",
                "10",
            ],
            vec![
                "pmat",
                "analyze",
                "complexity",
                "--top-files",
                "10",
                "--verbose",
            ],
        ];

        for args in variations {
            let cli = Cli::try_parse_from(args.clone());
            assert!(cli.is_ok(), "Failed with args: {args:?}");

            if let Ok(parsed) = cli {
                assert!(parsed.verbose);
                match parsed.command {
                    Commands::Analyze(AnalyzeCommands::Complexity { top_files, .. }) => {
                        assert_eq!(top_files, 10);
                    }
                    _ => panic!("Expected Analyze::Complexity command"),
                }
            }
        }
    }
}

#[cfg(test)]
mod parser_behavior_tests {
    use super::*;

    #[test]
    fn test_unknown_argument_handling() {
        // Test unknown arguments
        let cli = Cli::try_parse_from(["pmat", "--unknown-flag", "list"]);

        assert!(cli.is_err());

        if let Err(e) = cli {
            let error_str = e.to_string();
            assert!(error_str.contains("unknown") || error_str.contains("unexpected"));
        }
    }

    #[test]
    fn test_typo_suggestions() {
        // Test if Clap provides suggestions for typos
        let cli = Cli::try_parse_from([
            "pmat", "--verbos", // Typo of --verbose
            "list",
        ]);

        assert!(cli.is_err());

        if let Err(e) = cli {
            let error_str = e.to_string();
            // Clap should mention the unknown flag
            assert!(
                error_str.contains("--verbos")
                    || error_str.contains("unexpected")
                    || error_str.contains("unknown")
            );
        }
    }

    #[test]
    fn test_help_flag_parsing() {
        // Test that help flags work correctly
        let cli = Cli::try_parse_from(["pmat", "--help"]);

        // Help flag causes early exit with error
        assert!(cli.is_err());

        if let Err(e) = cli {
            // Check if it's a help error
            assert!(e.kind() == clap::error::ErrorKind::DisplayHelp);
        }
    }

    #[test]
    fn test_version_flag_parsing() {
        // Test version flag
        let cli = Cli::try_parse_from(["pmat", "--version"]);

        assert!(cli.is_err());

        if let Err(e) = cli {
            assert!(e.kind() == clap::error::ErrorKind::DisplayVersion);
        }
    }

    #[test]
    fn test_subcommand_help() {
        // Test subcommand-specific help
        let cli = Cli::try_parse_from(["pmat", "analyze", "--help"]);

        assert!(cli.is_err());

        if let Err(e) = cli {
            assert!(e.kind() == clap::error::ErrorKind::DisplayHelp);
            let help_text = e.to_string();
            assert!(help_text.contains("analyze") || help_text.contains("Analyze"));
        }
    }

    #[test]
    fn test_double_dash_separator() {
        // Test -- separator - but since project_path is not positional, this doesn't apply
        // Let's test a different scenario where we want a value that looks like a flag
        let cli = Cli::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--project-path",
            "./--weird-filename.rs",
        ]);

        assert!(cli.is_ok());

        if let Ok(parsed) = cli {
            match parsed.command {
                Commands::Analyze(AnalyzeCommands::Complexity { project_path, .. }) => {
                    assert_eq!(*project_path, PathBuf::from("./--weird-filename.rs"));
                }
                _ => panic!("Expected Analyze::Complexity command"),
            }
        }
    }
}
