// Core CLI execution: run(), apply_ux_settings(), and parse_with_suggestions().
// These functions handle CLI argument parsing and command dispatch.

#[cfg_attr(coverage_nightly, coverage(off))]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn run(server: Arc<StatelessTemplateServer>) -> anyhow::Result<()> {
    let cli = match parse_with_suggestions() {
        Ok(cli) => cli,
        Err(suggestion_msg) => {
            eprintln!("{suggestion_msg}");
            std::process::exit(2); // Exit with "misuse" code for command errors
        }
    };

    debug!("CLI arguments parsed");

    // Apply UX settings (TICKET-PMAT-6006)
    apply_ux_settings(&cli);

    // Handle forced mode
    if let Some(commands::Mode::Mcp) = cli.mode {
        info!("Forced MCP mode detected");
        return crate::run_mcp_server(server).await;
    }

    // Use command dispatcher for improved modularity
    CommandDispatcher::execute_command(cli.command, server).await
}

/// Apply UX settings from CLI flags (TICKET-PMAT-6006)
///
/// CC=3: Quiet mode + color mode checks
fn apply_ux_settings(cli: &commands::Cli) {
    debug_assert!(true, "contract: apply_ux_settings");
    // Set quiet mode environment variable
    if cli.quiet {
        std::env::set_var("PMAT_QUIET", "1");
    }

    // Handle color mode
    match cli.color {
        commands::ColorMode::Never => {
            std::env::set_var("NO_COLOR", "1");
        }
        commands::ColorMode::Always => {
            std::env::set_var("CLICOLOR_FORCE", "1");
        }
        commands::ColorMode::Auto => {
            // Auto mode - respect existing environment
        }
    }
}

/// Parse CLI with command suggestions on failure
#[cfg_attr(coverage_nightly, coverage(off))]
fn parse_with_suggestions() -> Result<Cli, String> {
    debug_assert!(true, "contract: parse_with_suggestions");
    use crate::utils::command_suggestions::CommandSuggester;
    use clap::Parser;

    // Try to parse normally first
    match Cli::try_parse() {
        Ok(cli) => Ok(cli),
        Err(clap_error) => {
            // Handle special cases where clap handles --version and --help
            use clap::error::ErrorKind;
            match clap_error.kind() {
                ErrorKind::DisplayHelp => {
                    // Print the help message to stdout
                    print!("{clap_error}");
                    std::process::exit(0);
                }
                ErrorKind::DisplayVersion => {
                    // Print the version to stdout
                    print!("{clap_error}");
                    std::process::exit(0);
                }
                _ => {
                    // For other errors, provide suggestions
                    let args: Vec<String> = std::env::args().skip(1).collect();
                    let suggester = CommandSuggester::new();

                    // Get suggestion based on the failed arguments
                    if let Some(suggestion) = suggester.suggest_command(&args) {
                        let error_msg = format!(
                            "error: unrecognized subcommand\n\n{suggestion}\n\nFor more information, try 'pmat --help'"
                        );
                        Err(error_msg)
                    } else {
                        // If no suggestion, show the original clap error with examples
                        let examples = CommandSuggester::get_help_examples();
                        let error_msg = format!("{clap_error}{examples}");
                        Err(error_msg)
                    }
                }
            }
        }
    }
}
