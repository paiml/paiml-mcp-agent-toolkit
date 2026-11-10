//! The five wave-2 detectors. Pure: workflow in, findings out.

use super::model::{Finding, Job, Report, Workflow};

fn finding(id: &'static str, wf: &Workflow, job: &Job<'_>, message: String) -> Finding {
    Finding {
        id,
        file: wf.path.clone(),
        job: job.id.clone(),
        message,
    }
}

/// Run-line prefixes that read live state from outside the checked-out commit.
const LIVE_READS: &[&str] = &[
    "gh api",
    "gh issue list",
    "gh issue view",
    "gh pr list",
    "gh pr view",
    "gh run list",
    "gh run view",
    "gh release view",
    "gh release list",
    "git ls-remote",
];

fn live_read(line: &str) -> Option<&'static str> {
    let l = line.trim_start_matches(|c: char| c == '$' || c.is_whitespace());
    if l.contains("api.github.com") && (l.starts_with("curl") || l.contains(" curl ")) {
        return Some("curl api.github.com");
    }
    LIVE_READS.iter().copied().find(|p| l.starts_with(p))
}

/// GS-03: a PR-path job's verdict depends on live external state.
pub fn detect_gs03(wf: &Workflow) -> Report {
    let mut r = Report::default();
    for job in wf.jobs().iter().filter(|j| wf.job_on_pr_path(j)) {
        if let Some(hit) = job.run_lines().iter().find_map(|l| live_read(l)) {
            r.findings.push(finding(
                "GS-03",
                wf,
                job,
                format!("PR job reads live external state (`{hit}`); verdict is not a function of the commit"),
            ));
        }
    }
    r
}

fn is_selftest(line: &str) -> bool {
    let l = line.to_lowercase();
    l.contains("--selftest")
        || l.contains("--self-test")
        || l.contains("falsifier")
        || l.contains("self-test")
}

/// A path filter covers product code when it is absent (every PR runs it) or names
/// a product tree or a catch-all.
fn covers_product(paths: &Option<Vec<String>>) -> bool {
    let Some(paths) = paths else { return true };
    paths.iter().any(|p| {
        let p = p.trim_start_matches('!');
        p == "*"
            || p == "**"
            || p.starts_with("src/")
            || p.starts_with("crates/")
            || p.starts_with("**/*.rs")
    })
}

/// GS-04: a gate self-test (falsifier, `--selftest`) runs where product code triggers it.
pub fn detect_gs04(wf: &Workflow) -> Report {
    let mut r = Report::default();
    if !covers_product(&wf.pr_paths()) {
        return r;
    }
    for job in wf.jobs().iter().filter(|j| wf.job_on_pr_path(j)) {
        if job.run_lines().iter().any(|l| is_selftest(l)) {
            r.findings.push(finding(
                "GS-04",
                wf,
                job,
                "gate self-test runs on the product path; run it only when the gate's own files change".into(),
            ));
        }
    }
    r
}

const HEAVY: &[&str] = &[
    "clean-room",
    "x64",
    "mutants",
    "gpu",
    "large",
    "xlarge",
    "16-core",
    "32-core",
];

/// A line that only inspects the result of a needed job.
fn is_result_check(line: &str) -> bool {
    let l = line.to_lowercase();
    let guard = l.starts_with("echo")
        || l.starts_with("exit")
        || l.starts_with("if ")
        || l.starts_with("fi")
        || l.starts_with("then")
        || l.starts_with("else")
        || l.starts_with("test ")
        || l.starts_with('[')
        || l.starts_with("for ")
        || l.starts_with("done")
        || l.starts_with("case")
        || l.starts_with("esac")
        || l.starts_with(";;")
        || l.starts_with("true")
        || l.starts_with("false")
        || l.starts_with("fail=")
        || l.starts_with("failed=");
    guard || l.contains("needs.") || l.contains("needs_json")
}

/// GS-05: a roll-up job (only `needs` and a result check) asks for a heavy runner.
pub fn detect_gs05(wf: &Workflow) -> Report {
    let mut r = Report::default();
    for job in wf.jobs() {
        if job.needs().is_empty() || !job.uses().is_empty() {
            continue;
        }
        let lines = job.run_lines();
        if lines.is_empty() || !lines.iter().all(|l| is_result_check(l)) {
            continue;
        }
        let heavy: Vec<String> = job
            .labels()
            .into_iter()
            .filter(|l| HEAVY.contains(&l.as_str()))
            .collect();
        if !heavy.is_empty() {
            r.findings.push(finding(
                "GS-05",
                wf_ref(wf),
                &job,
                format!("roll-up job only checks `needs`, yet asks for heavy runner label(s) {heavy:?}; use a hosted runner"),
            ));
        }
    }
    r
}

fn wf_ref(wf: &Workflow) -> &Workflow {
    wf
}

/// Where a reusable-workflow ref points. Parsed from `org/repo/.github/workflows/x.yml@ref`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SharedRef {
    pub repo: String,
    pub sha: String,
}

pub(crate) fn parse_shared_ref(uses: &str) -> Option<SharedRef> {
    let (path, rf) = uses.split_once('@')?;
    if !path.contains("/.github/workflows/") {
        return None;
    }
    let mut parts = path.splitn(3, '/');
    let (org, repo) = (parts.next()?, parts.next()?);
    let pinned = rf.len() == 40 && rf.chars().all(|c| c.is_ascii_hexdigit());
    pinned.then(|| SharedRef {
        repo: format!("{org}/{repo}"),
        sha: rf.to_string(),
    })
}

/// GS-12: a shared workflow pinned to a sha the default branch does not contain.
/// `reachable(repo, sha)` answers from a local checkout; `None` means it cannot say.
pub fn detect_gs12(wf: &Workflow, reachable: &dyn Fn(&str, &str) -> Option<bool>) -> Report {
    let mut r = Report::default();
    for job in wf.jobs() {
        for u in job.uses() {
            let Some(sr) = parse_shared_ref(&u) else {
                continue;
            };
            match reachable(&sr.repo, &sr.sha) {
                Some(false) => r.findings.push(finding(
                    "GS-12",
                    wf,
                    &job,
                    format!(
                        "`{u}` pins {} to {}, which is not on its default branch",
                        sr.repo,
                        &sr.sha[..12]
                    ),
                )),
                Some(true) => {}
                None => r.not_measured.push(format!(
                    "GS-12 {}:{}: cannot resolve {}@{} (no checkout)",
                    wf.path,
                    job.id,
                    sr.repo,
                    &sr.sha[..12]
                )),
            }
        }
    }
    r
}

const REPO_GLOBAL: &[&str] = &[
    "git log --all",
    "git rev-list --all",
    "git rev-list --count head",
    "--all-history",
    "traceability",
    "roadmap",
];

/// GS-14: a repo-global check (whole history, whole roadmap) on the per-PR path.
pub fn detect_gs14(wf: &Workflow) -> Report {
    let mut r = Report::default();
    for job in wf.jobs().iter().filter(|j| wf.job_on_pr_path(j)) {
        let hit = job.run_lines().into_iter().find_map(|l| {
            let low = l.to_lowercase();
            REPO_GLOBAL
                .iter()
                .find(|p| low.contains(*p))
                .map(|p| (*p).to_string())
        });
        if let Some(p) = hit {
            r.findings.push(finding(
                "GS-14",
                wf,
                job,
                format!("PR job runs a repo-global check (`{p}`); its inputs are not the diff, so move it to nightly"),
            ));
        }
    }
    r
}
