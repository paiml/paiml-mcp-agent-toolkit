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

use super::{
    build_corpus, help_for, parse_help, run, run_with_env, CorpusSize, HelpFlag, DEFAULT_TIMEOUT,
};
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

/// Whole command paths that mutate the fixture they are pointed at.
///
/// `DENY_ROOTS` cannot express these: `comply` and `analyze` are swept, but two
/// of their leaves rewrite the corpus underneath the rest of the sweep.
/// `comply enforce` installs pre-commit/pre-push hooks (and auto-proceeds when
/// stdin is not a tty, so `--yes` being in `DENY_FLAGS` does not stop it), and
/// `analyze clippy` runs `clippy --fix` and rewrites source in place —
/// verified: on a fixture that compiles it reports `"action": "applied"`,
/// `"fixed_files": ["src/lib.rs"]`. A sweep that edits its own corpus makes
/// every later verdict unreproducible.
const DENY_PATHS: &[(&str, &str)] = &[
    (
        "comply enforce",
        "installs git hooks into the fixture; auto-proceeds when non-interactive",
    ),
    (
        "analyze clippy",
        "runs clippy --fix and rewrites the fixture's source in place",
    ),
    // Mutating, like `comply enforce`: it writes ./.pmat/backup and can rewrite
    // project.toml, so sweeping it changes the fixture every other step
    // measures. Its `--no-backup` is NOT a no-op — on a project the migration
    // actually backs up, the flag removes the "✓ Created backup at:" line
    // (304B -> 266B, verified by hand). It reads inert only on this corpus,
    // which pins version 2.0.0 so `comply` has a version gap to react to; that
    // input sends migrate down the "3 breaking changes detected" path, which
    // returns before any backup is taken, leaving the flag nothing to suppress.
    (
        "comply migrate",
        "writes ./.pmat/backup and may rewrite project.toml, mutating the shared fixture",
    ),
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
    // `comply ratchet --lower` REWRITES `.pmat-ratchet.toml` — its own help says
    // "every baseline the tree has beaten is rewritten to the measured value,
    // and the entry's `justification` ... is removed". The sweep was invoking it
    // unattended and booking the result as a no-op.
    //
    // Precisely: probes run with the CORPUS as cwd, so this could not reach
    // THIS repository's baselines. It was a no-op only because no corpus
    // carries a `.pmat-ratchet.toml`, so the handler bails before it writes —
    // and the corpus is a thing we add files to (it just gained an `examples/`
    // and a `.min.` file so two satd counters could move). The first corpus to
    // carry a ratchet file would have its baselines silently lowered, and the
    // sweep would record that as "changes nothing".
    //
    // A flag that rewrites a file is not a flag to probe for observable output.
    "--lower",
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
/// someone has to defend in review.
///
/// **The policy the entries must satisfy: every reason names a demonstrated
/// real effect** — the code path that reads the flag and the input or format
/// under which it was observed changing something. "Legitimately inert" is not
/// a reason; "gates the per-proof evidence section of the markdown renderer,
/// which the default `summary` renderer does not emit" is. An entry that
/// cannot name an effect is a defect being documented rather than fixed, which
/// is exactly how 49 no-op flags shipped in 3.29.0.
///
/// Each entry below was triaged against the 3.30.0 artifact by reproducing the
/// flag's effect on an input the sweep's corpus does not provide, or on a
/// `--format` the sweep does not carry.
const ALLOWED_NOOPS: &[(&str, &str, &str)] = &[
    // (command path, flag, why it legitimately changes nothing)
    // --- --color on commands that emit no colour ------------------------------
    // The switch itself is live in the same binary and corpus: `pmat score
    // --color always` emits ANSI on 13 lines against 0 under `--color auto`.
    // It sets CLICOLOR_FORCE/NO_COLOR in apply_ux_settings
    // (src/cli/cli_run_command.rs:65-75), which colors_enabled()
    // (src/cli/colors.rs:126-140) resolves once per process. These commands
    // produce nothing colourable for it to switch.
    (
        "analyze dag",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR for colors_enabled(); this command's whole output is a machine-readable mermaid graph with no colourable element",
    ),
    (
        "analyze incremental-coverage",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; incremental_coverage_handler*.rs reference crate::cli::colors zero times and emit plain status lines",
    ),
    (
        "analyze lint-hotspot",
        "--color",
        "colours the clean-result confirmation (lint_hotspot_handlers/mod.rs:206-220 report_measured_clean -> c::pass); on a clean crate `--color always` emits an ANSI-green tick and `--color auto` does not — the corpus is dirty by construction so that branch is unreachable",
    ),
    (
        "analyze proof-annotations",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; proof_annotation_helpers_format.rs emits no colour in any of its five formats (summary/full/json/markdown/sarif all verified at 0 escape bytes)",
    ),
    (
        "comply cross-crate",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; nothing under cross_crate_handlers/ references crate::cli::colors, and the reachable output here is the plain single-crate discovery notice (handler.rs:81)",
    ),
    (
        "context",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; context emits a markdown/JSON document users redirect to a file, and colouring it would corrupt the artifact",
    ),
    (
        "explain",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; explain prints a static id/name catalogue (command_routing.rs:428-438) with no status or score element to colour",
    ),
    // `quality-gate --color` was here and has been deleted: the colour-adoption
    // work landed, and the entry's stated reason ("all fifteen
    // quality_gate_*.rs printers ... reference crate::cli::colors zero times")
    // is no longer true. Verified by hand on the large corpus: `--color always`
    // emits 189 ANSI-carrying lines, `--color never` emits 0. The sweep now
    // scores it Effective, and an allow-list entry that suppresses nothing is
    // how an exception outlives its reason.
    (
        "tdg config show",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; the output is a pretty-printed JSON config dump, which must not carry escapes",
    ),
    (
        "tdg config sources",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; the entire output is three plain println!s (config_command_handlers.rs:335-338)",
    ),
    (
        "tdg config validate",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; the entire output is one emoji literal line (config_command_handlers.rs:322)",
    ),
    (
        "tdg history",
        "--color",
        "sets CLICOLOR_FORCE/NO_COLOR; colour is applied by format_history_output, which is only reached with stored TDG records (history.rs:33-39) — an empty result set has nothing to colour",
    ),
    // --- --quiet on commands that emit no chatter -----------------------------
    // The switch itself is live in the same binary and corpus: this round took
    // `analyze bottleneck` from 40B of stderr to 0B, `analyze provability` from
    // 125B, `context` from 60B and `score` from 55B, in every case with the
    // stdout report byte-identical. --quiet suppresses NOISE and never RESULTS,
    // so a command whose entire output is its report has nothing for it to take
    // away — and each entry below was measured, not assumed: stderr is 0B on
    // the swept invocation, or the only stderr is an error, which must survive.
    (
        "analyze big-o",
        "--quiet",
        "emits only its report on stdout (820B) and nothing on stderr; its header/summary go through tracing `info!`, which is below the default filter. The one stderr line it can produce is the `--analyze-space` no-op warning about the user's own flag, which is a diagnostic, not chatter, and does not fire on the swept invocation",
    ),
    (
        "analyze defects",
        "--quiet",
        "emits only the Known Defects Report on stdout (3661B, exit 1) and nothing on stderr; src/cli/handlers/analyze_defects_handler/ contains no stderr macro at all",
    ),
    (
        "analyze deep-context",
        "--quiet",
        "the swept default format uses SimpleDeepContext, which prints no progress — stdout report only, stderr 0B. The flag's effect is demonstrable one format over: `--format sarif` runs the full DeepContextAnalyzer and emitted 62B of spinner text, now 62B -> 0B under --quiet with this round's progress-primitive fix",
    ),
    (
        "analyze entropy",
        "--quiet",
        "emits only the Entropy Analysis Summary on stdout (571B), stderr 0B. The four progress banners in its file (entropy_semantic.rs) belong to `analyze semantic`'s route, not to route_entropy_analysis",
    ),
    (
        "analyze models",
        "--quiet",
        "emits only the Model Inventory table on stdout (700B), stderr 0B. Its single stderr line is the 'No model files found' refusal, which is an error and must survive --quiet",
    ),
    (
        "deps-audit",
        "--quiet",
        "emits only the Dependency Audit Report on stdout (548B), stderr 0B; src/cli/handlers/deps_audit_handlers/ contains no stderr macro",
    ),
    (
        "explain",
        "--quiet",
        "emits only the check/metric listing on stdout (986B), stderr 0B. Its one stderr line is the miss message for an unknown pattern, which exits 1 and is an error",
    ),
    (
        "repo-score",
        "--quiet",
        "emits only the Repository Health Score report on stdout (889B), stderr 0B; repo_score_handlers*.rs contains no stderr macro",
    ),
    (
        "comply audit",
        "--quiet",
        "on the sweep's corpus audit refuses with 'ERROR: Audit requires clean git state.' (179B stderr, exit 1) before reaching any output, and errors are not suppressible. On a clean-git copy of the same corpus the flag is live where the banner exists: `-f markdown` prints 'PMAT Comply Audit (Layer 3: Governance)' + rule via status_println!, and --quiet drops both; the swept default `-f json` deliberately emits the artifact alone so it stays parseable",
    ),
    (
        "comply init",
        "--quiet",
        "on the corpus the command has already been initialised by an earlier sweep invocation and prints its 'Project already initialized at ./.pmat/project.toml' refusal on stdout, which is the result. On a fresh directory the banner and next-steps block are status_println! and --quiet takes stdout 378B -> 101B (measured), leaving only the created-artifact lines",
    ),
    (
        "comply report",
        "--quiet",
        "the swept invocation writes the compliance report itself to stdout (511B), and a report is a result. The flag's effect is on the --output branch: `comply report --output F` prints '✓ Compliance report written to F' via status_println! (43B), and --quiet suppresses it (measured: 43B -> 0B, file still written)",
    ),
    (
        "comply asset-validate",
        "--quiet",
        "emits only the CB-13xx contract findings and the '1 pass, 2 warn, 4 skip' tally on stdout (645B), stderr 0B",
    ),
    // --- flags that modify another flag ---------------------------------------
    (
        "analyze lint-hotspot",
        "--dry-run",
        "modifies --enforce only: `--enforce` prints '🚨 Quality gate is blocking' and `--enforce --dry-run` suppresses it (lint_hotspot_handlers/mod.rs:362 `enforce && !dry_run && blocking`); nothing is written to disk in either mode",
    ),
    (
        "enforce extreme",
        "--dry-run",
        "modifies --apply-suggestions only: `--apply-suggestions` walks Refactoring/Validating/Iteration-3 states, `--apply-suggestions --dry-run` stops at Violating (enforce_handlers/states.rs:92-95)",
    ),
    (
        "tdg diagnostics",
        "--storage",
        "selects the storage section, which is also the default when no section is chosen (tdg_diagnostic_handler.rs:86-88); with a sibling selected it is observable — `--scheduler` vs `--scheduler --storage` adds the whole 12-line Storage Diagnostics block",
    ),
    // --- flags the default renderer has no place to show ----------------------
    (
        "analyze proof-annotations",
        "--include-evidence",
        "gates write_detailed_proofs in the markdown and full renderers (proof_annotation_helpers_report.rs:158-175): `-f markdown --include-evidence` adds 1381 lines of '## Detailed Proofs'; the default `-f summary` emits only totals, so there is no per-proof section to attach evidence to",
    ),
    (
        "analyze provability",
        "--include-evidence",
        "gates the per-property evidence blocks (provability_helpers_json.rs:32, provability_helpers_detailed.rs:61,201): `-f json --include-evidence` adds a \"properties\": [...] array to each function; the summary renderer has no per-function section",
    ),
    (
        "enforce extreme",
        "--single-file-mode",
        "changes files_remaining from distinct_violation_files() to 0/1 (enforce_handlers/states.rs:51-56): `-f json` shows \"files_remaining\": 73 -> 1; the summary renderer does not print the progress block",
    ),
    (
        "quality-gate",
        "--include-provability",
        "populates results.provability_score via calculate_provability_score (quality_gate_project.rs:51-56): `-f json` shows \"provability_score\": null -> 0.919; the summary renderer never prints that field",
    ),
    (
        "analyze comprehensive",
        "--executive-summary",
        "prepends the Executive Summary block in format_as_markdown (comprehensive_analysis_handler/output.rs:46-60): `--format markdown --executive-summary` adds 15 lines; under the default `summary` format format_as_text already prints that section unconditionally, so the flag would ask for what is already on screen",
    ),
    (
        "analyze graph-metrics",
        "--top-k",
        "truncates the per-node ranking via filter_results (analysis/graph_metrics_handler.rs:41): 1 vs 50 changes detailed 38->528, json 22->463, csv 6->55 lines; the default `summary` format is documented as 'Summary statistics only' and emits no node list to cut",
    ),
    // --- filters the corpus cannot straddle -----------------------------------
    (
        "analyze entropy",
        "--min-severity",
        "filters the violation list by severity floor in every format; on this corpus `medium` vs `high` drops the Medium diversity violation (2 -> 1 violations). The sweep pits values[0]=low against values[1]=medium and the corpus contains no Low-severity violation, so both admit the same set",
    ),
    (
        "analyze graph-metrics",
        "--metrics",
        "gates which centrality algorithms run (graph_metrics_algorithms.rs:28 Brandes, :64 closeness, :78 PageRank); `--metrics closeness --format json` gives close_max 0.99 against 0.0, and centrality vs betweenness differ on a graph with 2-hop paths. The sweep pits centrality against betweenness under the default `summary` format, which prints no per-node metric",
    ),
    (
        "analyze proof-annotations",
        "--verification-method",
        "filters annotations by VerificationMethod: borrow-checker and all select the corpus's 259 annotations, formal-proof/model-checking/static-analysis/abstract-interpretation each select 0. The sweep pits values[0]=formal-proof against values[1]=model-checking, and pmat's syn-based analyser only ever emits BorrowChecker, so both correctly select the empty set",
    ),
    // --- effects the sweep's observable cannot reach --------------------------
    (
        "score",
        "--regression-check",
        "calls check_regression (score_handler.rs:111-121), which needs a persisted .pmat-metrics score from a different sha and a delta worse than -5.0; with two such scores present it exits 1 with 'REGRESSION: composite dropped -32.1 pts'. A freshly built corpus has no prior score history at any sha",
    ),
    (
        "score",
        "--stack",
        "appends the 'Stack Quality (CB-150)' block listing sovereign dependencies found in Cargo.toml (score_handler_display.rs:5-31); adding `aprender`/`trueno` to the fixture's Cargo.toml makes it appear. The corpus has an empty [dependencies] section, and the function early-returns when none are found",
    ),
    (
        "comply check",
        "--strict",
        "escalates warnings to a failing exit and nothing else — exit_policy (comply_handlers/check_handlers/check.rs:229-256) returns code 2 only when `is_compliant && warn > 0 && fail == 0`; a report with failures already exits 1 and the flag has nothing left to escalate. Demonstrated on this corpus: give it a `deny.toml` naming all four families and a `.github/workflows/ci.yml` whose `gate` job runs the CB-reaching commands, then `PMAT_REQUIRED_STATUS_CHECKS=gate pmat comply check` exits 0 and the same run with --strict exits 2 with 'the report is COMPLIANT (0 failures), but --strict treats warnings as errors: 9 warning(s)'. The swept corpus fails CB-1701 and CB-2100, so both runs exit 1 with byte-identical stdout (15,773B). Pinned as a tri-state by check_readonly_and_exemption_tests.rs::strict_exit_codes_are_a_documented_tri_state",
    ),
];

/// Where a probe writes when a flag's only observable is "a file was written".
///
/// Deliberately outside the corpus: writing into the fixture mid-sweep would
/// change the input every later command sees.
const PROBE_OUTPUT_FILE: &str = "/tmp/pmat-flag-efficacy-probe-output.txt";

/// Extra arguments a flag needs before its effect exists at all.
///
/// Some flags modify another flag, and some only speak through a non-default
/// renderer. Probing those bare compares two runs in which the flag has nothing
/// to do, and reports the fixture's shape as the flag's defect. The control run
/// carries the same extra arguments, so the comparison still isolates the flag.
///
/// Each entry is a claim that this context is where the flag lives, and is as
/// reviewable as an `ALLOWED_NOOPS` entry — with the difference that the flag
/// still has to prove itself.
/// Environment a flag needs before its effect is observable at all.
///
/// `("*", flag, env)` applies to that flag on every command; a concrete path
/// applies to that command only. Each row is a reviewable claim about WHERE the
/// flag's effect lives, exactly like `PROBE_CONTEXT` is for arguments.
///
/// PER-FLAG, NEVER GLOBAL, and that is the whole design. `--quiet` is honoured
/// above clap dispatch — `effective_trace_filter` drops the RUST_LOG fallback
/// and `log_level_directive` forces "error" — so it suppresses framework
/// chatter for every command in the tree. With RUST_LOG unset there is nothing
/// to suppress and it reads as a no-op on ~43 commands; the sweep was reporting
/// a property of the developer's shell.
///
/// Setting RUST_LOG globally in `run()` would fix `--quiet` and break
/// `--verbose`, which would then measure 129B against 129B and read as a no-op
/// on every command — the same trap as the NO_COLOR bug recorded in
/// `mod.rs::run`, which turned ~40 `--color` flags into false positives. The
/// overlay is therefore attached to the one flag whose effect it exposes.
/// `(command path or "*", flag, environment)`.
type ProbeEnvRow = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

const PROBE_ENV: &[ProbeEnvRow] = &[("*", "--quiet", &[("RUST_LOG", "info")])];

fn probe_env(path: &str, flag: &str) -> &'static [(&'static str, &'static str)] {
    PROBE_ENV
        .iter()
        .find(|(p, f, _)| (*p == "*" || *p == path) && *f == flag)
        .map_or(&[][..], |(_, _, env)| *env)
}

const PROBE_CONTEXT: &[(&str, &str, &[&str])] = &[
    // `--quiet` suppresses the "Diagnostic report written to: ..." notice, which
    // only exists on the --output branch (project_diag_handlers.rs:123).
    ("project-diag", "--quiet", &["--output", PROBE_OUTPUT_FILE]),
    // comply report's default format is markdown, which has no colour by
    // design; the coloured renderer is ComplyOutputFormat::Text
    // (migrate_handlers_enforce.rs:140-176).
    ("comply report", "--color", &["--format", "text"]),
    // The threshold's only consumer is the gate, and the gate's only consumer
    // is the threshold: `analyze dead-code` reports 3.8% dead against a default
    // limit of 15%, so neither is observable without the other.
    (
        "analyze dead-code",
        "--fail-on-violation",
        &["--max-percentage", "1.0"],
    ),
    (
        "analyze dead-code",
        "--max-percentage",
        &["--fail-on-violation"],
    ),
    // A PageRank convergence tolerance changes the ranking, not the six
    // aggregate numbers the summary format prints.
    (
        "analyze graph-metrics",
        "--convergence-threshold",
        &["--format", "json"],
    ),
];

/// Probe values that must straddle the fixture for a specific option.
///
/// `numeric_probe_values` guesses from the option's name; where the corpus's
/// own measured statistic is known, name the pair instead of guessing around
/// it. A pair that sits entirely on one side of the fixture cannot falsify a
/// filter: `--max-percentage 0.1` and `0.9` are both below the corpus's 3.8%
/// dead-code density, so both fail the gate identically.
const PROBE_VALUES: &[(&str, &str, &str, &str)] = &[
    ("analyze dead-code", "--max-percentage", "1.0", "50"),
    (
        "analyze graph-metrics",
        "--convergence-threshold",
        "0.000000000001",
        "0.5",
    ),
];

fn probe_context(path: &str, flag: &str) -> &'static [&'static str] {
    PROBE_CONTEXT
        .iter()
        .find(|(p, f, _)| *p == path && *f == flag)
        .map_or(&[][..], |(_, _, extra)| *extra)
}

fn probe_values(path: &str, flag: &str) -> Option<(&'static str, &'static str)> {
    PROBE_VALUES
        .iter()
        .find(|(p, f, _, _)| *p == path && *f == flag)
        .map(|(_, _, lo, hi)| (*lo, *hi))
}

/// Two probe values for an option clap did not enumerate, chosen from the
/// option's name. Returns `None` when the domain is unguessable (paths,
/// patterns, free text) rather than guessing and reporting a bogus verdict.
fn numeric_probe_values(long: &str) -> Option<(&'static str, &'static str)> {
    let n = long.trim_start_matches('-').to_lowercase();
    if n.contains("threshold") || n.contains("ratio") || n.contains("percentage") {
        return Some(("0.1", "0.9"));
    }
    // Line counts are compared against whole files, not against a handful of
    // items: the corpus's critical-risk file is ~985 lines, so `1` and `50`
    // both admit it and `analyze comprehensive --min-lines` looked inert. The
    // pair has to span the file-length distribution, not sit under it.
    if n.contains("lines") {
        return Some(("1", "1000"));
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
    // A subcommand compiled out of this build refuses identically for every
    // flag: `tdg dashboard` bails with "Dashboard requires the 'http-server'
    // feature", and its --color and --open verdicts were the refusal compared
    // against itself, not the flags.
    if all.contains("requires the") && all.contains("' feature") {
        return Some("feature-gated out of this build".into());
    }
    // Commands that decline their own precondition and then exit 0. These read
    // as healthy baselines — stdout is non-empty and the exit code is clean —
    // but the handler returned before it reached anything a flag controls, so
    // every flag under it compares equal.
    for (needle, why) in [
        (
            "requires at least 2 crates",
            "single-crate corpus: the command declined before any flag was read",
        ),
        (
            "No TDG history found",
            "no stored TDG history: the empty-set early return precedes the formatter",
        ),
        (
            "No model files found",
            "corpus has no model files: the handler returns before --check",
        ),
        (
            "No WebAssembly files found",
            "corpus has no WebAssembly files",
        ),
    ] {
        if all.contains(needle) {
            return Some(why.into());
        }
    }
    // A baseline that died inside an external tool is not a control: the
    // fixture, not pmat, decided the outcome.
    for (needle, why) in [
        (
            "No rule to make target",
            "fixture Makefile lacks the target this command shells out to",
        ),
        (
            "could not compile",
            "fixture does not compile; the command failed before reading its flags",
        ),
    ] {
        if all.contains(needle) {
            return Some(why.into());
        }
    }
    // STRUCTURAL, and deliberately last: a baseline that failed AND rendered
    // nothing is not a control, whatever it printed on stderr.
    //
    // Every needle above is a specific phrase someone had to observe first.
    // This rule needs no phrase, and it is the one that closes the class. CI
    // caught `analyze coverage-improve --fast` as a no-op where the whole
    // sweep was green locally: the harness corpus has no Makefile, so the
    // command exits 1 with an empty stdout BEFORE `--fast` is ever read, and
    // the flagged run reproduced the baseline exactly because both had failed
    // identically. The flag is in fact wired (`calculator.rs:212,297,316`).
    //
    // `compare_probes` has carried this exact guard for the two-value paths
    // since it booked `--format text|json` on this same command as a no-op.
    // The boolean-switch path never got it, so the rule lived in one of the
    // two places that needed it. Putting it HERE covers both, because both
    // paths validate their baseline through this function.
    //
    // The `rendered_nothing` conjunct is load-bearing: a non-zero exit is the
    // NORMAL outcome for the commands this gate exists to police — `analyze
    // lint-hotspot`, `quality-gate` and `enforce extreme` all exit 1 on the
    // defect-rich corpus while printing a full report. They keep a non-empty
    // stdout and stay usable as controls. Skipping on exit code alone would
    // excuse every no-op flag on every failing quality gate, trading one
    // silent pass for another.
    if !o.succeeded() && o.stdout.trim().is_empty() {
        return Some(format!(
            "baseline exited {} with empty stdout; the command failed before any              flag was read, so it is not a control",
            o.code.map_or_else(|| "by signal".to_string(), |c| c.to_string())
        ));
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
    let path_str = path.join(" ");
    let extra = probe_context(&path_str, &flag.long);
    let env = probe_env(&path_str, &flag.long);
    let mut refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
    refs.extend_from_slice(extra);

    // With a probe context OR a probe environment the control must carry it too,
    // or the comparison measures the context rather than the flag.
    let contextual_baseline;
    let baseline = if extra.is_empty() && env.is_empty() {
        baseline
    } else {
        contextual_baseline = run_with_env(&refs, cwd, DEFAULT_TIMEOUT, env);
        if let Some(why) = baseline_unusable(&contextual_baseline) {
            return Verdict::Skipped(format!("probe context {extra:?}: {why}"));
        }
        &contextual_baseline
    };
    let baseline_key = baseline.key();
    let baseline_key = baseline_key.as_str();

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
        let oa = run_with_env(&a, cwd, DEFAULT_TIMEOUT, env);
        let ob = run_with_env(&b, cwd, DEFAULT_TIMEOUT, env);
        return compare_probes(&oa, &ob);
    }

    // Numeric options can be probed without knowing the domain: two different
    // legal magnitudes must not produce identical output. `--top-files` was
    // among the 49 no-op flags in 3.29.0 and would otherwise be skipped here
    // purely because clap does not enumerate integers.
    if flag.takes_free_value {
        let Some((lo, hi)) =
            probe_values(&path_str, &flag.long).or_else(|| numeric_probe_values(&flag.long))
        else {
            return Verdict::Skipped("needs a value the harness cannot synthesise".into());
        };
        let mut a = refs.clone();
        a.push(&flag.long);
        a.push(lo);
        let mut b = refs.clone();
        b.push(&flag.long);
        b.push(hi);
        let oa = run_with_env(&a, cwd, DEFAULT_TIMEOUT, env);
        let ob = run_with_env(&b, cwd, DEFAULT_TIMEOUT, env);
        return compare_probes(&oa, &ob);
    }

    // Boolean switch: presence must change the observable.
    let mut with = refs.clone();
    with.push(&flag.long);
    let o = run_with_env(&with, cwd, DEFAULT_TIMEOUT, env);
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

/// Compare two probes of the same option at different values.
///
/// Difference is decided first, and only then is failure considered: a command
/// that exits non-zero for both values can still be honouring the option (a
/// gate that fails differently is doing its job). Two runs that *agree*, both
/// failed, **and rendered nothing** prove nothing about the option — the enum
/// branch lacked this guard, so `analyze coverage-improve --format text|json`,
/// which died identically inside `make` before any formatter ran, was booked
/// as a no-op.
///
/// "Rendered nothing" is load-bearing and deliberately narrow. A non-zero exit
/// is the *normal* outcome for the commands this gate cares most about:
/// `analyze lint-hotspot`, `quality-gate` and `enforce extreme` all exit 1 on
/// the defect-rich corpus while printing a full report. Skipping on exit code
/// alone would have excused every no-op flag on every failing quality gate —
/// swapping one silent pass for another.
fn compare_probes(a: &super::Observable, b: &super::Observable) -> Verdict {
    if a.timed_out || b.timed_out {
        return Verdict::Skipped("timed out".into());
    }
    if a.key() != b.key() {
        return Verdict::Effective;
    }
    let rendered_nothing = a.stdout.trim().is_empty() && b.stdout.trim().is_empty();
    if !a.succeeded() && !b.succeeded() && rendered_nothing {
        return Verdict::Skipped("both probe values rejected; no usable control".into());
    }
    Verdict::NoOp
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

        if let Some((_, why)) = DENY_PATHS.iter().find(|(p, _)| *p == path_str) {
            findings.push(Finding {
                path: path_str,
                flag: "*".into(),
                verdict: Verdict::Skipped(format!("denied: {why}")),
            });
            continue;
        }

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
        "flag-efficacy sweep — binary: {}\nmode: {}\ncorpus: {}\ncommands walked: {}\n",
        super::pmat_bin().display(),
        if full { "all roots" } else { "core roots" },
        super::corpus_fingerprint(cwd),
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
    let _ = writeln!(report, "DENIED PATHS (never swept, by design):");
    for (p, why) in DENY_PATHS {
        let _ = writeln!(report, "  {p:<18} {why}");
    }
    let _ = writeln!(
        report,
        "ALLOWED NO-OPS ({} entries; each names a demonstrated real effect):",
        ALLOWED_NOOPS.len()
    );
    for (p, f, why) in ALLOWED_NOOPS {
        let _ = writeln!(report, "  pmat {p} {f}\n      {why}");
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

    // An allow-list entry that no longer suppresses anything is stale: the
    // flag has either become effective or stopped being checked. Reported
    // rather than enforced — a command the sweep skips legitimately produces
    // no verdict for its entries to suppress — but a stale entry is how an
    // exception outlives the reason for it.
    let unexercised: Vec<&(&str, &str, &str)> = ALLOWED_NOOPS
        .iter()
        .filter(|(p, fl, _)| {
            !findings
                .iter()
                .any(|f| f.path == *p && f.flag == *fl && f.verdict == Verdict::NoOp)
        })
        .collect();
    let _ = writeln!(
        report,
        "\n--- ALLOW-LIST ENTRIES NOT EXERCISED ({} of {}; the flag is now effective, or its \
         command was skipped — re-verify and delete the entry) ---",
        unexercised.len(),
        ALLOWED_NOOPS.len()
    );
    for (p, fl, _) in &unexercised {
        let _ = writeln!(report, "  pmat {p} {fl}");
    }

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

/// A baseline that FAILED and printed nothing is not a control — and one that
/// failed while printing a full report still is.
///
/// RED-first: on the commit before the structural rule was added to
/// `baseline_unusable`, the first assertion here fails. That is not a
/// hypothetical — it is the CI failure that produced this test. The sweep was
/// green on this workstation and red on the runner, reporting
/// `pmat analyze coverage-improve --fast` as the one flag that "parses but
/// changes nothing". It does not: `--fast` reaches `fast_mode` in
/// `perfection_score/calculator.rs` at lines 212, 297 and 316. The corpus the
/// runner sweeps has no Makefile, so the command exits 1 with an empty stdout
/// before any flag is read, and the flagged run reproduced the baseline byte
/// for byte because BOTH had already failed.
///
/// Measured on a Makefile-less fixture, both probes:
///   `analyze coverage-improve --path FIX`          exit=1  stdout=0B
///   `analyze coverage-improve --path FIX --fast`   exit=1  stdout=0B
///
/// The second and third assertions are the counter-test, and they are the
/// reason the rule carries the `rendered_nothing` conjunct rather than keying
/// on the exit code alone. `quality-gate`, `analyze lint-hotspot` and `enforce
/// extreme` are the commands this gate exists to police, and exiting 1 while
/// printing a full report is their NORMAL, healthy outcome. Were they skipped,
/// every no-op flag on every failing quality gate would be excused — which is
/// the same silent pass the harness was built to end, just relocated.
#[test]
fn a_baseline_that_failed_silently_is_not_a_control() {
    let died_before_reading_flags = super::Observable {
        code: Some(1),
        stdout: String::new(),
        stderr: "Error: Could not find Makefile in /tmp/corpus or parent directories".into(),
        timed_out: false,
    };
    assert!(
        baseline_unusable(&died_before_reading_flags).is_some(),
        "a baseline that exited non-zero having rendered NOTHING cannot serve as          a control: every flag added to it reproduces it exactly and is booked a          no-op. This is the CI failure that reported `analyze coverage-improve          --fast` inert when it is wired to fast_mode."
    );

    // The counter-test. Without it, "skip every failing baseline" passes the
    // assertion above and silently excuses the entire defect-rich corpus.
    let failed_loudly = super::Observable {
        code: Some(1),
        stdout: "TDG Score: 42.1 (F)\n12 violations found\n".into(),
        stderr: String::new(),
        timed_out: false,
    };
    assert!(
        baseline_unusable(&failed_loudly).is_none(),
        "exiting non-zero while printing a full report is the NORMAL outcome for          `quality-gate`, `analyze lint-hotspot` and `enforce extreme` on the          defect corpus. Skipping these would excuse every no-op flag they carry."
    );

    // ...and success with empty stdout is still a control: plenty of commands
    // report on stderr and exit 0. Only the CONJUNCTION is disqualifying.
    let quiet_success = super::Observable {
        code: Some(0),
        stdout: String::new(),
        stderr: "analysed 12 files".into(),
        timed_out: false,
    };
    assert!(
        baseline_unusable(&quiet_success).is_none(),
        "a command that succeeded is a usable control even with an empty stdout"
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

/// Normalisation must not erase the measurement it is normalising.
///
/// The git-object-id rule `\b[0-9a-f]{7,40}\b` also matched the fractional
/// digits of a float, so `"pagerank": 0.008631379960390049` and
/// `"pagerank": 0.008631851530375838` both became `0.<SHA>` and compared equal.
/// `analyze graph-metrics --convergence-threshold` changed those numbers and
/// the sweep still called it a no-op — the harness deleted the evidence.
#[test]
fn normalisation_keeps_decimal_digits_and_still_erases_object_ids() {
    let a = super::normalize("\"pagerank\": 0.008631379960390049");
    let b = super::normalize("\"pagerank\": 0.008631851530375838");
    assert_ne!(
        a, b,
        "two different pagerank values must stay different after normalisation"
    );
    assert!(
        a.contains("0.008631379960390049"),
        "float digits must survive normalisation, got {a:?}"
    );
    assert_eq!(
        super::normalize("commit 4dea27ed4f00 landed"),
        "commit <SHA> landed",
        "git object ids must still be erased, or every sha-bearing baseline is nondeterministic"
    );
}

/// A flag whose effect only exists alongside another flag must be probed with
/// it, and against a control carrying the same context.
#[test]
fn probe_context_is_declared_for_flags_that_modify_another_flag() {
    assert_eq!(
        probe_context("analyze dead-code", "--fail-on-violation"),
        ["--max-percentage", "1.0"],
        "the gate is only observable against a threshold the corpus exceeds"
    );
    assert_eq!(
        probe_context("analyze dead-code", "--max-percentage"),
        ["--fail-on-violation"],
        "the threshold's only consumer is the gate"
    );
    assert_eq!(
        probe_values("analyze dead-code", "--max-percentage"),
        Some(("1.0", "50")),
        "0.1 vs 0.9 sit entirely below the corpus's 3.8% dead-code density"
    );
    assert!(
        probe_context("analyze complexity", "--top-files").is_empty(),
        "flags with no declared context must be probed bare"
    );
    assert_eq!(
        numeric_probe_values("--min-lines"),
        Some(("1", "1000")),
        "a line-count probe must span whole files, not a handful of items"
    );
}

/// Two probes that agree *and* both failed prove nothing about the option.
#[test]
fn agreeing_failed_probes_are_skipped_not_booked_as_noop() {
    let died = || super::Observable {
        code: Some(1),
        stdout: String::new(),
        stderr: "Error: make coverage failed with exit code Some(2)".into(),
        timed_out: false,
    };
    assert!(
        matches!(compare_probes(&died(), &died()), Verdict::Skipped(_)),
        "identical failures before the formatter ran are a fixture verdict, not a flag verdict"
    );

    // A gate that fails differently at two values is honouring the option.
    let fail_a = super::Observable {
        code: Some(1),
        stdout: "dead code 3.8% exceeds threshold of 1.0%".into(),
        stderr: String::new(),
        timed_out: false,
    };
    let ok_b = super::Observable {
        code: Some(0),
        stdout: "dead code 3.8% within threshold of 50%".into(),
        stderr: String::new(),
        timed_out: false,
    };
    assert_eq!(compare_probes(&fail_a, &ok_b), Verdict::Effective);
    // Two runs that agree and *worked* are the real no-op: the option was
    // honoured by a command that had every chance to act on it.
    assert_eq!(compare_probes(&ok_b, &ok_b), Verdict::NoOp);
    // And a failing command that still printed its report is a usable control.
    // `analyze lint-hotspot`, `quality-gate` and `enforce extreme` all exit 1
    // on the defect-rich corpus; skipping on exit code alone would excuse
    // every no-op flag on every quality gate that is doing its job.
    assert_eq!(
        compare_probes(&fail_a, &fail_a),
        Verdict::NoOp,
        "a non-zero exit with a rendered report is still a control"
    );
}

/// A command that declines its own precondition, or that this build compiled
/// out, is not a control — every flag under it compares equal for a reason
/// that has nothing to do with the flags.
#[test]
fn declined_preconditions_are_skipped_not_blamed() {
    let obs = |out: &str, err: &str, code: i32| super::Observable {
        code: Some(code),
        stdout: out.into(),
        stderr: err.into(),
        timed_out: false,
    };
    for (o, e, code, what) in [
        (
            "",
            "Error: Dashboard requires the 'http-server' feature. Rebuild with: cargo build --features http-server",
            1,
            "feature-gated subcommand",
        ),
        (
            "Cross-crate analysis requires at least 2 crates.\nDiscovery priority: ...",
            "",
            0,
            "single-crate corpus, exits 0 so it looks healthy",
        ),
        ("No TDG history found matching criteria.", "", 0, "empty result set"),
        (
            "",
            "Error: make coverage failed\nmake: *** No rule to make target 'coverage'.  Stop.",
            1,
            "missing Makefile target",
        ),
        (
            "",
            "Error: Clippy failed: error: could not compile `corpus` (lib)",
            1,
            "fixture does not compile",
        ),
    ] {
        assert!(
            baseline_unusable(&obs(o, e, code)).is_some(),
            "must be skipped, not blamed on its flags: {what}"
        );
    }
}

/// A flag whose own help says it REWRITES a tracked file must never be swept.
///
/// `comply ratchet --lower` was being swept, and booked as a no-op. It is one
/// only by accident: no corpus carries a `.pmat-ratchet.toml`, so the handler
/// bails before it writes. Probes run with the corpus as cwd, so this never
/// reached THIS repository — but the corpus is a thing we add files to, and the
/// first one to carry a ratchet file would have its baselines silently lowered
/// with the damage recorded as "changes nothing".
///
/// The two halves are asserted together so neither can drift alone: the help
/// must still describe a rewrite, AND the flag must be denied. If `--lower`
/// ever stops mutating, this fails and says so, rather than leaving a flag
/// permanently over-denied for a reason that expired.
#[test]
fn a_flag_that_rewrites_a_tracked_file_is_never_swept() {
    let cwd = std::env::current_dir().expect("cwd");
    let help = help_for(&["comply", "ratchet"], &cwd)
        .expect("`pmat comply ratchet --help` must be readable");

    assert!(
        help.contains("rewritten") || help.contains("rewrite"),
        "--lower's help no longer describes a rewrite. Re-check whether it still \
         mutates before leaving it in DENY_FLAGS:\n{help}"
    );
    assert!(
        DENY_FLAGS.contains(&"--lower"),
        "a flag whose own help says it rewrites a tracked file must be in \
         DENY_FLAGS, or the sweep will invoke it against whatever tree it is \
         pointed at"
    );
}

/// Every allow-listed no-op must name a demonstrated effect, and none may
/// shadow a command the sweep never reaches.
#[test]
fn allowed_noops_name_a_real_effect() {
    for (path, flag, why) in ALLOWED_NOOPS {
        assert!(
            flag.starts_with("--"),
            "{path} {flag}: the flag must be the long form"
        );
        assert!(
            why.len() > 60,
            "{path} {flag}: the reason must name the code path and the input or \
             format under which the flag was observed working, not assert inertness: {why:?}"
        );
        assert!(
            !DENY_PATHS.iter().any(|(p, _)| p == path),
            "{path} {flag}: allow-listing a flag on a command the sweep never runs \
             hides it instead of documenting it"
        );
    }
    let mut seen = BTreeSet::new();
    for (path, flag, _) in ALLOWED_NOOPS {
        assert!(
            seen.insert((*path, *flag)),
            "{path} {flag} is allow-listed twice"
        );
    }
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

    // Debt that is *not* a canonical marker, so `analyze satd --strict` has
    // something to narrow. With only TODO/FIXME/HACK/XXX present, strict and
    // non-strict returned the identical 63 violations.
    let satd = read("src/satd_00.rs");
    assert!(
        satd.contains("temporary workaround") && satd.contains("code smell"),
        "corpus must carry non-canonical debt phrasing, or --strict cannot be \
         distinguished from the default"
    );

    // Debt inside the test file, so `--include-tests` has something to
    // include: with a clean tests/basic.rs the flag changed nothing and was
    // indistinguishable from a flag that is never read.
    assert!(
        read("tests/basic.rs").contains("TODO"),
        "the corpus test file must carry its own debt"
    );

    // A dependency chain, so anything that walks edges has depth to walk.
    // Without it the corpus graph is a star: `analyze dag --max-depth 1` and
    // `--max-depth 50` rendered byte-identical output.
    assert!(
        read("src/chain_00.rs").contains("use crate::chain_01;")
            && read("src/chain_04.rs").contains("use crate::chain_05;"),
        "corpus must contain a multi-hop `use crate::` chain"
    );

    // `i * 0` is clippy's deny-by-default erasing_op; with it the fixture did
    // not compile under clippy and every command that shells out to clippy
    // returned Err before reading a flag.
    assert!(
        !dead.contains("* 0)"),
        "corpus must not trip deny-by-default clippy lints, or clippy-backed \
         commands fail before they read their own flags"
    );

    // WebAssembly, AssemblyScript and model inputs: without them those
    // commands report "Found 0 files" and every flag under them compares equal
    // because there was nothing on disk to act on.
    let wasm = std::fs::read(root.join("mod.wasm")).unwrap_or_default();
    assert_eq!(
        &wasm[..4],
        b"\0asm",
        "mod.wasm must carry valid wasm magic, or the binary reader rejects it"
    );
    assert!(wasm.len() > 40 && wasm.windows(3).any(|w| w == b"env"));
    assert!(
        std::fs::read(root.join("broken.wasm"))
            .unwrap_or_default()
            .starts_with(b"NOTWASM!"),
        "the corpus must also contain the malformed input the format validator exists to reject"
    );
    assert!(
        read("mod.wat").contains("(module") && read("two.wat").contains("(memory 1)"),
        "WAT sources must parse and one must declare memory"
    );
    assert!(
        read("assembly/mem.ts").contains("memory.grow"),
        "AssemblyScript fixture must contain the construct --memory-analysis claims to report"
    );
    assert!(
        read("assembly/index.ts").contains("i32") && read("extra.as").contains("i32"),
        "three AssemblyScript sources must be present (.ts and .as)"
    );
    let gguf = std::fs::read(root.join("models/tiny.gguf")).unwrap_or_default();
    assert!(
        gguf.starts_with(b"GGUF") && gguf.len() >= 24,
        "a readable GGUF header (magic + version + counts) must exist"
    );
    assert!(
        std::fs::read(root.join("models/tiny.apr"))
            .unwrap_or_default()
            .starts_with(b"APR2"),
        "a readable APR model must exist"
    );
    assert!(
        std::fs::read(root.join("models/tiny.safetensors"))
            .unwrap_or_default()
            .len()
            > 8,
        "a readable safetensors model must exist"
    );
    assert!(
        std::fs::read(root.join("models/garbage.gguf"))
            .unwrap_or_default()
            .starts_with(b"NOTAGGUF"),
        "a .gguf whose header does not parse must exist, or --check's unreadable-header \
         path is never taken"
    );
    assert!(
        !root.join("models/README.md").exists() && !root.join("models/tokenizer.json").exists(),
        "the model directory must lack a model card and tokenizer, or two of the three \
         --check findings never fire"
    );

    // `comply enforce` and every other hook-aware command define "is this a
    // git repository" as `.git/hooks` existing; `git init --template=` does
    // not create it.
    assert!(
        root.join(".git/hooks").is_dir(),
        "corpus .git must have a hooks directory, or hook-aware commands abort \
         before reading any flag"
    );

    // `main` must lag `HEAD`, or `analyze incremental-coverage` (which defaults
    // to `--base-branch main`) analyses 0 changed files and every flag under it
    // is compared against an empty result set.
    let changed = std::process::Command::new("git")
        .args(["diff", "--name-only", "main...HEAD"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);
    assert!(
        changed >= 8,
        "main..HEAD must contain changed files (got {changed}) — the second commit \
         has to land on a branch"
    );

    // One Critical-risk file, left uncommitted: `analyze comprehensive` only
    // emits the "Focus on N high-risk files" recommendation that
    // --confidence-threshold and --min-lines filter when defect-prediction
    // ranks a file High or Critical.
    let critical = read("src/critical.rs");
    let lines = critical.lines().count();
    assert!(
        (900..1000).contains(&lines),
        "critical.rs is {lines} lines; it must sit under the 1000-line probe so \
         --min-lines has a value that admits it and one that does not"
    );
    assert!(
        critical.matches("use std::").count() >= 20 && critical.matches("if ").count() >= 100,
        "critical.rs must be import-heavy and branch-heavy enough to be ranked Critical"
    );
    let tracked = std::process::Command::new("git")
        .args(["ls-files", "src/critical.rs"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    assert!(
        tracked.is_empty(),
        "critical.rs must stay uncommitted: with churn history its prediction \
         confidence saturates and a confidence threshold has no boundary to cross"
    );

    assert_large_corpus_spans_the_gate_a_inputs(root, &read);
}

/// The inputs added for the differential gate's 49 corpus gaps.
///
/// Split out only for length. Every assertion here pins a fixture that some
/// numeric leaf's *responsiveness* depends on: an unasserted fixture rots
/// silently, and the corpus has lost content that way before. Each one names
/// the leaf it feeds, so a failure says what breaks rather than only what
/// changed.
fn assert_large_corpus_spans_the_gate_a_inputs(root: &Path, read: &dyn Fn(&str) -> String) {
    // score :: sub_scores.file_health — `(1 - files_over_1000 / total_rs) * 100`,
    // and critical.rs is deliberately pinned under 1000 lines just above.
    let huge = read("src/huge.rs").lines().count();
    assert!(
        huge > 1000,
        "src/huge.rs is {huge} lines; file_health only moves off 100.0 when a \
         file exceeds 1000 lines"
    );

    // analyze tdg / build-tdg :: f_grade_count, grade_capped — TDG's penalty is
    // 69.9 * 0.6^(n-1), so ONE critical defect can never reach F. Exactly one
    // file may carry three, or the average falls under 80 and grade_capped (which
    // needs a good average *and* an F) stops being reachable.
    let f_grade_file = read("src/faults_00.rs").matches(".unwrap()").count();
    assert!(
        f_grade_file >= 3,
        "src/faults_00.rs has {f_grade_file} unwrap() calls; F needs at least 2 \
         critical defects and the fixture carries 3"
    );
    assert_eq!(
        read("src/faults_01.rs").matches(".unwrap()").count(),
        1,
        "only faults_00 may be F-grade: more F files pull the average under 80 \
         and grade_capped can never become true"
    );
    assert!(
        !read("src/unproven_00.rs").contains(".unwrap()"),
        "the unproven_* family must use .expect(), which TDG does not count as a \
         critical defect — with unwraps every one of its 40 files auto-fails to F"
    );

    // quality-gate :: results.provability_violations — the check averages the
    // first 50 functions walkdir yields and fires under 0.70. Each of these
    // scores 0.20: raw pointer (nullability + aliasing), .expect() with no `?`
    // (bounds), println! (purity).
    let unproven = read("src/unproven_39.rs");
    assert!(
        unproven.contains("*const u8")
            && unproven.contains("println!")
            && unproven.contains(".expect("),
        "unproven_* must be low on all four provability properties at once"
    );
    assert!(
        !unproven.contains("unsafe"),
        "the raw pointer must never be dereferenced: clippy::not_unsafe_ptr_arg_deref \
         is deny-by-default and a corpus that fails clippy takes every \
         clippy-backed command down with it"
    );

    // analyze tdg :: ungraded_files[] — sources in languages this build cannot
    // grade, so the "not part of the score" disclosure has entries.
    assert!(
        root.join("src/deploy.sh").exists() && root.join("src/mod.zig").exists(),
        "corpus must carry ungradable-language sources (.sh, .zig)"
    );

    // analyze proof-annotations :: summary.files_not_analyzed. Deliberately not
    // declared in lib.rs: rustc never sees it, so cargo check still succeeds.
    assert!(
        read("src/broken_syntax.rs").contains("this is not rust"),
        "corpus must carry a file the AST walkers cannot parse"
    );
    assert!(
        !read("src/lib.rs").contains("pub mod broken_syntax;"),
        "broken_syntax.rs must stay out of the module tree, or the whole corpus \
         stops compiling and every clippy-backed command fails first"
    );

    // analyze duplicates :: file_statistics ./src/lib.rs — lib.rs is the one
    // file all three corpora have, so it is the only per-file duplication row
    // they can be compared on.
    let lib = read("src/lib.rs");
    assert!(
        lib.contains("pub mod inline_dup_a") && lib.contains("pub mod inline_dup_b"),
        "lib.rs must contain its own clone pair"
    );

    // analyze big-o :: pattern_matches[], distribution.O(n log n), and
    // summary.high_complexity_truncated (which needs findings past --top-files 10).
    let searching = read("src/searching.rs");
    assert!(
        searching.contains(".sort()") && searching.contains(".binary_search("),
        "corpus must contain the sorting and binary-search patterns the matcher names"
    );
    assert!(
        searching.contains("fib(n - 1) + fib(n - 2)") && searching.contains("hi - lo"),
        "corpus must contain an exponential and a logarithmic algorithm — both \
         buckets stay empty, which is recorded in ALLOWED_CONSTANTS as a \
         classifier gap and must stay falsifiable"
    );
    let superlinear = (0..14)
        .filter(|i| root.join(format!("src/superlinear_{i:02}.rs")).exists())
        .count();
    assert!(
        superlinear >= 11,
        "only {superlinear} superlinear files; high_complexity_truncated needs \
         findings in more than --top-files (default 10) files"
    );

    // quality-gate :: results.dead_code_violations — the per-file warning walks
    // an UNORDERED `files_with_dead_code.iter().take(5)`, so every dead file has
    // to clear 20%, not just the worst one.
    let dead_heavy = read("src/dead_heavy.rs");
    assert!(
        dead_heavy.matches("fn buried_").count() >= 20,
        "dead_heavy.rs must be overwhelmingly dead"
    );
    assert!(
        read("src/dead_00.rs").contains("fn spare_00_a"),
        "every dead_* file needs its extra dead items to clear the 20% per-file \
         threshold; only the worst file clearing it is a coin flip"
    );

    // quality-gate :: results.security_violations — check_security reads only
    // the top level (fs::read_dir, no recursion), so a file under src/ is
    // invisible to it.
    let leak = read("leak.rs");
    assert!(
        leak.contains("password") && leak.contains("api_key"),
        "the hardcoded-secret fixture must sit at the repository root, where the \
         non-recursive security scan can see it"
    );

    // repo-score :: repository_hygiene / pmat_compliance / precommit_hooks.
    for cruft in [".DS_Store", "backup.bak", "old.orig", "scratch.tmp"] {
        assert!(
            root.join(cruft).exists(),
            "cruft fixture {cruft} missing; repository_hygiene C1 scores 5.0/5.0 without it"
        );
    }
    assert!(
        root.join(".idea/workspace.xml").exists() && root.join(".vscode/settings.json").exists(),
        "team-specific files missing; repository_hygiene C2 scores full marks without them"
    );
    let blob = std::fs::metadata(root.join("bigblob.dat"))
        .map(|m| m.len())
        .unwrap_or(0);
    assert!(
        blob > 1_000_000,
        "bigblob.dat is {blob} bytes; C3 deducts only for blobs over 1MB in git history"
    );
    let in_history = std::process::Command::new("git")
        .args(["ls-files", "bigblob.dat"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    assert!(
        !in_history.is_empty(),
        "bigblob.dat must be COMMITTED — C3 pipes `git rev-list --objects HEAD`, \
         so an uncommitted blob is invisible to it"
    );
    assert!(
        read(".pmat-gates.toml").contains("[complexity]"),
        "pmat_compliance F1/F2 both key off .pmat-gates.toml"
    );
    let hook = root.join(".git/hooks/pre-commit");
    assert!(
        read(".git/hooks/pre-commit").contains("clippy"),
        "precommit_hooks B2 grades the hook's contents"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&hook)
            .map(|m| m.permissions().mode())
            .unwrap_or(0);
        assert!(
            mode & 0o111 != 0,
            "the pre-commit hook must be executable, or B1 scores 5.0 instead of 10.0"
        );
    }

    // comply :: versions_behind, breaking_changes[], recommendations[],
    // is_compliant — all four move only for a project pinned to an older pmat.
    assert!(
        read(".pmat/project.toml").contains("version = \"2.0.0\""),
        "the comply surface has no other axis: an unpinned project is 0 versions \
         behind with nothing breaking, truthfully"
    );

    // quality-gate :: results.coverage_violations — the large corpus is the
    // below-floor branch; empty carries the passing report, tiny none at all.
    assert!(
        read(".pmat/coverage-cache.json").contains("\"1\":1"),
        "the below-floor coverage report must be a hit map, so the check does the \
         percentage arithmetic itself rather than echoing a number"
    );
    let lcov = read("target/coverage/lcov.info");
    assert!(
        lcov.contains(&format!("SF:{}", root.join("src/lib.rs").display())),
        "the lcov artifact must use absolute SF: paths, as cargo-llvm-cov writes \
         them — a ./-prefixed artifact would paper over the key mismatch that \
         keeps analyze incremental-coverage on the skip list"
    );

    // analyze churn :: summary.stable_files[] and analyze bottleneck ::
    // couplings[] — one hot pair against a quiet tree, over enough commits that
    // 0.6/max_commits fits under the 0.10 stable threshold.
    assert!(
        super::commit_count(root) >= 12,
        "large corpus has {} commits; a file touched once cannot score under 0.10 \
         until max_commits exceeds 6",
        super::commit_count(root)
    );

    // analyze assembly-script / web-assembly :: files_truncated — needs more
    // reportable files than --top-files' default of 10.
    let wasm_modules = (0..10)
        .filter(|i| root.join(format!("wasm/mod_{i:02}.wasm")).exists())
        .count();
    assert_eq!(
        wasm_modules, 10,
        "ten extra valid wasm modules are what push the report past --top-files"
    );
    let as_modules = (0..9)
        .filter(|i| root.join(format!("assembly/mod_{i:02}.ts")).exists())
        .count();
    assert_eq!(
        as_modules, 9,
        "nine extra AssemblyScript sources, same reason"
    );

    // score :: rps_categories."Dependency Health" and "Performance & Benchmarking"
    // are read straight out of the manifest — feature-flag tiers and a [[bench]]
    // section with harness = false.
    let manifest = read("Cargo.toml");
    assert!(
        manifest.contains("[features]"),
        "the large corpus manifest must declare feature flags, or Dependency \
         Health scores the same 5/12 for every project"
    );
    assert!(
        manifest.contains("[[bench]]") && manifest.contains("harness = false"),
        "the large corpus manifest must declare a custom-harness benchmark"
    );
    assert!(
        root.join("benches/throughput.rs").exists(),
        "a declared [[bench]] with no file makes cargo refuse the manifest, which \
         takes every cargo-backed command down with it"
    );
    assert!(
        read(".github/workflows/ci.yml").contains("cargo bench"),
        "3 of the 10 benchmarking points are for running benchmarks in CI"
    );
    // score :: rps_categories."Known Defects" is `20 - 5 * (unwraps / 100)`, and
    // "GPU/SIMD Quality" is N/A until a GPU file sits at the TOP LEVEL.
    let unwraps = read("src/unwrap_heavy.rs").matches(".unwrap()").count();
    assert!(
        unwraps >= 100,
        "src/unwrap_heavy.rs has {unwraps} unwraps; Known Defects does not move          below 100, and they must stay in ONE file or the F grades they create          drag the TDG average under 80 and grade_capped stops being reachable"
    );
    assert!(
        read("kernel.cu").contains("__syncthreads"),
        "the CUDA kernel must sit at the repository root: has_gpu_simd_code reads          only the top level, so the same file one directory down leaves the          category N/A"
    );
    assert!(
        read(".pmat-metrics/coverage.result").contains("coverage_pct"),
        "`pmat score` reads coverage from .pmat-metrics/coverage.result, which is \
         neither of the two files quality-gate reads"
    );
}

/// The empty and tiny corpora must stay poor.
///
/// The differential invariant is a *contrast*, so every input added to the large
/// corpus for it is only worth anything while the other two lack it. This is the
/// other half of `large_corpus_contains_every_defect_family`: it fails if a
/// future fixture is added to `build_corpus` unconditionally.
#[test]
fn small_corpora_stay_poor_so_the_contrast_survives() {
    for size in [CorpusSize::Empty, CorpusSize::Tiny] {
        let corpus = build_corpus(size);
        let root = corpus.path();
        for absent in [
            ".pmat-gates.toml",
            ".pmat/project.toml",
            ".git/hooks/pre-commit",
            "bigblob.dat",
            ".DS_Store",
            ".idea/workspace.xml",
            "leak.rs",
            "src/huge.rs",
            "src/broken_syntax.rs",
            "src/deploy.sh",
            "src/unproven_00.rs",
            "src/dead_heavy.rs",
            "models/tiny.gguf",
            "mod.wasm",
            "assembly/index.ts",
            "target/coverage/lcov.info",
            "benches/throughput.rs",
            "kernel.cu",
            "src/unwrap_heavy.rs",
            "src/nested.rs",
            ".pmat-metrics/coverage.result",
        ] {
            assert!(
                !root.join(absent).exists(),
                "{} corpus contains {absent}; the gate compares the three and an \
                 input present everywhere proves nothing",
                size.name()
            );
        }
    }
    // The one deliberate exception, and why: `results.passed` needs a project
    // that passes, and `coverage_violations` needs the above-floor branch.
    let empty = build_corpus(CorpusSize::Empty);
    assert!(
        empty.path().join(".pmat-metrics/coverage.json").exists(),
        "the empty corpus carries the passing coverage report — without it no \
         corpus passes quality-gate and results.passed is constant false"
    );
    let tiny = build_corpus(CorpusSize::Tiny);
    assert!(
        !tiny.path().join(".pmat-metrics/coverage.json").exists(),
        "tiny must stay reportless, or the not-measured branch is never exercised"
    );
}

/// PMAT-688: the 3.39.0 sweep booked four "errors out" rows that were the
/// ledger writers refusing a dirty tree (documented: "Commit or stash first,
/// or pass --allow-dirty") and clap refusing `--allow-dirty` without the
/// `--write-ledger` it `requires`. A refusal that names its precondition is
/// the honest alternative to silently doing nothing — the same class as
/// "--ml is not implemented", not the "flag broke a working command" class.
#[test]
fn precondition_refusals_are_honest() {
    let dirty = "Error: refusing to write .pmat/reachability-ledger.json from a dirty git \
                 tree: it would record whatever these uncommitted, possibly-unrelated changes \
                 happen to add or remove, not the tree at HEAD. Commit or stash first, or pass \
                 --allow-dirty.\n M src/lib.rs\n";
    let requires = "error: the following required arguments were not provided:\n  \
                    --write-ledger\n\nUsage: pmat analyze reachability --write-ledger \
                    --allow-dirty\n";
    assert!(
        is_honest_refusal(dirty),
        "a documented precondition refusal is honest"
    );
    assert!(
        is_honest_refusal(requires),
        "clap naming the missing companion flag is honest"
    );
    assert!(
        !is_honest_refusal("Error: called `Option::unwrap()` on a `None` value"),
        "a crash is not a refusal"
    );
    assert!(
        !is_honest_refusal("error: failed to parse config: missing field `gates`"),
        "an unrelated error: line is not a refusal"
    );
}

/// PMAT-688: three flags were booked as no-ops because the corpus carried
/// nothing for them to act on. `validate-docs --fail-on-error` needs a broken
/// link; `roadmap todos` needs a roadmap with a current sprint (its baseline
/// failed on the corpus with the 🔄 banner already on stdout, so the
/// "not a control" guard never fired and both `--color` and
/// `--include-quality-gates` were booked against a failing command);
/// `show-metrics --failures-only` needs a trend store holding one regressing
/// and one improving series inside its 30-day window.
#[test]
fn large_corpus_carries_the_flag_fixtures() {
    let corpus = build_corpus(CorpusSize::Large);
    let root = corpus.path();

    let links = std::fs::read_to_string(root.join("docs/links.md")).expect("docs/links.md");
    assert!(
        links.contains("](design-notes.md)") && !root.join("docs/design-notes.md").exists(),
        "one relative link to a file that does not exist:\n{links}"
    );

    // `popper-score --failures-only` hides a category at or above its 80% pass
    // line; the corpus had none (best: Transparency at 70%). An ADR directory
    // is the scorer's 4-point lever (transparency.rs C3) and lifts it to 90%.
    let adr = root.join("docs/adr");
    assert!(
        adr.is_dir() && std::fs::read_dir(&adr).expect("docs/adr").next().is_some(),
        "docs/adr/ with at least one decision record"
    );

    let roadmap = std::fs::read_to_string(root.join("docs/execution/roadmap.md"))
        .expect("docs/execution/roadmap.md");
    assert!(
        roadmap.contains("## Current Sprint:") && roadmap.contains("| ID | Description |"),
        "a current sprint with the task table the parser reads:\n{roadmap}"
    );
    assert_eq!(
        roadmap.matches("| PMAT-").count(),
        2,
        "two sprint tasks:\n{roadmap}"
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;
    for (metric, rising) in [("lint", true), ("test-fast", false)] {
        let raw = std::fs::read_to_string(root.join(format!(".pmat-metrics/trends/{metric}.json")))
            .unwrap_or_else(|e| panic!(".pmat-metrics/trends/{metric}.json: {e}"));
        let obs: Vec<serde_json::Value> = serde_json::from_str(&raw).expect("observations");
        assert!(obs.len() >= 6, "{metric}: {} observations", obs.len());
        let vals: Vec<f64> = obs
            .iter()
            .map(|o| o["value"].as_f64().expect("value"))
            .collect();
        let monotone = vals
            .windows(2)
            .all(|w| if rising { w[1] > w[0] } else { w[1] < w[0] });
        assert!(
            monotone,
            "{metric} must be strictly {}: {vals:?}",
            if rising { "rising" } else { "falling" }
        );
        let newest = obs
            .iter()
            .map(|o| o["timestamp"].as_i64().expect("timestamp"))
            .max()
            .expect("at least one observation");
        assert!(
            now - newest < 86_400 * 30,
            "{metric}: the newest observation must fall inside show-metrics' 30-day window"
        );
    }
}
