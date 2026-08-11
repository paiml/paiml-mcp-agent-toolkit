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

    // Handle forced mode.
    //
    // The binary decides this before clap runs (`cli::forced_mode_from_args`),
    // so this branch is only reached by other embedders of `cli::run`. It used
    // to call `crate::run_mcp_server`, a *second* MCP server whose 21-tool
    // inventory shares only 7 names with the unified server's 20 — and those 7
    // take different arguments (`project_path` string vs `paths` array), with 7
    // of the 21 describing themselves as "(unimplemented stub — KAIZEN-0200)"
    // (that server has since been deleted, #696). `--mode mcp` must not reach a
    // different server than
    // `MCP_VERSION=1 pmat` does — #697: this branch built its own
    // `UnifiedServer` while the binary built another, so "the same server" was
    // an unenforced coincidence. Both now call the one entry point.
    if let Some(commands::Mode::Mcp) = cli.mode {
        info!("Forced MCP mode detected");
        return crate::mcp_pmcp::run_stdio_server().await;
    }

    // Use command dispatcher for improved modularity
    let result = CommandDispatcher::execute_command(cli.command, server).await;

    // `maintain bug-report` reads a captured-error store that nothing in the
    // product ever wrote — `capture_command_error*` had no caller outside its
    // own unit tests — so it always answered "No captured error found. Run a
    // pmat command that fails first" even immediately after one did. The
    // failing command is what has to write it.
    if let Err(e) = &result {
        crate::cli::handlers::bug_report_handler::capture_cli_failure(
            &std::env::args().collect::<Vec<_>>(),
            e,
        );
    }

    result
}

/// Apply UX settings from CLI flags (TICKET-PMAT-6006)
///
/// CC=3: Quiet mode + color mode checks
fn apply_ux_settings(cli: &commands::Cli) {
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
