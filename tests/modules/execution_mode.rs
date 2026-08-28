// Tests for the main binary functionality
// We can't directly test the main function, but we can test the logic it uses

#[cfg(test)]
mod binary_main_tests {
    use std::env;

    /// The execution-mode decision, with MCP_VERSION passed IN rather than read
    /// from the process.
    ///
    /// It used to call `std::env::var("MCP_VERSION")`, which forced its two
    /// callers to `set_var`/`remove_var` around every assertion. Cargo runs a
    /// binary's tests as parallel THREADS in one process, so those mutations are
    /// global: `test_execution_mode_detection_without_mcp_version` removes the
    /// variable that `test_execution_mode_detection_with_mcp_version` has just
    /// set, and the second assertion sees "Cli" where it demands "Mcp".
    /// Measured on the built test binary before this change: 1 failure in 60
    /// runs at `--test-threads=2`.
    ///
    /// The blast radius was much wider than these two tests. Any test in this
    /// binary that spawns the pmat binary while the window is open gets a child
    /// that inherits MCP_VERSION, and pmat then ignores its subcommand and runs
    /// the stdio MCP server (src/bin/pmat.rs:41). That is how
    /// `pmat_serve_websocket_fails_loudly` came to assert exit 2 against a
    /// process that exited 0 — it failed the 3.32.0 release dogfood.
    ///
    /// Taking the value as an argument removes the shared mutable state
    /// entirely, which is strictly better than serialising access to it.
    ///
    /// ALL THREE inputs are parameters, and the third and fourth are why. The
    /// first version of this fix took only `mcp_version` and still read
    /// `stdin().is_terminal()` and `args().len()` from the process — so
    /// `detect_execution_mode_test(None)` returned "Cli" when a test FILTER was
    /// passed (argv > 1) and "Mcp" when the whole suite ran (argv == 1). It
    /// passed every filtered run and failed the release dogfood, which runs the
    /// suite whole.
    ///
    /// That is worse than the tautology it replaced. `mode == "Cli" || mode ==
    /// "Mcp"` could not fail; an assertion that depends on how the harness was
    /// invoked fails INTERMITTENTLY, which costs more to diagnose than an
    /// assertion that never fires. The fix for a tautology is to make the
    /// function decidable, not to assert one arbitrary branch of an ambient
    /// condition.
    ///
    /// The shape mirrors the real `classify_execution_mode` in src/bin/pmat.rs,
    /// which is already pure for exactly this reason: "so it can be unit-tested
    /// without touching real stdin / env vars (see GH-285 regression test)".
    fn detect_execution_mode_test(
        mcp_version: Option<&str>,
        no_args: bool,
        stdin_is_pipe: bool,
    ) -> String {
        let is_mcp = stdin_is_pipe && no_args || mcp_version.is_some();

        if is_mcp {
            "Mcp".to_string()
        } else {
            "Cli".to_string()
        }
    }

    #[test]
    fn test_execution_mode_detection_with_mcp_version() {
        // The opt-in wins whatever stdin and argv say — all four combinations.
        for (no_args, piped) in [(true, true), (true, false), (false, true), (false, false)] {
            assert_eq!(
                detect_execution_mode_test(Some("1.0.0"), no_args, piped),
                "Mcp",
                "MCP_VERSION must win with no_args={no_args} stdin_is_pipe={piped}"
            );
        }
    }

    /// Without the opt-in, the mode is decided by stdin and argv alone.
    ///
    /// This asserted `mode == "Cli" || mode == "Mcp"` over a function that
    /// returns one of exactly those two strings — a tautology, and it was paying
    /// a process-wide `remove_var` for the privilege. Under a test harness both
    /// conditions are false (stdin is not a terminal, but argv carries the test
    /// filter, so `args().len() == 1` does not hold), which is decidable.
    #[test]
    fn test_execution_mode_detection_without_mcp_version() {
        // Without the opt-in the decision is stdin AND argv, so each case is
        // named rather than one being asserted as if it were the only one.
        assert_eq!(
            detect_execution_mode_test(None, true, true),
            "Mcp",
            "bare `pmat` with piped stdin is the MCP auto-detect"
        );
        assert_eq!(
            detect_execution_mode_test(None, true, false),
            "Cli",
            "bare `pmat` on a terminal is CLI"
        );
        assert_eq!(
            detect_execution_mode_test(None, false, true),
            "Cli",
            "a subcommand with piped stdin is CLI — the subcommand wins"
        );
        assert_eq!(
            detect_execution_mode_test(None, false, false),
            "Cli",
            "a subcommand on a terminal is CLI"
        );
    }

    #[test]
    fn test_env_filter_creation() {
        use tracing_subscriber::EnvFilter;

        // Test the environment filter logic from main
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        // Should not panic and should create a valid filter
        assert!(format!("{filter:?}").contains("info") || !format!("{filter:?}").is_empty());
    }

    #[test]
    fn test_server_creation_logic() {
        use pmat::stateless_server::StatelessTemplateServer;
        use std::sync::Arc;

        // Test the server creation logic from main
        let server_result = StatelessTemplateServer::new();
        assert!(server_result.is_ok());

        let server = Arc::new(server_result.unwrap());
        assert!(Arc::strong_count(&server) > 0);
    }

    // `test_mcp_version_environment_variable` was deleted here, not fixed.
    //
    // Its whole body was `set_var(k, v); assert!(var(k).is_ok()); remove_var(k)`
    // over four values — an assertion about `std::env`, not about pmat, that
    // could not fail. It carried
    // `#[ignore = "Environment variable manipulation unsafe in parallel tests"]`,
    // so the hazard was known and the test hidden rather than the state removed.
    //
    // Hiding it made things worse in the one mode where it matters: `--ignored`
    // is what the release dogfood runs, so this was the only MCP_VERSION setter
    // that fired there — alongside 512 other ignored tests, many of which spawn
    // the pmat binary. Note the fourth value is the EMPTY string, and
    // `env::var(..).is_ok()` is true for "", so `MCP_VERSION=` hijacks a child
    // exactly as `MCP_VERSION=1.0.0` does.

    #[test]
    fn test_argument_count_behavior() {
        let args: Vec<String> = env::args().collect();

        // Test that we can detect argument count (used in main's mode detection)
        assert!(!args.is_empty()); // At least the program name

        // Simulate the condition from main
        let single_arg = args.len() == 1;
        // Just verify we can check the condition without panic
        let _ = single_arg;
    }

    #[tokio::test]
    async fn test_async_runtime_setup() {
        // Test that we can set up the tokio runtime (main is async)
        let result = tokio::spawn(async {
            // Simulate some async work like in main
            let server_result = pmat::stateless_server::StatelessTemplateServer::new();
            server_result.is_ok()
        })
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_tracing_initialization() {
        use tracing_subscriber::{fmt, EnvFilter};

        // Test that tracing can be initialized (similar to main)
        let result = std::panic::catch_unwind(|| {
            // Don't actually initialize to avoid conflicts, just test the builder
            let _subscriber = fmt().with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            );
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_terminal_detection() {
        // Test terminal detection logic used in main
        let is_terminal = std::io::IsTerminal::is_terminal(&std::io::stdin());

        // Should not panic and return a boolean
        // Just verify we can check terminal state without panic
        let _ = is_terminal;
    }

    #[test]
    fn test_error_handling_setup() {
        use anyhow::Result;

        // Test that Result<()> type works (main returns Result<()>)
        let test_result: Result<()> = Ok(());
        assert!(test_result.is_ok());

        let error_result: Result<()> = Err(anyhow::anyhow!("Test error"));
        assert!(error_result.is_err());
    }
}
