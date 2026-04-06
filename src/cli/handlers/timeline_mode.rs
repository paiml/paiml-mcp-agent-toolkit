#![cfg_attr(coverage_nightly, coverage(off))]
// Sprint 78: TUI-006 GREEN - Timeline CLI Integration
//
// Provides TimelineMode enum and helpers for CLI --interactive flag

use anyhow::Result;

/// Timeline playback mode (interactive TUI vs non-interactive)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelineMode {
    /// Interactive TUI mode (requires TTY)
    Interactive,
    /// Non-interactive batch mode
    #[default]
    NonInteractive,
}

impl TimelineMode {
    /// Parse mode from command-line arguments
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_args(args: &[&str]) -> Self {
        debug_assert!(!args.is_empty(), "args must not be empty");
        if args.contains(&"--interactive") || args.contains(&"-i") {
            TimelineMode::Interactive
        } else {
            TimelineMode::NonInteractive
        }
    }

    /// Check if this is interactive mode
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_interactive(&self) -> bool {
        matches!(self, TimelineMode::Interactive)
    }

    /// Check if this is non-interactive mode
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_non_interactive(&self) -> bool {
        matches!(self, TimelineMode::NonInteractive)
    }

    /// Check if mode requires a terminal (TTY)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn requires_terminal(&self) -> bool {
        self.is_interactive()
    }

    /// Validate terminal availability for this mode
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate_terminal_availability(&self, has_tty: bool) -> Result<()> {
        if self.requires_terminal() && !has_tty {
            anyhow::bail!("Interactive mode requires a TTY (terminal). Run without --interactive flag for batch mode.");
        }
        Ok(())
    }

    /// Get human-readable description
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn description(&self) -> &str {
        match self {
            TimelineMode::Interactive => "Interactive TUI mode",
            TimelineMode::NonInteractive => "Non-interactive batch mode",
        }
    }

    /// Validate arguments for conflicting flags
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate_args(args: &[&str]) -> Result<()> {
        debug_assert!(!args.is_empty(), "args must not be empty");
        let has_interactive = args.contains(&"--interactive") || args.contains(&"-i");
        let has_json = args.contains(&"--json");

        if has_interactive && has_json {
            anyhow::bail!("Conflicting flags: --interactive and --json cannot be used together");
        }

        Ok(())
    }

    /// Check if TUI feature is available
    #[cfg(feature = "tui")]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn check_feature_availability(&self) -> Result<()> {
        Ok(())
    }

    /// Check if TUI feature is available
    #[cfg(not(feature = "tui"))]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn check_feature_availability(&self) -> Result<()> {
        if self.is_interactive() {
            anyhow::bail!(
                "Interactive mode requires the 'tui' feature. Rebuild with --features tui"
            );
        }
        Ok(())
    }
}

/// Placeholder for handle_timeline function
/// This will be properly implemented when integrating with existing timeline command
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn handle_timeline(_args: &[&str]) -> Result<()> {
    debug_assert!(!_args.is_empty(), "_args must not be empty");
    Ok(())
}

/// Get timeline command help text
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn get_timeline_help_text() -> String {
    r#"USAGE:
    pmat timeline <recording.pmat> [OPTIONS]

OPTIONS:
    --interactive, -i    Launch interactive TUI for timeline navigation
    --json              Output timeline data as JSON (conflicts with --interactive)
    --help, -h          Print this help message

EXAMPLES:
    # Interactive mode (TUI)
    pmat timeline recording.pmat --interactive

    # Non-interactive mode (default)
    pmat timeline recording.pmat

    # JSON output
    pmat timeline recording.pmat --json
"#
    .to_string()
}

// --- Include files for module organization ---
include!("timeline_mode_tests.rs");
