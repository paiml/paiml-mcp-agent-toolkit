/// Test discovery subcommands for systematic test fixing
#[derive(Debug, Clone, Subcommand)]
pub enum TestDiscoveryCommands {
    /// Discover all test failures in workspace
    #[command(visible_aliases = &["d"])]
    Run {
        /// Project path (defaults to current directory)
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,

        /// Output file for failures JSON
        #[arg(short = 'o', long = "output", default_value = "test-failures.json")]
        output: PathBuf,

        /// Use cargo nextest (faster, parallel); `--use-nextest false` runs `cargo test`
        ///
        /// `#[arg(long, default_value = "true")]` derived `ArgAction::SetTrue`,
        /// which ignores the default and can only ever set the flag to true:
        /// the value was true whether or not the flag was passed, `--use-nextest
        /// false` was rejected as an unexpected argument, and the `cargo test`
        /// branch in `handle_discovery_run` was unreachable. `ArgAction::Set`
        /// with an optional value keeps nextest the default while making the
        /// flag actually settable.
        #[arg(
            long,
            action = clap::ArgAction::Set,
            num_args = 0..=1,
            default_value_t = true,
            default_missing_value = "true"
        )]
        use_nextest: bool,

        /// Maximum test timeout in seconds
        #[arg(long, default_value = "600")]
        timeout: u64,
    },

    /// Categorize test failures by root cause
    #[command(visible_aliases = &["cat"])]
    Categorize {
        /// Input failures JSON from discovery
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Output categories JSON
        #[arg(short = 'o', long = "output", default_value = "test-categories.json")]
        output: PathBuf,
    },

    /// Mark tests as #[ignore] with reasons
    #[command(visible_aliases = &["m"])]
    Mark {
        /// Input categories JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Actually apply changes (default: dry-run)
        #[arg(long)]
        apply: bool,
    },

    /// Verify all tests pass after marking
    #[command(visible_aliases = &["v"])]
    Verify {
        /// Project path
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,
    },

    /// Create GitHub issues from categorized test failures (Phase 5)
    #[command(visible_aliases = &["tickets", "t"])]
    CreateTickets {
        /// Input categories JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Actually create GitHub issues (default: dry-run)
        #[arg(long)]
        create: bool,

        /// Output tickets summary
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// GitHub repository (owner/repo format)
        #[arg(long)]
        repo: Option<String>,

        /// Labels to add to created issues
        #[arg(long, value_delimiter = ',')]
        labels: Option<Vec<String>>,
    },

    /// Resolve test file paths from test names
    #[command(visible_aliases = &["resolve", "r"])]
    ResolvePaths {
        /// Input failures JSON
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Output with resolved paths
        #[arg(short = 'o', long = "output")]
        output: PathBuf,

        /// Project path to search for test files
        #[arg(short = 'p', long = "path", default_value = ".")]
        path: PathBuf,
    },
}

/// Test discovery output formats
#[derive(Debug, Clone, clap::ValueEnum, PartialEq)]
pub enum TestDiscoveryFormat {
    Json,
    Markdown,
    Text,
}

#[cfg(test)]
mod use_nextest_flag_tests {
    //! `--use-nextest` must be a real switch: it used to derive
    //! `ArgAction::SetTrue`, so the value was true no matter what the user
    //! typed and `cargo test` could never run.
    use super::TestDiscoveryCommands;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct Harness {
        #[command(subcommand)]
        cmd: TestDiscoveryCommands,
    }

    fn use_nextest_of(args: &[&str]) -> bool {
        match Harness::try_parse_from(args).expect("parse test-discovery run").cmd {
            TestDiscoveryCommands::Run { use_nextest, .. } => use_nextest,
            other => panic!("expected `run`, got {other:?}"),
        }
    }

    #[test]
    fn use_nextest_defaults_on_but_can_be_switched_off() {
        assert!(use_nextest_of(&["td", "run"]), "nextest stays the default");
        assert!(use_nextest_of(&["td", "run", "--use-nextest"]));
        assert!(
            !use_nextest_of(&["td", "run", "--use-nextest", "false"]),
            "--use-nextest false must select `cargo test`"
        );
    }
}
