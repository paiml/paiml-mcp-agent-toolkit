//! Wire name and category for every subcommand `pmat` advertises — decided
//! once, in a build every gate compiles.
//!
//! # Why this lives here and not in the protocol adapter
//!
//! This table is a pure function of [`crate::cli::Commands`], which is
//! default-features code. It used to live inside
//! `unified_protocol::adapters::cli`, behind `#[cfg(feature =
//! "unified-protocol")]` — a feature in neither `default` nor `full`. The
//! consequence was not academic:
//!
//! * The exhaustiveness of the match — the only thing that turns "somebody
//!   added a `Commands` variant and forgot to name it" from a runtime
//!   `unreachable!()` into a compile error — was checked by no build the
//!   project ships. `cargo check` on a default build never type-checked it.
//! * The test that guards it (`command_name_totality_tests`) was gated behind
//!   the same flag, so `cargo test --lib -- --list` returned zero matches for
//!   it. A guard that does not run guards nothing.
//!
//! The variants being guarded are default-features types, so the decision
//! table belongs next to them. The protocol adapter now delegates here, which
//! means the compiler enforces totality on every ordinary build and the guard
//! runs in the default test suite.
//!
//! # The invariant
//!
//! Both matches below are total: **no catch-all arm**. That is deliberate and
//! load-bearing. Adding a `Commands` or `AnalyzeCommands` variant must be a
//! compile error here, not a process abort the first time a user types it.
//! Do not add a `_` arm to make that error go away — name the variant.

use crate::cli::commands::QddCommands;
use crate::cli::{AnalyzeCommands, Commands};

/// Categories for `analyze` subcommand dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzeCommandCategory {
    /// Core analysis commands (basic metrics): churn, complexity, dead code, SATD, TDG, lint hotspots
    Basic,
    /// Advanced analysis commands (comprehensive): deep context, comprehensive, defect prediction, duplicates, `BigO`
    Advanced,
    /// Graph and structural analysis: DAG, graph metrics, symbol table, name similarity
    Structural,
    /// Specialized analysis commands: makefile, provability, proof annotations, coverage, WebAssembly
    Specialized,
}

/// Categories for general CLI command dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    /// Generation and creation commands: generate, scaffold
    Generation,
    /// Analysis and assessment commands: analyze (delegated), quality-gate, report
    Analysis,
    /// Operations and maintenance commands: serve, cache, memory, telemetry
    Operations,
    /// Development workflow commands: refactor, test, roadmap, validate
    Workflow,
    /// System interaction commands: list, search, context, diagnose
    System,
    /// Configuration and setup commands: config, agent, tdg
    Configuration,
    /// Demo and examples: demo
    Demo,
    /// Runtime enforcement: enforce
    Enforcement,
}

/// Category **and** wire name for an `analyze` subcommand, decided once.
///
/// No catch-all arm: see the module docs. A new `AnalyzeCommands` variant must
/// fail to compile here.
#[must_use]
pub fn classify_analyze_command(
    analyze_cmd: &AnalyzeCommands,
) -> (AnalyzeCommandCategory, &'static str) {
    use AnalyzeCommandCategory::{Advanced, Basic, Specialized, Structural};
    match analyze_cmd {
        // Core analysis commands (basic metrics)
        AnalyzeCommands::Bottleneck { .. } => (Basic, "analyze-bottleneck"),
        AnalyzeCommands::Churn { .. } => (Basic, "analyze-churn"),
        AnalyzeCommands::Complexity { .. } => (Basic, "analyze-complexity"),
        AnalyzeCommands::DeadCode { .. } => (Basic, "analyze-dead-code"),
        AnalyzeCommands::Defects { .. } => (Basic, "analyze-defects"),
        AnalyzeCommands::Satd { .. } => (Basic, "analyze-satd"),
        AnalyzeCommands::Tdg { .. } => (Basic, "analyze-tdg"),
        AnalyzeCommands::BuildTdg { .. } => (Basic, "analyze-build-tdg"),
        AnalyzeCommands::LintHotspot { .. } => (Basic, "analyze-lint-hotspot"),
        AnalyzeCommands::Clippy { .. } => (Basic, "analyze-clippy"),
        AnalyzeCommands::Entropy { .. } => (Basic, "analyze-entropy"),
        // Reachability is Basic: it reads the module graph and `git
        // ls-files`, with no AST pass and no cargo build.
        AnalyzeCommands::Reachability { .. } => (Basic, "analyze-reachability"),
        // HardcodedPaths likewise: `git ls-files` plus a text scan.
        AnalyzeCommands::HardcodedPaths { .. } => (Basic, "analyze-hardcoded-paths"),
        // VacuousTests parses with syn but runs no cargo build.
        AnalyzeCommands::VacuousTests { .. } => (Basic, "analyze-vacuous-tests"),
        // UnrunTests parses the lib module graph with syn and reads the
        // workflow files; no cargo build, no network.
        AnalyzeCommands::UnrunTests { .. } => (Basic, "analyze-unrun-tests"),

        // Advanced analysis commands (comprehensive)
        AnalyzeCommands::DeepContext { .. } => (Advanced, "analyze-deep-context"),
        AnalyzeCommands::Comprehensive { .. } => (Advanced, "analyze-comprehensive"),
        AnalyzeCommands::DefectPrediction { .. } => (Advanced, "analyze-defect-prediction"),
        AnalyzeCommands::Duplicates { .. } => (Advanced, "analyze-duplicates"),
        AnalyzeCommands::BigO { .. } => (Advanced, "analyze-big-o"),

        // Graph and structural analysis
        AnalyzeCommands::Dag { .. } => (Structural, "analyze-dag"),
        AnalyzeCommands::GraphMetrics { .. } => (Structural, "analyze-graph-metrics"),
        AnalyzeCommands::SymbolTable { .. } => (Structural, "analyze-symbol-table"),
        AnalyzeCommands::NameSimilarity { .. } => (Structural, "analyze-name-similarity"),

        // Specialized analysis commands
        AnalyzeCommands::Makefile { .. } => (Specialized, "analyze-makefile"),
        AnalyzeCommands::Provability { .. } => (Specialized, "analyze-provability"),
        AnalyzeCommands::ProofAnnotations { .. } => (Specialized, "analyze-proof-annotations"),
        AnalyzeCommands::IncrementalCoverage { .. } => {
            (Specialized, "analyze-incremental-coverage")
        }
        AnalyzeCommands::CoverageImprove { .. } => (Specialized, "analyze-coverage-improve"),
        AnalyzeCommands::AssemblyScript { .. } => (Specialized, "analyze-assemblyscript"),
        AnalyzeCommands::WebAssembly { .. } => (Specialized, "analyze-webassembly"),
        AnalyzeCommands::Wasm { .. } => (Specialized, "analyze-wasm"),
        AnalyzeCommands::Cluster { .. } => (Specialized, "analyze-cluster"), // PMAT-SEARCH-011
        AnalyzeCommands::Topics { .. } => (Specialized, "analyze-topics"),   // PMAT-SEARCH-011
        AnalyzeCommands::Models { .. } => (Specialized, "analyze-models"),

        #[cfg(feature = "mutation-testing")]
        AnalyzeCommands::Mutate { .. } => (Specialized, "analyze-mutate"),

        #[cfg(feature = "deep-wasm")]
        AnalyzeCommands::DeepWasm { .. } => (Specialized, "analyze-deep-wasm"),
    }
}
/// Category **and** wire name for a top-level command, decided once.
///
/// No catch-all arm: see the module docs. A new `Commands` variant must fail
/// to compile here.
#[must_use]
pub fn classify_command(command: &Commands) -> (CommandCategory, &'static str) {
    use CommandCategory::{
        Analysis, Configuration, Demo, Enforcement, Generation, Operations, System, Workflow,
    };
    match command {
        // Sub-command delegation: the name comes from the inner enum, which
        // is classified by its own total match.
        Commands::Analyze(analyze_cmd) => (Analysis, classify_analyze_command(analyze_cmd).1),
        // QDD create/refactor/validate are development-workflow verbs, the
        // same group as `refactor` and `validate`.
        Commands::Qdd(qdd_cmd) => (Workflow, qdd_command_name(qdd_cmd)),

        // Generation and creation
        Commands::Generate { .. } => (Generation, "generate"),
        Commands::Scaffold { .. } => (Generation, "scaffold"),

        // Analysis and assessment
        Commands::QualityGate { .. } => (Analysis, "quality-gate"),
        Commands::QualityGates { .. } => (Analysis, "quality-gates"),
        Commands::Report { .. } => (Analysis, "report"),
        Commands::Score { .. } => (Analysis, "score"),
        Commands::RepoScore { .. } => (Analysis, "repo-score"),
        Commands::RustProjectScore { .. } => (Analysis, "rust-project-score"),
        Commands::BrickScore { .. } => (Analysis, "brick-score"),
        Commands::PopperScore { .. } => (Analysis, "popper-score"),
        Commands::DemoScore { .. } => (Analysis, "demo-score"),
        Commands::InfraScore { .. } => (Analysis, "infra-score"),
        Commands::PerfectionScore { .. } => (Analysis, "perfection-score"),
        Commands::ValidateDocs(_) => (Analysis, "validate-docs"),
        Commands::ValidateReadme(_) => (Analysis, "validate-readme"),
        Commands::RedTeam(_) => (Analysis, "red-team"),
        Commands::Org(_) => (Analysis, "org"),
        Commands::Prompt(_) => (Analysis, "prompt"),
        Commands::Embed(_) => (Analysis, "embed"),
        Commands::Semantic(_) => (Analysis, "semantic"),
        Commands::ShowMetrics { .. } => (Analysis, "show-metrics"),
        Commands::PredictQuality { .. } => (Analysis, "predict-quality"),
        Commands::RecordMetric { .. } => (Analysis, "record-metric"),
        Commands::DepsAudit { .. } => (Analysis, "deps-audit"),
        Commands::Comply { .. } => (Analysis, "comply"),
        Commands::ProjectDiag { .. } => (Analysis, "project-diag"),
        Commands::TestDiscovery { .. } => (Analysis, "test-discovery"),
        Commands::DebugFiveWhys { .. } => (Analysis, "five-whys"),
        Commands::Localize { .. } => (Analysis, "localize"),
        Commands::CudaTdg { .. } => (Analysis, "cuda-tdg"),
        Commands::Falsify { .. } => (Analysis, "falsify"),
        Commands::Sql { .. } => (Analysis, "sql"),
        Commands::Verify(_) => (Analysis, "verify"),
        #[cfg(feature = "mutation-testing")]
        Commands::Mutate(_) => (Analysis, "mutate"),

        // Operations and maintenance
        Commands::Serve { .. } => (Operations, "serve"),
        Commands::Cache { .. } => (Operations, "cache"),
        Commands::Memory { .. } => (Operations, "memory"),
        Commands::Telemetry { .. } => (Operations, "telemetry"),

        // Development workflow
        Commands::Refactor(_) => (Workflow, "refactor"),
        Commands::Test { .. } => (Workflow, "test"),
        Commands::Roadmap(_) => (Workflow, "roadmap"),
        Commands::Maintain { .. } => (Workflow, "maintain"), // TICKET-PMAT-5032
        Commands::Hooks(_) => (Workflow, "hooks"),           // TICKET-PMAT-5034
        Commands::Validate { .. } => (Workflow, "validate"),
        Commands::Work { .. } => (Workflow, "work"), // Issue #75
        Commands::Oracle { .. } => (Workflow, "oracle"),
        Commands::QaWork { .. } => (Workflow, "qa-work"), // GH-102
        Commands::Spec { .. } => (Workflow, "spec"),
        Commands::Kaizen { .. } => (Workflow, "kaizen"),
        Commands::Extract { .. } => (Workflow, "extract"),
        Commands::Split { .. } => (Workflow, "split"),
        Commands::CiLocal { .. } => (Workflow, "ci-local"),
        Commands::TestStability { .. } => (Workflow, "test-stability"),
        Commands::Stack { .. } => (Workflow, "stack"),

        // System interaction
        Commands::List { .. } => (System, "list"),
        Commands::Search { .. } => (System, "search"),
        Commands::Context { .. } => (System, "context"),
        Commands::Diagnose(_) => (System, "diagnose"),
        Commands::Debug { .. } => (System, "debug"),
        Commands::Query { .. } => (System, "query"),
        Commands::Explain { .. } => (System, "explain"),
        Commands::Mcp(_) => (System, "mcp"),
        Commands::Agy(_) => (System, "agy"), // Anti-Gravity translator
        Commands::Init { .. } => (System, "init"),

        // Configuration and setup
        Commands::Config { .. } => (Configuration, "config"),
        Commands::Agent { .. } => (Configuration, "agent"),
        Commands::Tdg { .. } => (Configuration, "tdg"),

        Commands::Demo { .. } => (Demo, "demo"),
        Commands::Enforce(_) => (Enforcement, "enforce"),
    }
}
/// Wire name for a QDD subcommand.
#[must_use]
pub fn qdd_command_name(qdd_cmd: &QddCommands) -> &'static str {
    match qdd_cmd {
        QddCommands::Create { .. } => "qdd-create",
        QddCommands::Refactor { .. } => "qdd-refactor",
        QddCommands::Validate { .. } => "qdd-validate",
    }
}
/// The wire name `pmat` uses for a parsed command, on any protocol.
#[must_use]
pub fn command_name(command: &Commands) -> &'static str {
    classify_command(command).1
}
