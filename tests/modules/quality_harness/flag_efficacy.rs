//! Harness B — the flag-efficacy sweep.
//!
//! Premise: a flag that the parser accepts must change something a caller can
//! observe. If `cmd --flag` and `cmd` produce byte-identical exit code, stdout
//! and stderr on a corpus built to trigger the flag, the flag is decoration.
//!
//! This is the single largest defect class in the 3.29.0 artifact: **49 flags
//! parsed and changed nothing**. Every one was reachable from `--help`, so
//! every one was findable by walking the help tree — which is what this does.
//!
//! Enum-valued options are checked the same way, by pitting two legal values
//! against each other: `--format json` must differ from `--format summary`.
//!
//! ```text
//! cargo test --test all -- --ignored flag_efficacy --nocapture   # workspace build
//! PMAT_BIN=$(which pmat) cargo test ... # the artifact users install
//! PMAT_FLAG_SWEEP=all cargo test ...    # the whole tree, for a release gate
//! ```

use super::{build_corpus, help_for, parse_help, run, CorpusSize, HelpFlag, DEFAULT_TIMEOUT};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

/// Command roots that mutate state, block, or take minutes.
///
/// Excluded because the harness must be safe to run anywhere and finish, not
/// because they are exempt from the invariant. Each entry names why.
const DENY_ROOTS: &[(&str, &str)] = &[
    ("agent", "spawns a background daemon"),
    ("serve", "binds a port and blocks"),
    ("oracle", "auto-fixes and commits"),
    ("kaizen", "auto-fixes, commits, files issues"),
    ("refactor", "rewrites source in place"),
    ("hooks", "installs git hooks into the repo"),
    ("maintain", "deletes files"),
    ("scaffold", "writes a project tree"),
    ("generate", "writes template output"),
    ("config", "persists user configuration"),
    ("record-metric", "writes to the metrics store"),
    ("cache", "mutates the shared cache"),
    ("memory", "mutates process/system state"),
    ("telemetry", "mutates the telemetry store"),
    ("embed", "builds an embedding index (minutes)"),
    ("ci-local", "runs the full CI pipeline (minutes)"),
    ("test-discovery", "edits test files"),
    ("work", "calls the GitHub API"),
    ("qa-work", "calls the GitHub API"),
    ("debug", "attaches to a live trace session"),
    ("sql", "arbitrary SQL against the index"),
    ("stack", "reaches into sibling repositories"),
    ("demo", "not available in the default build"),
    ("org", "not available in the default build"),
];

/// Flags that mutate or block regardless of which command carries them.
const DENY_FLAGS: &[&str] = &[
    "--fix",
    "--force",
    "--write",
    "--install",
    "--uninstall",
    "--yes",
    "--delete",
    "--clean",
    "--watch",
    "--daemon",
    "--serve",
    "--interactive",
    "--auto-commit",
    "--commit",
    "--apply",
];

/// The fast subset, run by default so this is affordable on every commit.
///
/// `PMAT_FLAG_SWEEP=all` walks everything else too. The roots omitted here are
/// printed in the report — a bounded sweep that does not say what it bounded
/// reads as full coverage, which is how 49 no-op flags stayed invisible.
const CORE_ROOTS: &[&str] = &[
    "analyze",
    "tdg",
    "score",
    "quality-gate",
    "enforce",
    "query",
    "context",
    "comply",
    "repo-score",
    "explain",
    "extract",
    "split",
    "deps-audit",
    "project-diag",
];

/// Flags proven to be legitimately observable-free, each with a reason.
///
/// This list is the only way a no-op flag passes, and every entry is a claim
/// someone has to defend in review. It starts empty on purpose.
const ALLOWED_NOOPS: &[(&str, &str, &str)] = &[
    // (command path, flag, why it legitimately changes nothing)
];

/// Two probe values for an option clap did not enumerate, chosen from the
/// option's name. Returns `None` when the domain is unguessable (paths,
/// patterns, free text) rather than guessing and reporting a bogus verdict.
fn numeric_probe_values(long: &str) -> Option<(&'static str, &'static str)> {
    let n = long.trim_start_matches('-').to_lowercase();
    if n.contains("threshold") || n.contains("ratio") || n.contains("percentage") {
        return Some(("0.1", "0.9"));
    }
    const COUNTS: &[&str] = &[
        "top",
        "limit",
        "depth",
        "count",
        "lines",
        "iterations",
        "cases",
        "jobs",
        "workers",
        "parallel",
    ];
    // The two values must *straddle* the corpus, not merely differ. Probing
    // `--min-dead-lines` with 1 and 5 reported a no-op because every dead
    // region in the fixture is ~13 lines, so both values admitted everything
    // and the output was identical.
    if COUNTS.iter().any(|k| n.contains(k)) {
        return Some(("1", "50"));
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Verdict {
    /// The flag changed the observable output.
    Effective,
    /// The flag parsed and changed nothing. A defect.
    NoOp,
    /// The flag is unimplemented and says so, refusing rather than pretending.
    ///
    /// This satisfies the invariant and is the *desired* outcome for a feature
    /// that does not exist yet: the whole point of the gate is that a flag must
    /// not silently do nothing, and exiting non-zero with "--ml is not
    /// implemented ... this flag would relabel them without changing them" is
    /// the honest alternative. Classifying it as a failure penalised precisely
    /// the fix that earlier rounds landed.
    Refuses,
    /// The flag turned a working command into a failing one.
    Errors { code: Option<i32> },
    /// Not checkable; the reason is reported.
    Skipped(String),
}

struct Finding {
    path: String,
    flag: String,
    verdict: Verdict,
}

/// Walk the help tree from `root`, depth-first, collecting leaf command paths.
fn leaf_commands(root: &str, cwd: &Path, out: &mut Vec<Vec<String>>, depth: usize) {
    if depth > 3 {
        return;
    }
    let path_owned = vec![root.to_string()];
    walk(path_owned, cwd, out, depth);

    fn walk(path: Vec<String>, cwd: &Path, out: &mut Vec<Vec<String>>, depth: usize) {
        if depth > 3 {
            out.push(path);
            return;
        }
        let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
        let Some(help) = help_for(&refs, cwd) else {
            out.push(path);
            return;
        };
        let (subs, _) = parse_help(&help);
        if subs.is_empty() {
            out.push(path);
            return;
        }
        for s in subs {
            let mut next = path.clone();
            next.push(s);
            walk(next, cwd, out, depth + 1);
        }
    }
}

/// Does the baseline invocation tell us the command is unusable as invoked?
fn baseline_unusable(o: &super::Observable) -> Option<String> {
    if o.timed_out {
        return Some(format!("timed out after {}s", DEFAULT_TIMEOUT.as_secs()));
    }
    let all = format!("{}{}", o.stdout, o.stderr);
    if all.contains("required arguments were not provided")
        || all.contains("requires a value")
        || all.contains("the following required")
        // Hand-written argument checks that clap never sees. `pmat split`
        // prints "FILE argument is required unless --auto is used" and exits 1;
        // without this the baseline looks usable and every flag under it gets
        // blamed for the pre-existing failure.
        || all.contains("argument is required")
        || all.contains("is required unless")
    {
        return Some("needs positional arguments".into());
    }
    // A panicking baseline emits the same panic text no matter which flag is
    // added, so every flag under it compares equal and is booked as a no-op.
    // `pmat query` with no search term panics at query_execution.rs:76 and
    // took all 28 of its flags down with it.
    if all.contains("panicked at") || o.code == Some(101) {
        return Some("baseline panics; no usable control".into());
    }
    if all.contains("NOT AVAILABLE") || all.contains("NOT IMPLEMENTED") {
        return Some("not built in this configuration".into());
    }
    None
}

fn check_flag(
    path: &[String],
    flag: &HelpFlag,
    baseline: &super::Observable,
    cwd: &Path,
) -> Verdict {
    if DENY_FLAGS.contains(&flag.long.as_str()) {
        return Verdict::Skipped("mutating flag".into());
    }
    let baseline_key = baseline.key();
    let baseline_key = baseline_key.as_str();
    let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();

    // Enumerated values: two legal values must not agree.
    if let Some(values) = &flag.values {
        if values.len() < 2 {
            return Verdict::Skipped("single legal value".into());
        }
        let mut a = refs.clone();
        a.push(&flag.long);
        a.push(&values[0]);
        let mut b = refs.clone();
        b.push(&flag.long);
        b.push(&values[1]);
        let oa = run(&a, cwd, DEFAULT_TIMEOUT);
        let ob = run(&b, cwd, DEFAULT_TIMEOUT);
        if oa.timed_out || ob.timed_out {
            return Verdict::Skipped("timed out".into());
        }
        return if oa.key() == ob.key() {
            Verdict::NoOp
        } else {
            Verdict::Effective
        };
    }

    // Numeric options can be probed without knowing the domain: two different
    // legal magnitudes must not produce identical output. `--top-files` was
    // among the 49 no-op flags in 3.29.0 and would otherwise be skipped here
    // purely because clap does not enumerate integers.
    if flag.takes_free_value {
        let Some((lo, hi)) = numeric_probe_values(&flag.long) else {
            return Verdict::Skipped("needs a value the harness cannot synthesise".into());
        };
        let mut a = refs.clone();
        a.push(&flag.long);
        a.push(lo);
        let mut b = refs.clone();
        b.push(&flag.long);
        b.push(hi);
        let oa = run(&a, cwd, DEFAULT_TIMEOUT);
        let ob = run(&b, cwd, DEFAULT_TIMEOUT);
        if oa.timed_out || ob.timed_out {
            return Verdict::Skipped("timed out".into());
        }
        // If neither value is accepted the probe proves nothing about the flag.
        if !oa.succeeded() && !ob.succeeded() {
            return Verdict::Skipped("synthesised values rejected".into());
        }
        return if oa.key() == ob.key() {
            Verdict::NoOp
        } else {
            Verdict::Effective
        };
    }

    // Boolean switch: presence must change the observable.
    let mut with = refs.clone();
    with.push(&flag.long);
    let o = run(&with, cwd, DEFAULT_TIMEOUT);
    if o.timed_out {
        return Verdict::Skipped("timed out".into());
    }
    if o.key() == baseline_key {
        return Verdict::NoOp;
    }
    // Output differs only because the flag broke the command.
    //
    // The baseline's own exit code is the control. Without it, a command that
    // already fails for its own reasons (`pmat split` without a FILE) blames
    // every flag it carries — twenty such false "errors out" verdicts in one
    // sweep, all of them the command's pre-existing state.
    if baseline.succeeded()
        && !o.succeeded()
        && (o.stderr.contains("error:") || o.stderr.contains("Error:"))
        && o.stdout.trim().is_empty()
    {
        return if is_honest_refusal(&o.stderr) {
            Verdict::Refuses
        } else {
            Verdict::Errors { code: o.code }
        };
    }
    Verdict::Effective
}

/// Does the failure say the flag is deliberately unimplemented?
fn is_honest_refusal(stderr: &str) -> bool {
    let s = stderr.to_lowercase();
    s.contains("not implemented")
        || s.contains("does not implement")
        || s.contains("would be accepted and ignored")
        || s.contains("no longer supported")
}

#[test]
#[ignore = "artifact-level sweep: spawns hundreds of processes; run via `make gate-flag-efficacy`"]
fn flag_efficacy_sweep() {
    let corpus = build_corpus(CorpusSize::Large);
    let cwd = corpus.path();

    let full = std::env::var("PMAT_FLAG_SWEEP").as_deref() == Ok("all");
    let denied: BTreeSet<&str> = DENY_ROOTS.iter().map(|(r, _)| *r).collect();

    let roots: Vec<String> = if full {
        let help = help_for(&[], cwd).expect("pmat --help must work");
        let (subs, _) = parse_help(&help);
        subs.into_iter()
            .filter(|s| !denied.contains(s.as_str()))
            .collect()
    } else {
        CORE_ROOTS.iter().map(|s| s.to_string()).collect()
    };

    let mut commands = Vec::new();
    for root in &roots {
        leaf_commands(root, cwd, &mut commands, 0);
    }

    assert!(
        commands.len() >= 10,
        "discovered only {} leaf command(s) — the help walker is broken, not \
         the codebase. A sweep with no subjects must fail rather than report \
         a clean bill of health.",
        commands.len()
    );

    let mut findings: Vec<Finding> = Vec::new();
    for path in &commands {
        let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
        let path_str = path.join(" ");

        let Some(help) = help_for(&refs, cwd) else {
            findings.push(Finding {
                path: path_str,
                flag: "*".into(),
                verdict: Verdict::Skipped("no help output".into()),
            });
            continue;
        };
        let (_, flags) = parse_help(&help);
        if flags.is_empty() {
            continue;
        }

        // Two identical runs: if they disagree, normalisation is insufficient
        // for this command and every flag verdict under it would be noise.
        let b1 = run(&refs, cwd, DEFAULT_TIMEOUT);
        let b2 = run(&refs, cwd, DEFAULT_TIMEOUT);
        if let Some(reason) = baseline_unusable(&b1) {
            findings.push(Finding {
                path: path_str,
                flag: "*".into(),
                verdict: Verdict::Skipped(reason),
            });
            continue;
        }
        if b1.key() != b2.key() {
            findings.push(Finding {
                path: path_str,
                flag: "*".into(),
                verdict: Verdict::Skipped("nondeterministic baseline".into()),
            });
            continue;
        }
        for flag in &flags {
            let verdict = check_flag(path, flag, &b1, cwd);
            findings.push(Finding {
                path: path_str.clone(),
                flag: flag.long.clone(),
                verdict,
            });
        }
    }

    // ---- report -----------------------------------------------------------
    let mut report = String::new();
    let _ = writeln!(
        report,
        "flag-efficacy sweep — binary: {}\nmode: {}\ncommands walked: {}\n",
        super::pmat_bin().display(),
        if full { "all roots" } else { "core roots" },
        commands.len()
    );
    if !full {
        let _ = writeln!(
            report,
            "NOT SWEPT (core mode; run PMAT_FLAG_SWEEP=all for these): every root outside {CORE_ROOTS:?}"
        );
    }
    let _ = writeln!(report, "DENIED ROOTS (never swept, by design):");
    for (r, why) in DENY_ROOTS {
        let _ = writeln!(report, "  {r:<18} {why}");
    }

    let noops: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.verdict == Verdict::NoOp)
        .filter(|f| {
            !ALLOWED_NOOPS
                .iter()
                .any(|(p, fl, _)| *p == f.path && *fl == f.flag)
        })
        .collect();
    let errors: Vec<&Finding> = findings
        .iter()
        .filter(|f| matches!(f.verdict, Verdict::Errors { .. }))
        .collect();
    let effective = findings
        .iter()
        .filter(|f| f.verdict == Verdict::Effective)
        .count();
    let refuses: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.verdict == Verdict::Refuses)
        .collect();
    let skipped: Vec<&Finding> = findings
        .iter()
        .filter(|f| matches!(f.verdict, Verdict::Skipped(_)))
        .collect();

    let _ = writeln!(
        report,
        "\nsummary: {effective} effective, {} refuses-honestly, {} no-op, {} error-out, {} skipped",
        refuses.len(),
        noops.len(),
        errors.len(),
        skipped.len()
    );

    let _ = writeln!(report, "\n--- NO-OP (flag parses, changes nothing) ---");
    for f in &noops {
        let _ = writeln!(report, "  pmat {} {}", f.path, f.flag);
    }
    let _ = writeln!(
        report,
        "\n--- REFUSES HONESTLY (unimplemented, and says so — this is a pass) ---"
    );
    for f in &refuses {
        let _ = writeln!(report, "  pmat {} {}", f.path, f.flag);
    }
    let _ = writeln!(
        report,
        "\n--- ERRORS OUT (flag breaks a working command) ---"
    );
    for f in &errors {
        if let Verdict::Errors { code } = f.verdict {
            let _ = writeln!(report, "  pmat {} {}  -> exit {:?}", f.path, f.flag, code);
        }
    }
    let _ = writeln!(report, "\n--- SKIPPED (unchecked, not passed) ---");
    for f in skipped {
        if let Verdict::Skipped(why) = &f.verdict {
            let _ = writeln!(report, "  pmat {} {}  [{}]", f.path, f.flag, why);
        }
    }

    let out_path = std::env::temp_dir().join("pmat-flag-efficacy-report.txt");
    let _ = std::fs::write(&out_path, &report);
    println!("{report}\nreport written to {}", out_path.display());

    // Skipping is legitimate per flag; skipping *everything* means the sweep
    // proved nothing, and must not be reported as a pass.
    let actually_checked = effective + noops.len() + errors.len() + refuses.len();
    assert!(
        actually_checked >= 20,
        "only {actually_checked} flag(s) were actually exercised out of {} \
         findings — the sweep degenerated into skips and proves nothing",
        findings.len()
    );

    assert!(
        noops.is_empty(),
        "{} flag(s) parse but change nothing. Each is a defect: either wire \
         it up or remove it. To accept one, add it to ALLOWED_NOOPS with a \
         reason.\n{}",
        noops.len(),
        noops
            .iter()
            .map(|f| format!("  pmat {} {}", f.path, f.flag))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The harness must be able to fail. A sweep that cannot produce a NoOp
/// verdict would pass forever and be exactly the kind of vacuous gate these
/// harnesses were built to replace.
#[test]
fn noop_detection_is_load_bearing() {
    let corpus = build_corpus(CorpusSize::Tiny);
    let cwd = corpus.path();
    let path = vec!["--version".to_string()];
    let baseline = run(&["--version"], cwd, DEFAULT_TIMEOUT);

    // A flag that does not exist cannot be Effective, and repeating an
    // idempotent invocation must compare equal — proving the comparator
    // detects sameness rather than always reporting a difference.
    let repeat = run(&["--version"], cwd, DEFAULT_TIMEOUT);
    assert_eq!(
        baseline.key(),
        repeat.key(),
        "identical invocations must compare equal, else every flag looks effective"
    );

    let synthetic = HelpFlag {
        long: "--version".into(),
        values: None,
        takes_free_value: false,
    };
    let verdict = check_flag(&path, &synthetic, &baseline, cwd);
    assert_eq!(
        verdict,
        Verdict::NoOp,
        "a flag that reproduces the baseline must be classified NoOp"
    );
}

/// Help parsing is the harness's only input; if it silently returns nothing,
/// the sweep reports zero defects and looks green.
#[test]
fn help_parsing_finds_commands_and_flags() {
    let help = "\
Usage: pmat analyze complexity [OPTIONS]

Commands:
  complexity  Analyze complexity
  churn       Analyze churn
  help        Print this message

Options:
  -p, --project-path <PROJECT_PATH>  Where to look
      --format <FORMAT>              Output format [default: summary] [possible values: summary, json, sarif]
      --top-files                    Only the worst offenders
  -h, --help                         Print help
";
    let (subs, flags) = parse_help(help);
    assert_eq!(subs, vec!["complexity", "churn"], "help must be excluded");

    let by = |n: &str| flags.iter().find(|f| f.long == n).cloned();
    assert!(by("--help").is_none(), "--help must never be swept");

    let fmt = by("--format").expect("--format parsed");
    assert_eq!(
        fmt.values.as_deref(),
        Some(
            &[
                "summary".to_string(),
                "json".to_string(),
                "sarif".to_string()
            ][..]
        ),
        "enumerated values drive the two-value comparison"
    );

    let top = by("--top-files").expect("--top-files parsed");
    assert!(
        top.values.is_none() && !top.takes_free_value,
        "a valueless switch must be checked by presence/absence"
    );

    let pp = by("--project-path").expect("--project-path parsed");
    assert!(
        pp.takes_free_value,
        "an unenumerated value option must be skipped, not guessed at"
    );
}

/// A baseline that cannot run is not a control, and every flag under it must
/// be skipped rather than blamed.
#[test]
fn unusable_baselines_are_skipped_not_blamed() {
    let panicking = super::Observable {
        code: Some(101),
        stdout: String::new(),
        stderr:
            "thread 'main' panicked at src/cli/handlers/query_handler/query_execution.rs:76:5:\n\
                 query string required unless --coverage-gaps or --extract-candidates"
                .into(),
        timed_out: false,
    };
    assert!(
        baseline_unusable(&panicking).is_some(),
        "a panicking baseline emits identical text for every flag, so all 28 \
         `pmat query` flags compared equal and were booked as no-ops"
    );

    let missing_arg = super::Observable {
        code: Some(1),
        stdout: String::new(),
        stderr: "Error: FILE argument is required unless --auto is used.".into(),
        timed_out: false,
    };
    assert!(baseline_unusable(&missing_arg).is_some());

    let healthy = super::Observable {
        code: Some(0),
        stdout: "{\"score\": 1.0}".into(),
        stderr: String::new(),
        timed_out: false,
    };
    assert!(
        baseline_unusable(&healthy).is_none(),
        "a working command must remain a usable control"
    );
}

/// A flag that refuses because it is unimplemented satisfies the invariant.
///
/// pmat exits non-zero with, e.g., "--ml is not implemented: complexity scores
/// are still computed by the heuristic formulas, so this flag would relabel
/// them without changing them". That is the fix an earlier round landed for
/// exactly this defect class, and the gate must not report it as a failure.
#[test]
fn honest_refusal_is_a_pass_not_a_failure() {
    for stderr in [
        "Error: --ml is not implemented: complexity scores are still computed by the heuristic formulas",
        "Error: analyze deep-context does not implement --full; the flag(s) would be accepted and ignored",
        "Error: --evolution is not implemented for `analyze satd`",
    ] {
        assert!(
            is_honest_refusal(stderr),
            "a stated refusal must be recognised, not booked as a broken command: {stderr}"
        );
    }
    for stderr in [
        "Error: FILE argument is required unless --auto is used.",
        "error: failed to parse manifest",
        "Error: connection refused",
    ] {
        assert!(
            !is_honest_refusal(stderr),
            "a genuine failure must not be excused as a refusal: {stderr}"
        );
    }
}

/// Materialise a corpus on disk for manual reproduction of a sweep finding.
///
/// Triage is worthless without this: four of the first six "defects" this
/// harness reported were fixture artifacts, and each was only identifiable by
/// running the failing command against the exact corpus the sweep used.
///
/// ```text
/// PMAT_CORPUS_OUT=/tmp/corpus PMAT_CORPUS_SIZE=large \
///   cargo test --test all -- --ignored dump_corpus --nocapture
/// ```
#[test]
#[ignore = "debugging aid: writes a corpus to PMAT_CORPUS_OUT"]
fn dump_corpus() {
    let out =
        std::env::var("PMAT_CORPUS_OUT").expect("set PMAT_CORPUS_OUT to the directory to write");
    let size = match std::env::var("PMAT_CORPUS_SIZE").as_deref() {
        Ok("empty") => CorpusSize::Empty,
        Ok("tiny") => CorpusSize::Tiny,
        _ => CorpusSize::Large,
    };
    let dir = build_corpus(size);
    let src = dir.keep();
    std::fs::rename(&src, &out)
        .or_else(|_| {
            // Cross-device rename fails; fall back to a recursive copy.
            std::process::Command::new("cp")
                .args(["-a", &src.display().to_string(), &out])
                .status()
                .map(|_| ())
        })
        .expect("materialise corpus");
    println!("corpus ({}) written to {out}", size.name());
}

/// clap's *expanded* layout — what `--help` emits, and what the first version
/// of this parser could not read. It found zero enumerated values, classified
/// every `--format` as unsynthesisable, discovered no JSON commands, and
/// passed clean. Pinned here so that regression cannot recur silently.
#[test]
fn expanded_help_layout_yields_enumerated_values() {
    let help = "\
Analyze code complexity

Usage: pmat analyze complexity [OPTIONS]

Options:
      --mode <MODE>
          Force cli or mcp mode

          [possible values: cli, mcp]

  -p, --path <PATH>
          Path to analyze

          [default: .]

  -v, --verbose
          Enable verbose output

      --format <FORMAT>
          Output format

          Possible values:
          - summary: Summary statistics only
          - json:    Machine-readable
          - sarif:   SARIF for code scanning

  -h, --help
          Print help
";
    let (_, flags) = parse_help(help);
    let by = |n: &str| flags.iter().find(|f| f.long == n).cloned();

    let fmt = by("--format").expect("--format must be discovered");
    assert_eq!(
        fmt.values.as_deref(),
        Some(
            &[
                "summary".to_string(),
                "json".to_string(),
                "sarif".to_string()
            ][..]
        ),
        "the multi-line `Possible values:` block must be read; missing it is \
         what made the first sweep vacuous"
    );
    assert!(!fmt.takes_free_value, "enumerated options are testable");

    let mode = by("--mode").expect("--mode must be discovered");
    assert_eq!(
        mode.values.as_deref(),
        Some(&["cli".to_string(), "mcp".to_string()][..]),
        "the bracketed layout must be read on a following line too"
    );

    let verbose = by("--verbose").expect("--verbose must be discovered");
    assert!(
        verbose.values.is_none() && !verbose.takes_free_value,
        "a valueless switch must stay a presence/absence test"
    );

    let path = by("--path").expect("--path must be discovered");
    assert!(
        path.takes_free_value,
        "`[default: .]` marks an option that takes an unenumerated value"
    );
    assert!(by("--help").is_none(), "--help is never swept");
}

/// The corpus must actually contain the defects it claims to, or every
/// downstream verdict is drawn from an empty room.
#[test]
fn large_corpus_contains_every_defect_family() {
    let corpus = build_corpus(CorpusSize::Large);
    let root = corpus.path();
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap_or_default();

    assert!(read("src/satd_00.rs").contains("TODO"), "SATD present");
    assert!(
        read("src/faults_00.rs").contains("unwrap()"),
        "faults present"
    );
    assert!(
        read("src/dup_a_00.rs") == read("src/dup_b_00.rs"),
        "duplicate pair must be identical"
    );
    assert!(
        read("src/complex_07.rs").matches("if ").count() > 3,
        "complexity present"
    );
    let dead = read("src/dead_00.rs");
    assert!(dead.contains("never_called_00"), "dead code present");
    // `analyze dead-code` filters files under `--min-dead-lines` (default 10),
    // so short dead functions produce a zero indistinguishable from a broken
    // detector. The fixture must clear the tool's own default threshold.
    let dead_lines = dead
        .lines()
        .skip_while(|l| !l.contains("never_called_00"))
        .take_while(|l| !l.contains("pub fn entry_00"))
        .count();
    assert!(
        dead_lines > 20,
        "dead region is only {dead_lines} lines; --min-dead-lines defaults to \
         10 and would filter the file out entirely"
    );
    assert!(
        root.join(".git").exists(),
        "corpus must be a git repo; several defects only appear with history"
    );
    // The user's `init.templateDir` installs a pre-commit hook into every new
    // repository. On the first run it ran pmat's own gate, failed, and aborted
    // every corpus commit — so the corpus had no history, `analyze churn`
    // truthfully reported `total_commits: 0`, and the differential gate booked
    // that as a pmat defect. A fixture must never manufacture its own findings.
    assert!(
        super::commit_count(root) >= 2,
        "large corpus must carry real history (got {} commits) — check that \
         git hooks are disarmed in git_init",
        super::commit_count(root)
    );
    let files = std::fs::read_dir(root.join("src")).expect("src").count();
    assert!(
        files > 100,
        "large corpus should be large, got {files} files"
    );

    // A metric can only be checked across a range the corpus spans. Without
    // nested loops `analyze big-o` truthfully reports an empty O(n^2) bucket;
    // without a pathological file `f_grade_count` is truthfully zero and the
    // F-grade gate truthfully passes. Both read as defects to a differential
    // check that never supplied the input needed to tell them apart.
    let sup = read("src/superlinear_00.rs");
    assert!(
        sup.matches("for ").count() >= 5,
        "corpus must contain genuinely superlinear code, not the sequential \
         loops the complex_* family emits"
    );
    let awful = read("src/awful.rs");
    assert!(
        awful.matches("if ").count() > 50 && awful.contains("TODO"),
        "corpus must contain one pathological file to populate the bad tail of \
         the grade distribution"
    );

    // Commits must fall inside the default analysis windows (churn: 30 days,
    // defect-prediction: 90). Pinning them to a fixed calendar date put every
    // commit seven months in the past, so churn honestly reported zero and the
    // differential gate attributed that zero to pmat.
    let age_days = std::process::Command::new("git")
        .args(["log", "-1", "--format=%cr"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    assert!(
        age_days.contains("day") || age_days.contains("hour") || age_days.contains("minute"),
        "newest corpus commit is {age_days:?} old — it must sit inside the \
         30-day churn window or churn metrics read as broken"
    );
}
