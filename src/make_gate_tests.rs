//! PMAT-1365 — `make gate` is declared, and its table cannot quietly stop covering CI.
//!
//! paiml-implement discovers a repository's gate by probing `make -n gate`. pmat had
//! no such target, so discovery fell back to `cargo test --workspace` — a guess that
//! runs none of the status contexts master requires, and whose green was trusted.
//! `make gate` now runs `scripts/gate.sh`, and `scripts/gate-control.sh` proves each of
//! its properties can fail. That control needs `make` and PyYAML, and `make gate`
//! itself is not a CI job, so neither reaches a required check.
//!
//! These tests are the half that does: `cargo test --lib` runs inside `ci / gate`. They
//! re-read the table with this crate's own YAML parser — an independent reading, not a
//! second runner — and pin the five things a later edit could remove while every other
//! test stayed green:
//!
//!   1. the Makefile target exists and runs the table;
//!   2. every required context has a row, and every CI-only row says why;
//!   3. every `step` row still names exactly one runnable step in its workflow;
//!   4. no CI-only row blames cost for a check that runs nothing but the pmat binary,
//!      which `make gate` already builds (its `build-pmat` leg);
//!   5. no CI-only row blames a credential that is a GitHub token `gh auth token` supplies;
//!   6. no leg runs a hand-written `target/debug/pmat`: `cmd` rows run `$PMAT_BIN`, the
//!      executable cargo reports building, and the one spelling `step` text may use is the
//!      one `scripts/gate.sh` rewrites to it.
//!
//! Contract: `contracts/make-gate-v1.yaml`.

use serde_yaml_ng::Value;
use std::fs;
use std::path::PathBuf;

const SCRIPT: &str = "scripts/gate.sh";
const MARKER: &str = "── EXTENSION POINT";
const CATEGORIES: &[&str] = &[
    "platform",
    "credential",
    "cost",
    "trigger",
    "not-gating",
    "not-required",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    let text = fs::read_to_string(&p);
    assert!(text.is_ok(), "{} missing: {:?}", p.display(), text.err());
    text.unwrap_or_default()
}

struct Row {
    kind: String,
    contexts: Vec<String>,
    leg: String,
    source: String,
    note: String,
    command: String,
}

/// The `REQUIRED_CONTEXTS=( ... )` block of the script.
fn required_contexts(script: &str) -> Vec<String> {
    script
        .lines()
        .skip_while(|l| !l.starts_with("REQUIRED_CONTEXTS=("))
        .skip(1)
        .take_while(|l| l.trim() != ")")
        .map(|l| l.trim().trim_matches('"').to_string())
        .collect()
}

/// The rows of the heredoc in `legs_table`, split exactly as the script splits them:
/// six `|` fields, the last one keeping any further `|`.
fn rows(script: &str) -> Vec<Row> {
    script
        .lines()
        .skip_while(|l| l.trim() != "cat <<'LEGS'")
        .skip(1)
        .take_while(|l| l.trim() != "LEGS")
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|l| {
            let f: Vec<&str> = l.splitn(6, '|').map(str::trim).collect();
            assert_eq!(f.len(), 6, "{SCRIPT}: row does not have six fields: {l}");
            Row {
                kind: f[0].to_string(),
                contexts: f[1].split(';').map(|c| c.trim().to_string()).collect(),
                leg: f[2].to_string(),
                source: f[3].to_string(),
                note: f[4].to_string(),
                command: f[5].to_string(),
            }
        })
        .collect()
}

#[test]
fn make_gate_is_declared_and_runs_the_table() {
    let makefile = read("Makefile");
    let lines: Vec<&str> = makefile.lines().collect();
    let at = lines.iter().position(|l| l.starts_with("gate:"));
    assert!(
        at.is_some(),
        "the Makefile declares no `gate:` target, so discovery falls back to a guess"
    );
    let at = at.unwrap_or_default();
    let recipe: Vec<&str> = lines[at + 1..]
        .iter()
        .take_while(|l| l.starts_with('\t'))
        .copied()
        .collect();
    assert!(
        recipe.iter().any(|l| l.contains(SCRIPT)),
        "`make gate` must run {SCRIPT}, the one table of what the gate covers; recipe: {recipe:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with(".PHONY:") && l.split_whitespace().any(|t| t == "gate")),
        "`gate` must be .PHONY, or a file named `gate` would silently satisfy it"
    );
}

#[test]
fn every_required_context_has_a_row_and_every_ci_only_row_a_reason() {
    let script = read(SCRIPT);
    let required = required_contexts(&script);
    assert!(
        required.len() >= 6,
        "REQUIRED_CONTEXTS parsed to {required:?} — the parser or the block is broken"
    );
    let rows = rows(&script);
    assert!(
        rows.iter().any(|r| r.kind == "step" || r.kind == "cmd"),
        "no row runs anything"
    );
    for c in &required {
        assert!(
            rows.iter().any(|r| r.contexts.iter().any(|x| x == c)),
            "required context `{c}` has no row in {SCRIPT}: it would be neither run nor named"
        );
    }
    for r in rows.iter().filter(|r| r.kind == "ci-only") {
        let (category, reason) = r.note.split_once(':').unwrap_or(("", ""));
        assert!(
            CATEGORIES.contains(&category) && !reason.trim().is_empty(),
            "CI-only row `{}` must say `<category>: <reason>`, got `{}`",
            r.leg,
            r.note
        );
    }
    assert!(
        script.contains(MARKER),
        "the extension-point marker is gone from {SCRIPT}; sibling gates append rows after it"
    );
}

#[test]
fn every_step_leg_names_one_runnable_step_in_its_workflow() {
    let script = read(SCRIPT);
    let steps: Vec<Row> = rows(&script)
        .into_iter()
        .filter(|r| r.kind == "step")
        .collect();
    assert!(!steps.is_empty(), "no step rows parsed");
    for r in &steps {
        let parts: Vec<&str> = r.source.splitn(3, '#').collect();
        assert_eq!(
            parts.len(),
            3,
            "step `{}`: source is not <workflow>#<job>#<step>",
            r.leg
        );
        let (wf, job, name) = (parts[0], parts[1], parts[2]);
        let doc = serde_yaml_ng::from_str::<Value>(&read(wf));
        assert!(
            doc.is_ok(),
            "step `{}`: {wf} does not parse: {:?}",
            r.leg,
            doc.as_ref().err()
        );
        let doc = doc.unwrap_or_default();
        let job_v = doc.get("jobs").and_then(|j| j.get(job));
        assert!(job_v.is_some(), "step `{}`: {wf} has no job `{job}`", r.leg);
        let null = Value::Null;
        let job_v = job_v.unwrap_or(&null);
        let hits: Vec<&Value> = job_v
            .get("steps")
            .and_then(Value::as_sequence)
            .map(|s| {
                s.iter()
                    .filter(|st| st.get("name").and_then(Value::as_str) == Some(name))
                    .collect()
            })
            .unwrap_or_default();
        assert_eq!(
            hits.len(),
            1,
            "step `{}`: {wf} job `{job}` has {} steps named `{name}` — renamed or removed in CI?",
            r.leg,
            hits.len()
        );
        let step = hits[0];
        let run = step.get("run").and_then(Value::as_str).unwrap_or("");
        assert!(
            !run.trim().is_empty(),
            "step `{}`: `{name}` has no run: script",
            r.leg
        );
        for key in ["if", "env", "working-directory"] {
            assert!(
                step.get(key).is_none(),
                "step `{}`: `{name}` has `{key}:`, which only GitHub Actions can honour",
                r.leg
            );
        }
        assert!(
            matches!(
                step.get("shell").and_then(Value::as_str),
                None | Some("bash")
            ),
            "step `{}`: `{name}` runs under a non-bash shell",
            r.leg
        );
        assert!(
            !run.contains("${{"),
            "step `{}`: `{name}` uses a GitHub expression",
            r.leg
        );
    }
}

/// Setup actions a CI job may use without doing any checking of its own.
const SETUP_ACTIONS: &[&str] = &[
    "actions/checkout@",
    "dtolnay/rust-toolchain@",
    "Swatinem/rust-cache@",
];

/// True when `job` does nothing but build pmat and run it: every `uses:` step is setup,
/// every `run:` step is a one-line `cargo build … --bin pmat` or `cargo run … --bin pmat -- …`
/// with no `env:`/`if:`, at least one of them runs it, and the job sets no env or matrix.
fn runs_only_pmat(job: &Value) -> bool {
    if ["env", "strategy", "services", "container"]
        .iter()
        .any(|k| job.get(*k).is_some())
    {
        return false;
    }
    let steps = job
        .get("steps")
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mut runs_pmat = false;
    for step in &steps {
        if let Some(uses) = step.get("uses").and_then(Value::as_str) {
            if !SETUP_ACTIONS.iter().any(|a| uses.starts_with(a)) {
                return false;
            }
            continue;
        }
        let run = step.get("run").and_then(Value::as_str).unwrap_or("").trim();
        if run.is_empty()
            || run.contains('\n')
            || step.get("env").is_some()
            || step.get("if").is_some()
        {
            return false;
        }
        let builds = run.starts_with("cargo build ") && run.contains(" --bin pmat");
        let runs = run.starts_with("cargo run ") && run.contains(" --bin pmat -- ");
        if !builds && !runs {
            return false;
        }
        runs_pmat |= runs;
    }
    runs_pmat
}

/// The CI jobs behind every CI-only row of `category` whose workflow is in this tree, as
/// `(leg, "<workflow> job `<job>`", job)`. A row's source is `<workflow> <job id> ...`.
/// Rows whose workflow lives elsewhere (sovereign-ci.yml is in paiml/.github) are skipped;
/// a row naming an in-tree workflow but no job in it fails.
fn ci_only_jobs(category: &str) -> Vec<(String, String, Value)> {
    let script = read(SCRIPT);
    let mut jobs = Vec::new();
    for r in rows(&script)
        .iter()
        .filter(|r| r.kind == "ci-only" && r.note.split_once(':').map(|c| c.0) == Some(category))
    {
        let mut words = r.source.split_whitespace();
        let (wf, job) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
        let wf = if wf.contains('/') {
            wf.to_string()
        } else {
            format!(".github/workflows/{wf}")
        };
        let Ok(text) = fs::read_to_string(repo_root().join(&wf)) else {
            continue;
        };
        let doc = serde_yaml_ng::from_str::<Value>(&text).unwrap_or_default();
        let job_v = doc.get("jobs").and_then(|j| j.get(job)).cloned();
        assert!(
            job_v.is_some(),
            "CI-only row `{}`: source `{}` names no job `{job}` in {wf}",
            r.leg,
            r.source
        );
        jobs.push((
            r.leg.clone(),
            format!("{wf} job `{job}`"),
            job_v.unwrap_or_default(),
        ));
    }
    jobs
}

/// `unrun-tests` and `reachability-ledger` were CI-only rows reading "cost: a release build
/// of pmat". Neither CI job does anything but build pmat and run one of its subcommands,
/// and the gate already builds pmat: on the debug binary the two checks take 14s and 1.5s.
/// The reason was never measured, and the reachability leg went red in CI on this branch's
/// own new file while `make gate` read green (PR #1368, feature-gate on 1d8c64f40).
#[test]
fn no_ci_only_cost_row_hides_a_check_that_only_runs_pmat() {
    let jobs = ci_only_jobs("cost");
    assert!(
        jobs.len() >= 5,
        "only {} CI-only cost rows resolved to a job in this tree — the source parser is broken",
        jobs.len()
    );
    let hiding: Vec<String> = jobs
        .iter()
        .filter(|(_, _, job)| runs_only_pmat(job))
        .map(|(leg, at, _)| format!("{leg} ({at})"))
        .collect();
    assert!(
        hiding.is_empty(),
        "CI-only rows blame cost for a CI job that only builds and runs pmat, which `make gate` \
         already builds (leg build-pmat) — make each a cmd row running `cargo run --locked --bin pmat --`: {hiding:?}"
    );
}

/// True when a step of `job`, or the job itself, hands a GitHub token to its commands.
fn reads_github_with_a_token(job: &Value) -> bool {
    let has_token = |v: &Value| {
        v.get("env")
            .is_some_and(|e| e.get("GH_TOKEN").is_some() || e.get("GITHUB_TOKEN").is_some())
    };
    has_token(job)
        || job
            .get("steps")
            .and_then(Value::as_sequence)
            .is_some_and(|s| s.iter().any(has_token))
}

/// `cb-2115` was CI-only on "credential: the workflow's GH_TOKEN", and traceability went red
/// on PR #1368 while `make gate` read green; with `gh auth token` it runs here in ~7s.
/// `dependabot-alerts-live` blamed the DEPENDABOT_TOKEN secret and runs here in 0.65s. A GitHub
/// token is a credential every clone that can push already holds, so it is no reason to leave a
/// check unrun: the leg takes it from `$GH_TOKEN`, else `gh auth token`, and fails without one.
#[test]
fn no_ci_only_credential_row_blames_a_github_token_gh_already_holds() {
    let ci =
        serde_yaml_ng::from_str::<Value>(&read(".github/workflows/ci.yml")).unwrap_or_default();
    let job = |id: &str| {
        ci.get("jobs")
            .and_then(|j| j.get(id))
            .cloned()
            .unwrap_or_default()
    };
    assert!(
        reads_github_with_a_token(&job("traceability")),
        "control: ci.yml traceability hands CB-2115 a GH_TOKEN, and the detector no longer sees it"
    );
    assert!(
        !reads_github_with_a_token(&job("windows-check")),
        "control: ci.yml windows-check uses no GitHub token, and the detector says it does"
    );
    let blaming: Vec<String> = ci_only_jobs("credential")
        .iter()
        .filter(|(_, _, job)| reads_github_with_a_token(job))
        .map(|(leg, at, _)| format!("{leg} ({at})"))
        .collect();
    assert!(
        blaming.is_empty(),
        "CI-only rows blame a credential that is a GitHub token — run each as a cmd leg with \
         GH_TOKEN=\"${{GH_TOKEN:-$(gh auth token)}}\", failing without one: {blaming:?}"
    );
}

/// Every hand-written spelling of the pmat binary a leg could run.
const BINARY_PATHS: &[&str] = &["target/debug/pmat", "target/release/pmat"];
/// The one spelling `scripts/gate.sh` rewrites, in a step's text, to `$PMAT_BIN`.
const REWRITTEN: &str = "./target/debug/pmat";

/// The `run:` text of a `<workflow>#<job>#<step>` source, or "" when it does not resolve
/// (`every_step_leg_names_one_runnable_step_in_its_workflow` fails that case by name).
fn step_run(source: &str) -> String {
    let parts: Vec<&str> = source.splitn(3, '#').collect();
    if parts.len() != 3 {
        return String::new();
    }
    let doc = serde_yaml_ng::from_str::<Value>(&read(parts[0])).unwrap_or_default();
    doc.get("jobs")
        .and_then(|j| j.get(parts[1]))
        .and_then(|j| j.get("steps"))
        .and_then(Value::as_sequence)
        .and_then(|s| {
            s.iter()
                .find(|st| st.get("name").and_then(Value::as_str) == Some(parts[2]))
        })
        .and_then(|st| st.get("run"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// `cb-2113-cb-2115` and `pmat-score` ran `./target/debug/pmat`, and eight `step` legs run the
/// same path from ci.yml. `build-pmat` writes to `$CARGO_TARGET_DIR/debug/pmat`, so under an
/// isolated target dir every one of them judged whatever binary an earlier build had left in
/// `./target` — a PASS about some other tree (PMAT-1365, sixth session). A leg runs the
/// executable cargo reports building: `cmd` rows name `$PMAT_BIN` (or `cargo run`), and `step`
/// text, which is CI's and cannot change, may only spell the path the gate rewrites.
/// The behaviour itself — a planted stale binary is never run — is arm 12 of gate-control.sh.
#[test]
fn no_leg_runs_a_hand_written_pmat_binary_path() {
    let script = read(SCRIPT);
    let all = rows(&script);
    let hand_written: Vec<String> = all
        .iter()
        .filter(|r| r.kind == "cmd" && BINARY_PATHS.iter().any(|p| r.command.contains(p)))
        .map(|r| r.leg.clone())
        .collect();
    assert!(
        hand_written.is_empty(),
        "cmd legs run a hand-written pmat path, which ignores CARGO_TARGET_DIR and judges a \
         stale binary — run \"$PMAT_BIN\" instead: {hand_written:?}"
    );
    let steps: Vec<(String, String)> = all
        .iter()
        .filter(|r| r.kind == "step")
        .map(|r| (r.leg.clone(), step_run(&r.source)))
        .collect();
    let runs_pmat: Vec<&String> = steps
        .iter()
        .filter(|(_, run)| run.contains(REWRITTEN))
        .map(|(leg, _)| leg)
        .collect();
    assert!(
        runs_pmat.iter().any(|l| l.as_str() == "roadmap-validate"),
        "control: ci.yml roadmap-validate runs {REWRITTEN}, and the reading no longer sees it"
    );
    let unrewritten: Vec<&String> = steps
        .iter()
        .filter(|(_, run)| {
            BINARY_PATHS
                .iter()
                .map(|p| run.matches(p).count())
                .sum::<usize>()
                != run.matches(REWRITTEN).count()
        })
        .map(|(leg, _)| leg)
        .collect();
    assert!(
        unrewritten.is_empty(),
        "step legs run pmat under a spelling scripts/gate.sh does not rewrite to $PMAT_BIN: \
         {unrewritten:?}"
    );
    assert!(
        script.contains(&format!("\"{REWRITTEN}\"")) && script.contains("--message-format json"),
        "{SCRIPT} no longer rewrites {REWRITTEN} to the executable cargo reports building"
    );
}
