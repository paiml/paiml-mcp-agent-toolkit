#![cfg_attr(coverage_nightly, coverage(off))]
//! Commands enum - all top-level CLI subcommands
//!
//! The `Commands` enum is the central dispatch type for all PMAT CLI subcommands.
//! The enum definition is in `definition.rs`; imports are here in `mod.rs`.
//!
//! ## Variant groups (by semantic domain)
//!
//! ### Template commands (lines ~1-70)
//! `Generate`, `Scaffold`, `List`, `Search`, `Validate`
//!
//! ### Context & Query (lines ~71-303)
//! `Context`, `Query` (RAG-powered semantic search with 40+ options)
//!
//! ### Analysis & Demo (lines ~304-403)
//! `Analyze`, `Qdd`, `Demo`, `ValidateDocs`, `ValidateReadme`, `RedTeam`, `Org`, `Prompt`
//!
//! ### Scoring (lines ~404-698)
//! `QualityGate`, `Report`, `RepoScore`, `RustProjectScore`, `PopperScore`,
//! `DemoScore`, `BrickScore`, `DepsAudit`
//!
//! ### Infrastructure (lines ~699-896)
//! `Serve`, `Diagnose`, `Enforce`, `Refactor`, `Roadmap`, `Test`, `Memory`,
//! `Cache`, `Telemetry`, `Config`, `ShowMetrics`, `PredictQuality`, `RecordMetric`, `Agent`
//!
//! ### Dev tools (lines ~897-1030)
//! `Tdg`, `QualityGates`, `Maintain`, `Hooks`, `Embed`, `Semantic`, `Mutate`, `Debug`, `Work`
//!
//! ### Operations (lines ~1031-1407)
//! `Falsify`, `QaWork`, `DebugFiveWhys`, `Oracle`, `PerfectionScore`, `Spec`,
//! `Comply`, `Kaizen`, `ProjectDiag`, `TestDiscovery`, `Localize`, `Extract`,
//! `Split`, `CudaTdg`, `Sql`

use crate::cli::diagnose::DiagnoseArgs;
use crate::cli::handlers::cache::CacheCommand;
use crate::cli::handlers::memory::MemoryCommand;
use crate::cli::verify::VerifyArgs;
use crate::cli::{
    AnalysisType, ContextFormat, DebugOutputFormat, DemoProtocol, OutputFormat, QualityCheckType,
    QualityGateOutputFormat, QueryOutputFormat, RepoScoreOutputFormat, ReportOutputFormat,
    TdgOutputFormat,
};
use clap::Subcommand;
use serde_json::Value;
use std::path::PathBuf;

use super::analyze_commands::AnalyzeCommands;
use super::config_hooks::HooksCommands;
use super::mcp_commands::McpCommands;
use super::misc_commands::{
    ComplyCommands, CudaTdgCommand, CudaTdgOutputFormat, DebugCommands, KaizenOutputFormat,
    MaintainCommands, OracleCommands, PerfectionScoreOutputFormat, ProjectDiagOutputFormat,
    QualityGatesCommand, SpecCommands, StackCommands, TdgCommand,
};
use super::org_prompt::{OrgCommands, PromptCommands};
use super::quality_commands::{EnforceCommands, QddCommands};
use super::refactor_scaffold::{RefactorCommands, ScaffoldCommands};
use super::roadmap_agent::{AgentCommands, RoadmapCommands, ServeTransport, TestSuite};
#[cfg(feature = "mutation-testing")]
use super::semantic_search::MutateArgs;
use super::semantic_search::{EmbedCommands, SemanticCommands};
use super::work_commands::{QaWorkCommands, TestDiscoveryCommands, WorkCommands};

include!("definition.rs");
