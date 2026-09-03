//! #1029 — the MCP tool list was hand-curated, so a new `analyze` subcommand
//! was CLI-only by omission and nothing said so.
//!
//! The compile-time half of the fix lives in the parent module: a total match
//! with no catch-all arm, so a new `AnalyzeCommands` variant cannot be built
//! until somebody declares what MCP does with it. These tests guard the halves
//! a compiler cannot see:
//!
//! * the declared `cli_name`s really are the names clap prints (a typo in a row
//!   would otherwise satisfy the compiler and describe nothing);
//! * every row the registry marks [`McpExposure::Tool`] is really served by the
//!   live server, and nothing else is;
//! * a `CliOnly` row states a reason with substance, and a `Backlog` row names
//!   the issue tracking it;
//! * the undecided count only goes down.
//!
//! Both directions are checked throughout, for the same reason: a registry row
//! that outlives its subcommand is a stale claim, and a stale claim is how an
//! exception outlives its reason.

use super::{
    analyze_mcp_exposure, declaration_for, required_analyze_tools, AnalyzeMcpDecl, McpExposure,
    ANALYZE_MCP_REGISTRY, BACKLOG_CEILING,
};
use crate::cli::commands::on_big_stack;
use crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS;
use clap::CommandFactory;

/// Every direct subcommand name under `pmat analyze`, from the clap tree.
///
/// Aliases are deliberately excluded: the canonical name is what a tool name
/// is derived from.
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

fn mcp_advertises(tool: &str) -> bool {
    LIVE_MCP_TOOLS.iter().any(|(n, _)| *n == tool)
}

/// The walker must actually find the subcommands.
///
/// Without this, a rename of `analyze` or a clap-API change turns the whole
/// file into a vacuous pass — an empty list satisfies every `for` loop below.
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

/// Every `pmat analyze` subcommand carries a declaration.
///
/// The compiler already forces a row to EXIST for each variant; this catches
/// the remaining hole, a row whose `cli_name` literal does not match the name
/// clap actually prints.
#[test]
fn every_analyze_subcommand_has_a_declaration() {
    let undeclared: Vec<String> = analyze_subcommands()
        .into_iter()
        .filter(|sub| declaration_for(sub).is_none())
        .collect();
    assert!(
        undeclared.is_empty(),
        "these `pmat analyze` subcommands have no row in ANALYZE_MCP_REGISTRY: {undeclared:?}\n\
         The total match in analyze_mcp_exposure.rs forces a row per VARIANT, so this can \
         only mean a row's `cli_name` literal is misspelled against the clap `#[command(name)]`."
    );
}

/// ...and every declaration names a live subcommand.
#[test]
fn every_declaration_names_a_live_subcommand() {
    let subs = analyze_subcommands();
    for decl in ANALYZE_MCP_REGISTRY {
        assert!(
            subs.iter().any(|s| s == decl.cli_name),
            "ANALYZE_MCP_REGISTRY declares `pmat analyze {}`, but no such subcommand \
             exists — delete the row rather than leaving a claim about nothing. \
             Live names: {subs:?}",
            decl.cli_name
        );
    }
}

/// Every tool the registry promises is really served.
///
/// This is the link the compiler cannot make: `McpExposure::Tool("analyze_x")`
/// is just a string until the server registers a handler under it. Declaring a
/// tool without registering it would advertise MCP parity that does not exist —
/// the same lie as the omission, told the other way round.
#[test]
fn every_advertised_tool_is_actually_served() {
    let unserved: Vec<&str> = required_analyze_tools()
        .into_iter()
        .filter(|tool| !mcp_advertises(tool))
        .collect();
    assert!(
        unserved.is_empty(),
        "ANALYZE_MCP_REGISTRY promises these MCP tools but LIVE_MCP_TOOLS does not \
         carry them: {unserved:?}\nRegister the handler in \
         `mcp_pmcp::simple_unified_server::build_server` and add it to LIVE_MCP_TOOLS."
    );
}

/// ...and every `analyze_*` tool served maps back to a `Tool` row.
///
/// The other direction of the same drift: a tool that outlives its subcommand
/// answers agents about an analyzer a human can no longer run.
#[test]
fn every_served_analyze_tool_maps_back_to_a_declaration() {
    let promised = required_analyze_tools();
    for (tool, _) in LIVE_MCP_TOOLS {
        if !tool.starts_with("analyze_") {
            continue;
        }
        assert!(
            promised.contains(tool),
            "MCP serves `{tool}` but no ANALYZE_MCP_REGISTRY row declares it \
             (declared: {promised:?})"
        );
    }
}

/// No `CliOnly` or `Backlog` row is secretly on MCP.
///
/// Catches a tool being registered without the row being updated, leaving the
/// registry asserting an absence that is no longer true.
#[test]
fn no_unexposed_row_is_secretly_served() {
    for decl in ANALYZE_MCP_REGISTRY {
        if decl.mcp_tool().is_some() {
            continue;
        }
        let would_be = format!("analyze_{}", decl.cli_name.replace('-', "_"));
        assert!(
            !mcp_advertises(&would_be),
            "`pmat analyze {}` is recorded as unexposed but MCP advertises `{would_be}` — \
             change the row to McpExposure::Tool(\"{would_be}\")",
            decl.cli_name
        );
    }
}

/// Every deliberate `CliOnly` states why.
///
/// A row whose reason is blank or a placeholder records nothing; it converts
/// this registry back into the hand-curated silence it exists to replace, while
/// looking like a decision.
#[test]
fn every_cli_only_reason_states_why() {
    for decl in ANALYZE_MCP_REGISTRY {
        let Some(reason) = decl.cli_only_reason() else {
            continue;
        };
        let lower = reason.to_lowercase();
        assert!(
            reason.len() >= 40,
            "the reason for `pmat analyze {}` is {} chars ({reason:?}) — state why an \
             agent must not reach this analyzer, or downgrade the row to \
             McpExposure::Backlog, which is at least honest",
            decl.cli_name,
            reason.len()
        );
        assert!(
            !lower.starts_with("todo") && !lower.starts_with("tbd") && lower != "n/a",
            "`pmat analyze {}` carries a placeholder reason ({reason:?})",
            decl.cli_name
        );
    }
}

/// Every `Backlog` row names the issue tracking it.
///
/// `Backlog` is the honest answer for an analyzer nobody has weighed, but only
/// while it is attached to something that can be worked. A bare marker with no
/// issue is a filing cabinet.
#[test]
fn every_backlog_row_names_an_issue() {
    for decl in ANALYZE_MCP_REGISTRY {
        let Some(issue) = decl.backlog_issue() else {
            continue;
        };
        assert!(
            issue.starts_with('#') && issue[1..].chars().all(|c| c.is_ascii_digit()),
            "`pmat analyze {}` is on the backlog against {issue:?}, which is not an \
             issue reference of the form `#NNNN`",
            decl.cli_name
        );
    }
}

/// The undecided count may only go down.
///
/// This is the half that makes the registry a ratchet rather than a filing
/// cabinet. Without it, a new analyzer is added as one more `Backlog` row, the
/// parity tests go green, and #1029 is back — the omission recorded but still
/// unexamined.
#[test]
fn the_backlog_count_only_goes_down() {
    let pending: Vec<&str> = ANALYZE_MCP_REGISTRY
        .iter()
        .filter_map(|d| d.backlog_issue().map(|_| d.cli_name))
        .collect();
    assert!(
        pending.len() <= BACKLOG_CEILING,
        "{} `pmat analyze` subcommands are on the MCP-parity backlog against a ceiling \
         of {BACKLOG_CEILING}: {pending:?}\n\n\
         Either register the new one as an MCP tool, or record a real \
         McpExposure::CliOnly reason for it. Raising BACKLOG_CEILING is not a fix — it \
         is the omission this registry exists to catch, written down.",
        pending.len()
    );
}

/// COUNTER-TEST: being CLI-only is a legitimate, reachable answer.
///
/// Without this, the cheapest way to make every check above go green is to
/// advertise all 35 analyzers — which would put `pmat analyze clippy`, a
/// command that REWRITES SOURCE BY DEFAULT, behind a tool named `analyze_*` on
/// an agent-facing surface. That is a worse defect than the omission #1029
/// reports, and it must fail here rather than pass as an over-correction.
///
/// So: the mutating analyzers must stay off MCP, and their reasons must still
/// be on record. Both halves matter — absence alone is what the issue is about.
#[test]
fn a_reasoned_cli_only_declaration_is_still_a_legitimate_answer() {
    for mutating in ["clippy", "coverage-improve", "build-tdg"] {
        let found = declaration_for(mutating);
        assert!(
            found.is_some(),
            "`pmat analyze {mutating}` must carry a registry row"
        );
        let Some(decl) = found else { continue };
        assert_eq!(
            decl.mcp_tool(),
            None,
            "`pmat analyze {mutating}` mutates the working tree or runs an unbounded \
             build; advertising it over MCP is an over-correction of #1029, not a fix"
        );
        let reason = decl.cli_only_reason();
        assert!(
            reason.is_some(),
            "`pmat analyze {mutating}` is CLI-only but records no reason — an undecided \
             gap and a decision must not look the same"
        );
        assert!(
            reason.is_some_and(|r| r.len() >= 40),
            "`pmat analyze {mutating}` records only {reason:?}"
        );
    }

    // ...and the registry is genuinely a mixture, not "expose everything" nor
    // "expose nothing" — either extreme would make the checks above vacuous.
    let exposed = required_analyze_tools().len();
    let total = ANALYZE_MCP_REGISTRY.len();
    assert!(
        exposed > 0 && exposed < total,
        "the registry declares {exposed} of {total} analyzers exposed; a registry that \
         is all-or-nothing is not recording decisions"
    );
}

/// The total match and the enumerable table agree.
///
/// They are generated from the same rows, so this is a guard on the macro
/// rather than on the data — but a macro that quietly dropped the `$cfg` slot,
/// or expanded the two halves from different lists, would produce exactly the
/// two-inventory drift #1029 is about.
#[test]
fn the_match_and_the_table_return_the_same_row() {
    let cmd = crate::cli::AnalyzeCommands::Reachability {
        path: std::path::PathBuf::from("."),
        format: "summary".to_string(),
        fail_on_orphan: false,
        write_ledger: false,
        allow_dirty: false,
        check_ledger: false,
    };
    let from_match: AnalyzeMcpDecl = analyze_mcp_exposure(&cmd);
    let from_table = declaration_for("reachability").copied();
    assert_eq!(
        Some(from_match),
        from_table,
        "analyze_mcp_exposure() and ANALYZE_MCP_REGISTRY disagree about `reachability`"
    );
    assert_eq!(
        from_match.exposure,
        McpExposure::Tool("analyze_reachability")
    );
}

/// The match must exist, and must have no catch-all arm.
///
/// The compile-time guarantee this whole module rests on is exactly "a match
/// over `AnalyzeCommands` with no `_` arm". Both halves are one keystroke to
/// undo — delete the generated function, or add `_ =>` — and either silences
/// the error a new variant is supposed to raise while `ANALYZE_MCP_REGISTRY`
/// keeps generating and every other test in this file keeps passing. So both
/// are asserted against the source rather than trusted.
#[test]
fn the_registry_match_exists_and_has_no_catch_all_arm() {
    let src = include_str!("analyze_mcp_exposure.rs");
    let macro_body_start = src
        .find("macro_rules! analyze_mcp_registry")
        .expect("the generating macro must still exist");
    let generated = &src[macro_body_start..];

    for required in ["match cmd {", "AnalyzeCommands::$variant { .. } =>"] {
        assert!(
            generated.contains(required),
            "analyze_mcp_registry! no longer generates a match over AnalyzeCommands \
             (missing {required:?}). Without it the table still builds and these tests \
             still pass, but a new variant compiles in silence — #1029, restored."
        );
    }
    for forbidden in ["_ =>", "_=>"] {
        assert!(
            !generated.contains(forbidden),
            "analyze_mcp_registry! grew a `{forbidden}` catch-all arm. That is the ONE \
             thing this module may not have: with it, a new AnalyzeCommands variant \
             compiles silently and is CLI-only by omission again — #1029, restored."
        );
    }
}

/// The reasons must be readable prose, not a shrug.
///
/// Separate from the length check because length is easy to satisfy with
/// filler. Every deliberate reason has to mention the analyzer's actual
/// behaviour — the thing a reader needs in order to disagree with it.
#[test]
fn deliberate_reasons_describe_behaviour() {
    let behavioural = [
        "write",
        "rewrit",
        "mutat",
        "build",
        "run",
        "feature",
        "unbounded",
        "edit",
        "generat",
    ];
    for decl in ANALYZE_MCP_REGISTRY {
        let Some(reason) = decl.cli_only_reason() else {
            continue;
        };
        let lower = reason.to_lowercase();
        assert!(
            behavioural.iter().any(|w| lower.contains(w)),
            "the reason for `pmat analyze {}` never says what the analyzer DOES \
             ({reason:?}); a reason a reader cannot check is not a reason",
            decl.cli_name
        );
    }
}
