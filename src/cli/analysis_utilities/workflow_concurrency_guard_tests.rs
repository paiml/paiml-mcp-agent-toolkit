//! Every `pull_request` workflow must cancel its own superseded runs.
//!
//! A new push to a PR must cancel that PR's older queued/running runs, or CI
//! queues pile up behind dead commits. The block is the same everywhere:
//!
//! ```yaml
//! concurrency:
//!   group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}
//!   cancel-in-progress: ${{ github.event_name == 'pull_request' }}
//! ```
//!
//! `cancel-in-progress` is true ONLY for `pull_request` events, so push to
//! master, schedule, merge_group, release and train runs are never cancelled.
//! `pull_request_target` (the authorization gate) is a different trigger and is
//! deliberately out of scope.

const GROUP: &str =
    "group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}";
const CANCEL: &str = "cancel-in-progress: ${{ github.event_name == 'pull_request' }}";

/// True when the workflow text lists `pull_request` as a trigger.
fn triggers_on_pull_request(text: &str) -> bool {
    text.lines().any(|l| {
        let t = l.trim_end();
        t == "  pull_request:" || t.starts_with("  pull_request: ") || t == "on: pull_request"
    })
}

/// `None` when the workflow is fine, else the reason it is not.
fn concurrency_violation(text: &str) -> Option<&'static str> {
    if !triggers_on_pull_request(text) {
        return None;
    }
    let lines: Vec<&str> = text.lines().collect();
    let Some(at) = lines.iter().position(|l| l.trim_end() == "concurrency:") else {
        return Some("no top-level concurrency block");
    };
    let body: Vec<&str> = lines[at + 1..]
        .iter()
        .take_while(|l| l.starts_with("  "))
        .map(|l| l.trim())
        .collect();
    if !body.contains(&GROUP) {
        return Some("concurrency group is not workflow + PR number/ref");
    }
    if !body.contains(&CANCEL) {
        return Some("cancel-in-progress is not limited to pull_request events");
    }
    None
}

#[test]
fn every_pull_request_workflow_cancels_superseded_runs() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows");
    let mut checked = 0;
    let mut bad = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("workflows dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read workflow");
        if triggers_on_pull_request(&text) {
            checked += 1;
        }
        if let Some(why) = concurrency_violation(&text) {
            bad.push(format!("{}: {why}", path.display()));
        }
    }
    assert!(
        checked >= 6,
        "expected the pull_request workflows, saw {checked}"
    );
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn checker_accepts_the_standard_block_and_rejects_each_defect() {
    let ok = format!("on:\n  pull_request:\nconcurrency:\n  {GROUP}\n  {CANCEL}\njobs:\n");
    assert_eq!(concurrency_violation(&ok), None);
    let none = "on:\n  pull_request:\njobs:\n";
    assert_eq!(
        concurrency_violation(none),
        Some("no top-level concurrency block")
    );
    let bad_group = format!("on:\n  pull_request:\nconcurrency:\n  group: x\n  {CANCEL}\n");
    assert!(concurrency_violation(&bad_group).unwrap().contains("group"));
    let always =
        format!("on:\n  pull_request:\nconcurrency:\n  {GROUP}\n  cancel-in-progress: true\n");
    assert!(concurrency_violation(&always)
        .unwrap()
        .contains("cancel-in-progress"));
    let push_only = "on:\n  push:\njobs:\n";
    assert_eq!(concurrency_violation(push_only), None);
    let target = "on:\n  pull_request_target:\njobs:\n";
    assert!(!triggers_on_pull_request(target));
}
