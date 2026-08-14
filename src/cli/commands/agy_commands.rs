use clap::Subcommand;
use std::path::PathBuf;

/// Commands for Google Anti-Gravity customizations translator
#[derive(Debug, Clone, Subcommand, PartialEq)]
pub enum AgyCommands {
    /// Report the PMAT work contracts an AGY transpiler would consume, then refuse
    /// the transpile: no Anti-Gravity target schema is defined (MACS-017, #984)
    Sync {
        /// Directory of PMAT work contracts (`<dir>/<id>/contract.json`) to inventory
        #[arg(long, default_value = ".pmat-work")]
        work_dir: PathBuf,
        /// Where transpiled AGY customizations would be written. Nothing is written
        /// yet: the target format is undefined, and the three candidate conventions
        /// (root AGENTS.md + skills.json, .agents/skills, .gemini/config/skills)
        /// disagree — see #984
        #[arg(long, default_value = ".agents/skills")]
        out_dir: PathBuf,
    },
}
