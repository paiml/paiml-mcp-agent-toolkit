#[cfg(test)]
mod tests {
    use crate::cli::{AnalyzeCommands, Cli, Commands};
    use crate::models::churn::ChurnOutputFormat;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_analyze_churn_command_parsing() {
        // Test basic analyze churn command
        let args = vec!["paiml-mcp-agent-toolkit", "analyze", "churn"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Analyze(analyze_cmd) => {
                match analyze_cmd {
                    AnalyzeCommands::Churn {
                        days,
                        project_path,
                        format,
                        output,
                    } => {
                        assert_eq!(days, 30); // Default value
                        assert_eq!(project_path, PathBuf::from(".")); // Default value
                        assert_eq!(format, ChurnOutputFormat::Summary); // Default
                        assert!(output.is_none());
                    }
                    AnalyzeCommands::Dag { .. } => {
                        panic!("Expected Churn command, got Dag");
                    }
                    AnalyzeCommands::Complexity { .. } => {
                        panic!("Expected Churn command, got Complexity");
                    }
                    AnalyzeCommands::DeadCode { .. } => {
                        panic!("Expected Churn command, got DeadCode");
                    }
                    AnalyzeCommands::Satd { .. } => {
                        panic!("Expected Churn command, got Satd");
                    }
                    AnalyzeCommands::DeepContext { .. } => {
                        panic!("Expected Churn command, got DeepContext");
                    }
                    AnalyzeCommands::Tdg { .. } => {
                        panic!("Expected Churn command, got Tdg");
                    }
                    AnalyzeCommands::Makefile { .. } => {
                        panic!("Expected Churn command, got Makefile");
                    }
                }
            }
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn test_analyze_churn_with_all_options() {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "analyze",
            "churn",
            "--days",
            "90",
            "--project-path",
            "/tmp/test",
            "--format",
            "markdown",
            "-o",
            "report.md",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Analyze(analyze_cmd) => match analyze_cmd {
                AnalyzeCommands::Churn {
                    days,
                    project_path,
                    format,
                    output,
                } => {
                    assert_eq!(days, 90);
                    assert_eq!(project_path, PathBuf::from("/tmp/test"));
                    assert_eq!(format, ChurnOutputFormat::Markdown);
                    assert_eq!(output, Some(PathBuf::from("report.md")));
                }
                AnalyzeCommands::Dag { .. } => {
                    panic!("Expected Churn command, got Dag");
                }
                AnalyzeCommands::Complexity { .. } => {
                    panic!("Expected Churn command, got Complexity");
                }
                AnalyzeCommands::DeadCode { .. } => {
                    panic!("Expected Churn command, got DeadCode");
                }
                AnalyzeCommands::Satd { .. } => {
                    panic!("Expected Churn command, got Satd");
                }
                AnalyzeCommands::DeepContext { .. } => {
                    panic!("Expected Churn command, got DeepContext");
                }
                AnalyzeCommands::Tdg { .. } => {
                    panic!("Expected Churn command, got Tdg");
                }
                AnalyzeCommands::Makefile { .. } => {
                    panic!("Expected Churn command, got Makefile");
                }
            },
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn test_analyze_churn_format_options() {
        // Test each format option
        let formats = vec!["json", "markdown", "csv", "summary"];

        for fmt in formats {
            let args = vec![
                "paiml-mcp-agent-toolkit",
                "analyze",
                "churn",
                "--format",
                fmt,
            ];
            let cli = Cli::try_parse_from(args).unwrap();

            match cli.command {
                Commands::Analyze(AnalyzeCommands::Churn { format, .. }) => match fmt {
                    "json" => assert_eq!(format, ChurnOutputFormat::Json),
                    "markdown" => assert_eq!(format, ChurnOutputFormat::Markdown),
                    "csv" => assert_eq!(format, ChurnOutputFormat::Csv),
                    "summary" => assert_eq!(format, ChurnOutputFormat::Summary),
                    _ => unreachable!(),
                },
                _ => panic!("Expected Analyze command"),
            }
        }
    }

    #[test]
    fn test_analyze_churn_invalid_format() {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "analyze",
            "churn",
            "--format",
            "invalid",
        ];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_churn_short_flags() {
        let args = vec![
            "paiml-mcp-agent-toolkit",
            "analyze",
            "churn",
            "-d",
            "7", // Short form of --days
            "-p",
            "/tmp", // Short form of --project-path
            "--format",
            "csv",
            "-o",
            "out.csv", // Short form of --output
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Analyze(AnalyzeCommands::Churn {
                days,
                project_path,
                format,
                output,
            }) => {
                assert_eq!(days, 7);
                assert_eq!(project_path, PathBuf::from("/tmp"));
                assert_eq!(format, ChurnOutputFormat::Csv);
                assert_eq!(output, Some(PathBuf::from("out.csv")));
            }
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn test_analyze_subcommand_help() {
        // Test that help works
        let args = vec!["paiml-mcp-agent-toolkit", "analyze", "--help"];
        let result = Cli::try_parse_from(args);
        // Help should cause an error (but a specific kind)
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_analyze_churn_help() {
        // Test that help works for churn subcommand
        let args = vec!["paiml-mcp-agent-toolkit", "analyze", "churn", "--help"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }
}
