#![cfg_attr(coverage_nightly, coverage(off))]
//! Clap surface for `pmat init` (#1030, #1031, #1032).
//!
//! The value enums live here rather than in `crate::services::workspace_init`
//! so that nothing under `src/services` has to depend on clap; the service
//! carries its own plain [`Target`](crate::services::workspace_init::Target)
//! and this converts into it.

use crate::services::workspace_init::Target;

/// Which agent client's layout `pmat init` should write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum InitTarget {
    /// Antigravity and the cross-client `.agents/` layout: `hooks.json`
    /// (PreToolUse), `mcp_config.json`, a pinned-effort skill, `AGENTS.md`.
    Agy,
    /// Claude Code: `.claude/settings.json`, `.claude/skills/`, the
    /// project-scoped `.mcp.json`, `AGENTS.md`.
    Claude,
    /// Claude Code plus a committed ultracode judgment workflow under
    /// `contracts/workflows/`. Ultracode is a session-only harness effort
    /// setting, not a separate client, so this is `claude` plus the one
    /// artifact the convention actually defines.
    Ultracode,
}

impl InitTarget {
    /// Convert to the clap-free target the generator works in.
    pub fn to_service(self) -> Target {
        match self {
            InitTarget::Agy => Target::Agy,
            InitTarget::Claude => Target::Claude,
            InitTarget::Ultracode => Target::Ultracode,
        }
    }
}

/// Output format for the `pmat init` report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum InitFormat {
    /// Aligned, human-readable summary.
    #[default]
    Human,
    /// Machine-readable: per-artifact outcomes plus the refusal list.
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cli_target_maps_to_a_service_target() {
        assert_eq!(InitTarget::Agy.to_service(), Target::Agy);
        assert_eq!(InitTarget::Claude.to_service(), Target::Claude);
        assert_eq!(InitTarget::Ultracode.to_service(), Target::Ultracode);
    }

    #[test]
    fn default_format_is_human() {
        assert_eq!(InitFormat::default(), InitFormat::Human);
    }
}
