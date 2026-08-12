//! Harness A — the differential-corpus invariant.
//!
//! Premise: a number that is identical for an empty project and a 107-file
//! project riddled with every defect pmat claims to detect is not a
//! measurement. It does not matter what the correct value would be — and for a
//! code-quality tool nobody knows what it would be, which is precisely why
//! `let coverage = 65.0; // Simulated coverage` survived to a release.
//!
//! So this asserts nothing about correctness. It asserts *responsiveness*:
//!
//! ```text
//! for every numeric leaf L in a command's JSON output:
//!     L(empty) , L(tiny) , L(large)  must not all be equal
//! ```
//!
//! That single property would have caught the four-literal 0.79 enforcement
//! score, `compression_ratio: 0.33`, and the SATD analyser that always
//! returned an empty vector. None of them require knowing the right answer.
//!
//! Array *lengths* count as leaves: a `files[]` of the same length for an
//! empty and a large project is the same defect wearing a different hat.

use super::{build_corpus, help_for, parse_help, run, CorpusSize, DEFAULT_TIMEOUT};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Command roots worth sweeping: the ones that report measurements.
const ANALYSIS_ROOTS: &[&str] = &[
    "analyze",
    "tdg",
    "score",
    "quality-gate",
    "enforce",
    "context",
    "repo-score",
    "deps-audit",
    "comply",
    "project-diag",
];

/// Key fragments whose constancy across corpora is expected, not a defect.
///
/// These are configuration echoed back, not measurements: a threshold is
/// supposed to be the same for every project. Kept deliberately narrow — the
/// wider this list grows, the less the harness proves.
const CONFIG_KEY_FRAGMENTS: &[&str] = &[
    "version",
    "schema",
    "threshold",
    "target",
    "limit",
    "max_",
    "min_",
    "budget",
    "timestamp",
    "generated_at",
    "duration",
    "elapsed",
    "_ms",
    "tool",
    "format",
    "capacity",
    // Analysis-window and scaling constants echoed into the report. Narrow and
    // explicit rather than a bare "days"/"window", which would also excuse
    // genuine day-denominated measurements.
    "period_days",
    "window_days",
    "at_full_scale",
];

/// Commands whose output is not a measurement of the project under analysis,
/// or which the corpus cannot exercise. Excluded with a stated reason, and
/// listed in the report so the bound is visible.
const NON_MEASURING: &[(&str, &str)] = &[
    (
        "tdg config show",
        "prints configuration, not a measurement — constant by definition",
    ),
    (
        "analyze models",
        "inventories ML models on disk, not project source",
    ),
    (
        "analyze assembly-script",
        "corpus contains no AssemblyScript sources to measure",
    ),
    (
        "analyze web-assembly",
        "corpus contains no WebAssembly modules to measure",
    ),
    (
        "tdg baseline list",
        "lists stored baselines; a fresh corpus has none by construction",
    ),
    (
        "tdg diagnostics",
        "reports cache/storage state, not a property of the project",
    ),
    (
        "analyze incremental-coverage",
        "requires prior coverage data; the corpus ships none",
    ),
];

/// Leaves proven constant for a legitimate reason, each with that reason.
///
/// As with the flag sweep, this is the only escape hatch and every entry is a
/// claim someone must defend.
const ALLOWED_CONSTANTS: &[(&str, &str, &str)] = &[
    // (command path, json leaf path, why it is legitimately constant)
    (
        "comply check",
        "checks[].len",
        "a fixed checklist; the count is structural, the verdicts are what vary",
    ),
    (
        "comply report",
        "checks[].len",
        "a fixed checklist; the count is structural",
    ),
    (
        "comply review",
        "[].len",
        "a fixed review checklist; the count is structural",
    ),
    (
        "project-diag",
        "checks[].len",
        "20 checks by specification (docs/specifications/components/repo-health.md)",
    ),
    (
        "project-diag",
        "categories[].len",
        "5 categories by specification",
    ),
];

#[derive(Debug, Clone, PartialEq)]
struct Leaf {
    path: String,
    value: f64,
}

/// Flatten JSON to numeric leaves, recording array lengths instead of
/// descending into them.
///
/// Array elements are skipped on purpose: index 0 of an empty project's
/// `files[]` and of a large one's are different things, so comparing them
/// would produce noise. The *length* is the comparable aggregate.
fn numeric_leaves(v: &Value, prefix: &str, out: &mut BTreeMap<String, f64>) {
    match v {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                out.insert(prefix.to_string(), f);
            }
        }
        Value::Bool(b) => {
            out.insert(prefix.to_string(), if *b { 1.0 } else { 0.0 });
        }
        Value::Array(items) => {
            out.insert(format!("{prefix}[].len"), items.len() as f64);
        }
        Value::Object(map) => {
            for (k, val) in map {
                let next = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                numeric_leaves(val, &next, out);
            }
        }
        _ => {}
    }
}

fn is_config_key(path: &str) -> bool {
    let lower = path.to_lowercase();
    CONFIG_KEY_FRAGMENTS.iter().any(|f| lower.contains(f))
}

/// Extract the first JSON document from mixed output.
///
/// Several commands print a banner before their JSON; a strict parse would
/// skip them and quietly shrink the sweep.
fn extract_json(s: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(s.trim()) {
        return Some(v);
    }
    let start = s.find(['{', '['])?;
    let candidate = &s[start..];
    let mut de = serde_json::Deserializer::from_str(candidate).into_iter::<Value>();
    de.next()?.ok()
}

struct CommandResult {
    path: String,
    /// Leaf -> value per corpus, in Empty/Tiny/Large order.
    per_corpus: Vec<BTreeMap<String, f64>>,
    skipped: Option<String>,
}

/// Find leaf commands that can emit JSON, by asking the binary's own help.
fn json_capable_commands(cwd: &std::path::Path) -> Vec<Vec<String>> {
    let mut found = Vec::new();
    for root in ANALYSIS_ROOTS {
        walk(vec![root.to_string()], cwd, &mut found, 0);
    }
    return found;

    fn walk(path: Vec<String>, cwd: &std::path::Path, out: &mut Vec<Vec<String>>, depth: usize) {
        if depth > 3 {
            return;
        }
        let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
        let Some(help) = help_for(&refs, cwd) else {
            return;
        };
        let (subs, flags) = parse_help(&help);
        if subs.is_empty() {
            let emits_json = flags.iter().any(|f| {
                f.long == "--format"
                    && f.values
                        .as_ref()
                        .is_some_and(|v| v.iter().any(|x| x.eq_ignore_ascii_case("json")))
            });
            if emits_json {
                out.push(path);
            }
            return;
        }
        for s in subs {
            let mut next = path.clone();
            next.push(s);
            walk(next, cwd, out, depth + 1);
        }
    }
}

#[test]
#[ignore = "artifact-level sweep: builds three corpora and runs every analyser; run via `make gate-differential`"]
fn metrics_must_respond_to_the_corpus() {
    let corpora = [
        (CorpusSize::Empty, build_corpus(CorpusSize::Empty)),
        (CorpusSize::Tiny, build_corpus(CorpusSize::Tiny)),
        (CorpusSize::Large, build_corpus(CorpusSize::Large)),
    ];

    let commands = json_capable_commands(corpora[2].1.path());

    // A sweep that discovers nothing reports nothing and exits 0 — which is
    // the pass-by-default defect this harness exists to catch, committed by
    // the harness itself. It happened on the first run: the help parser missed
    // clap's expanded layout, found zero JSON commands, and passed clean.
    assert!(
        commands.len() >= 10,
        "discovered only {} JSON-capable command(s) under {:?} — the help \
         parser is broken, not the codebase. A sweep with no subjects must \
         fail loudly rather than report a clean bill of health.",
        commands.len(),
        ANALYSIS_ROOTS
    );

    let mut results = Vec::new();

    for path in &commands {
        let joined = path.join(" ");
        if let Some((_, why)) = NON_MEASURING.iter().find(|(c, _)| *c == joined) {
            results.push(CommandResult {
                path: joined,
                per_corpus: Vec::new(),
                skipped: Some((*why).to_string()),
            });
            continue;
        }
        let mut per_corpus = Vec::new();
        let mut skipped = None;
        for (_, dir) in &corpora {
            let mut args: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
            args.push("--format");
            args.push("json");
            let o = run(&args, dir.path(), DEFAULT_TIMEOUT);
            if o.timed_out {
                skipped = Some("timed out".to_string());
                break;
            }
            let Some(json) = extract_json(&o.stdout) else {
                skipped = Some(format!("no JSON on stdout (exit {:?})", o.code));
                break;
            };
            let mut leaves = BTreeMap::new();
            numeric_leaves(&json, "", &mut leaves);
            if leaves.is_empty() {
                skipped = Some("JSON carried no numeric leaves".to_string());
                break;
            }
            per_corpus.push(leaves);
        }
        results.push(CommandResult {
            path: path.join(" "),
            per_corpus,
            skipped,
        });
    }

    // ---- verdicts ---------------------------------------------------------
    let mut constant_leaves: Vec<(String, String, f64)> = Vec::new();
    let mut wholly_inert: Vec<String> = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();

    for r in &results {
        if let Some(why) = &r.skipped {
            skipped.push((r.path.clone(), why.clone()));
            continue;
        }
        if r.per_corpus.len() != 3 {
            skipped.push((r.path.clone(), "incomplete corpus run".into()));
            continue;
        }
        // Only leaves the command reports for all three corpora are comparable.
        let shared: Vec<&String> = r.per_corpus[0]
            .keys()
            .filter(|k| r.per_corpus[1].contains_key(*k) && r.per_corpus[2].contains_key(*k))
            .collect();

        let mut responsive = 0usize;
        let mut inert = Vec::new();
        for k in &shared {
            let a = r.per_corpus[0][*k];
            let b = r.per_corpus[1][*k];
            let c = r.per_corpus[2][*k];
            if a == b && b == c {
                if is_config_key(k) {
                    continue;
                }
                if ALLOWED_CONSTANTS
                    .iter()
                    .any(|(p, leaf, _)| *p == r.path && *leaf == k.as_str())
                {
                    continue;
                }
                inert.push(((*k).clone(), a));
            } else {
                responsive += 1;
            }
        }
        if responsive == 0 && !shared.is_empty() {
            wholly_inert.push(r.path.clone());
        }
        for (k, v) in inert {
            constant_leaves.push((r.path.clone(), k, v));
        }
    }

    // ---- report -----------------------------------------------------------
    let mut report = String::new();
    let _ = writeln!(
        report,
        "differential-corpus sweep — binary: {}\ncommands with JSON output: {}\n",
        super::pmat_bin().display(),
        commands.len()
    );
    for (size, dir) in &corpora {
        let _ = writeln!(
            report,
            "corpus {:<6} {}",
            size.name(),
            super::corpus_fingerprint(dir.path())
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(
        report,
        "--- WHOLLY INERT (no leaf responded to any corpus) ---"
    );
    for c in &wholly_inert {
        let _ = writeln!(report, "  pmat {c}");
    }
    let _ = writeln!(
        report,
        "\n--- CONSTANT LEAVES (identical for empty and large) ---"
    );
    for (cmd, leaf, val) in &constant_leaves {
        let _ = writeln!(report, "  pmat {cmd:<40} {leaf} = {val}");
    }
    let _ = writeln!(report, "\n--- SKIPPED (unchecked, not passed) ---");
    for (cmd, why) in &skipped {
        let _ = writeln!(report, "  pmat {cmd:<40} [{why}]");
    }
    let _ = writeln!(
        report,
        "\nsummary: {} inert command(s), {} constant leaf/leaves, {} skipped",
        wholly_inert.len(),
        constant_leaves.len(),
        skipped.len()
    );

    let out_path = std::env::temp_dir().join("pmat-differential-corpus-report.txt");
    let _ = std::fs::write(&out_path, &report);
    println!("{report}\nreport written to {}", out_path.display());

    assert!(
        wholly_inert.is_empty() && constant_leaves.is_empty(),
        "{} command(s) measure nothing and {} numeric leaf/leaves are identical \
         for an empty project and a defect-rich one. Either the value is not \
         measured, or it is configuration — if configuration, add it to \
         ALLOWED_CONSTANTS with a reason.\n\ninert: {:?}\nconstant: {}",
        wholly_inert.len(),
        constant_leaves.len(),
        wholly_inert,
        constant_leaves
            .iter()
            .map(|(c, l, v)| format!("\n  pmat {c} :: {l} = {v}"))
            .collect::<String>()
    );
}

// ---------------------------------------------------------------------------
// Harness self-tests — these run in the normal suite.
//
// A falsification harness that cannot itself fail is the defect it hunts, so
// each mechanism is checked against a case it must reject.
// ---------------------------------------------------------------------------

/// The sweep's own measurement must match a hand-run of the same command.
///
/// This exists because the sweep reported `analyze dead-code` as constant-zero
/// across all three corpora while running that exact command by hand on the
/// same fixture returned 195 dead lines. A gate whose numbers disagree with
/// the tool it is auditing is worse than no gate: it manufactures defects.
#[test]
#[ignore = "spawns the binary against three corpora; run with the sweeps"]
fn sweep_readings_match_a_hand_run() {
    let corpora = [
        (CorpusSize::Empty, build_corpus(CorpusSize::Empty)),
        (CorpusSize::Tiny, build_corpus(CorpusSize::Tiny)),
        (CorpusSize::Large, build_corpus(CorpusSize::Large)),
    ];
    let mut readings = Vec::new();
    for (size, dir) in &corpora {
        let o = run(
            &["analyze", "dead-code", "--format", "json"],
            dir.path(),
            DEFAULT_TIMEOUT,
        );
        let json = extract_json(&o.stdout);
        let mut leaves = BTreeMap::new();
        if let Some(j) = &json {
            numeric_leaves(j, "", &mut leaves);
        }
        let dead = leaves.get("summary.total_dead_lines").copied();
        println!(
            "{:<6} {} -> total_dead_lines={:?} (exit {:?}, {}B)",
            size.name(),
            super::corpus_fingerprint(dir.path()),
            dead,
            o.code,
            o.stdout.len()
        );
        readings.push((size.name(), dead));
    }

    let large = readings[2].1.expect("large corpus must yield a reading");
    assert!(
        large > 0.0,
        "the large corpus contains 15 files of dead code exceeding \
         --min-dead-lines, so the sweep must read a non-zero value; it read \
         {large}. Readings: {readings:?}"
    );
    assert_ne!(
        readings[0].1, readings[2].1,
        "an empty and a defect-rich corpus must not report the same dead-code \
         total. Readings: {readings:?}"
    );
}

#[test]
fn flattening_records_array_length_not_elements() {
    let v: Value = serde_json::json!({
        "summary": { "score": 0.79, "files": [1, 2, 3] },
        "ok": true
    });
    let mut leaves = BTreeMap::new();
    numeric_leaves(&v, "", &mut leaves);

    assert_eq!(leaves.get("summary.score"), Some(&0.79));
    assert_eq!(
        leaves.get("summary.files[].len"),
        Some(&3.0),
        "array length is the comparable aggregate"
    );
    assert!(
        !leaves.contains_key("summary.files.0"),
        "array elements must not be compared across corpora"
    );
    assert_eq!(leaves.get("ok"), Some(&1.0), "bools are comparable too");
}

#[test]
fn constant_detection_catches_a_fabricated_score() {
    // The exact shape of the shipped defect: four literals averaged, so the
    // score is identical no matter what is being scored.
    let fabricated =
        |_corpus: &str| -> Value { serde_json::json!({ "score": 0.79, "violations": [] }) };
    let mut maps = Vec::new();
    for c in ["empty", "tiny", "large"] {
        let mut m = BTreeMap::new();
        numeric_leaves(&fabricated(c), "", &mut m);
        maps.push(m);
    }
    let all_equal = maps[0]["score"] == maps[1]["score"] && maps[1]["score"] == maps[2]["score"];
    assert!(all_equal, "fixture must reproduce the defect");
    assert!(
        !is_config_key("score"),
        "`score` must not be excused as configuration, or the harness is blind \
         to the single largest defect class"
    );
}

#[test]
fn config_keys_are_excused_but_measurements_are_not() {
    for excused in [
        "summary.threshold",
        "max_complexity",
        "schema_version",
        "elapsed_ms",
    ] {
        assert!(is_config_key(excused), "{excused} should be excused");
    }
    for measured in [
        "summary.score",
        "total_violations",
        "coverage_percent",
        "satd_count",
        "files[].len",
    ] {
        assert!(
            !is_config_key(measured),
            "{measured} is a measurement and must never be excused"
        );
    }
}

#[test]
fn json_is_recovered_from_banner_prefixed_output() {
    let mixed = "Analyzing project...\nDone in 1.2s\n{\"score\": 1.0}\n";
    let v = extract_json(mixed).expect("JSON after a banner must still parse");
    assert_eq!(v["score"], 1.0);
    assert!(
        extract_json("no json here at all").is_none(),
        "absence of JSON must be reported, not papered over with a default"
    );
}

#[test]
fn corpora_differ_in_the_ways_metrics_should_notice() {
    let empty = build_corpus(CorpusSize::Empty);
    let large = build_corpus(CorpusSize::Large);
    let count = |d: &tempfile::TempDir| {
        std::fs::read_dir(d.path().join("src"))
            .map(|r| r.count())
            .unwrap_or(0)
    };
    assert_eq!(count(&empty), 1, "empty corpus is a single stub file");
    assert!(
        count(&large) > 100,
        "large corpus must dwarf the empty one, else the invariant proves nothing"
    );
}
