//! No catch-all arm may stand between a `Commands`/`AnalyzeCommands` variant
//! and the code that handles it.
//!
//! Both dispatch chains in this crate were written as *two* matches: an
//! exhaustive one that sorts variants into families, and a second one per
//! family that ends in `_ => unreachable!(..)`. Only the first is checked by
//! the compiler. Adding a variant to a family therefore compiles and panics at
//! run time, which is exactly how `Reachability`, `HardcodedPaths` and
//! `VacuousTests` shipped categorised-but-unnamed, and how `Mcp` and `Agy`
//! did before them.
//!
//! The repair is structural — one total match per decision — and the compiler
//! enforces it from then on. This test exists because the compiler cannot
//! enforce it *everywhere at once*:
//!
//! * The command-name table used to sit behind `#[cfg(feature =
//!   "unified-protocol")]`, in neither `default` nor `full`, so no build the
//!   project ships ever type-checked it and its totality had to be pinned by
//!   reading the source. It now lives in `src/cli/command_wire_names.rs`,
//!   which every build compiles, so the compiler enforces that half directly;
//!   the text check is kept as a second line of defence against someone
//!   silencing the compiler with a `_` arm.
//! * In this file the top-level `dispatch_analyze_command` is exhaustive, so a
//!   *new* variant is a compile error; but moving an *existing* variant into a
//!   family whose router does not handle it still compiles and still panics.
//!
//! Scope note: only these two files are checked, and only wildcard arms over
//! the command enums. `let ... else { unreachable!(..) }` narrowings in the
//! leaf routers are left alone — they match exactly one variant, so a new
//! variant cannot reach them. The `_ => Err(UnsupportedProtocol(..))` arms in
//! `unified_protocol/adapters/cli/dispatch_methods*.rs` are a softer instance
//! of the same problem (a new variant is silently rejected rather than
//! panicking) and are deliberately out of scope here.

/// This file, as compiled.
const DISPATCH_SRC: &str = include_str!("mod.rs");

/// The command-name table. Compiled by every build since it moved out of the
/// unified-protocol adapter, but still read here so that a `_` arm added to
/// silence the compiler is caught by name.
const CLI_INPUT_SRC: &str = include_str!("../../command_wire_names.rs");

/// Wildcard match arms whose enclosing `match` is over `Commands` /
/// `AnalyzeCommands`. Returns `(line number, arm text)`.
///
/// Deliberately narrow: a `_ =>` over some other enum (an output format, a
/// severity) is ordinary Rust and must not be reported, or the check would be
/// a blanket ban that everyone learns to route around.
fn wildcard_arms_over_command_enums(src: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = src.lines().collect();
    let mut found = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let arm = line.trim_start();
        if !arm.starts_with("_ =>") {
            continue;
        }
        let arm_indent = line.len() - arm.len();
        // rustfmt puts the `match` head exactly one level out from its arms,
        // so the nearest earlier line that is less indented is that head.
        let head = lines[..i].iter().rposition(|l| {
            let t = l.trim_start();
            !t.is_empty() && (l.len() - t.len()) < arm_indent
        });
        let Some(head) = head else { continue };
        if !lines[head].contains("match ") {
            continue;
        }
        let body = lines[head..i].join("\n");
        if body.contains("AnalyzeCommands::") || body.contains("Commands::") {
            found.push((i + 1, arm.to_string()));
        }
    }
    found
}

/// Reality check on the detector itself, so a green run of the two tests below
/// cannot mean "the check looked at nothing". Green before the fix and after.
#[test]
fn the_detector_finds_a_command_wildcard_and_ignores_other_enums() {
    let over_commands = "\
async fn route(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Churn { .. } => churn(cmd).await,
        _ => unreachable!(\"Expected core analysis command\"),
    }
}
";
    let hits = wildcard_arms_over_command_enums(over_commands);
    assert_eq!(
        hits.len(),
        1,
        "a wildcard over AnalyzeCommands must be reported, got {hits:?}"
    );
    assert_eq!(hits[0].0, 4, "reported line number, got {hits:?}");

    let over_some_other_enum = "\
fn label(f: OutputFormat) -> &'static str {
    match f {
        OutputFormat::Json => \"json\",
        _ => \"text\",
    }
}
";
    assert!(
        wildcard_arms_over_command_enums(over_some_other_enum).is_empty(),
        "an ordinary wildcard over an unrelated enum must not be reported"
    );

    let nested_inside_a_command_arm = "\
async fn route(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Cluster { method, .. } => {
            let m = match method {
                ClusterMethod::Kmeans => \"kmeans\",
                _ => \"other\",
            };
            cluster(m).await
        }
    }
}
";
    assert!(
        wildcard_arms_over_command_enums(nested_inside_a_command_arm).is_empty(),
        "a wildcard over a nested, unrelated enum must not be reported"
    );
}

/// Reality check on the fixtures: if either file stopped containing its
/// dispatcher the two tests below would pass vacuously.
#[test]
fn both_dispatch_files_still_contain_their_dispatchers() {
    assert!(
        DISPATCH_SRC.contains("async fn dispatch_analyze_command"),
        "analysis_handlers/mod.rs must still hold the analyze dispatcher"
    );
    assert!(
        DISPATCH_SRC.contains("AnalyzeCommands::Wasm"),
        "the analyze dispatcher must still route every variant by name"
    );
    assert!(
        CLI_INPUT_SRC.contains("Commands::Generate"),
        "command_wire_names.rs must still name top-level commands"
    );
    assert!(
        CLI_INPUT_SRC.contains("AnalyzeCommands::Churn"),
        "command_wire_names.rs must still name analyze subcommands"
    );
}

/// The shipped dispatcher: six family routers each ended in
/// `_ => unreachable!("Expected <family> analysis command")`.
#[test]
fn the_analyze_dispatcher_has_no_catch_all_arm() {
    let hits = wildcard_arms_over_command_enums(DISPATCH_SRC);
    assert!(
        hits.is_empty(),
        "src/cli/handlers/analysis_handlers/mod.rs must dispatch every \
         AnalyzeCommands variant by name so that adding one is a compile \
         error; catch-all arms found at {hits:?}"
    );
}

/// The command-name table. The compiler now checks its exhaustiveness on every
/// build; this catches the other way out, a `_` arm added to silence it.
#[test]
fn the_command_name_classifier_has_no_catch_all_arm() {
    let hits = wildcard_arms_over_command_enums(CLI_INPUT_SRC);
    assert!(
        hits.is_empty(),
        "src/cli/command_wire_names.rs must name every Commands variant by \
         name so that adding one is a compile error; catch-all arms found at \
         {hits:?}"
    );
    // Comment lines are excluded: the fix's doc comment quotes the arms it
    // replaced, and pinning prose is not the point.
    let code_only: String = CLI_INPUT_SRC
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code_only.contains("unreachable!"),
        "the classifier reaches every variant; no unreachable!() belongs in \
         its code"
    );
}
