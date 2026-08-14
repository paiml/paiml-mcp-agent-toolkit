#![cfg_attr(
    all(coverage_nightly, not(coverage_attr_stable)),
    feature(coverage_attribute)
)]
#![cfg_attr(coverage_nightly, coverage(off))]

// Minimal stub when built without standard-deps — pmat's CLI requires the
// full analysis pipeline which lives behind the `standard-deps` feature.
// `cargo check --no-default-features` exists for packaging/Cargo.toml validity
// checks only; it is not a supported runtime build.
#[cfg(not(feature = "standard-deps"))]
fn main() {
    eprintln!("pmat requires the `standard-deps` feature. Build with `--features standard-deps` or use the default feature set.");
    std::process::exit(2);
}

#[cfg(feature = "standard-deps")]
mod full {
    use anyhow::Result;
    use pmat::{cli, stateless_server::StatelessTemplateServer};
    use std::io::IsTerminal;
    use std::process;
    use std::sync::Arc;
    use tracing::{debug, info, trace};
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    enum ExecutionMode {
        Mcp,
        Cli,
        /// No subcommand given and stdin is piped (not a TTY) without an explicit
        /// MCP opt-in. Print help and exit non-zero instead of blocking on stdin
        /// (see GH-285).
        HelpAndExit,
    }

    /// POSIX-compliant exit codes for CLI interface
    /// Per SPECIFICATION.md Section 23: CLI Interface
    #[derive(Debug, Clone, Copy)]
    pub enum ExitCode {
        /// Success
        Success = 0,
        /// General error
        GeneralError = 1,
        /// Misuse of shell command
        MisuseError = 2,
        /// Permission denied
        PermissionDenied = 126,
        /// Command not found
        CommandNotFound = 127,
        /// Invalid argument to exit
        InvalidExitArg = 128,
        /// Quality gate failure (custom)
        QualityGateFailure = 3,
        /// Configuration error (custom)
        ConfigurationError = 4,
        /// Analysis error (custom)
        AnalysisError = 5,
    }

    impl From<ExitCode> for i32 {
        fn from(code: ExitCode) -> Self {
            code as i32
        }
    }

    fn detect_execution_mode() -> ExecutionMode {
        let args: Vec<String> = std::env::args().collect();
        classify_execution_mode(
            cli::forced_mode_from_args(&args),
            std::env::var("MCP_VERSION").is_ok(),
            args.len() == 1,
            !std::io::stdin().is_terminal(),
        )
    }

    /// Pure decision function for execution mode so it can be unit-tested
    /// without touching real stdin / env vars (see GH-285 regression test).
    fn classify_execution_mode(
        forced: Option<cli::commands::Mode>,
        mcp_version_env: bool,
        no_args: bool,
        stdin_is_pipe: bool,
    ) -> ExecutionMode {
        // `--help` calls `--mode` "Force specific mode (auto-detected by
        // default)", so it outranks every auto-detection signal, MCP_VERSION
        // included — otherwise `--mode cli` could not override a host's env.
        // It used to be read nowhere at all; `--mode mcp <subcommand>` swallowed
        // the subcommand and started a legacy MCP server with a *different*
        // tool inventory. See `cli::forced_mode_from_args`.
        match forced {
            Some(cli::commands::Mode::Mcp) => return ExecutionMode::Mcp,
            Some(cli::commands::Mode::Cli) => return ExecutionMode::Cli,
            None => {}
        }

        // Explicit MCP opt-in via env var always wins (e.g. Claude Desktop sets this).
        if mcp_version_env {
            return ExecutionMode::Mcp;
        }

        // GH-285: If the user ran bare `pmat` with stdin piped (e.g. `echo "" | pmat`)
        // without opting into MCP via `MCP_VERSION`, previous behavior entered the MCP
        // server and blocked forever on stdin. Treat that case the same as bare `pmat`
        // on a terminal: print help and exit non-zero.
        if no_args && stdin_is_pipe {
            return ExecutionMode::HelpAndExit;
        }

        ExecutionMode::Cli
    }

    /// Initialize the enhanced tracing system based on CLI flags
    fn init_tracing(cli: &cli::EarlyCliArgs) -> Result<()> {
        let filter = create_env_filter(cli)?;
        // `fmt::layer()` defaults to `ansi = true` unconditionally, so
        // `--color never` on a pipe still produced
        // `\x1b[2m…\x1b[0m WARN churn not measured …` on stderr — the flag was
        // obeyed by the command's output and ignored by the log beside it. This
        // is decided from raw argv because clap has not parsed yet; see
        // `cli::tracing_ansi_enabled`.
        let ansi = cli::tracing_ansi_enabled(
            &std::env::args().collect::<Vec<_>>(),
            std::io::stderr().is_terminal(),
        );

        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(cli.debug || cli.trace)
                    .with_thread_ids(cli.trace)
                    .with_file(cli.trace)
                    .with_line_number(cli.trace)
                    .with_ansi(ansi)
                    .compact()
                    .with_writer(std::io::stderr),
            )
            .init();

        Ok(())
    }

    /// Create environment filter based on CLI flags
    fn create_env_filter(cli: &cli::EarlyCliArgs) -> Result<EnvFilter> {
        if cli.is_mcp_server {
            Ok(create_mcp_filter(cli.debug))
        } else {
            create_cli_filter(cli)
        }
    }

    /// Create filter for MCP server mode
    fn create_mcp_filter(debug: bool) -> EnvFilter {
        if debug {
            EnvFilter::new("warn,pmat=debug")
        } else {
            EnvFilter::new("off")
        }
    }

    /// Create filter for CLI mode
    fn create_cli_filter(cli: &cli::EarlyCliArgs) -> Result<EnvFilter> {
        if let Some(ref custom) = cli.trace_filter {
            return Ok(EnvFilter::try_new(custom)?);
        }

        // The decision itself lives in the library (`cli::log_level_directive`)
        // so it is covered by the `--lib` suite CI runs. This match used to be
        // spelled out here over `(trace, debug, verbose)` only — with no arm for
        // `--quiet`, which is why "Enable quiet mode (errors only)" named a log
        // level no argv could select.
        let Some(filter_str) =
            cli::log_level_directive(cli.trace, cli.debug, cli.verbose, cli.quiet)
        else {
            return Ok(get_default_filter());
        };

        Ok(EnvFilter::new(filter_str))
    }

    /// Get default production filter
    fn get_default_filter() -> EnvFilter {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    }

    #[tokio::main]
    #[allow(unreachable_pub)]
    pub async fn real_main() {
        let exit_code = match run_main().await {
            Ok(()) => ExitCode::Success,
            Err(e) => {
                // NOT `tracing::error!` — MCP-server mode installs
                // `EnvFilter::new("off")`, which silently discarded this, so
                // `pmat agent mcp-server` exited 1 printing nothing at all.
                // See `cli::write_fatal_error` for the full history.
                cli::write_fatal_error(std::io::stderr(), &e);
                categorize_error(&e)
            }
        };

        if exit_code as i32 != 0 {
            process::exit(exit_code.into());
        }
    }

    fn categorize_error(error: &anyhow::Error) -> ExitCode {
        let error_str = error.to_string().to_lowercase();

        match () {
            _ if is_quality_gate_error(&error_str) => ExitCode::QualityGateFailure,
            _ if is_configuration_error(&error_str) => ExitCode::ConfigurationError,
            _ if is_analysis_error(&error_str) => ExitCode::AnalysisError,
            _ if is_permission_error(&error_str) => ExitCode::PermissionDenied,
            _ => ExitCode::GeneralError,
        }
    }

    fn is_quality_gate_error(error_str: &str) -> bool {
        error_str.contains("quality gate") || error_str.contains("violation")
    }

    fn is_configuration_error(error_str: &str) -> bool {
        error_str.contains("config") || error_str.contains("parse")
    }

    fn is_analysis_error(error_str: &str) -> bool {
        error_str.contains("analysis") || error_str.contains("complexity")
    }

    fn is_permission_error(error_str: &str) -> bool {
        error_str.contains("permission") || error_str.contains("access")
    }

    async fn run_main() -> Result<()> {
        // Parse CLI to get tracing configuration early
        let cli = cli::parse_early_for_tracing();

        // Initialize enhanced tracing system
        init_tracing(&cli)?;

        // Only log to stdout if not running MCP server mode
        if !cli.is_mcp_server {
            info!(
                "Starting PAIML MCP Agent Toolkit v{}",
                env!("CARGO_PKG_VERSION")
            );
            debug!("Debug logging enabled");
            trace!("Trace logging enabled");
        }

        // Create shared template server
        let server = Arc::new(StatelessTemplateServer::new()?);

        if !cli.is_mcp_server {
            debug!("Template server initialized");
        }

        match detect_execution_mode() {
            ExecutionMode::Mcp => {
                if !cli.is_mcp_server {
                    info!("Running unified MCP server (pmcp SDK)");
                }
                // #697: this used to construct the server here, and `cli::run`
                // constructed a second one for its `--mode mcp` branch. Two
                // call sites, free to drift into two inventories. There is now
                // exactly one, and `exactly_one_mcp_stdio_entry_point` fails if
                // either route grows its own again.
                pmat::mcp_pmcp::run_stdio_server().await
            }
            ExecutionMode::Cli => {
                if !cli.is_mcp_server {
                    info!("Running in CLI mode");
                }
                cli::run(server).await
            }
            ExecutionMode::HelpAndExit => {
                // GH-285: No subcommand + piped stdin + no MCP_VERSION. Print help on
                // stderr and exit with code 2 (misuse), matching the convention for
                // "no command supplied" on a terminal.
                let mut cmd = <pmat::cli::Cli as clap::CommandFactory>::command();
                let _ = cmd.print_help();
                // Names only the route that has been run end-to-end. This used
                // to also offer `pmat agent mcp-server`, which is compiled out
                // unless `--features agent-daemon` is set (it is not in
                // `default`), so on every published binary it exits 1 — and,
                // until `write_fatal_error` existed, did so in total silence.
                eprintln!(
                    "\nerror: no subcommand given and stdin is not a terminal. \
                 Set MCP_VERSION=1 to start the MCP server (`MCP_VERSION=1 pmat`)."
                );
                process::exit(ExitCode::MisuseError as i32);
            }
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[cfg(test)]
    mod gh285_tests {
        //! Regression tests for GH-285: `pmat` hangs when invoked with no args
        //! and stdin is a pipe. See `classify_execution_mode`.
        //!
        //! Manual end-to-end repro (should NOT hang or exit 124):
        //! ```text
        //! echo "" | timeout 3 target/debug/pmat 2>&1; echo exit=$?
        //! timeout 3 target/debug/pmat < /dev/null; echo exit=$?
        //! ```
        //! Both should print help and exit with code 2.
        use super::{classify_execution_mode, cli, ExecutionMode};

        #[test]
        fn bare_invocation_on_tty_is_cli() {
            // `pmat` typed into a terminal with no pipe -> normal CLI (clap
            // will then print help on missing subcommand).
            assert!(matches!(
                classify_execution_mode(None, false, true, false),
                ExecutionMode::Cli
            ));
        }

        #[test]
        fn piped_stdin_with_no_args_prints_help_instead_of_hanging() {
            // GH-285: This used to enter MCP mode and block on stdin forever.
            assert!(matches!(
                classify_execution_mode(None, false, true, true),
                ExecutionMode::HelpAndExit
            ));
        }

        #[test]
        fn explicit_mcp_version_env_still_enters_mcp_mode() {
            // Claude Desktop and similar hosts set MCP_VERSION; they must still
            // get the MCP server regardless of TTY state.
            assert!(matches!(
                classify_execution_mode(None, true, true, true),
                ExecutionMode::Mcp
            ));
            assert!(matches!(
                classify_execution_mode(None, true, false, false),
                ExecutionMode::Mcp
            ));
        }

        #[test]
        fn subcommand_with_piped_stdin_is_still_cli() {
            // `echo foo | pmat analyze ...` must dispatch to the CLI subcommand,
            // not to help-and-exit.
            assert!(matches!(
                classify_execution_mode(None, false, false, true),
                ExecutionMode::Cli
            ));
        }

        /// `--mode mcp` is advertised in `--help` but was read nowhere: with no
        /// subcommand it failed "requires a subcommand", and with one it
        /// swallowed the subcommand and started a legacy MCP server whose
        /// 21-tool inventory shares only 7 names with the unified server's 20.
        /// It must now reach the same server `MCP_VERSION=1 pmat` does.
        #[test]
        fn mode_mcp_selects_the_mcp_server_with_or_without_a_subcommand() {
            let mcp = Some(cli::commands::Mode::Mcp);
            assert!(matches!(
                classify_execution_mode(mcp.clone(), false, true, false),
                ExecutionMode::Mcp
            ));
            assert!(matches!(
                classify_execution_mode(mcp.clone(), false, false, false),
                ExecutionMode::Mcp
            ));
            // …and it must not be defeated by the GH-285 help-and-exit rule.
            assert!(matches!(
                classify_execution_mode(mcp, false, true, true),
                ExecutionMode::Mcp
            ));
        }

        /// "Force specific mode" has to be able to force the *other* way too,
        /// or a host that exports MCP_VERSION could never run a subcommand.
        #[test]
        fn mode_cli_overrides_the_mcp_version_env() {
            assert!(matches!(
                classify_execution_mode(Some(cli::commands::Mode::Cli), true, false, true),
                ExecutionMode::Cli
            ));
        }
    }
} // end of mod full

#[cfg(feature = "standard-deps")]
fn main() {
    full::real_main();
}
