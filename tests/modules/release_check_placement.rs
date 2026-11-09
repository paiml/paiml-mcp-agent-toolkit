//! PMAT-ANDON-1117Z order 1: `release-check` must not gate master.
//!
//! A version bump merged to master is not a release. release-check ran inside
//! quality-gate.yml on every push, so the 3.43.0 bump (#1619) turned master red
//! and, via the PR gates, every PR red for three days. It now lives in its own
//! workflow that only runs on a schedule or by hand.

use std::fs;
use std::path::Path;

fn workflow(name: &str) -> Option<String> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".github/workflows")
        .join(name);
    if !p.parent().is_some_and(Path::is_dir) {
        eprintln!("NOT-SHIPPED: .github/workflows absent (packaged crate)");
        return None;
    }
    Some(fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} unreadable: {e}", p.display())))
}

/// Text of the top-level `on:` block (up to the next top-level key).
fn on_block(yaml: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for l in yaml.lines() {
        if l.starts_with("on:") {
            inside = true;
            continue;
        }
        if inside && !l.is_empty() && !l.starts_with(' ') && !l.starts_with('#') {
            break;
        }
        if inside {
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}

#[test]
fn quality_gate_does_not_run_release_check() {
    let Some(q) = workflow("quality-gate.yml") else {
        return;
    };
    assert!(
        !q.contains("release-check"),
        "quality-gate.yml (push + pull_request) must not run release-check: a version \
         bump without a release would turn master red"
    );
}

#[test]
fn release_check_runs_only_on_schedule_or_by_hand() {
    let Some(r) = workflow("release-check.yml") else {
        return;
    };
    let on = on_block(&r);
    assert!(
        on.contains("schedule:"),
        "release-check must run daily:\n{on}"
    );
    assert!(
        on.contains("workflow_dispatch:"),
        "release-check must be runnable by hand:\n{on}"
    );
    for bad in [
        "push:",
        "pull_request:",
        "pull_request_target:",
        "merge_group:",
    ] {
        assert!(
            !on.contains(bad),
            "release-check must not trigger on {bad}\n{on}"
        );
    }
    assert!(
        r.contains("scripts/release-check.sh"),
        "the workflow must still run the check"
    );
}

#[test]
fn a_tag_without_a_release_is_caught_by_the_release_workflow() {
    let Some(r) = workflow("release.yml") else {
        return;
    };
    assert!(
        on_block(&r).contains("tags:"),
        "release.yml must fire on v* tags"
    );
}
