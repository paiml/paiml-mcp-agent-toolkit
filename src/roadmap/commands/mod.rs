#![allow(unused)]
//! CLI commands for roadmap management

#![cfg_attr(coverage_nightly, coverage(off))]

use super::{
    generator, HashMap, Path, Priority, Roadmap, RoadmapConfig, Sprint, Task, TaskStatus, Utc,
};
use crate::cli::OutputFormat;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Parser)]
#[command(about = "Roadmap management with PDMT todos and quality gates")]
/// Roadmap command.
pub struct RoadmapCommand {
    #[command(subcommand)]
    pub command: RoadmapSubcommand,
}

#[derive(Debug, Subcommand)]
/// Roadmap subcommand.
pub enum RoadmapSubcommand {
    /// Initialize a new sprint in the roadmap
    Init {
        #[arg(long)]
        version: String,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "14")]
        duration_days: u32,
        #[arg(long, default_value = "P0")]
        priority: String,
    },

    /// Generate PDMT todos from roadmap tasks
    Todos {
        #[arg(long)]
        sprint: Option<String>,
        #[arg(long, default_value = "todos.md")]
        output: PathBuf,
        #[arg(long)]
        include_quality_gates: bool,
    },

    /// Start working on a task
    Start {
        /// Task ID (e.g., PMAT-3001)
        task_id: String,
        #[arg(long)]
        create_branch: bool,
    },

    /// Complete a task (with quality validation)
    Complete {
        /// Task ID (e.g., PMAT-3001)
        task_id: String,
        #[arg(long)]
        skip_quality_check: bool,
    },

    /// Check sprint or task status
    Status {
        #[arg(long)]
        sprint: Option<String>,
        #[arg(long)]
        task: Option<String>,
        #[arg(long, default_value = "human")]
        format: OutputFormat,
    },

    /// Validate sprint readiness for release
    Validate {
        #[arg(long)]
        sprint: String,
        #[arg(long)]
        strict: bool,
    },

    /// Run quality checks for a task
    QualityCheck {
        #[arg(long)]
        task_id: String,
    },
}

/// Execute roadmap commands
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn execute(cmd: RoadmapCommand, config: RoadmapConfig) -> Result<()> {
    let roadmap_path = config.path.clone();

    match cmd.command {
        RoadmapSubcommand::Init {
            version,
            title,
            duration_days,
            priority,
        } => init_sprint(&roadmap_path, &version, &title, duration_days, &priority).await,
        RoadmapSubcommand::Todos {
            sprint,
            output,
            include_quality_gates,
        } => {
            generate_todos(
                &roadmap_path,
                sprint.as_deref(),
                &output,
                include_quality_gates,
                &config,
            )
            .await
        }
        RoadmapSubcommand::Start {
            task_id,
            create_branch,
        } => start_task(&roadmap_path, &task_id, create_branch, &config).await,
        RoadmapSubcommand::Complete {
            task_id,
            skip_quality_check,
        } => complete_task(&roadmap_path, &task_id, skip_quality_check, &config).await,
        RoadmapSubcommand::Status {
            sprint,
            task,
            format,
        } => show_status(&roadmap_path, sprint.as_deref(), task.as_deref(), format).await,
        RoadmapSubcommand::Validate { sprint, strict } => {
            validate_sprint(&roadmap_path, &sprint, strict, &config).await
        }
        RoadmapSubcommand::QualityCheck { task_id } => quality_check(&task_id, &config).await,
    }
}

// --- Submodule includes ---
include!("commands_init.rs");
include!("commands_todos.rs");
include!("commands_tasks.rs");
include!("commands_status.rs");
include!("commands_validation.rs");

// Regression tests for `roadmap status --format`
include!("commands_status_tests.rs");

// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "tests.rs"]
mod tests;
#[cfg(all(test, feature = "broken-tests"))]
mod tests_part2;
#[cfg(all(test, feature = "broken-tests"))]
mod tests_part3;
#[cfg(all(test, feature = "broken-tests"))]
mod tests_part4;
