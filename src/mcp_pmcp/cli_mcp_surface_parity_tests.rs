//! #1029 — the MCP tool list was hand-curated, so a new `analyze` subcommand
//! was CLI-only by omission and nothing said so.
//!
//! # Where the parity checks moved, and why
//!
//! This file used to carry `CLI_ONLY_ANALYZERS`: a table of subcommand names
//! and reasons, maintained here, beside the clap enum. It caught the omission —
//! but it was itself a hand-maintained list keyed on strings, so it could only
//! ever fail AFTER a variant landed, and only if the author of the next
//! subcommand happened to run the test. A new variant compiled fine.
//!
//! The decision now lives in [`crate::cli::analyze_mcp_exposure`], as a **total
//! match over `AnalyzeCommands` with no catch-all arm**, generated together
//! with its enumerable table from one list of rows. A new variant does not fail
//! a test; it fails to COMPILE until somebody writes down what MCP does with
//! it. Tests for it live next to it, in `cli/analyze_mcp_exposure_tests.rs`.
//!
//! Deleting the table rather than keeping it in sync was the point: two lists
//! of the same fact was the root cause the issue named, and a duplicate here —
//! however well tested — would be the same shape of bug.
//!
//! # What stays here
//!
//! One check that belongs to this module rather than to the CLI: the module's
//! own prose may not advertise tools the server does not serve. It is a
//! different failure from the registry's (a doc drifting away from the code,
//! not a decision going unmade), and its subject is `mcp_pmcp/mod.rs`.

use crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS;

fn mcp_advertises(tool: &str) -> bool {
    LIVE_MCP_TOOLS.iter().any(|(n, _)| *n == tool)
}

/// The module's own documentation may not carry a tool inventory that drifts.
///
/// `mcp_pmcp/mod.rs` documented "24 MCP tools" as a hand-written bullet list —
/// four `refactor.*` tools unregistered in EV-0 (#999) and six `tdg_*` tools
/// this server has never registered, against a live surface of 16. A reader, or
/// an agent, taking that list at face value asks for tools that do not exist.
/// The list is gone; this keeps any replacement honest rather than forbidding
/// one, so a future author may write the inventory back as long as every entry
/// is real.
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
         {bogus:?}\nLIVE_MCP_TOOLS is the inventory; a hand-written copy beside it \
         drifts, which is how a doc claiming 24 tools survived over a surface of 16."
    );
}

/// The HTTP transport must keep deriving its tools from the stdio builder.
///
/// `mcp-http` is in `default` as of 3.32.0, so a released binary serves THREE
/// surfaces — CLI, MCP over stdio, MCP over streamable HTTP — and the exposure
/// question is three-way, not two-way.
///
/// Today they cannot disagree: `serve_http` calls
/// `SimpleUnifiedServer::build_server`, the same function `run()` uses, so the
/// registry chain (`cli::analyze_mcp_exposure` -> `LIVE_MCP_TOOLS` ->
/// `build_server`) reaches HTTP for free. But "for free" is the same standing
/// as the MCP tool list had before #1029: correct by habit, guarded by nothing.
/// A second builder for HTTP would restore exactly the two-inventory split this
/// issue is about, one transport further out — and `manifest_matches_server`
/// would not see it, because that guard scans only `run()`'s registrations.
///
/// So: HTTP may call the shared builder and may not register tools of its own.
#[test]
fn the_http_transport_serves_the_stdio_tool_surface() {
    let src = include_str!("http_server.rs");
    assert!(
        src.contains("SimpleUnifiedServer::build_server"),
        "mcp_pmcp/http_server.rs no longer builds its server from \
         SimpleUnifiedServer::build_server. If it grew its own registrations, the HTTP \
         surface is now a second answer to 'which tools does pmat advertise' and the \
         #1029 registry does not reach it."
    );
    let own_registrations: Vec<&str> = src
        .lines()
        .filter(|l| l.contains(".tool(") && !l.trim_start().starts_with("//"))
        .collect();
    assert!(
        own_registrations.is_empty(),
        "mcp_pmcp/http_server.rs registers tools of its own: {own_registrations:?}\n\
         Every transport must take its surface from build_server, or CLI-vs-stdio parity \
         stops implying CLI-vs-HTTP parity."
    );
}

/// This file must not grow a second inventory again.
///
/// The regression being prevented is the one the header describes: someone
/// re-adds a `&[(&str, ...)]` table of subcommand names here, next to the
/// server, and it drifts from the registry that the compiler enforces. Keyed on
/// the shape rather than a name, because the name is the easy part to change.
#[test]
fn the_parity_table_is_not_re_added_beside_the_server() {
    let src = include_str!("cli_mcp_surface_parity_tests.rs");
    // Assembled at runtime so this line is not itself the match it is hunting
    // for. Written literally, the check failed on its own source — a scanner
    // that finds only itself reports a defect that is not there, which is the
    // mirror image of the one that reports nothing.
    let needle = ["const", "CLI_ONLY"].join(" ");
    let declarations = src.matches(needle.as_str()).count();
    assert_eq!(
        declarations, 0,
        "a CLI-only exclusion table has reappeared in this file. The decision belongs in \
         cli/analyze_mcp_exposure.rs, where a missing row is a compile error; a copy here \
         can only be a second answer to the same question (#1029)."
    );
}
