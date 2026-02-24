#![cfg_attr(coverage_nightly, coverage(off))]
// Refactor and Scaffold commands - extracted for file health (CB-040)

use crate::cli::{
    ExplainLevel, QualityProfile, RefactorAutoOutputFormat, RefactorDocsOutputFormat,
    RefactorMode, RefactorOutputFormat,
};
use clap::Subcommand;
use std::path::PathBuf;
use serde_json::Value;

include!("refactor_commands_def.rs");

include!("scaffold_commands_def.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    // Types from sibling command modules and CLI enums
    use crate::cli::commands::{
        AgentCommands, AnalyzeCommands, Cli, Commands, DiagnosticOutputFormat, Mode,
        RoadmapCommands, ServeTransport, StorageCommand, TdgCommand, TestSuite,
    };
    use crate::cli::{ComplexityOutputFormat, ContextFormat, OutputFormat};
    use crate::models::churn::ChurnOutputFormat;
    use clap::Parser;

    include!("tests_cli_parsing.rs");

    include!("tests_command_variants.rs");
}
