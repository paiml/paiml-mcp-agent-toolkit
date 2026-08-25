//! Regressions for the two contradictions the dead-code report shipped with:
//! a suppressed function that belonged to no category (#928) and a `--timeout`
//! that was printed and never enforced (#929).
//!
//! Both run a real `cargo check` over a throwaway crate, because both defects
//! lived in the seam between this handler and the compiler — a hand-built
//! `FileDeadCode` would have passed on the broken code.

use super::{
    format_dead_code_result, run_dead_code_analysis_with_filters, DeadCodeAnalysisFilters,
};
use std::time::{Duration, Instant};

/// A budget the finding tests cannot hit, for the tests that are not about the
/// budget.
///
/// Dead-code analysis of a Rust crate shells out to `cargo check` and is bounded
/// by **wall clock**, so any finite deadline in a test that asserts *findings* is
/// a bet on how loaded the machine is. This one was tuned twice and lost twice:
/// 120s failed under `cargo test`, and the 600s it was raised to failed again
/// under `cargo llvm-cov`, where the instrumented harness runs ~19,800 tests and
/// starves the blocking task (#1013 — `ci / test` passed the same commit that
/// `ci / coverage` failed).
///
/// A third number would only move the odds. The phase has no completion
/// guarantee, so the deadline is removed from the assertion path entirely rather
/// than re-tuned. Nothing is lost: the budget IS covered, deterministically, by
/// `a_cargo_check_that_outruns_the_budget_is_killed_and_reported` below, which
/// uses one second against a crate rigged to outlive it.
///
/// The tradeoff, stated: if the analysis ever genuinely hangs, this test hangs
/// with it and the CI job timeout reports it instead of an assertion. That is a
/// worse message for a real bug, in exchange for never again reporting a fake
/// one — and a hang is a bug we would want to see, not a threshold to tune.
const NO_DEADLINE: Duration = Duration::from_secs(24 * 60 * 60);

fn filters(min_dead_lines: usize) -> DeadCodeAnalysisFilters {
    DeadCodeAnalysisFilters {
        include_unreachable: false,
        include_tests: false,
        min_dead_lines,
        top_files: None,
        include: Vec::new(),
        exclude: Vec::new(),
        max_depth: 10,
    }
}

/// A bin crate: `main` is a live root rustc and the reachability heuristic both
/// recognise, so the two analyzers agree on the findings and any disagreement
/// between the two SURFACES is the report's own doing.
fn write_bin_crate(root: &std::path::Path, name: &str, main_rs: &str) {
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"),
    )
    .expect("Cargo.toml");
    std::fs::write(root.join("src/main.rs"), main_rs).expect("main.rs");
    crate::services::cargo_dead_code_analyzer::write_fixture_lockfile(root);
}

/// One dead function, worth 5 estimated dead lines — under the 10 that
/// `--min-dead-lines` used to default to.
const ONE_DEAD_FUNCTION: &str = "fn main() {\n    println!(\"{}\", entry(1));\n}\n\n\
     fn entry(n: u64) -> u64 {\n    n + 1\n}\n\n\
     fn never_called_dead_fn() -> u64 {\n    7\n}\n";

/// `--min-dead-lines` as the CLI itself defaults it.
///
/// Asked of clap rather than written down: a constant that merely looks like the
/// default cannot fail when the default moves, and it is the default that this
/// pair of tests is about.
fn cli_default_min_dead_lines() -> usize {
    crate::cli::commands::on_big_stack(|| {
        use clap::Parser;
        let cli = crate::cli::Cli::try_parse_from(["pmat", "analyze", "dead-code"])
            .expect("`pmat analyze dead-code` parses");
        let crate::cli::Commands::Analyze(crate::cli::commands::AnalyzeCommands::DeadCode {
            min_dead_lines,
            ..
        }) = cli.command
        else {
            panic!("`pmat analyze dead-code` must parse as Analyze/DeadCode");
        };
        min_dead_lines
    })
}

fn write_crate(root: &std::path::Path, name: &str, lib_rs: &str, build_rs: Option<&str>) {
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    let build_line = if build_rs.is_some() {
        "build=\"build.rs\"\n"
    } else {
        ""
    };
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n{build_line}"),
    )
    .expect("Cargo.toml");
    std::fs::write(root.join("src/lib.rs"), lib_rs).expect("lib.rs");
    if let Some(build) = build_rs {
        std::fs::write(root.join("build.rs"), build).expect("build.rs");
    }
    crate::services::cargo_dead_code_analyzer::write_fixture_lockfile(root);
}

/// The report listed six dead functions and headed them with
/// `dead_functions: 0, dead_classes: 0, dead_modules: 0, unreachable_blocks: 0`
/// — the twelve dead lines belonged to no category at all — and typed each one
/// `item_type: "variable"` in a record whose own `reason` said `fn`. The cause
/// was `DeadCodeKind::Suppressed`, a "kind" that replaced the item's real kind
/// with the way it had been discovered.
#[tokio::test]
async fn a_suppressed_function_is_counted_and_typed_as_a_function() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let root = tmp.path();
    // The attribute is assembled rather than written literally: pmat's own
    // suppression scanner would otherwise find this file and report it.
    let allow = format!("#[allow({})]", "dead_code");
    write_crate(
        root,
        "suppressed_fn_crate",
        &format!(
            "{allow}\nfn admitted_dead(x: i32) -> i32 {{ x + 1 }}\npub fn used() -> i32 {{ 1 }}\n"
        ),
        None,
    );

    let outcome = run_dead_code_analysis_with_filters(root, filters(0), NO_DEADLINE)
        .await
        .expect("analysis runs");

    let summary = &outcome.report.summary;
    let items: Vec<_> = outcome
        .report
        .files
        .iter()
        .flat_map(|f| f.items.iter())
        .collect();

    assert_eq!(
        items.len(),
        1,
        "expected the one suppressed item: {:?}",
        outcome.report.files
    );
    assert_eq!(
        items[0].item_type,
        crate::models::dead_code::DeadCodeType::Function,
        "an item whose reason says `fn` was typed {:?}",
        items[0].item_type
    );
    assert_eq!(
        summary.dead_functions, 1,
        "the summary counts 0 dead functions over a listed dead function: {summary:?}"
    );
    // The contradiction in one assertion: nothing may be listed that no counter
    // accounts for.
    assert!(
        summary.dead_functions + summary.dead_classes + summary.dead_modules > 0,
        "{} dead lines in {} files, and every category counter is 0: {summary:?}",
        summary.total_dead_lines,
        summary.files_with_dead_code
    );
}

/// `--timeout N` printed "⏰ Analysis timeout set to N seconds" and ran to
/// completion: the work was a blocking `Command::output()` inside an `async`
/// block, so neither of the two `tokio::time::timeout`s wrapped around it could
/// fire. Measured at 20.2s under `--timeout 1`, exit 0.
///
/// The 20-second sleep is in a build script, so `cargo check` is deterministically
/// slower than the budget without depending on machine speed.
#[tokio::test(flavor = "multi_thread")]
async fn a_cargo_check_that_outruns_the_budget_is_killed_and_reported() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let root = tmp.path();
    write_crate(
        root,
        "slowcheck_crate",
        "fn dead_one() -> i32 { 1 }\npub fn used() -> i32 { 2 }\n",
        Some("fn main() { std::thread::sleep(std::time::Duration::from_secs(20)); }\n"),
    );

    let started = Instant::now();
    let result =
        run_dead_code_analysis_with_filters(root, filters(0), Duration::from_secs(1)).await;
    let elapsed = started.elapsed();

    let error = result
        .err()
        .unwrap_or_else(|| panic!("--timeout 1 ran the 20s check to completion in {elapsed:?}"));
    assert!(
        error.to_string().contains("timed out after 1 seconds"),
        "unexpected error: {error}"
    );
    // Generous, because the budget only starts once cargo is spawned and the
    // build script has to be compiled first; the point is that it is nowhere
    // near the 20s the check itself takes.
    assert!(
        elapsed < Duration::from_secs(15),
        "the budget was not enforced: {elapsed:?}"
    );
}

/// A lib crate. This is the case the earlier version of these tests could not
/// see: for a LIBRARY, an un-called `pub` item is not dead — the public API is
/// the entry point — and it is the case the reachability analyzer gets wrong.
fn write_lib_crate(root: &std::path::Path, name: &str, lib_rs: &str) {
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"),
    )
    .expect("Cargo.toml");
    std::fs::write(root.join("src/lib.rs"), lib_rs).expect("lib.rs");
    crate::services::cargo_dead_code_analyzer::write_fixture_lockfile(root);
}

/// One dead function and TWO dead types, in a bin crate.
///
/// The dead structs are the half the reachability analyzer cannot represent at
/// all: it has no notion of a dead type, so `dead_classes` could never reach the
/// MCP payload however the two surfaces were compared.
const ONE_DEAD_FUNCTION_TWO_DEAD_TYPES: &str =
    "fn main() {\n    println!(\"{}\", entry(1));\n}\n\n\
     fn entry(n: u64) -> u64 {\n    n + 1\n}\n\n\
     fn never_called_dead_fn() -> u64 {\n    7\n}\n\n\
     struct NeverConstructedA {\n    _x: u64,\n}\n\n\
     struct NeverConstructedB {\n    _y: u64,\n}\n";

/// A library whose public API is used by nobody in the crate, plus real dead
/// code of both kinds behind it.
///
/// `pub fn entry` is LIVE: it is this library's API. `helper` is live because
/// `entry` calls it. Everything else is dead.
const LIB_WITH_A_PUBLIC_API: &str = "pub fn entry(n: u64) -> u64 {\n    helper(n) + 1\n}\n\n\
     fn helper(n: u64) -> u64 {\n    n * 2\n}\n\n\
     fn never_called_one() -> u64 {\n    1\n}\n\n\
     fn never_called_two() -> u64 {\n    2\n}\n\n\
     struct NeverConstructed {\n    _z: u64,\n}\n\n\
     impl NeverConstructed {\n    fn dead_method(&self) -> u64 {\n        self._z\n    }\n}\n";

/// One finding, as `(file, kind, name)`.
///
/// The KIND is part of it on purpose. The version of this test that shipped
/// compared dead FUNCTION names only, on a BIN crate only — the two conditions
/// under which the two analyzers happen to agree — so it passed while the two
/// surfaces disagreed about dead types on every crate and about a library's
/// whole public API. A comparison that cannot see the kinds cannot see the
/// defect. So is the LINE: a payload that names a finding without saying where
/// it is has published half of it.
type Finding = (String, String, String, u32);

fn file_name_of(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// Every finding the CLI report NAMES.
fn cli_findings(report: &crate::models::dead_code::DeadCodeResult) -> Vec<Finding> {
    let mut found: Vec<Finding> = report
        .files
        .iter()
        .flat_map(|f| {
            f.items.iter().map(move |i| {
                (
                    file_name_of(&f.path),
                    serde_json::to_value(i.item_type)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_string))
                        .unwrap_or_else(|| format!("{:?}", i.item_type)),
                    i.name.clone(),
                    i.line,
                )
            })
        })
        .collect();
    found.sort();
    found
}

/// Every finding the MCP payload NAMES, read out of the per-kind lists rather
/// than out of one hardcoded key, so a kind the payload stops publishing shows
/// up here as a missing finding instead of being skipped.
fn mcp_findings(payload: &serde_json::Value) -> Vec<Finding> {
    let kinds = [
        ("dead_functions", "function"),
        ("dead_classes", "class"),
        ("dead_variables", "variable"),
        ("dead_modules", "module"),
        ("unreachable_blocks", "unreachable"),
        ("other", "other"),
    ];
    let mut found: Vec<Finding> = Vec::new();
    for file in payload["results"]["files"]
        .as_array()
        .expect("results.files must be an array")
    {
        let name = file_name_of(file["file"].as_str().expect("file"));
        for (key, kind) in kinds {
            for item in file[key]
                .as_array()
                .unwrap_or_else(|| panic!("results.files[].{key} must be an array: {file}"))
            {
                found.push((
                    name.clone(),
                    kind.to_string(),
                    item["name"].as_str().expect("name").to_string(),
                    u32::try_from(item["line"].as_u64().expect("line")).expect("line fits"),
                ));
            }
        }
    }
    found.sort();
    found
}

/// Run both surfaces over `root` and require them to name the same findings.
///
/// Returns the agreed list so each test can then state what that list has to
/// contain — parity over an empty list is not evidence of anything.
async fn surfaces_must_agree(root: &std::path::Path) -> Vec<Finding> {
    let mcp = crate::mcp_pmcp::tool_functions::analyze_dead_code(&[root.to_path_buf()], false)
        .await
        .expect("MCP dead-code analysis runs");
    assert_eq!(
        mcp["results"]["paths_not_analyzed"]
            .as_array()
            .expect("paths_not_analyzed")
            .len(),
        0,
        "MCP could not analyse the fixture: {mcp}"
    );

    let outcome = run_dead_code_analysis_with_filters(
        root,
        filters(cli_default_min_dead_lines()),
        NO_DEADLINE,
    )
    .await
    .expect("analysis runs");

    let cli = cli_findings(&outcome.report);
    let from_mcp = mcp_findings(&mcp);
    assert_eq!(
        cli, from_mcp,
        "`analyze dead-code` and MCP `analyze_dead_code` disagree on the same path: \
         CLI {cli:?} vs MCP {from_mcp:?} (payload {mcp})"
    );

    // The count that heads the MCP list has to agree with the list, in both
    // directions: `total_dead_code` was the dead FUNCTION count under a name
    // that promises all dead code, so a crate with one dead function and two
    // dead structs answered 1.
    assert_eq!(
        mcp["results"]["total_dead_code"].as_u64(),
        Some(from_mcp.len() as u64),
        "total_dead_code does not count the findings beneath it: {mcp}"
    );
    // …and the CLI summary heads its own list with the same categories.
    let summary = &outcome.report.summary;
    assert_eq!(
        summary.dead_functions,
        cli.iter()
            .filter(|(_, kind, _, _)| kind == "function")
            .count(),
        "the CLI summary contradicts its own item list: {summary:?}"
    );
    assert_eq!(
        summary.dead_classes,
        cli.iter().filter(|(_, kind, _, _)| kind == "class").count(),
        "the CLI summary contradicts its own item list: {summary:?}"
    );

    // The default hid nothing, so there is nothing to declare.
    assert!(
        outcome.scope.omitted.is_empty(),
        "the DEFAULT invocation dropped findings: {:?}",
        outcome.scope.omitted
    );
    assert_eq!(
        outcome.report.summary.files_with_dead_code, outcome.report.files_with_dead_code_found,
        "`files_with_dead_code` and `files_with_dead_code_found` contradict each other \
         in one object with nothing omitted"
    );

    from_mcp
}

fn names_of(findings: &[Finding], kind: &str) -> Vec<String> {
    findings
        .iter()
        .filter(|(_, k, _, _)| k == kind)
        .map(|(_, _, name, _)| name.clone())
        .collect()
}

/// #928 EXTENDED, ACROSS SURFACES. The invariant was "nothing may be listed
/// that no counter accounts for"; this is its mirror image, which shipped:
/// **nothing found may go unaccounted for.**
///
/// On this fixture MCP `analyze_dead_code` answered `{total_dead_code: 1}` while
/// `analyze dead-code` answered `{dead_functions: 1, dead_classes: 2}` — the two
/// machine-readable surfaces of one tool contradicting each other about the same
/// path, because MCP ran a reachability analyzer that has no notion of a dead
/// TYPE at all.
#[tokio::test]
async fn the_two_surfaces_agree_on_a_bin_crate_with_dead_types() {
    let tmp = tempfile::Builder::new()
        .prefix("dcparity-bin")
        .tempdir()
        .expect("tempdir");
    let root = tmp.path();
    write_bin_crate(root, "surface_parity_bin", ONE_DEAD_FUNCTION_TWO_DEAD_TYPES);

    let findings = surfaces_must_agree(root).await;

    assert_eq!(
        names_of(&findings, "function"),
        vec!["never_called_dead_fn".to_string()],
        "found {findings:?}"
    );
    // The half that could not cross to MCP at all.
    assert_eq!(
        names_of(&findings, "class"),
        vec![
            "NeverConstructedA".to_string(),
            "NeverConstructedB".to_string()
        ],
        "found {findings:?}"
    );
}

/// A LIBRARY, which is the case the shipped test could not reach.
///
/// The reachability analyzer calls every un-called item dead, so it named this
/// crate's `pub fn entry` — the library's entire reason to exist — while missing
/// `NeverConstructed`, which is genuinely dead:
///
/// ```text
///   CLI  {never_called_one, never_called_two, dead_method, NeverConstructed}
///   MCP  {entry, never_called_one, never_called_two, dead_method}
/// ```
///
/// Reporting a library's public API as dead code is a false positive of the kind
/// that ends a tool's usefulness, and no disclosure field fixes it.
#[tokio::test]
async fn the_two_surfaces_agree_on_a_lib_crate_and_neither_calls_its_api_dead() {
    let tmp = tempfile::Builder::new()
        .prefix("dcparity-lib")
        .tempdir()
        .expect("tempdir");
    let root = tmp.path();
    write_lib_crate(root, "surface_parity_lib", LIB_WITH_A_PUBLIC_API);

    let findings = surfaces_must_agree(root).await;

    let mut functions = names_of(&findings, "function");
    functions.sort();
    assert_eq!(
        functions,
        vec![
            "dead_method".to_string(),
            "never_called_one".to_string(),
            "never_called_two".to_string()
        ],
        "found {findings:?}"
    );
    assert!(
        !functions.contains(&"entry".to_string()),
        "a library's public API was reported dead: {findings:?}"
    );
    assert!(
        !functions.contains(&"helper".to_string()),
        "a function the public API calls was reported dead: {findings:?}"
    );
    assert_eq!(
        names_of(&findings, "class"),
        vec!["NeverConstructed".to_string()],
        "found {findings:?}"
    );
}

/// Copy every `.rs` file under `from` into `to`, preserving the layout.
fn copy_rust_tree(from: &std::path::Path, to: &std::path::Path) -> usize {
    let mut copied = 0;
    let entries = match std::fs::read_dir(from) {
        Ok(entries) => entries,
        Err(_) => return 0,
    };
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            let child = to.join(&name);
            std::fs::create_dir_all(&child).expect("mkdir");
            copied += copy_rust_tree(&path, &child);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            std::fs::create_dir_all(to).expect("mkdir");
            std::fs::copy(&path, to.join(&name)).expect("copy");
            copied += 1;
        }
    }
    copied
}

/// A REAL directory of this repository — `src/models`, the sharpest case there
/// is — and the finding set MCP `analyze_dead_code` published over it:
///
/// ```text
///   pmat analyze dead-code -p src/models    ->  0
///   MCP  analyze_dead_code {src/models}     -> 50
/// ```
///
/// Fifty. `fmt`, `serialize`, `visit_str`, `deserialize_bool_lenient` — trait
/// impls the compiler can see being called, and `#[serde(default = "…")]`
/// helpers the derive macro calls — all reported dead, because the reachability
/// analyzer calls every un-called item dead and a library's API is un-called by
/// construction. That is the false positive this fix is about, and it is not
/// something a disclosure field could have made true.
///
/// The control is that same analyzer, run here over the same content, so this
/// test cannot pass because the fixture went blunt: if the 50 ever stop being
/// producible, the control says so rather than leaving an empty comparison
/// looking like a pass.
///
/// The in-repo control and the copy do NOT report the same set, and the
/// difference is the point: `src/models` is inside this crate, so the analyzer
/// walks up to the manifest and treats the crate's exports as roots; the copy
/// has no crate above it and says so. The copy is therefore the superset, and it
/// is the set the MCP tool is held to.
///
/// The analysis runs over a COPY, in a temp dir, and that is deliberate: rooting
/// it at the real `src/models` makes `cargo check` compile this entire crate —
/// measured at 4m13s here — inside a unit test of that crate. The copy carries
/// the same 89 real files and the same names; what it does not carry is a
/// manifest, so the run ends in a stated failure instead of a four-minute one.
/// Both halves of the contract are then checked at once: the tool does not
/// republish a single one of the reachability analyzer's names, and it does not
/// answer with a silent zero either — it says which path it could not analyse
/// and why, with the same error the CLI fails with.
#[tokio::test]
async fn the_mcp_tool_does_not_republish_reachability_findings_over_a_real_repo_directory() {
    use crate::services::dead_code_multi_language::analyze_dead_code_multi_language;

    let models = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/models");
    assert!(models.is_dir(), "{} is not a directory", models.display());

    // CONTROL, over the real directory.
    let control = analyze_dead_code_multi_language(&models).expect("reachability analyzer runs");
    let control_names: std::collections::BTreeSet<String> = control
        .dead_functions
        .iter()
        .map(|f| f.name.clone())
        .collect();
    assert!(
        control_names.len() >= 25,
        "the control stopped being the sharp case: {} name(s) over {}",
        control_names.len(),
        models.display()
    );
    // Two of them, spelled out: a `Display::fmt` and a `Serialize::serialize`.
    // Neither can be dead — the compiler reports neither — and both were in the
    // 50.
    for live in ["fmt", "serialize"] {
        assert!(
            control_names.contains(live),
            "control no longer names `{live}`, so this test would no longer be about \
             live API being called dead: {control_names:?}"
        );
    }

    let tmp = tempfile::Builder::new()
        .prefix("dcparity-models")
        .tempdir()
        .expect("tempdir");
    let copy = tmp.path();
    let copied = copy_rust_tree(&models, copy);
    assert!(
        copied >= 25,
        "expected a real tree, copied {copied} file(s)"
    );

    // The copy carries the same tree, minus the one thing a copy cannot carry:
    // the crate above it. `src/models` is part of THIS crate, so the analyzer
    // walks up to the manifest, finds a library target and seeds the crate's
    // exports as roots — eight `pub fn`s of `src/models` are rescued that way,
    // which is the same false positive #1013 removed from the cargo path. The
    // copy has no manifest above it, says so (`library_target: undetermined`)
    // and seeds nothing, so it reports everything the in-repo control reports
    // AND the exports the enclosing crate rescued.
    //
    // That superset is the sharper fixture of the two, and it is the set the MCP
    // tool is checked against below — stricter than the control, not looser.
    let copy_control =
        analyze_dead_code_multi_language(copy).expect("reachability analyzer runs on the copy");
    let copy_names: std::collections::BTreeSet<String> = copy_control
        .dead_functions
        .iter()
        .map(|f| f.name.clone())
        .collect();
    assert!(
        copy_names.is_superset(&control_names),
        "the copy is not the same tree as {}: it lost {:?}",
        models.display(),
        control_names.difference(&copy_names).collect::<Vec<_>>()
    );
    for live in ["fmt", "serialize"] {
        assert!(
            copy_names.contains(live),
            "the copy no longer names `{live}`, so this test would no longer be about \
             live API being called dead: {copy_names:?}"
        );
    }

    let mcp = crate::mcp_pmcp::tool_functions::analyze_dead_code(&[copy.to_path_buf()], false)
        .await
        .expect("MCP dead-code analysis returns a payload");
    let reported: std::collections::BTreeSet<String> = mcp_findings(&mcp)
        .into_iter()
        .map(|(_, _, name, _)| name)
        .collect();
    let repeated: Vec<&String> = reported.intersection(&copy_names).collect();
    assert!(
        repeated.is_empty(),
        "MCP `analyze_dead_code` republished {} of the reachability analyzer's findings over \
         a real library subtree of this repo, including {repeated:?}; \
         `pmat analyze dead-code -p src/models` reports none of them",
        repeated.len()
    );

    // And the zero is not a silent one. The CLI fails on this path; the payload
    // has to name the path and say the same thing, or a client cannot tell "no
    // dead code" from "not analysed".
    let cli = run_dead_code_analysis_with_filters(copy, filters(0), NO_DEADLINE).await;
    let cli_error = cli
        .err()
        .map(|e| e.to_string())
        .expect("a directory with no manifest cannot be cargo-checked");
    let not_analyzed = mcp["results"]["paths_not_analyzed"]
        .as_array()
        .expect("paths_not_analyzed must be present");
    assert_eq!(
        not_analyzed.len(),
        1,
        "the CLI failed with `{cli_error}` and the payload declares nothing: {mcp}"
    );
    assert_eq!(
        not_analyzed[0]["path"].as_str(),
        Some(copy.display().to_string().as_str()),
        "the declared path is not the requested one: {mcp}"
    );
    let reason = not_analyzed[0]["reason"].as_str().expect("reason");
    // WORD FOR WORD, not a substring of it. This used to pin the literal
    // "Cargo check failed" — cargo's own message, leaked through — so the day
    // the CLI started refusing a manifest-less tree in its own words the two
    // surfaces could have drifted apart with the test still green. What the
    // contract is about is that a client reading the payload learns exactly
    // what an operator reading the CLI learns.
    assert_eq!(
        reason, cli_error,
        "the payload's reason is not the error the CLI fails with"
    );
    assert!(
        reason.contains("no dead-code measurement was taken"),
        "the reason must say that nothing was measured, so a client cannot read \
         `total_dead_code: 0` as a clean bill of health: {reason}"
    );
    assert_eq!(
        mcp["results"]["total_dead_code"].as_u64(),
        Some(0),
        "findings over a path that was not analysed: {mcp}"
    );
}

/// The residual case, and the invariant that now covers it: when a threshold IS
/// asked for, what it removes must still be counted, in the units the summary
/// counts in, in the same object.
///
/// `files_with_dead_code_found` was the whole disclosure the JSON carried — a
/// bare file count, so `dead_functions: 0` over three cut dead functions was
/// unfalsifiable from the payload. The text renderer had said it all along;
/// JSON, the surface agents and CI read, had not.
#[tokio::test]
async fn a_threshold_that_hides_a_file_still_counts_what_it_hid() {
    let tmp = tempfile::Builder::new()
        .prefix("dcomit")
        .tempdir()
        .expect("tempdir");
    let root = tmp.path();
    write_bin_crate(root, "omission_crate", ONE_DEAD_FUNCTION);

    // 100 estimated dead lines in an 11-line file: nothing can clear it, so the
    // list is empty and every summary counter is zero.
    let outcome = run_dead_code_analysis_with_filters(root, filters(100), NO_DEADLINE)
        .await
        .expect("analysis runs");

    assert!(
        outcome.report.files.is_empty(),
        "the threshold was supposed to empty the list: {:?}",
        outcome.report.files
    );
    assert_eq!(outcome.report.summary.dead_functions, 0);

    let rendered = format_dead_code_result(
        &outcome.report,
        &crate::cli::DeadCodeOutputFormat::Json,
        outcome.scope,
    )
    .expect("json renders");
    let json: serde_json::Value = serde_json::from_str(&rendered).expect("json parses");

    let summary_files = json["summary"]["files_with_dead_code"]
        .as_u64()
        .expect("files_with_dead_code");
    let found_files = json["files_with_dead_code_found"]
        .as_u64()
        .expect("files_with_dead_code_found");
    let omitted = &json["omitted"];
    assert!(
        !omitted.is_null(),
        "the payload says {found_files} file(s) were found with dead code and \
         {summary_files} have it, and accounts for the difference nowhere: {rendered}"
    );
    assert_eq!(
        omitted["dead_functions"].as_u64(),
        Some(1),
        "one dead function was cut and no counter says so: {rendered}"
    );
    assert_eq!(
        summary_files + omitted["files"].as_u64().expect("omitted.files"),
        found_files,
        "listed + omitted must account for every file found: {rendered}"
    );
    assert_eq!(
        json["summary"]["total_dead_lines"].as_u64().expect("lines")
            + omitted["dead_lines"].as_u64().expect("omitted.dead_lines"),
        5,
        "the 5 estimated dead lines of the cut function are in neither figure: {rendered}"
    );
    let reasons = omitted["reasons"].as_array().expect("reasons");
    assert!(
        reasons.iter().any(|r| r == "below --min-dead-lines"),
        "the omission names no knob to turn: {rendered}"
    );
    assert!(
        !reasons.iter().any(|r| r == "beyond --top-files"),
        "a filter that did not run must not be blamed: {rendered}"
    );
}

/// #928 RESIDUAL. Every `DeadCodeKind` the parser can produce must reach a
/// `DeadCodeType` that NAMES it. `Module` and the unclassified `Other` both used
/// to land on `Variable`, so a record could read
/// `"item_type": "variable"` beside `"reason": "module `x` is never used"` —
/// the report contradicting itself inside one object.
///
/// This is a pure mapping test on purpose: rustc emits the `module` wording
/// rarely enough that a fixture cannot be relied on to produce one, while the
/// parser accepts it unconditionally (`("module `", "` is never used", …)`).
#[test]
fn every_dead_code_kind_maps_to_a_type_that_names_it() {
    use crate::models::dead_code::DeadCodeType;
    use crate::services::cargo_dead_code_analyzer::{DeadCodeKind, DeadItem};

    let item = |kind: DeadCodeKind, message: &str| DeadItem {
        name: "x".to_string(),
        kind,
        line: 1,
        column: 1,
        message: message.to_string(),
    };

    let cases = [
        (
            item(DeadCodeKind::Module, "module `x` is never used"),
            DeadCodeType::Module,
        ),
        (
            item(
                DeadCodeKind::Other("union".to_string()),
                "union `x` is never used",
            ),
            DeadCodeType::Other,
        ),
        (
            item(DeadCodeKind::Constant, "constant `x` is never used"),
            DeadCodeType::Variable,
        ),
        (
            item(DeadCodeKind::Function, "function `x` is never used"),
            DeadCodeType::Function,
        ),
    ];

    for (dead_item, expected) in cases {
        let reason = dead_item.message.clone();
        let reported = super::dead_items_to_report_items(std::slice::from_ref(&dead_item));
        assert_eq!(
            reported[0].item_type, expected,
            "`{reason}` must not be reported as {:?}",
            reported[0].item_type
        );
    }
}

/// What `--include-tests` means where there is no cargo to ask.
///
/// Moved here from `mcp_pmcp::tool_functions::analysis_tools`, which applied
/// this predicate to the reachability analyzer's output on its own: the flag
/// therefore worked over MCP and was inert on the CLI for every non-Rust
/// project, because `filters.include_tests` reached
/// `run_multi_language_dead_code` and was never read. One predicate, inside the
/// analysis, so both surfaces get the same answer.
#[test]
fn is_test_path_recognises_the_usual_layouts() {
    assert!(super::is_test_path("tests/integration.rs"));
    assert!(super::is_test_path("crate/tests/helpers/mod.rs"));
    assert!(super::is_test_path("src/foo_tests.rs"));
    assert!(super::is_test_path("src/test_foo.py"));
    assert!(!super::is_test_path("src/lib.rs"));
    assert!(!super::is_test_path("src/latest.rs"));
}

/// A path naming ONE FILE.
///
/// `analyze_dead_code`'s schema takes "at least one file or directory", but the
/// analysis is rooted at a directory — cargo runs in one — so a file is answered
/// by analysing its directory and listing only that file. Both halves have to be
/// right, and the first one was not: cargo's rows are relative to the CRATE
/// root, not to the directory the analysis was pointed at, so joining them to
/// the analysis root produced `…/fx5/src/src/main.rs`, which matched nothing.
/// The tool then reported `files_listed: 0` for a file holding three findings
/// and filed all three under "outside the requested path".
#[tokio::test]
async fn a_file_path_reports_that_files_findings_and_says_where_it_looked() {
    let tmp = tempfile::Builder::new()
        .prefix("dcparity-file")
        .tempdir()
        .expect("tempdir");
    let root = tmp.path();
    write_bin_crate(
        root,
        "surface_parity_file",
        ONE_DEAD_FUNCTION_TWO_DEAD_TYPES,
    );
    let main_rs = root.join("src/main.rs");

    let mcp =
        crate::mcp_pmcp::tool_functions::analyze_dead_code(std::slice::from_ref(&main_rs), false)
            .await
            .expect("MCP dead-code analysis runs");

    let findings = mcp_findings(&mcp);
    assert_eq!(
        findings.len(),
        3,
        "the requested file holds all three findings: {mcp}"
    );
    assert_eq!(
        mcp["results"]["files"][0]["file"].as_str(),
        Some(main_rs.display().to_string().as_str()),
        "the listed path must be the file that was asked about: {mcp}"
    );

    // …and the widening is declared, with nothing left over: the whole
    // directory's findings are the file's, so nothing was restricted away.
    let scope = &mcp["results"]["paths"][0];
    assert_eq!(
        scope["analysis_root"].as_str(),
        Some(root.join("src").display().to_string().as_str()),
        "the payload must say which directory was analysed: {mcp}"
    );
    assert_eq!(scope["files_listed"].as_u64(), Some(1), "{mcp}");
    assert_eq!(
        scope["findings_outside_requested_path"]["dead_functions"].as_u64(),
        Some(0),
        "findings inside the requested file were filed as outside it: {mcp}"
    );
    assert_eq!(
        scope["findings_outside_requested_path"]["dead_classes"].as_u64(),
        Some(0),
        "findings inside the requested file were filed as outside it: {mcp}"
    );
}

/// The shared runner's defaults ARE the CLI's defaults.
///
/// [`super::run_dead_code_suite`] is what makes one name mean one analyzer, and
/// it is only that if it runs the analysis at the same settings `pmat analyze
/// dead-code` runs it at. A constant that merely looks like a clap default
/// cannot fail when the default moves — so the defaults are asked of clap, the
/// same way `cli_default_min_dead_lines` above asks for one.
#[test]
fn the_suite_defaults_are_the_cli_defaults() {
    crate::cli::commands::on_big_stack(|| {
        use clap::Parser;
        let cli = crate::cli::Cli::try_parse_from(["pmat", "analyze", "dead-code"])
            .expect("`pmat analyze dead-code` parses");
        let crate::cli::Commands::Analyze(crate::cli::commands::AnalyzeCommands::DeadCode {
            min_dead_lines,
            max_depth,
            timeout,
            include_unreachable,
            top_files,
            include,
            exclude,
            ..
        }) = cli.command
        else {
            panic!("`pmat analyze dead-code` must parse as Analyze/DeadCode");
        };

        assert_eq!(min_dead_lines, super::DEAD_CODE_DEFAULT_MIN_DEAD_LINES);
        assert_eq!(max_depth, super::DEAD_CODE_DEFAULT_MAX_DEPTH);
        assert_eq!(timeout, super::DEAD_CODE_DEFAULT_TIMEOUT_SECS);
        // The three the suite hardcodes rather than names, because each one is
        // the value that hides nothing: no unreachable blocks unless asked for,
        // no cap on the list, no path filter.
        assert!(!include_unreachable);
        assert_eq!(top_files, None);
        assert!(include.is_empty());
        assert!(exclude.is_empty());
    });
}

/// Issue #1058 — the transport-parity gate's dead-code finding.
///
/// Not a population disagreement: the CLI and the MCP tool measured copia
/// identically at 29 analysed files. They published that number under
/// DIFFERENT NAMES, and the CLI additionally published a second, larger one:
///
/// ```text
///   CLI  total_files 38   analyzed_files 29     (no `files_analyzed`)
///   MCP  files_analyzed 29                      (no discovered count at all)
/// ```
///
/// A checker asking each transport for "dead-code files" was answered 38 and
/// 29, and both were right about their own field. A key-name split reads as a
/// measurement disagreement, which is worse than either number being wrong,
/// because each surface looks correct alone.
///
/// RED CONTROL: removing the two `object.insert` calls from
/// `format_dead_code_as_json_scoped` fails the first assertion here.
#[test]
fn the_json_report_publishes_both_counts_under_the_canonical_names() {
    let report = crate::models::dead_code::DeadCodeResult {
        summary: crate::models::dead_code::DeadCodeSummary {
            total_files_analyzed: 29,
            files_with_dead_code: 0,
            total_dead_lines: 0,
            dead_percentage: 0.0,
            dead_functions: 0,
            dead_classes: 0,
            dead_modules: 0,
            unreachable_blocks: 0,
        },
        files: vec![],
        total_files: 38,
        analyzed_files: 29,
        files_with_dead_code_found: 0,
        files_truncated: false,
        library_target: None,
        compiler_scan: None,
    };

    let rendered = format_dead_code_result(
        &report,
        &crate::cli::DeadCodeOutputFormat::Json,
        super::DeadCodeReportScope::default(),
    )
    .expect("json renders");
    let json: serde_json::Value = serde_json::from_str(&rendered).expect("json parses");

    assert_eq!(
        json["files_analyzed"].as_u64(),
        Some(29),
        "the name the MCP payload uses must exist here: {rendered}"
    );
    assert_eq!(
        json["files_discovered"].as_u64(),
        Some(38),
        "…and the discovered count under the name `analyze complexity` uses: {rendered}"
    );

    // COUNTER-TEST: two names for one number, never two numbers, and never a
    // number invented to fill a key. The legacy spellings must still be there
    // and must still agree with the new ones — a fix that made the two counts
    // equal would "pass parity" by destroying the denominator.
    assert_eq!(json["files_analyzed"], json["analyzed_files"]);
    assert_eq!(json["files_discovered"], json["total_files"]);
    assert_ne!(
        json["files_analyzed"], json["files_discovered"],
        "38 discovered and 29 analysed are different facts and must stay different"
    );
}
