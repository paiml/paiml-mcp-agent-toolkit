//! #1029 — the MCP tool list is hand-curated, so a new `analyze` subcommand is
//! CLI-only by omission and nothing says so.
//!
//! `pmat analyze --help` grew `reachability`, `hardcoded-paths` and
//! `vacuous-tests` in 3.32.0. All three appeared on the CLI the moment the
//! clap variant existed, and on MCP never — because the MCP surface is
//! [`LIVE_MCP_TOOLS`], a list maintained *beside* the clap enum rather than
//! derived from it. Nothing failed. The tools were simply absent, which is this
//! project's signature defect: absence rendered as success.
//!
//! This file makes absence loud. Every `pmat analyze <sub>` either has an MCP
//! counterpart or carries an entry in [`CLI_ONLY_ANALYZERS`] saying why not.
//! Adding a subcommand without doing one of those two things fails the build.
//!
//! It does NOT decide that a subcommand *should* be on MCP — that is a judgement
//! per analyzer, and several genuinely should not be. What it removes is the
//! third option: shipping one without anybody deciding.
//!
//! Modelled on `src/cli/commands/inert_flag_disclosure_tests.rs`, which walks the
//! clap tree bidirectionally and refuses to pass on silence. Both directions are
//! checked here for the same reason: a registry entry that outlives its
//! subcommand is a stale claim, and a stale claim is how an exception outlives
//! its reason.

use crate::cli::commands::on_big_stack;
use crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS;
use clap::CommandFactory;

/// Why a `pmat analyze` subcommand is absent from the MCP surface.
///
/// Two statuses, and the difference between them is the whole point. A
/// *decision* not to expose something is a fact about the design. A *gap* is a
/// fact about the backlog. Collapsing the two into one "excluded" bucket is how
/// an unreviewed omission acquires the appearance of a choice, so they are kept
/// apart and counted separately.
#[derive(Clone, Copy)]
enum Absence {
    /// A positive decision: this must not be agent-callable, for the stated
    /// reason. Checked for substance by `every_deliberate_absence_states_why`.
    Deliberate(&'static str),
    /// Nobody has decided. The subcommand predates or postdates the MCP list
    /// and no one weighed it. Tracked by #1029; the count may only go down.
    UnreviewedGap,
}

/// Every `pmat analyze <name>` that MCP does not advertise.
///
/// An entry is a claim someone must defend. Registering the tool on the server
/// is how a row leaves this list; `no_absence_entry_is_secretly_on_mcp` then
/// requires the row to be deleted, so an excuse cannot outlive its reason.
const CLI_ONLY_ANALYZERS: &[(&str, Absence)] = &[
    // ---- decided ----
    (
        "clippy",
        Absence::Deliberate(
            "rewrites source. `--dry-run` is opt-IN, so the default behaviour of \
             `pmat analyze clippy` is to apply fixes; MCP advertises analyzers, and an \
             agent calling a tool named `analyze_*` must not have its working tree edited.",
        ),
    ),
    (
        "coverage-improve",
        Absence::Deliberate(
            "not an analyzer: it drives an Extreme-TDD loop that writes test files and \
             re-runs coverage to a target percentage. Unbounded runtime and a mutated \
             tree are both wrong shapes for a request/response tool call.",
        ),
    ),
    (
        "build-tdg",
        Absence::Deliberate(
            "runs `cargo build` and then gates on a TDG threshold — a CI step, not a \
             query. The analysis half is already reachable as `quality_gate`, and the \
             build half is the caller's own job.",
        ),
    ),
    (
        "wasm",
        Absence::Deliberate(
            "absent from the default build. Its own help opens `[NOT AVAILABLE in the \
             default build] ... needs --features wasm-ast`, so advertising it over MCP \
             would offer every installed binary a tool that cannot run — worse than \
             absent, because absence is at least honest.",
        ),
    ),
    (
        "deep-wasm",
        Absence::Deliberate(
            "behind `--features deep-wasm`, so it is absent from the default build for \
             the same reason `wasm` is: advertising it over MCP would offer every \
             installed binary a tool that cannot run, and absence is at least honest. \
             It also takes a WASM BINARY and DWARF symbols as separate inputs — an \
             artifact-inspection pipeline, not a question about a source tree.",
        ),
    ),
    (
        "mutate",
        Absence::Deliberate(
            "behind `--features mutation-testing`, and the same shape as \
             `coverage-improve`: it GENERATES mutants, writes them, and re-runs the \
             test suite per mutant. Unbounded runtime and a mutated tree are both \
             wrong for a request/response tool call, whatever the speedup. An agent \
             wanting the verdict rather than the run can read the mutation gate's \
             recorded result.",
        ),
    ),
    // ---- gaps: nothing decided these, they are simply not on the list ----
    // The three that opened #1029 (all new in 3.32.0) head the list.
    ("reachability", Absence::UnreviewedGap),
    ("hardcoded-paths", Absence::UnreviewedGap),
    ("vacuous-tests", Absence::UnreviewedGap),
    ("unrun-tests", Absence::UnreviewedGap),
    ("assembly-script", Absence::UnreviewedGap),
    ("bottleneck", Absence::UnreviewedGap),
    ("churn", Absence::UnreviewedGap),
    ("cluster", Absence::UnreviewedGap),
    ("comprehensive", Absence::UnreviewedGap),
    ("defect-prediction", Absence::UnreviewedGap),
    ("defects", Absence::UnreviewedGap),
    ("duplicates", Absence::UnreviewedGap),
    ("entropy", Absence::UnreviewedGap),
    ("graph-metrics", Absence::UnreviewedGap),
    ("incremental-coverage", Absence::UnreviewedGap),
    ("lint-hotspot", Absence::UnreviewedGap),
    ("makefile", Absence::UnreviewedGap),
    ("models", Absence::UnreviewedGap),
    ("name-similarity", Absence::UnreviewedGap),
    ("proof-annotations", Absence::UnreviewedGap),
    ("provability", Absence::UnreviewedGap),
    ("symbol-table", Absence::UnreviewedGap),
    ("tdg", Absence::UnreviewedGap),
    ("topics", Absence::UnreviewedGap),
    ("web-assembly", Absence::UnreviewedGap),
];

/// The number of `UnreviewedGap` rows at the time #1029 was filed.
///
/// A ceiling, not a target: it may only go down. It is deliberately not an
/// equality, because a row moving from `UnreviewedGap` to `Deliberate` is
/// progress and must not fail the build. Adding a NEW analyzer and leaving it
/// unreviewed pushes the count over the ceiling and fails — which is the
/// behaviour #1029 asks for: a new subcommand cannot be CLI-only in silence.
const UNREVIEWED_GAP_CEILING: usize = 25;

/// Every direct subcommand name under `pmat analyze`, from the clap tree.
///
/// Aliases are deliberately excluded: the canonical name is what a tool name
/// would be derived from.
fn analyze_subcommands() -> Vec<String> {
    on_big_stack(|| {
        let root = <crate::cli::Cli as CommandFactory>::command();
        let analyze = root
            .get_subcommands()
            .find(|c| c.get_name() == "analyze")
            .cloned()
            .expect("`pmat analyze` exists");
        let mut names: Vec<String> = analyze
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .filter(|n| n != "help")
            .collect();
        names.sort_unstable();
        names
    })
}

/// The MCP tool name a subcommand would carry, by the convention every one of
/// the six live analyze tools already follows: `dead-code` -> `analyze_dead_code`.
fn mcp_tool_name_for(sub: &str) -> String {
    format!("analyze_{}", sub.replace('-', "_"))
}

fn mcp_advertises(tool: &str) -> bool {
    LIVE_MCP_TOOLS.iter().any(|(n, _)| *n == tool)
}

fn declared_cli_only(sub: &str) -> bool {
    CLI_ONLY_ANALYZERS.iter().any(|(n, _)| *n == sub)
}

/// The walker must actually find the subcommands.
///
/// Without this, a rename of `analyze` or a clap-API change turns the whole
/// file into a vacuous pass — an empty list satisfies every `for` loop below.
/// The floor is well under the real count so it does not fight ordinary churn,
/// but far above zero.
#[test]
fn the_walker_discovers_the_analyze_surface() {
    let subs = analyze_subcommands();
    assert!(
        subs.len() >= 30,
        "only {} `pmat analyze` subcommands discovered ({subs:?}) — the walker is \
         broken, and every parity check in this file is passing vacuously",
        subs.len()
    );
    for known in ["complexity", "satd", "dead-code"] {
        assert!(
            subs.iter().any(|s| s == known),
            "`pmat analyze {known}` not discovered; walker is reading the wrong tree"
        );
    }
}

/// Every `pmat analyze` subcommand is either on MCP or declared CLI-only.
#[test]
fn every_analyze_subcommand_is_on_mcp_or_declared_cli_only() {
    let mut undeclared = Vec::new();
    for sub in analyze_subcommands() {
        let tool = mcp_tool_name_for(&sub);
        if !mcp_advertises(&tool) && !declared_cli_only(&sub) {
            undeclared.push(format!("pmat analyze {sub}  (would be MCP tool `{tool}`)"));
        }
    }
    assert!(
        undeclared.is_empty(),
        "these `pmat analyze` subcommands are CLI-only and nothing records why:\n  {}\n\n\
         The MCP tool list is hand-curated (mcp_pmcp/tool_manifest.rs::LIVE_MCP_TOOLS), so a \
         new subcommand is absent from MCP unless someone adds it. Either register the tool \
         on the server, or add a `(name, reason)` row to CLI_ONLY_ANALYZERS in this file \
         saying why an agent cannot reach it.",
        undeclared.join("\n  ")
    );
}

/// ...and every CLI-only entry still names a live subcommand.
#[test]
fn every_cli_only_entry_names_a_live_subcommand() {
    let subs = analyze_subcommands();
    for (name, _) in CLI_ONLY_ANALYZERS {
        assert!(
            subs.iter().any(|s| s == name),
            "CLI_ONLY_ANALYZERS claims `pmat analyze {name}` is CLI-only, but no such \
             subcommand exists any more — delete the entry rather than leaving a claim \
             about nothing"
        );
    }
}

/// ...and no CLI-only entry is secretly on MCP.
///
/// The failure mode this catches is a tool being registered without the excuse
/// being retired, leaving the file asserting an absence that is no longer true.
#[test]
fn no_absence_entry_is_secretly_on_mcp() {
    for (name, _) in CLI_ONLY_ANALYZERS {
        let tool = mcp_tool_name_for(name);
        assert!(
            !mcp_advertises(&tool),
            "`pmat analyze {name}` is recorded as CLI-only but MCP advertises `{tool}` — \
             remove the CLI_ONLY_ANALYZERS entry"
        );
    }
}

/// Every deliberate absence states why.
///
/// A `Deliberate` row whose reason is blank or a placeholder records nothing;
/// it converts this registry back into the hand-curated silence it exists to
/// replace, while looking like a decision.
#[test]
fn every_deliberate_absence_states_why() {
    for (name, absence) in CLI_ONLY_ANALYZERS {
        let Absence::Deliberate(reason) = absence else {
            continue;
        };
        let lower = reason.to_lowercase();
        assert!(
            reason.len() >= 40,
            "the reason for `pmat analyze {name}` is {} chars ({reason:?}) — state why an \
             agent must not reach this analyzer, or downgrade the row to \
             Absence::UnreviewedGap, which is at least honest",
            reason.len()
        );
        assert!(
            !lower.starts_with("todo") && !lower.starts_with("tbd") && lower != "n/a",
            "`pmat analyze {name}` carries a placeholder reason ({reason:?})"
        );
    }
}

/// The unreviewed-gap count may only go down.
///
/// This is the half that makes the registry a ratchet rather than a filing
/// cabinet. Without it, a new analyzer is added as one more `UnreviewedGap`
/// row, the parity test goes green, and #1029 is back — the omission recorded
/// but still unexamined.
#[test]
fn the_unreviewed_gap_count_only_goes_down() {
    let pending: Vec<&str> = CLI_ONLY_ANALYZERS
        .iter()
        .filter(|(_, a)| matches!(a, Absence::UnreviewedGap))
        .map(|(n, _)| *n)
        .collect();
    assert!(
        pending.len() <= UNREVIEWED_GAP_CEILING,
        "{} `pmat analyze` subcommands are on MCP-parity backlog against a ceiling of \
         {UNREVIEWED_GAP_CEILING}: {pending:?}\n\n\
         Either register the new one as an MCP tool, or record a real \
         Absence::Deliberate reason for it. Raising UNREVIEWED_GAP_CEILING is not a fix — \
         it is the omission this file exists to catch, written down.",
        pending.len()
    );
}

/// Every `analyze_*` MCP tool maps back to a live CLI subcommand.
///
/// The other direction of the same drift: a tool that outlives its subcommand
/// answers agents about an analyzer a human can no longer run.
#[test]
fn every_analyze_mcp_tool_has_a_cli_subcommand() {
    let subs = analyze_subcommands();
    for (tool, _) in LIVE_MCP_TOOLS {
        let Some(rest) = tool.strip_prefix("analyze_") else {
            continue;
        };
        assert!(
            subs.iter().any(|s| mcp_tool_name_for(s) == *tool),
            "MCP advertises `{tool}` but `pmat analyze {}` does not exist (nearest CLI \
             names: {subs:?})",
            rest.replace('_', "-")
        );
    }
}

/// The module's own documentation may not carry a tool inventory that drifts.
///
/// `mcp_pmcp/mod.rs` documented "24 MCP tools" as a hand-written bullet list —
/// four `refactor.*` tools unregistered in EV-0 (#999) and six `tdg_*` tools
/// this server has never registered, against a live surface of 16. A reader,
/// or an agent, taking that list at face value asks for tools that do not
/// exist. The list is gone; this keeps any replacement honest rather than
/// forbidding one, so a future author may write the inventory back as long as
/// every entry is real.
#[test]
fn the_module_doc_names_no_tool_the_server_does_not_serve() {
    let mut bogus = Vec::new();
    for line in include_str!("mod.rs").lines() {
        let trimmed = line.trim_start();
        // The shape that drifted:  //! - `tool_name` - description
        let Some(rest) = trimmed.strip_prefix("//! - `") else {
            continue;
        };
        let Some(end) = rest.find('`') else {
            continue;
        };
        let name = &rest[..end];
        if !mcp_advertises(name) {
            bogus.push(name.to_string());
        }
    }
    assert!(
        bogus.is_empty(),
        "the mcp_pmcp module doc advertises tools the server does not register: \
         {bogus:?}\\nLIVE_MCP_TOOLS is the inventory; a hand-written copy beside it \
         drifts, which is how a doc claiming 24 tools survived over a surface of 16."
    );
}
