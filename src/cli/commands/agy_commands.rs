use clap::Subcommand;
use std::path::PathBuf;

/// Commands for Google Anti-Gravity customizations translator
#[derive(Debug, Clone, Subcommand, PartialEq)]
pub enum AgyCommands {
    /// Transpile PMAT skill requirements and contracts into AGY formats (MACS-017)
    Sync {
        /// Optional path to .pmat-work contracts
        #[arg(long, default_value = ".pmat-work")]
        work_dir: PathBuf,
        /// Target AGY config dir (e.g. .gemini/config/skills)
        #[arg(long, default_value = ".agents/skills")]
        out_dir: PathBuf,
    },
}
