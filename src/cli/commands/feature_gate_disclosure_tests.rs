//! `--help` must name every Cargo feature a command needs to work at all.
//!
//! A subcommand that parses, prints a confident description, and then exits
//! with "requires the 'X' feature" sends the reader to debug their input for a
//! problem that is in the binary they installed. The repo already treats that
//! as a defect for `demo`, `org` and `serve --transport http`, whose entries
//! carry a `[NOT AVAILABLE in the default build] … needs --features …` label.
//!
//! Audit of every hard precondition reachable from the default
//! (`cargo install pmat`) build — `default = core-languages, viz, http-client,
//! standard-deps` — found four more surfaces that were still silent, measured
//! against the 3.32.0 release binary:
//!
//! | surface                          | feature        | runtime error |
//! |----------------------------------|----------------|---------------|
//! | `pmat analyze wasm`              | `wasm-ast`     | "WASM analysis requires the 'wasm-ast' feature" |
//! | `pmat tdg dashboard`             | `http-server`  | "Dashboard requires the 'http-server' feature" |
//! | `pmat agent <any subcommand>`    | `agent-daemon` | "Agent daemon feature not enabled" |
//! | `pmat analyze complexity --watch`| `watch`        | "Watch mode requires the 'watch' feature" |
//!
//! Deliberately excluded, and why: `analyze deep-wasm` and `mutate` are
//! `#[cfg]`-ed out of the enum entirely, so they never appear in `--help` and
//! cannot mislead; `debug replay --interactive` is already labelled
//! `[NOT IMPLEMENTED]` and its `tui` check has no caller; `github-api`,
//! `git-lib` and `diagnostics` degrade to a `gh`/`git` subprocess or a zeroed
//! field instead of failing; `doc-indexing` only shortens PDF text. None of
//! those is a precondition for the command running.
//!
//! The label is unconditional on purpose. "[NOT AVAILABLE in the default
//! build]" is a statement about the default build, which stays true in a build
//! that does enable the feature, so the help text does not fork per feature
//! set — the same choice `demo` and `org` already make.

use super::on_big_stack;
use clap::CommandFactory;

/// Walk the clap tree to a (possibly nested) subcommand.
fn command_at(path: &[&str]) -> clap::Command {
    let owned: Vec<String> = path.iter().map(|s| (*s).to_string()).collect();
    on_big_stack(move || {
        let mut cmd = <crate::cli::Cli as CommandFactory>::command();
        for name in &owned {
            let next = cmd
                .get_subcommands()
                .find(|s| s.get_name() == name.as_str())
                .cloned()
                .unwrap_or_else(|| panic!("subcommand `{name}` must exist"));
            cmd = next;
        }
        cmd
    })
}

/// The one-line entry clap prints for this command in its parent's
/// `Commands:` list.
fn list_entry(path: &[&str]) -> String {
    command_at(path)
        .get_about()
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// What `pmat <path> --help` prints as the description: `long_about` when there
/// is one, `about` otherwise — clap's own fallback.
fn help_text(path: &[&str]) -> String {
    let cmd = command_at(path);
    cmd.get_long_about()
        .or_else(|| cmd.get_about())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// What `pmat <path> --help` prints next to one flag.
fn flag_help(path: &[&str], id: &str) -> String {
    let cmd = command_at(path);
    let arg = cmd
        .get_arguments()
        .find(|a| a.get_id() == id)
        .unwrap_or_else(|| panic!("`--{id}` must exist on `pmat {}`", path.join(" ")));
    arg.get_long_help()
        .or_else(|| arg.get_help())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn assert_text_discloses(what: &str, where_: &str, text: &str, feature: &str) {
    assert!(
        text.contains("NOT AVAILABLE") || text.contains("NOT IMPLEMENTED"),
        "{what} cannot run in the default build; its {where_} must say so, \
         got: {text}"
    );
    assert!(
        text.contains(&format!("--features {feature}")),
        "{what}'s {where_} must name the feature that would enable it \
         (--features {feature}), got: {text}"
    );
}

/// Both places a reader looks: the parent's command list, and the command's
/// own `--help` body. Labelling only one of them still strands whoever read
/// the other — which is how `org` ended up labelled in the list and silent in
/// `pmat org --help`.
fn assert_discloses(what: &str, path: &[&str], feature: &str) {
    assert_text_discloses(what, "command-list entry", &list_entry(path), feature);
    assert_text_discloses(what, "--help body", &help_text(path), feature);
}

/// `pmat analyze wasm --help` described the analysis in full and said nothing
/// about `wasm-ast`; the requirement only appeared after you supplied a real
/// `.wasm` file and the command exited rc=1.
#[test]
fn analyze_wasm_help_names_the_wasm_ast_feature() {
    assert_discloses("`pmat analyze wasm`", &["analyze", "wasm"], "wasm-ast");
}

/// The reality the help above has to match, measured rather than assumed.
#[cfg(not(feature = "wasm-ast"))]
#[tokio::test]
async fn analyze_wasm_really_refuses_without_the_feature() {
    let cmd = on_big_stack(|| {
        let cli = <crate::cli::Cli as clap::Parser>::try_parse_from([
            "pmat", "analyze", "wasm", "mod.wasm",
        ])
        .expect("clap accepts `pmat analyze wasm mod.wasm`");
        match cli.command {
            crate::cli::Commands::Analyze(a) => a,
            _ => panic!("must parse as an analyze command"),
        }
    });
    let err = crate::cli::handlers::analysis_handlers::route_analyze_command(cmd)
        .await
        .expect_err("`analyze wasm` must refuse without --features wasm-ast")
        .to_string();
    assert!(
        err.contains("wasm-ast"),
        "the refusal names the feature, got: {err}"
    );
}

/// `pmat tdg --help` listed `dashboard  Start TDG web dashboard server`, and
/// `pmat tdg dashboard --help` documented `--port`, `--host` and `--open`, for
/// a server that cannot be built in this binary.
#[test]
fn tdg_dashboard_help_names_the_http_server_feature() {
    assert_discloses("`pmat tdg dashboard`", &["tdg", "dashboard"], "http-server");
}

#[cfg(not(feature = "http-server"))]
#[tokio::test]
async fn tdg_dashboard_really_refuses_without_the_feature() {
    let command = on_big_stack(|| {
        let cli = <crate::cli::Cli as clap::Parser>::try_parse_from(["pmat", "tdg", "dashboard"])
            .expect("clap accepts `pmat tdg dashboard`");
        match cli.command {
            crate::cli::Commands::Tdg { command, .. } => {
                command.expect("`dashboard` is a subcommand of `tdg`")
            }
            _ => panic!("must parse as a tdg command"),
        }
    });
    let err = crate::cli::handlers::tdg_diagnostic_handler::handle_tdg_diagnostics(
        &command,
        &std::path::PathBuf::from("."),
    )
    .await
    .expect_err("`tdg dashboard` must refuse without --features http-server")
    .to_string();
    assert!(
        err.contains("http-server"),
        "the refusal names the feature, got: {err}"
    );
}

/// Every one of `agent`'s nine subcommands exits with "Agent daemon feature
/// not enabled" in the shipped build, while the command list advertised
/// "Start Claude Code background agent for continuous quality monitoring".
/// Same defect `demo` and `org` were fixed for.
#[test]
fn agent_help_names_the_agent_daemon_feature() {
    assert_discloses("`pmat agent`", &["agent"], "agent-daemon");
}

/// `--watch` is documented on `analyze complexity` as "Watch mode for
/// continuous analysis", with no hint that passing it is an immediate error.
#[test]
fn complexity_watch_flag_names_the_watch_feature() {
    assert_text_discloses(
        "`pmat analyze complexity --watch`",
        "flag help",
        &flag_help(&["analyze", "complexity"], "watch"),
        "watch",
    );
}

#[cfg(not(feature = "watch"))]
#[tokio::test]
async fn complexity_watch_really_refuses_without_the_feature() {
    let cmd = on_big_stack(|| {
        let cli = <crate::cli::Cli as clap::Parser>::try_parse_from([
            "pmat",
            "analyze",
            "complexity",
            "--watch",
        ])
        .expect("clap accepts `pmat analyze complexity --watch`");
        match cli.command {
            crate::cli::Commands::Analyze(a) => a,
            _ => panic!("must parse as an analyze command"),
        }
    });
    let err = crate::cli::handlers::analysis_handlers::route_analyze_command(cmd)
        .await
        .expect_err("`--watch` must refuse without --features watch")
        .to_string();
    assert!(
        err.contains("watch"),
        "the refusal names the feature, got: {err}"
    );
}

/// COUNTER-TEST. Green before the disclosures were added and green after: the
/// commands that work in the shipped build must not be labelled unavailable.
/// A "fix" that stamped a feature warning onto everything, or that labelled
/// commands whose only gate is optional, fails here.
#[test]
fn commands_that_work_in_the_default_build_claim_no_feature_requirement() {
    for path in [
        &["analyze", "churn"][..],
        &["analyze", "complexity"][..],
        &["analyze", "dead-code"][..],
        &["analyze", "satd"][..],
        &["analyze", "dag"][..],
        &["analyze", "deep-context"][..],
        &["quality-gate"][..],
        &["context"][..],
        &["report"][..],
        &["list"][..],
        &["tdg"][..],
        &["verify"][..],
    ] {
        let text = help_text(path);
        assert!(
            !text.contains("--features"),
            "`pmat {}` runs in the default build; its help must not claim a \
             feature requirement, got: {text}",
            path.join(" ")
        );
        assert!(
            !text.contains("NOT AVAILABLE"),
            "`pmat {}` runs in the default build; its help must not call it \
             unavailable, got: {text}",
            path.join(" ")
        );
    }
}

/// COUNTER-TEST. The three disclosures that already existed must survive: a
/// change to this area that dropped them would be a regression of the same
/// defect, not a fix for it.
#[test]
fn the_disclosures_that_already_existed_are_still_there() {
    // These two carry their label in the command-list entry only; widening
    // them to the `--help` body is a separate fix and is not asserted here.
    assert_text_discloses(
        "`pmat demo`",
        "command-list entry",
        &list_entry(&["demo"]),
        "demo",
    );
    assert_text_discloses(
        "`pmat org`",
        "command-list entry",
        &list_entry(&["org"]),
        "org-intelligence",
    );
    assert!(
        help_text(&["serve"]).contains("--features mcp-http"),
        "`pmat serve --help` must keep naming the feature its HTTP transport \
         needs"
    );
}
