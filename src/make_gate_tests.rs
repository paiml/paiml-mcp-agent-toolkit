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
//! second runner — and pin the three things a later edit could remove while every other
//! test stayed green:
//!
//!   1. the Makefile target exists and runs the table;
//!   2. every required context has a row, and every CI-only row says why;
//!   3. every `step` row still names exactly one runnable step in its workflow.
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
