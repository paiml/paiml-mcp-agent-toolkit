//! API-measured runner smells (pmat#1673, paiml/infra#3828): the three
//! resource smells a workflow file cannot show, only the GitHub API can.
//!
//! | id    | smell                                  | warn      | fail      |
//! |-------|----------------------------------------|-----------|-----------|
//! | RS-13 | workflow runs per merged PR, 24 h      | > 15      | > 30      |
//! | RS-14 | share of finished runs that were cancelled | > 40% | > 60%     |
//! | RS-15 | run queue time p95 (created -> started)   | > 300 s | > 900 s |
//!
//! The ids continue `paiml/.github` `scripts/runner-smells.sh` (RS-01..RS-12).
//! Every bound is a tunable `.pmat.yaml` option. The engine is pure: it judges
//! a [`Window`] that an injected [`ActionsApi`] produced, so every test runs
//! offline on a fixture. A read that failed or was rate limited is
//! [`Verdict::NotMeasured`], never a pass (doctrine 2, GH-1).
//!
//! GitHub GraphQL has no workflow-run object: merged PRs are read through a
//! GraphQL `search` count (GH-2: reads by GraphQL), runs through the REST
//! `actions/runs` list (the only API that has them), capped at [`MAX_PAGES`]
//! pages and reported as a sample when the cap bites.
use chrono::{DateTime, Utc};

#[cfg(test)]
mod tests;

/// REST pages of 100 runs read per window; past it the window is a sample.
pub const MAX_PAGES: usize = 5;

#[derive(Debug, Clone, PartialEq)]
pub struct RunSample {
    pub created_at: DateTime<Utc>,
    /// `run_started_at`; `None` while the run is still queued.
    pub started_at: Option<DateTime<Utc>>,
    /// `None` while the run has not finished.
    pub conclusion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub repo: String,
    pub since: DateTime<Utc>,
    /// The API's `total_count` of runs created in the window.
    pub runs_total: u64,
    /// The runs actually read (at most `MAX_PAGES * 100`).
    pub runs: Vec<RunSample>,
    pub merged_prs: u64,
}

/// The injected read side: live it is [`GhCli`], in tests a fixture.
pub trait ActionsApi {
    fn window(&self, repo: &str, since: DateTime<Utc>) -> Result<Window, String>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    pub runs_per_merge_warn: f64,
    pub runs_per_merge_fail: f64,
    pub cancelled_warn: f64,
    pub cancelled_fail: f64,
    pub queue_p95_warn_secs: i64,
    pub queue_p95_fail_secs: i64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            runs_per_merge_warn: 15.0,
            runs_per_merge_fail: 30.0,
            cancelled_warn: 0.40,
            cancelled_fail: 0.60,
            queue_p95_warn_secs: 300,
            queue_p95_fail_secs: 900,
        }
    }
}

impl Thresholds {
    /// Defaults overridden by `.pmat.yaml` options. `get` answers `None` for
    /// an absent key and `Some(None)` for a key whose value is not a number,
    /// which is refused by name rather than ignored.
    pub fn from_lookup(get: &dyn Fn(&str) -> Option<Option<f64>>) -> Result<Self, String> {
        let mut t = Self::default();
        let num = |key: &str, slot: &mut f64| -> Result<(), String> {
            match get(key) {
                None => Ok(()),
                Some(Some(v)) => {
                    *slot = v;
                    Ok(())
                }
                Some(None) => Err(format!("option {key} is not a number")),
            }
        };
        num("runs_per_merge_warn", &mut t.runs_per_merge_warn)?;
        num("runs_per_merge_fail", &mut t.runs_per_merge_fail)?;
        num("cancelled_warn", &mut t.cancelled_warn)?;
        num("cancelled_fail", &mut t.cancelled_fail)?;
        let mut warn = t.queue_p95_warn_secs as f64;
        let mut fail = t.queue_p95_fail_secs as f64;
        num("queue_p95_warn_secs", &mut warn)?;
        num("queue_p95_fail_secs", &mut fail)?;
        t.queue_p95_warn_secs = warn as i64;
        t.queue_p95_fail_secs = fail as i64;
        Ok(t)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
    NotMeasured,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub id: &'static str,
    pub verdict: Verdict,
    pub message: String,
}

fn graded(value: f64, warn: f64, fail: f64) -> Verdict {
    if value > fail {
        Verdict::Fail
    } else if value > warn {
        Verdict::Warn
    } else {
        Verdict::Pass
    }
}

/// `read` of the window as a phrase, saying when the runs are a sample.
fn basis(w: &Window) -> String {
    if (w.runs.len() as u64) < w.runs_total {
        format!("sample of {} of {} runs", w.runs.len(), w.runs_total)
    } else {
        format!("{} runs", w.runs.len())
    }
}

/// RS-13: workflow runs created in the window divided by PRs merged in it.
pub fn runs_per_merge(w: &Window, t: &Thresholds) -> Finding {
    let denom = w.merged_prs.max(1);
    let ratio = w.runs_total as f64 / denom as f64;
    let note = if w.merged_prs == 0 && w.runs_total > 0 {
        " (no merged PR in the window: counted against one)"
    } else {
        ""
    };
    Finding {
        id: "RS-13",
        verdict: graded(ratio, t.runs_per_merge_warn, t.runs_per_merge_fail),
        message: format!(
            "{} workflow run(s) per merged PR ({} runs, {} merged since {}){note}; warn above {:.1}, fail above {:.1}",
            format_args!("{ratio:.1}"),
            w.runs_total,
            w.merged_prs,
            w.since.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            t.runs_per_merge_warn,
            t.runs_per_merge_fail
        ),
    }
}

/// RS-14: of the runs that finished, the share that ended `cancelled`.
pub fn cancelled_share(w: &Window, t: &Thresholds) -> Finding {
    let finished: Vec<&RunSample> = w.runs.iter().filter(|r| r.conclusion.is_some()).collect();
    if finished.is_empty() {
        return Finding {
            id: "RS-14",
            verdict: Verdict::NotMeasured,
            message: format!(
                "no finished run in the window to measure a cancelled share from ({})",
                basis(w)
            ),
        };
    }
    let cancelled = finished
        .iter()
        .filter(|r| r.conclusion.as_deref() == Some("cancelled"))
        .count();
    let share = cancelled as f64 / finished.len() as f64;
    Finding {
        id: "RS-14",
        verdict: graded(share, t.cancelled_warn, t.cancelled_fail),
        message: format!(
            "{:.1}% of finished runs were cancelled ({cancelled} of {}; {}); warn above {:.0}%, fail above {:.0}%",
            share * 100.0,
            finished.len(),
            basis(w),
            t.cancelled_warn * 100.0,
            t.cancelled_fail * 100.0
        ),
    }
}

/// Nearest-rank percentile of an ascending slice (`p` in 0..=100).
pub fn percentile(sorted: &[i64], p: f64) -> i64 {
    let n = sorted.len();
    let rank = ((p / 100.0) * n as f64).ceil() as usize;
    sorted[rank.clamp(1, n) - 1]
}

/// RS-15: p95 of the wait between a run's creation and its start.
pub fn queue_p95(w: &Window, t: &Thresholds) -> Finding {
    let mut waits: Vec<i64> = w
        .runs
        .iter()
        .filter_map(|r| {
            r.started_at
                .map(|s| (s - r.created_at).num_seconds().max(0))
        })
        .collect();
    if waits.is_empty() {
        return Finding {
            id: "RS-15",
            verdict: Verdict::NotMeasured,
            message: format!(
                "no started run in the window to measure a queue time from ({})",
                basis(w)
            ),
        };
    }
    waits.sort_unstable();
    let p95 = percentile(&waits, 95.0);
    Finding {
        id: "RS-15",
        verdict: graded(
            p95 as f64,
            t.queue_p95_warn_secs as f64,
            t.queue_p95_fail_secs as f64,
        ),
        message: format!(
            "run queue time p95 is {p95} s ({} started runs; {}); warn above {} s, fail above {} s",
            waits.len(),
            basis(w),
            t.queue_p95_warn_secs,
            t.queue_p95_fail_secs
        ),
    }
}

/// Read the window once and judge it by all three rules. A failed read is
/// `NotMeasured` on every rule, carrying the reason.
pub fn evaluate(
    api: &dyn ActionsApi,
    repo: &str,
    since: DateTime<Utc>,
    t: &Thresholds,
) -> Vec<Finding> {
    match api.window(repo, since) {
        Ok(w) => vec![
            runs_per_merge(&w, t),
            cancelled_share(&w, t),
            queue_p95(&w, t),
        ],
        Err(e) => ["RS-13", "RS-14", "RS-15"]
            .into_iter()
            .map(|id| Finding {
                id,
                verdict: Verdict::NotMeasured,
                message: format!("{repo}: {e}"),
            })
            .collect(),
    }
}

/// One REST page of `actions/runs`: `(total_count, runs)`.
pub fn parse_runs_page(body: &str) -> Result<(u64, Vec<RunSample>), String> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("runs page is not JSON: {e}"))?;
    let total = v
        .get("total_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            format!(
                "not a runs page (no total_count): {}",
                v.get("message").and_then(|m| m.as_str()).unwrap_or("?")
            )
        })?;
    let list = v
        .get("workflow_runs")
        .and_then(serde_json::Value::as_array)
        .ok_or("not a runs page (no workflow_runs)")?;
    let ts = |x: &serde_json::Value| -> Result<Option<DateTime<Utc>>, String> {
        match x.as_str() {
            None => Ok(None),
            Some(s) => DateTime::parse_from_rfc3339(s)
                .map(|d| Some(d.with_timezone(&Utc)))
                .map_err(|e| format!("bad timestamp {s:?}: {e}")),
        }
    };
    let mut runs = Vec::with_capacity(list.len());
    for r in list {
        let created_at = ts(&r["created_at"])?.ok_or("run without created_at")?;
        runs.push(RunSample {
            created_at,
            started_at: ts(&r["run_started_at"])?,
            conclusion: r["conclusion"].as_str().map(str::to_string),
        });
    }
    Ok((total, runs))
}

/// The GraphQL search that counts PRs merged since `since`.
pub fn merged_query(repo: &str, since: DateTime<Utc>) -> String {
    format!(
        "query {{ search(query: \"repo:{repo} is:pr is:merged merged:>={}\", type: ISSUE, first: 1) {{ issueCount }} }}",
        since.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    )
}

pub fn parse_merged_count(body: &str) -> Result<u64, String> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("merged count is not JSON: {e}"))?;
    if let Some(errors) = v.get("errors") {
        return Err(format!("GraphQL error: {errors}"));
    }
    v["data"]["search"]["issueCount"]
        .as_u64()
        .ok_or_else(|| "GraphQL answer has no data.search.issueCount".to_string())
}

/// The live client: the `gh` CLI, so it uses the operator's own auth.
pub struct GhCli;

impl GhCli {
    fn gh(args: &[&str]) -> Result<String, String> {
        let out = std::process::Command::new("gh")
            .args(args)
            .output()
            .map_err(|e| format!("cannot run gh: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "gh {} failed: {}",
                args.first().copied().unwrap_or(""),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        String::from_utf8(out.stdout).map_err(|e| format!("gh output is not UTF-8: {e}"))
    }
}

impl ActionsApi for GhCli {
    fn window(&self, repo: &str, since: DateTime<Utc>) -> Result<Window, String> {
        let stamp = since.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let merged = parse_merged_count(&Self::gh(&[
            "api",
            "graphql",
            "-f",
            &format!("query={}", merged_query(repo, since)),
        ])?)?;
        let mut runs = Vec::new();
        let mut total = 0;
        for page in 1..=MAX_PAGES {
            let path =
                format!("repos/{repo}/actions/runs?created=%3E%3D{stamp}&per_page=100&page={page}");
            let (t, mut batch) = parse_runs_page(&Self::gh(&["api", &path])?)?;
            total = t;
            let got = batch.len();
            runs.append(&mut batch);
            if got < 100 || runs.len() as u64 >= total {
                break;
            }
        }
        Ok(Window {
            repo: repo.to_string(),
            since,
            runs_total: total,
            runs,
            merged_prs: merged,
        })
    }
}
