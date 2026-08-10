// Debug Adapter Protocol (DAP) handlers - Sprint 74
//
// Stub: Not yet implemented
//
// Honest-failure policy: these exit with
// [`DEBUG_UNIMPLEMENTED_EXIT_CODE`] (2, "misuse"), the same code `pmat serve`
// uses for the same situation (utility_serve_handlers). They used to
// `anyhow::bail!`, which surfaced as exit 1 — the code reserved for "the
// command ran and found a problem" — so a CI script could not tell "this
// subcommand does not exist yet" from "the DAP server failed to start".

/// Exit code for a `pmat debug` subcommand that is not implemented.
///
/// Kept numerically in step with
/// [`crate::cli::handlers::utility_serve_handlers::SERVE_UNIMPLEMENTED_EXIT_CODE`].
pub const DEBUG_UNIMPLEMENTED_EXIT_CODE: i32 = 2;

/// Emit the honest-failure diagnostic for an unimplemented `debug` subcommand.
///
/// Extracted so tests can assert on the exact bytes without the process exit.
pub fn write_debug_unimplemented_message<W: std::io::Write>(
    mut out: W,
    subcommand: &str,
    ticket: &str,
    requested: &str,
) -> std::io::Result<()> {
    writeln!(
        out,
        "error: pmat debug {subcommand} is not implemented ({ticket})"
    )?;
    writeln!(out, "  requested: {requested}")?;
    writeln!(
        out,
        "hint: no DAP socket is bound and no recording is read — nothing is running"
    )?;
    Ok(())
}

// Placeholder for DAP server handler
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_debug_serve(
    port: u16,
    host: String,
    record_dir: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    let _ = write_debug_unimplemented_message(
        std::io::stderr(),
        "serve",
        "DEBUG-002",
        &format!("host={host} port={port} record_dir={record_dir:?}"),
    );
    std::process::exit(DEBUG_UNIMPLEMENTED_EXIT_CODE);
}

// Placeholder for DAP replay handler
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_debug_replay(
    recording: std::path::PathBuf,
    position: Option<usize>,
    interactive: bool,
) -> anyhow::Result<()> {
    let _ = write_debug_unimplemented_message(
        std::io::stderr(),
        "replay",
        "DEBUG-003",
        &format!("recording={recording:?} position={position:?} interactive={interactive}"),
    );
    std::process::exit(DEBUG_UNIMPLEMENTED_EXIT_CODE);
}

// Placeholder for DAP compare handler
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_debug_compare() -> anyhow::Result<()> {
    anyhow::bail!("Debug compare command not yet implemented")
}

// Placeholder for DAP timeline handler
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_debug_timeline() -> anyhow::Result<()> {
    anyhow::bail!("Debug timeline command not yet implemented")
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // The serve/replay tests below used to call the handlers and assert
    // `result.is_err()`. Those handlers now exit(2) like `pmat serve` does, so
    // they are exercised through the message writer instead — same coverage
    // intent (the diagnostic names the ticket and says it is not implemented),
    // without killing the test process.

    /// The `debug serve` diagnostic still names DEBUG-002 and says so plainly.
    #[test]
    fn serve_message_states_not_implemented_and_names_the_ticket() {
        let mut buf = Vec::new();
        write_debug_unimplemented_message(
            &mut buf,
            "serve",
            "DEBUG-002",
            "host=localhost port=8080 record_dir=None",
        )
        .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("DEBUG-002"), "got: {s}");
        assert!(s.contains("not implemented"), "got: {s}");
    }

    /// The diagnostic echoes what was requested, so logs show the port/host.
    #[test]
    fn serve_message_echoes_the_request() {
        let mut buf = Vec::new();
        write_debug_unimplemented_message(
            &mut buf,
            "serve",
            "DEBUG-002",
            "host=0.0.0.0 port=65535 record_dir=Some(\"/tmp/debug-recordings\")",
        )
        .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("0.0.0.0"), "got: {s}");
        assert!(s.contains("65535"), "got: {s}");
        assert!(s.contains("/tmp/debug-recordings"), "got: {s}");
    }

    /// The `debug replay` diagnostic still names DEBUG-003.
    #[test]
    fn replay_message_states_not_implemented_and_names_the_ticket() {
        let mut buf = Vec::new();
        write_debug_unimplemented_message(
            &mut buf,
            "replay",
            "DEBUG-003",
            "recording=\"/tmp/recording.dap\" position=Some(42) interactive=true",
        )
        .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("DEBUG-003"), "got: {s}");
        assert!(s.contains("not implemented"), "got: {s}");
        assert!(s.contains("/tmp/recording.dap"), "got: {s}");
    }

    /// `debug serve` must exit 2 ("misuse"), not 1, and must match `pmat serve`.
    ///
    /// Exit 1 is the code a command uses when it ran and found a problem; using
    /// it for "this subcommand does not exist yet" made the two indistinguishable.
    #[test]
    fn unimplemented_exit_code_matches_serve() {
        assert_eq!(DEBUG_UNIMPLEMENTED_EXIT_CODE, 2);
        assert_eq!(
            DEBUG_UNIMPLEMENTED_EXIT_CODE,
            crate::cli::handlers::utility_serve_handlers::SERVE_UNIMPLEMENTED_EXIT_CODE
        );
    }

    /// The `debug` subcommand help must carry the `[NOT IMPLEMENTED]` marker
    /// that `pmat serve` carries. Without it the only way to learn that
    /// `debug serve` binds no socket was to run it.
    #[test]
    fn serve_and_replay_are_labelled_not_implemented() {
        use clap::Subcommand;
        let cmd =
            crate::cli::commands::DebugCommands::augment_subcommands(clap::Command::new("debug"));
        for name in ["serve", "replay"] {
            let about = cmd
                .get_subcommands()
                .find(|s| s.get_name() == name)
                .unwrap_or_else(|| panic!("{name} subcommand must exist"))
                .get_about()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            assert!(
                about.contains("NOT IMPLEMENTED"),
                "`debug {name}` must be labelled unimplemented in the command list, got: {about}"
            );
        }
    }

    /// Test that handle_debug_compare returns the expected error
    #[tokio::test]
    async fn test_handle_debug_compare_returns_not_implemented() {
        let result = handle_debug_compare().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not yet implemented"));
    }

    /// Test that handle_debug_timeline returns the expected error
    #[tokio::test]
    async fn test_handle_debug_timeline_returns_not_implemented() {
        let result = handle_debug_timeline().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not yet implemented"));
    }
}
