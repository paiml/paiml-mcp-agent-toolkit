//! A flag whose own `--help` says it does nothing must SAY so at runtime.
//!
//! Three flags in this CLI are accepted and deliberately inert. Two of them
//! already print a named `*_NOOP_NOTE` const on stderr when passed —
//! `analyze big-o --analyze-space` and `analyze assembly-script --wasm-complexity` —
//! and neither carries an `ALLOWED_NOOPS` entry in the flag-efficacy harness,
//! because the disclosure makes the flag observable on its own bare probe. That
//! is the house convention.
//!
//! `quality-gate --fail-on-violation` was the one that missed it: parsed,
//! discarded into `_` on BOTH dispatch routes, and accepted in silence. A user
//! who asks for something deserves to be told the request was redundant rather
//! than honoured.
//!
//! This file is the poka-yoke for the NEXT one. Adding a flag whose help
//! declares it inert now fails this test until it is registered here, so the
//! convention is enforced by the compiler-and-test loop rather than by whoever
//! remembers it. That matters because "everybody knows to do X" is exactly how
//! 49 no-op flags shipped in 3.29.0.
//!
//! Maintenance is O(1) in the number of commands: the check is driven by the
//! doc comment the author already writes to declare the flag inert.

use super::on_big_stack;
use clap::CommandFactory;

/// Flags that are accepted and do nothing, each with the runtime disclosure
/// that makes them observable.
///
/// `(command path, clap arg id, the substring the note must carry)`. The
/// substring is the flag's own long name, so a note that discloses the wrong
/// flag fails.
const INERT_FLAG_DISCLOSURES: &[(&[&str], &str, &str)] = &[
    (&["analyze", "big-o"], "analyze_space", "--analyze-space"),
    // NOT `analyze complexity` — this test caught that on its first run. The
    // flag lives on `analyze assembly-script`, and a registry entry naming the
    // wrong command is exactly the stale claim the second test exists to reject.
    (
        &["analyze", "assembly-script"],
        "wasm_complexity",
        "--wasm-complexity",
    ),
    (
        &["quality-gate"],
        "fail_on_violation",
        "--fail-on-violation",
    ),
];

/// The vocabulary an author uses to declare a flag inert in its help text.
///
/// Matched case-insensitively. If you are adding a flag that does nothing and
/// none of these words appears in its help, the flag is undocumented as well as
/// inert — write the help first.
const INERT_VOCABULARY: &[&str] = &[
    "no-op",
    "noop",
    "has no effect",
    "accepted for compatibility",
];

/// Every `(path, arg id, help text)` in the whole clap tree.
fn every_flag() -> Vec<(Vec<String>, String, String)> {
    on_big_stack(|| {
        let root = <crate::cli::Cli as CommandFactory>::command();
        let mut out = Vec::new();
        let mut stack = vec![(Vec::<String>::new(), root)];
        while let Some((path, cmd)) = stack.pop() {
            for arg in cmd.get_arguments() {
                let help = arg
                    .get_long_help()
                    .or_else(|| arg.get_help())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                out.push((path.clone(), arg.get_id().to_string(), help));
            }
            for sub in cmd.get_subcommands() {
                let mut p = path.clone();
                p.push(sub.get_name().to_string());
                stack.push((p, sub.clone()));
            }
        }
        out
    })
}

fn declares_itself_inert(help: &str) -> bool {
    let lower = help.to_lowercase();
    INERT_VOCABULARY.iter().any(|v| lower.contains(v))
}

fn registered(path: &[String], id: &str) -> bool {
    INERT_FLAG_DISCLOSURES.iter().any(|(p, a, _)| {
        *a == id && p.len() == path.len() && p.iter().zip(path).all(|(x, y)| *x == y.as_str())
    })
}

/// A flag that declares itself inert must be registered here.
///
/// This is the half that catches the NEXT one. Without it, the fourth inert
/// flag ships accepted-in-silence and nothing notices until a sweep months
/// later calls it a no-op.
#[test]
fn every_flag_that_declares_itself_inert_is_registered() {
    let mut unregistered = Vec::new();
    for (path, id, help) in every_flag() {
        if declares_itself_inert(&help) && !registered(&path, &id) {
            unregistered.push(format!(
                "pmat {} --{}  (help says: {:?})",
                path.join(" "),
                id.replace('_', "-"),
                help.chars().take(90).collect::<String>()
            ));
        }
    }
    assert!(
        unregistered.is_empty(),
        "these flags declare themselves inert in --help but are not registered in \
         INERT_FLAG_DISCLOSURES, so nothing checks that they disclose it at runtime:\n  {}\n\
         Add a `*_NOOP_NOTE` const, print it when the flag is passed, and register it here.",
        unregistered.join("\n  ")
    );
}

/// ...and a registered flag must still declare itself inert.
///
/// The other half. Without it an entry outlives the reason it was added — the
/// flag-efficacy harness carries a tombstone for exactly that: "an allow-list
/// entry that suppresses nothing is how an exception outlives its reason".
#[test]
fn every_registered_flag_still_declares_itself_inert() {
    for (path, id, _) in INERT_FLAG_DISCLOSURES {
        let owned: Vec<String> = path.iter().map(|s| (*s).to_string()).collect();
        let found = every_flag()
            .into_iter()
            .find(|(p, a, _)| p == &owned && a == id);
        let (_, _, help) = found.unwrap_or_else(|| {
            panic!(
                "registered flag `--{}` on `pmat {}` does not exist any more — \
                 remove the entry",
                id.replace('_', "-"),
                path.join(" ")
            )
        });
        assert!(
            declares_itself_inert(&help),
            "`--{}` on `pmat {}` is registered as inert but its help no longer says so. \
             Either it does something now (delete the entry and the note) or the help \
             regressed. Help: {help:?}",
            id.replace('_', "-"),
            path.join(" ")
        );
    }
}

/// The three notes name their own flag.
///
/// A disclosure that names the wrong flag is worse than none: it sends the
/// reader to the wrong place.
#[test]
fn each_disclosure_names_its_own_flag() {
    use crate::cli::analysis_utilities::FAIL_ON_VIOLATION_NOOP_NOTE;
    assert!(FAIL_ON_VIOLATION_NOOP_NOTE.contains("--fail-on-violation"));
    assert!(FAIL_ON_VIOLATION_NOOP_NOTE.contains("no-op"));
    assert!(
        FAIL_ON_VIOLATION_NOOP_NOTE.contains("--report-only"),
        "the note must name what to pass INSTEAD, or it tells the user their flag is \
         useless without telling them what to do"
    );
    // The registry's third column is the substring each note must carry; assert
    // the one we can reach from here matches it.
    let entry = INERT_FLAG_DISCLOSURES
        .iter()
        .find(|(_, id, _)| *id == "fail_on_violation")
        .expect("fail_on_violation is registered");
    assert!(FAIL_ON_VIOLATION_NOOP_NOTE.contains(entry.2));
}
