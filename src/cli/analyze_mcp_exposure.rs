//! Which `pmat analyze` subcommands the MCP surface advertises — decided once,
//! in a build every gate compiles, with no way to skip the decision.
//!
//! # The defect this replaces (#1029)
//!
//! The MCP tool list was maintained *beside* [`AnalyzeCommands`], never derived
//! from it. A new clap variant appeared in `pmat analyze --help` the moment it
//! existed and on MCP never. Nothing failed — the tool was simply absent, which
//! is this project's signature defect: absence rendered as success. Measured on
//! the running server at the time #1029 was filed, 29 of 35 `analyze`
//! subcommands were CLI-only and not one of the 29 was written down anywhere.
//!
//! # The invariant
//!
//! [`analyze_mcp_exposure`] is a **total match with no catch-all arm**. Adding a
//! variant to `AnalyzeCommands` is therefore a COMPILE ERROR here until somebody
//! writes down what happens to it on MCP. That is the whole mechanism: not a
//! list to remember to update, but a build that will not proceed. Do not add a
//! `_` arm to make the error go away — name the variant, the way
//! [`crate::cli::command_wire_names`] next door already requires for wire names.
//!
//! The match and the enumerable table are generated from ONE source by
//! `analyze_mcp_registry!`. Two lists were the root cause the issue named; a
//! second copy here, however well tested, would be the same shape of bug.
//!
//! # What a declaration may say
//!
//! Three answers, and the differences between them are the point:
//!
//! * [`McpExposure::Tool`] — advertised, under that tool name.
//! * [`McpExposure::CliOnly`] — a positive decision that agents must NOT reach
//!   it, with the reason. Reasons are checked for substance, because a blank
//!   reason converts this registry back into the silence it exists to replace
//!   while looking like a decision.
//! * [`McpExposure::Backlog`] — nobody has decided yet, with the issue tracking
//!   that. Kept distinct from `CliOnly` deliberately: a decision not to expose
//!   something is a fact about the design, an undecided gap is a fact about the
//!   backlog, and collapsing the two is how an unreviewed omission acquires the
//!   appearance of a choice. The count is ratcheted and may only go down.
//!
//! Tests live in `analyze_mcp_exposure_tests.rs`, including the two directions
//! that keep the table honest: every row names a live subcommand, and every
//! live subcommand has a row.

use crate::cli::AnalyzeCommands;

/// Whether `pmat analyze <sub>` is reachable over MCP, and if not, why not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpExposure {
    /// Advertised over MCP under this tool name.
    Tool(&'static str),
    /// Deliberately not advertised, for the stated reason.
    ///
    /// The reason is load-bearing and is checked for substance. It must say why
    /// an *agent* must not reach the analyzer — not merely that it is absent.
    CliOnly(&'static str),
    /// Not advertised and not yet decided, tracked by the named issue (`#NNNN`).
    ///
    /// Honest rather than flattering: fabricating a rationale for an analyzer
    /// nobody has weighed would make the registry look complete while recording
    /// nothing true.
    Backlog(&'static str),
}

/// One `pmat analyze` subcommand's MCP status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalyzeMcpDecl {
    /// The clap subcommand name, exactly as `pmat analyze --help` prints it.
    pub cli_name: &'static str,
    /// What the MCP surface does with it.
    pub exposure: McpExposure,
}

impl AnalyzeMcpDecl {
    /// The MCP tool name, when this subcommand is advertised.
    #[must_use]
    pub fn mcp_tool(&self) -> Option<&'static str> {
        match self.exposure {
            McpExposure::Tool(name) => Some(name),
            McpExposure::CliOnly(_) | McpExposure::Backlog(_) => None,
        }
    }

    /// The written reason this subcommand is deliberately CLI-only.
    #[must_use]
    pub fn cli_only_reason(&self) -> Option<&'static str> {
        match self.exposure {
            McpExposure::CliOnly(reason) => Some(reason),
            McpExposure::Tool(_) | McpExposure::Backlog(_) => None,
        }
    }

    /// The issue tracking an undecided gap.
    #[must_use]
    pub fn backlog_issue(&self) -> Option<&'static str> {
        match self.exposure {
            McpExposure::Backlog(issue) => Some(issue),
            McpExposure::Tool(_) | McpExposure::CliOnly(_) => None,
        }
    }
}

/// Generate the enumerable table AND the total match from one list of rows.
///
/// The `$(#[$cfg])*` slot carries `#[cfg(feature = ...)]` for the two variants
/// that are feature-gated, so a row exists in exactly the builds in which its
/// subcommand exists — otherwise the "every row names a live subcommand" test
/// would, correctly, read an ungated row as a claim about nothing.
macro_rules! analyze_mcp_registry {
    ($(
        $(#[$cfg:meta])*
        $variant:ident => $cli_name:literal, $exposure:expr,
    )+) => {
        /// Every `pmat analyze` subcommand and what MCP does with it.
        ///
        /// Enumerable half of the registry. Generated from the same rows as
        /// [`analyze_mcp_exposure`], so the two cannot disagree.
        pub const ANALYZE_MCP_REGISTRY: &[AnalyzeMcpDecl] = &[
            $(
                $(#[$cfg])*
                AnalyzeMcpDecl { cli_name: $cli_name, exposure: $exposure },
            )+
        ];

        /// What MCP does with a parsed `analyze` subcommand.
        ///
        /// **Total match, no catch-all arm.** A new `AnalyzeCommands` variant
        /// must fail to compile here until its MCP exposure is declared. See
        /// the module docs before reaching for a `_` arm.
        #[must_use]
        pub fn analyze_mcp_exposure(cmd: &AnalyzeCommands) -> AnalyzeMcpDecl {
            match cmd {
                $(
                    $(#[$cfg])*
                    AnalyzeCommands::$variant { .. } =>
                        AnalyzeMcpDecl { cli_name: $cli_name, exposure: $exposure },
                )+
            }
        }
    };
}

analyze_mcp_registry! {
    // ── Advertised over MCP ────────────────────────────────────────────────
    Complexity => "complexity", McpExposure::Tool("analyze_complexity"),
    Satd => "satd", McpExposure::Tool("analyze_satd"),
    DeadCode => "dead-code", McpExposure::Tool("analyze_dead_code"),
    Dag => "dag", McpExposure::Tool("analyze_dag"),
    DeepContext => "deep-context", McpExposure::Tool("analyze_deep_context"),
    BigO => "big-o", McpExposure::Tool("analyze_big_o"),
    // The three that opened #1029. All read-only, all bounded by `git
    // ls-files`, all answering a question an agent asks constantly ("what in
    // this tree is not real?"), which is exactly the shape that belongs on an
    // agent-facing surface.
    Reachability => "reachability", McpExposure::Tool("analyze_reachability"),
    HardcodedPaths => "hardcoded-paths", McpExposure::Tool("analyze_hardcoded_paths"),
    VacuousTests => "vacuous-tests", McpExposure::Tool("analyze_vacuous_tests"),

    // ── Deliberately CLI-only ──────────────────────────────────────────────
    Clippy => "clippy", McpExposure::CliOnly(
        "rewrites source. `--dry-run` is opt-IN, so the default behaviour of \
         `pmat analyze clippy` is to apply fixes; MCP advertises analyzers, and an \
         agent calling a tool named `analyze_*` must not have its working tree edited.",
    ),
    CoverageImprove => "coverage-improve", McpExposure::CliOnly(
        "not an analyzer: it drives an Extreme-TDD loop that writes test files and \
         re-runs coverage to a target percentage. Unbounded runtime and a mutated \
         tree are both wrong shapes for a request/response tool call.",
    ),
    BuildTdg => "build-tdg", McpExposure::CliOnly(
        "runs `cargo build` and then gates on a TDG threshold — a CI step, not a \
         query. The analysis half is already reachable as `quality_gate`, and the \
         build half is the caller's own job.",
    ),
    Wasm => "wasm", McpExposure::CliOnly(
        "absent from the default build. Its own help opens `[NOT AVAILABLE in the \
         default build] ... needs --features wasm-ast`, so advertising it over MCP \
         would offer every installed binary a tool that cannot run — worse than \
         absent, because absence is at least honest.",
    ),
    // The next two rows are gated to exactly the builds in which the subcommand
    // exists. Ungating either one turns every default build red; deleting
    // either one turns `--features full` red.
    #[cfg(feature = "deep-wasm")]
    DeepWasm => "deep-wasm", McpExposure::CliOnly(
        "behind `--features deep-wasm`, so it is absent from the default build for \
         the same reason `wasm` is: advertising it over MCP would offer every \
         installed binary a tool that cannot run, and absence is at least honest. \
         It also takes a WASM BINARY and DWARF symbols as separate inputs — an \
         artifact-inspection pipeline, not a question about a source tree.",
    ),
    #[cfg(feature = "mutation-testing")]
    Mutate => "mutate", McpExposure::CliOnly(
        "behind `--features mutation-testing`, and the same shape as \
         `coverage-improve`: it GENERATES mutants, writes them, and re-runs the \
         test suite per mutant. Unbounded runtime and a mutated tree are both \
         wrong for a request/response tool call, whatever the speedup. An agent \
         wanting the verdict rather than the run can read the mutation gate's \
         recorded result.",
    ),

    // ── Undecided: on the #1029 backlog, and the count may only go down ─────
    UnrunTests => "unrun-tests", McpExposure::Backlog("#1029"),
    AssemblyScript => "assembly-script", McpExposure::Backlog("#1029"),
    Bottleneck => "bottleneck", McpExposure::Backlog("#1029"),
    Churn => "churn", McpExposure::Backlog("#1029"),
    Cluster => "cluster", McpExposure::Backlog("#1029"),
    Comprehensive => "comprehensive", McpExposure::Backlog("#1029"),
    DefectPrediction => "defect-prediction", McpExposure::Backlog("#1029"),
    Defects => "defects", McpExposure::Backlog("#1029"),
    Duplicates => "duplicates", McpExposure::Backlog("#1029"),
    Entropy => "entropy", McpExposure::Backlog("#1029"),
    GraphMetrics => "graph-metrics", McpExposure::Backlog("#1029"),
    IncrementalCoverage => "incremental-coverage", McpExposure::Backlog("#1029"),
    LintHotspot => "lint-hotspot", McpExposure::Backlog("#1029"),
    Makefile => "makefile", McpExposure::Backlog("#1029"),
    Models => "models", McpExposure::Backlog("#1029"),
    NameSimilarity => "name-similarity", McpExposure::Backlog("#1029"),
    ProofAnnotations => "proof-annotations", McpExposure::Backlog("#1029"),
    Provability => "provability", McpExposure::Backlog("#1029"),
    SymbolTable => "symbol-table", McpExposure::Backlog("#1029"),
    Tdg => "tdg", McpExposure::Backlog("#1029"),
    Topics => "topics", McpExposure::Backlog("#1029"),
    WebAssembly => "web-assembly", McpExposure::Backlog("#1029"),
}

/// How many rows may still be undecided.
///
/// A ceiling, not a target: it may only go down. It is deliberately not an
/// equality, because a row moving from `Backlog` to `Tool` or `CliOnly` is
/// progress and must not fail the build. Adding a NEW analyzer and declaring it
/// `Backlog` pushes the count over the ceiling and fails — which is the
/// behaviour #1029 asks for: a new subcommand cannot be CLI-only in silence.
///
/// 25 when #1029 was filed; 22 after the three analyzers it names were exposed.
pub const BACKLOG_CEILING: usize = 22;

/// Tool names the MCP surface is required to advertise, sorted.
///
/// The projection `tool_manifest.rs` is checked against: the registry decides
/// which `analyze_*` tools exist, the server must serve exactly those.
#[must_use]
pub fn required_analyze_tools() -> Vec<&'static str> {
    let mut tools: Vec<&'static str> = ANALYZE_MCP_REGISTRY
        .iter()
        .filter_map(AnalyzeMcpDecl::mcp_tool)
        .collect();
    tools.sort_unstable();
    tools
}

/// The registry row for a subcommand name, if it has one.
#[must_use]
pub fn declaration_for(cli_name: &str) -> Option<&'static AnalyzeMcpDecl> {
    ANALYZE_MCP_REGISTRY
        .iter()
        .find(|decl| decl.cli_name == cli_name)
}

/// One line stating how much of `pmat analyze` an agent can actually reach.
///
/// Printed by `pmat mcp manifest` so the gap has a number a reader can see
/// without opening this file. #1029 was open for a release because "which
/// analyzers are missing from MCP" had no answer anywhere — not in the
/// manifest, not in `tools/list`, not in the docs. It has one now, and it is
/// computed rather than written down.
#[must_use]
pub fn parity_summary() -> String {
    let exposed = ANALYZE_MCP_REGISTRY
        .iter()
        .filter(|d| d.mcp_tool().is_some())
        .count();
    let deliberate = ANALYZE_MCP_REGISTRY
        .iter()
        .filter(|d| d.cli_only_reason().is_some())
        .count();
    let backlog = ANALYZE_MCP_REGISTRY
        .iter()
        .filter(|d| d.backlog_issue().is_some())
        .count();
    format!(
        "{exposed} of {} `pmat analyze` subcommands are advertised over MCP; \
         {deliberate} are deliberately CLI-only and {backlog} are on the #1029 backlog",
        ANALYZE_MCP_REGISTRY.len()
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
#[path = "analyze_mcp_exposure_tests.rs"]
mod analyze_mcp_exposure_tests;
