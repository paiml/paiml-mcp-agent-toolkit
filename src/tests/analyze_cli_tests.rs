#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::cli::{AnalyzeCommands, Cli, Commands};
    use crate::models::churn::ChurnOutputFormat;
    use clap::Parser;
    use std::path::PathBuf;

    /// Run a closure on a thread with 8MB stack to avoid stack overflow
    /// from clap derive's large Cli enum.
    fn with_large_stack<F: FnOnce() + Send + 'static>(f: F) {
        let builder = std::thread::Builder::new().stack_size(8 * 1024 * 1024);
        let handle = builder.spawn(f).expect("Failed to spawn thread");
        handle.join().expect("Thread panicked");
    }

    #[test]
    fn test_analyze_churn_command_parsing() {
        with_large_stack(|| {
            let args = vec!["pmat", "analyze", "churn"];
            let cli = Cli::try_parse_from(args).unwrap();

            match cli.command {
                Commands::Analyze(analyze_cmd) => {
                    match analyze_cmd {
                        AnalyzeCommands::Churn {
                            days,
                            project_path,
                            format,
                            output,
                            top_files,
                            ..
                        } => {
                            assert_eq!(days, 30);
                            assert_eq!(project_path, PathBuf::from("."));
                            assert_eq!(format, ChurnOutputFormat::Summary);
                            assert!(output.is_none());
                            assert_eq!(top_files, 10);
                        }
                        _ => {
                            panic!("Expected Churn command, got a different analyze command");
                        }
                    }
                }
                _ => panic!("Expected Analyze command"),
            }
        });
    }

    #[test]
    fn test_analyze_churn_with_all_options() {
        with_large_stack(|| {
            let args = vec![
                "pmat",
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
                        top_files,
                        ..
                    } => {
                        assert_eq!(days, 90);
                        assert_eq!(project_path, PathBuf::from("/tmp/test"));
                        assert_eq!(format, ChurnOutputFormat::Markdown);
                        assert_eq!(output, Some(PathBuf::from("report.md")));
                        assert_eq!(top_files, 10);
                    }
                    _ => {
                        panic!("Expected Churn command, got a different analyze command");
                    }
                },
                _ => panic!("Expected Analyze command"),
            }
        });
    }

    #[test]
    fn test_analyze_churn_format_options() {
        with_large_stack(|| {
            let formats = vec!["json", "markdown", "csv", "summary"];

            for fmt in formats {
                let args = vec!["pmat", "analyze", "churn", "--format", fmt];
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
        });
    }

    #[test]
    fn test_analyze_churn_invalid_format() {
        with_large_stack(|| {
            let args = vec!["pmat", "analyze", "churn", "--format", "invalid"];
            let result = Cli::try_parse_from(args);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_analyze_churn_short_flags() {
        with_large_stack(|| {
            let args = vec![
                "pmat", "analyze", "churn", "-d", "7", "-p", "/tmp", "--format", "csv", "-o",
                "out.csv",
            ];
            let cli = Cli::try_parse_from(args).unwrap();

            match cli.command {
                Commands::Analyze(AnalyzeCommands::Churn {
                    days,
                    project_path,
                    format,
                    output,
                    top_files,
                    ..
                }) => {
                    assert_eq!(days, 7);
                    assert_eq!(project_path, PathBuf::from("/tmp"));
                    assert_eq!(format, ChurnOutputFormat::Csv);
                    assert_eq!(output, Some(PathBuf::from("out.csv")));
                    assert_eq!(top_files, 10);
                }
                _ => panic!("Expected Analyze command"),
            }
        });
    }

    #[test]
    fn test_analyze_subcommand_help() {
        with_large_stack(|| {
            let args = vec!["pmat", "analyze", "--help"];
            let result = Cli::try_parse_from(args);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        });
    }

    #[test]
    fn test_analyze_churn_help() {
        with_large_stack(|| {
            let args = vec!["pmat", "analyze", "churn", "--help"];
            let result = Cli::try_parse_from(args);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        });
    }
}
