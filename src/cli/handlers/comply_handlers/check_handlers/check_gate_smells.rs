// GS-01, GS-02, GS-07, GS-13 (#1673): gate smells found by reading workflow files.
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// Static and deterministic: the scan reads `.github/workflows/*.y{,a}ml` and nothing
// else — no network, no git, no `gh`. Warn-level: a smell is reported, never a failure.
// The scan is line-based on purpose: a workflow that does not parse as YAML must still
// be scanned, and these four smells are all visible on one line or one job block.

const GS_01_TITLE: &str = "Network Fetch In PR Gate";
const GS_02_TITLE: &str = "Floating Version";
const GS_07_TITLE: &str = "No Timeout Or Supersede";
const GS_13_TITLE: &str = "Hosted Runner In Sovereign Repo";

/// Commands that download something at job time.
const GS_FETCH_CMDS: &[&str] = &[
    "curl ",
    "wget ",
    "apt install",
    "apt-get install",
    "cargo install",
    "cargo binstall",
    "rustup ",
    "git fetch",
];
/// Actions that download a toolchain or tool at job time.
const GS_FETCH_ACTIONS: &[&str] = &[
    "dtolnay/rust-toolchain",
    "actions-rs/toolchain",
    "taiki-e/install-action",
    "actions/setup-",
];
/// Refs that move: an action or toolchain pinned to one is not pinned.
const GS_FLOATING_REFS: &[&str] = &[
    "stable", "beta", "nightly", "main", "master", "latest", "develop", "HEAD",
];
/// GitHub-hosted runner label prefixes.
const GS_HOSTED_PREFIXES: &[&str] = &["ubuntu-", "windows-", "macos-"];
/// How many hits a verdict message names before it says "(+N more)".
const GS_MESSAGE_HITS: usize = 8;

/// One workflow file reduced to its code lines: `(1-based line number, text)`,
/// comments and blank lines dropped, indentation kept.
struct GsWorkflow {
    file: String,
    lines: Vec<(usize, String)>,
}

/// One job of a workflow: its id, where it starts, and the lines of its block.
struct GsJob<'a> {
    id: &'a str,
    line: usize,
    indent: usize,
    body: Vec<(usize, &'a str)>,
}

fn gs_indent(s: &str) -> usize {
    s.len() - s.trim_start().len()
}

fn gs_strip_comment(raw: &str) -> &str {
    if raw.trim_start().starts_with('#') {
        return "";
    }
    raw.find(" #").map_or(raw, |i| &raw[..i])
}

fn gs_unquote(v: &str) -> &str {
    v.trim().trim_matches(|c| c == '"' || c == '\'')
}

/// `key: value` of a line, with a leading `- ` list marker removed.
fn gs_kv(line: &str) -> Option<(&str, &str)> {
    let t = line.trim_start();
    let t = t.strip_prefix("- ").unwrap_or(t);
    let (k, v) = t.split_once(':')?;
    Some((k.trim(), v.trim()))
}

fn gs_is_jobs_header(line: &str) -> bool {
    gs_indent(line) == 0 && line.trim_start().starts_with("jobs:")
}

/// `cmd` appears in `line` as a word of its own, not inside a longer word.
fn gs_names_cmd(line: &str, cmd: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '-' || c == '_';
    line.match_indices(cmd).any(|(i, _)| {
        let before_ok = line[..i].chars().next_back().is_none_or(|c| !word(c));
        let after_ok = line[i + cmd.len()..]
            .chars()
            .next()
            .is_none_or(|c| !word(c))
            || !cmd.ends_with(word);
        before_ok && after_ok
    })
}

impl GsWorkflow {
    fn parse(file: &str, text: &str) -> Self {
        let lines = text
            .lines()
            .enumerate()
            .filter_map(|(i, raw)| {
                let code = gs_strip_comment(raw).trim_end();
                (!code.trim().is_empty()).then(|| (i + 1, code.to_string()))
            })
            .collect();
        Self {
            file: file.to_string(),
            lines,
        }
    }

    fn hit(&self, line: usize, what: &str) -> String {
        format!("{}:{line}: {what}", self.file)
    }

    /// Does the workflow run on `pull_request`? Only the trigger block counts.
    fn is_pr_workflow(&self) -> bool {
        self.lines
            .iter()
            .take_while(|(_, l)| !gs_is_jobs_header(l))
            .any(|(_, l)| l.contains("pull_request"))
    }

    fn jobs(&self) -> Vec<GsJob<'_>> {
        let mut it = self
            .lines
            .iter()
            .skip_while(|(_, l)| !gs_is_jobs_header(l))
            .skip(1)
            .peekable();
        let child = match it.peek() {
            Some((_, l)) => gs_indent(l),
            None => return vec![],
        };
        if child == 0 {
            return vec![];
        }
        let mut jobs: Vec<GsJob<'_>> = vec![];
        for (n, l) in it {
            let ind = gs_indent(l);
            if ind < child {
                break;
            }
            let t = l.trim();
            if ind == child && t.ends_with(':') && !t.starts_with('-') {
                jobs.push(GsJob {
                    id: t.trim_end_matches(':'),
                    line: *n,
                    indent: ind,
                    body: vec![],
                });
            } else if let Some(j) = jobs.last_mut() {
                j.body.push((*n, l.as_str()));
            }
        }
        jobs
    }
}

impl<'a> GsJob<'a> {
    /// The value of a key set directly on the job (not on a step).
    fn key(&self, key: &str) -> Option<(usize, &'a str)> {
        let at = self.body.first().map(|(_, l)| gs_indent(l))?;
        self.body.iter().find_map(|(n, l)| {
            let (k, v) = gs_kv(l)?;
            (gs_indent(l) == at && k == key).then_some((*n, v))
        })
    }

    /// Every `runs-on` label of the job, inline or as a block list.
    fn runs_on_labels(&self) -> Vec<(usize, String)> {
        let Some((at, inline)) = self.key("runs-on") else {
            return vec![];
        };
        let mut labels: Vec<(usize, String)> = inline
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(gs_unquote)
            .filter(|s| !s.is_empty())
            .map(|s| (at, s.to_string()))
            .collect();
        let own = self
            .body
            .iter()
            .find(|(n, _)| *n == at)
            .map_or(0, |(_, l)| gs_indent(l));
        for (n, l) in self.body.iter().filter(|(n, _)| *n > at) {
            if gs_indent(l) <= own {
                break;
            }
            let t = l.trim().trim_start_matches("- ");
            let v = t.split_once(':').map_or(t, |(_, v)| v);
            labels.push((*n, gs_unquote(v).to_string()));
        }
        labels
    }

    fn mentions_fork(&self) -> bool {
        let named = self
            .key("name")
            .is_some_and(|(_, v)| v.to_lowercase().contains("fork"));
        named || self.id.to_lowercase().contains("fork")
    }
}

/// GS-01: a `pull_request` workflow that fetches something at job time.
fn gs_scan_network_fetch(workflows: &[GsWorkflow]) -> Vec<String> {
    let mut hits = vec![];
    for wf in workflows.iter().filter(|w| w.is_pr_workflow()) {
        for (n, l) in wf.lines.iter().skip_while(|(_, l)| !gs_is_jobs_header(l)) {
            if let Some(what) = gs_fetch_what(l) {
                hits.push(wf.hit(*n, what.trim()));
            }
        }
    }
    hits
}

fn gs_fetch_what(line: &str) -> Option<&'static str> {
    if gs_kv(line).is_some_and(|(k, _)| k == "name") {
        return None;
    }
    let cmd = GS_FETCH_CMDS.iter().find(|c| gs_names_cmd(line, c));
    let uses_line = gs_kv(line).is_some_and(|(k, _)| k == "uses");
    let action = GS_FETCH_ACTIONS
        .iter()
        .find(|a| uses_line && line.contains(**a));
    cmd.or(action).copied()
}

/// GS-02: an action or toolchain pinned to a ref that moves.
fn gs_scan_floating(workflows: &[GsWorkflow]) -> Vec<String> {
    let mut hits = vec![];
    for wf in workflows {
        for (n, l) in &wf.lines {
            if let Some(what) = gs_floating_what(l) {
                hits.push(wf.hit(*n, &what));
            }
        }
    }
    hits
}

fn gs_floating_what(line: &str) -> Option<String> {
    let (k, v) = gs_kv(line)?;
    let v = gs_unquote(v);
    match k {
        "uses" => {
            let r = v.rsplit_once('@')?.1;
            GS_FLOATING_REFS.contains(&r).then(|| format!("uses {v}"))
        }
        "toolchain" => matches!(v, "stable" | "beta" | "nightly").then(|| format!("toolchain {v}")),
        _ => None,
    }
}

/// GS-07: a `pull_request` job with no `timeout-minutes`, or a `pull_request`
/// workflow that never cancels a superseded run.
fn gs_scan_timeout_supersede(workflows: &[GsWorkflow]) -> Vec<String> {
    let mut hits = vec![];
    for wf in workflows.iter().filter(|w| w.is_pr_workflow()) {
        for job in wf.jobs() {
            // A job that calls a reusable workflow cannot carry its own timeout.
            if job.key("timeout-minutes").is_none() && job.key("uses").is_none() {
                hits.push(wf.hit(job.line, &format!("job {} has no timeout-minutes", job.id)));
            }
        }
        if !gs_cancels_superseded(wf) {
            let first = wf.lines.first().map_or(1, |(n, _)| *n);
            hits.push(wf.hit(first, "no concurrency.cancel-in-progress"));
        }
    }
    hits
}

fn gs_cancels_superseded(wf: &GsWorkflow) -> bool {
    wf.lines.iter().any(|(_, l)| {
        gs_kv(l).is_some_and(|(k, v)| k == "cancel-in-progress" && !v.is_empty() && v != "false")
    })
}

/// GS-13: a job on a GitHub-hosted runner, outside a fork-refusal job.
fn gs_scan_hosted_runner(workflows: &[GsWorkflow]) -> Vec<String> {
    let mut hits = vec![];
    for wf in workflows {
        for job in wf.jobs().iter().filter(|j| !j.mentions_fork()) {
            for (n, label) in job.runs_on_labels() {
                if GS_HOSTED_PREFIXES.iter().any(|p| label.starts_with(p)) {
                    hits.push(wf.hit(n, &format!("job {} runs-on {label}", job.id)));
                }
            }
        }
    }
    hits
}

/// The workflow files of a repo, sorted by name; `None` when there are none.
fn gs_load_workflows(project_path: &Path) -> Option<Vec<GsWorkflow>> {
    let dir = project_path.join(".github/workflows");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("yml" | "yaml")))
        .collect();
    files.sort();
    let workflows: Vec<GsWorkflow> = files
        .iter()
        .filter_map(|p| {
            let text = std::fs::read_to_string(p).ok()?;
            let name = p.file_name()?.to_str()?;
            Some(GsWorkflow::parse(
                &format!(".github/workflows/{name}"),
                &text,
            ))
        })
        .collect();
    (!workflows.is_empty()).then_some(workflows)
}

/// One verdict: Pass with no hits, Warn naming the first few otherwise.
fn gs_verdict(id: &str, title: &str, hits: &[String]) -> ComplianceCheck {
    let name = format!("{id}: {title}");
    if hits.is_empty() {
        return ComplianceCheck {
            name,
            status: CheckStatus::Pass,
            message: "no smell found".into(),
            severity: Severity::Info,
        };
    }
    let shown = hits
        .iter()
        .take(GS_MESSAGE_HITS)
        .cloned()
        .collect::<Vec<_>>()
        .join("; ");
    let more = hits.len().saturating_sub(GS_MESSAGE_HITS);
    let tail = if more > 0 {
        format!(" (+{more} more)")
    } else {
        String::new()
    };
    ComplianceCheck {
        name,
        status: CheckStatus::Warn,
        message: format!("{} hit(s): {shown}{tail}", hits.len()),
        severity: Severity::Warning,
    }
}

/// GS-01, GS-02, GS-07 and GS-13, then the wave-2 rules, for one repository, in id order.
pub(crate) fn check_gate_smells(project_path: &Path) -> Vec<ComplianceCheck> {
    let Some(workflows) = gs_load_workflows(project_path) else {
        return [
            ("GS-01", GS_01_TITLE),
            ("GS-02", GS_02_TITLE),
            ("GS-07", GS_07_TITLE),
            ("GS-13", GS_13_TITLE),
        ]
        .iter()
        .chain(GS_WAVE2)
        .map(|(id, title)| gs_wave2_skip(id, title, "decline: no .github/workflows/*.yml".into()))
        .collect();
    };
    let mut checks = vec![
        gs_verdict("GS-01", GS_01_TITLE, &gs_scan_network_fetch(&workflows)),
        gs_verdict("GS-02", GS_02_TITLE, &gs_scan_floating(&workflows)),
        gs_verdict("GS-07", GS_07_TITLE, &gs_scan_timeout_supersede(&workflows)),
        gs_verdict("GS-13", GS_13_TITLE, &gs_scan_hosted_runner(&workflows)),
    ];
    checks.extend(gs_wave2_checks(project_path));
    checks
}
