use super::detectors::parse_shared_ref;
use super::*;

fn wf(text: &str) -> Workflow {
    Workflow::parse("ci.yml", text).expect("parse")
}

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn ids(r: &Report) -> Vec<&'static str> {
    r.findings.iter().map(|f| f.id).collect()
}

// ---- GS-03 ----
const GS03_SMELLY: &str = "on: pull_request\njobs:\n  gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: gh issue list --label p0 | grep -q .\n";
const GS03_CLEAN: &str = "on: pull_request\njobs:\n  gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: cargo test\n";
const GS03_NIGHTLY: &str = "on:\n  schedule:\n    - cron: '0 3 * * *'\njobs:\n  gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: gh api repos/x/y\n";

#[test]
fn gs03_flags_live_read_on_pr_path() {
    assert_eq!(ids(&detect_gs03(&wf(GS03_SMELLY))), ["GS-03"]);
}
#[test]
fn gs03_clean_and_nightly_pass() {
    assert!(detect_gs03(&wf(GS03_CLEAN)).findings.is_empty());
    assert!(detect_gs03(&wf(GS03_NIGHTLY)).findings.is_empty());
}
#[test]
fn gs03_curl_api_and_ls_remote() {
    let t = "on: pull_request\njobs:\n  a:\n    runs-on: x\n    steps:\n      - run: curl -s https://api.github.com/repos/a/b\n  b:\n    runs-on: x\n    steps:\n      - run: git ls-remote origin main\n";
    assert_eq!(detect_gs03(&wf(t)).findings.len(), 2);
}

// ---- GS-04 ----
const GS04_SMELLY: &str = "on: pull_request\njobs:\n  gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: ./scripts/gate.sh --selftest\n";
const GS04_CLEAN: &str = "on:\n  pull_request:\n    paths:\n      - 'scripts/gate.sh'\njobs:\n  gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: ./scripts/gate.sh --selftest\n";

#[test]
fn gs04_flags_unfiltered_selftest() {
    assert_eq!(ids(&detect_gs04(&wf(GS04_SMELLY))), ["GS-04"]);
}
#[test]
fn gs04_gate_only_path_filter_passes() {
    assert!(detect_gs04(&wf(GS04_CLEAN)).findings.is_empty());
}
#[test]
fn gs04_product_path_filter_flags() {
    let t = GS04_CLEAN.replace("scripts/gate.sh'", "crates/**'");
    assert_eq!(detect_gs04(&wf(&t)).findings.len(), 1);
}

// ---- GS-05 ----
const GS05_SMELLY: &str = "on: pull_request\njobs:\n  build:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: cargo build\n  rollup:\n    needs: [build]\n    runs-on: [self-hosted, clean-room]\n    steps:\n      - run: |\n          if [ \"${{ needs.build.result }}\" != success ]; then exit 1; fi\n";
const GS05_CLEAN: &str = "on: pull_request\njobs:\n  build:\n    runs-on: [self-hosted, clean-room]\n    steps:\n      - run: cargo build\n  rollup:\n    needs: [build]\n    runs-on: ubuntu-24.04\n    steps:\n      - run: |\n          if [ \"${{ needs.build.result }}\" != success ]; then exit 1; fi\n";

#[test]
fn gs05_flags_heavy_rollup() {
    let r = detect_gs05(&wf(GS05_SMELLY));
    assert_eq!(ids(&r), ["GS-05"]);
    assert_eq!(r.findings[0].job, "rollup");
}
#[test]
fn gs05_hosted_rollup_and_heavy_worker_pass() {
    assert!(detect_gs05(&wf(GS05_CLEAN)).findings.is_empty());
}
#[test]
fn gs05_job_that_does_work_is_not_a_rollup() {
    let t = GS05_SMELLY.replace("exit 1; fi", "exit 1; fi\n          cargo test");
    assert!(detect_gs05(&wf(&t)).findings.is_empty());
}

// ---- GS-12 ----
fn shared(sha: &str) -> String {
    format!("on: pull_request\njobs:\n  ci:\n    uses: paiml/.github/.github/workflows/sovereign-ci.yml@{sha}\n")
}

#[test]
fn gs12_parse_shared_ref() {
    let r = parse_shared_ref(&format!("paiml/.github/.github/workflows/x.yml@{SHA}"))
        .expect("a 40-hex ref parses");
    assert_eq!((r.repo.as_str(), r.sha.as_str()), ("paiml/.github", SHA));
    assert!(parse_shared_ref("paiml/.github/.github/workflows/x.yml@main").is_none());
    assert!(parse_shared_ref(&format!("actions/checkout@{SHA}")).is_none());
}
#[test]
fn gs12_flags_unreachable_sha() {
    let r = detect_gs12(&wf(&shared(SHA)), &|_, _| Some(false));
    assert_eq!(ids(&r), ["GS-12"]);
}
#[test]
fn gs12_reachable_sha_and_branch_ref_pass() {
    assert!(detect_gs12(&wf(&shared(SHA)), &|_, _| Some(true))
        .findings
        .is_empty());
    assert!(detect_gs12(&wf(&shared("main")), &|_, _| Some(false))
        .findings
        .is_empty());
}
#[test]
fn gs12_unresolvable_is_not_measured_never_a_pass() {
    let r = detect_gs12(&wf(&shared(SHA)), &|_, _| None);
    assert!(r.findings.is_empty());
    assert_eq!(r.not_measured.len(), 1);
}

// ---- GS-14 ----
const GS14_SMELLY: &str = "on: pull_request\njobs:\n  trace:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: pmat comply check --checks traceability\n";
const GS14_CLEAN: &str = "on: pull_request\njobs:\n  t:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: cargo test\n";
const GS14_NIGHTLY: &str = "on:\n  schedule:\n    - cron: '0 3 * * *'\njobs:\n  trace:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: pmat comply check --checks traceability\n";

#[test]
fn gs14_flags_repo_global_on_pr_path() {
    assert_eq!(ids(&detect_gs14(&wf(GS14_SMELLY))), ["GS-14"]);
}
#[test]
fn gs14_clean_and_nightly_pass() {
    assert!(detect_gs14(&wf(GS14_CLEAN)).findings.is_empty());
    assert!(detect_gs14(&wf(GS14_NIGHTLY)).findings.is_empty());
}

// ---- scan ----
#[test]
fn scan_reports_each_smell_and_unparseable_file() {
    let d = tempfile::TempDir::new().expect("tempdir");
    std::fs::write(d.path().join("a.yml"), GS03_SMELLY).expect("write fixture");
    std::fs::write(d.path().join("b.yml"), GS14_SMELLY).expect("write fixture");
    std::fs::write(d.path().join("c.yml"), "jobs: [unclosed").expect("write fixture");
    std::fs::write(d.path().join("d.txt"), "ignored").expect("write fixture");
    let r = scan_workflows(d.path(), &|_, _| None).expect("scan succeeds");
    let got = ids(&r);
    assert!(got.contains(&"GS-03") && got.contains(&"GS-14"), "{got:?}");
    assert_eq!(r.not_measured.len(), 1);
}

#[test]
fn gs03_curl_mid_line_and_leading_curl_each_flag() {
    let mid = "on: pull_request\njobs:\n  a:\n    runs-on: x\n    steps:\n      - run: sudo curl -s https://api.github.com/x\n";
    let lead = "on: pull_request\njobs:\n  a:\n    runs-on: x\n    steps:\n      - run: curl https://api.github.com/x\n";
    assert_eq!(detect_gs03(&wf(mid)).findings.len(), 1);
    assert_eq!(detect_gs03(&wf(lead)).findings.len(), 1);
}
