// RS-01..RS-12 (pmat#1673): static scan of a repo's GitHub Actions configuration for the
// runner and gate defects measured in the 2026-11-10 build outage. A line-for-line port of
// paiml/.github `scripts/runner-smells.sh`: same rule ids, same messages, same line numbers
// (the parity test replays that script's findings from tests/fixtures/runner-smells).
//
// Included from check.rs: no `use` imports, so every path is fully qualified. Static, no
// network, no YAML parser (the awk original is line-based, and so is this).
//
// Level. The script's default mode is `report`: it prints every finding, RS-12 included,
// and exits 0. RS-12 is hard only under its `--ratchet` and `--strict` modes. So every rule
// here is a Warn by default and `pmat comply check --strict` is the blocking mode, as in the
// script. A rule may block by default only once CI shows it PASS on the last shipped
// artifact and FAIL on a planted defect; the script @5f7f049 finds 2 RS-12 lines in this
// repository, so RS-12 as Fail would be an unproven blocking gate.
//
// Parity choices, each measured against the script @5f7f049 (see the tests):
// - bytes, not UTF-8: a workflow that is not valid UTF-8 is still scanned (lossy decode);
// - records end at `\n` only, as in awk and sed: a `\r` stays on the line, so a CRLF file
//   has no job lines here either. A CRLF fix belongs in the script first, then here;
// - the shell glob skips dotfiles, and so does this; files are ordered by bytes (the glob
//   under LC_ALL=C). Another locale changes the order of findings only, never the set;
// - paths print as the script prints them when run as `runner-smells.sh .` in the repo;
// - a workflow that cannot be read is not measured: all twelve rules Skip, none Pass.

const RS_WORKFLOWS: &str = "./.github/workflows";

const RS_TITLES: [(&str, &str); 12] = [
    ("RS-01", "No Concurrency Cancel"),
    ("RS-02", "Job Without Timeout"),
    ("RS-03", "Hosted Runner On PR"),
    ("RS-04", "Heavy Suite On Every PR Push"),
    ("RS-05", "pull_request_target Trigger"),
    ("RS-06", "Cargo Without CI Profile"),
    ("RS-07", "Heavy Gate Roll-Up Runner"),
    ("RS-08", "Action Not Pinned To Sha"),
    ("RS-09", "Draft PRs Start CI"),
    ("RS-10", "Too Many PR Workflows"),
    ("RS-11", "Examples Built On PR Path"),
    ("RS-12", "Mutation Outside Nightly"),
];
const RS_MAX_PR_WORKFLOWS: usize = 3;
const RS_MESSAGE_HITS: usize = 8;

/// One finding, the script's `RULE<TAB>file:line<TAB>message` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RsFinding {
    pub(crate) rule: &'static str,
    pub(crate) file: String,
    pub(crate) line: usize,
    pub(crate) message: String,
}

impl RsFinding {
    fn at(rule: &'static str, file: &str, line: usize, message: impl Into<String>) -> Self {
        Self {
            rule,
            file: file.to_string(),
            line,
            message: message.into(),
        }
    }

    /// `RULE<TAB>file:line<TAB>message`, the script's output format.
    pub(crate) fn tsv(&self) -> String {
        format!(
            "{}\t{}:{}\t{}",
            self.rule, self.file, self.line, self.message
        )
    }
}

fn rs_re(pattern: &str) -> regex::Regex {
    regex::Regex::new(pattern).expect("runner-smells pattern is valid")
}

static RS_RE_COMMENT: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"(^|[[:space:]])#.*$"));
static RS_RE_ON: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r#"^("on"|on|true)[ \t]*:"#));
static RS_RE_PR_VALUE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"(^|[^a-z_])pull_request(_target)?([^a-z_]|$)"));
static RS_RE_PR_KEY: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    rs_re(r#"^[ \t]+(- )?["']?pull_request(_target)?["']?[ \t]*(:|$|#)"#)
});
static RS_RE_JOB: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"^  [A-Za-z0-9_-]+:[ \t]*$"));
static RS_RE_KEY4: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"^    [A-Za-z0-9_-]+:"));
static RS_RE_HOSTED: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"(ubuntu|macos|windows)-(latest|[0-9])"));
static RS_RE_HEAVY: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    rs_re(r"(?i)mutation|mutant|coverage|msrv|bench|fuzz|convergence|behavior")
});
static RS_RE_CARGO_RUN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    rs_re(
        r"(^|[^[:alnum:]_-])cargo([[:space:]]+[+][^[:space:]]+)?[[:space:]]+(build|test|nextest|clippy|llvm-cov|check)([[:space:]]|$)",
    )
});
static RS_RE_USES: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    rs_re(r"^[[:space:]]*(- )?uses:[[:space:]]*[^.[:space:]][^[:space:]]*")
});
static RS_RE_USES_PREFIX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"^[[:space:]]*(- )?uses:[[:space:]]*"));
static RS_RE_SHA: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"@[0-9a-f]{40}([[:space:]]|$)"));
static RS_RE_ALL_TARGETS: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    rs_re(r"cargo[^#]*[[:space:]](--all-targets|--examples)([[:space:]]|$)")
});
static RS_RE_CLIPPY_CHECK: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"cargo[[:space:]]+(clippy|check)"));
static RS_RE_MUTANTS: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| rs_re(r"(^|[^[:alnum:]_])cargo[ -]mutants([^[:alnum:]_]|$)"));

/// The records awk and sed see: split at `\n` only (a `\r` is kept), and a final
/// newline does not open an empty last record. `str::lines` would strip the `\r`.
fn rs_lines(text: &str) -> impl Iterator<Item = &str> {
    let body = text.strip_suffix('\n').unwrap_or(text);
    body.split('\n').filter(move |_| !text.is_empty())
}

/// The script's `nocomment`: `sed -E 's/(^|[[:space:]])#.*$//'`, line by line.
fn rs_nocomment(text: &str) -> Vec<String> {
    rs_lines(text)
        .map(|l| RS_RE_COMMENT.replace(l, "").into_owned())
        .collect()
}

/// `is_pr`: `on:` names pull_request or pull_request_target.
fn rs_is_pr(text: &str) -> bool {
    let (mut in_on, mut pr) = (false, false);
    for line in rs_lines(text) {
        if RS_RE_ON.is_match(line) {
            in_on = true;
            let value = line
                .split_once(':')
                .map_or("", |(_, v)| v.trim_start_matches([' ', '\t']));
            if RS_RE_PR_VALUE.is_match(value) {
                pr = true;
            }
            continue;
        }
        in_on &= !rs_ends_block(line);
        if in_on && RS_RE_PR_KEY.is_match(line) {
            pr = true;
        }
    }
    pr
}

fn rs_is_nightly_key(key: &str) -> bool {
    key == "schedule" || key == "workflow_dispatch"
}

/// `non_nightly`: `on:` names a trigger other than schedule / workflow_dispatch.
fn rs_non_nightly(text: &str) -> bool {
    let (mut in_on, mut bad) = (false, false);
    let strip =
        |s: &str, drop: &[char]| s.chars().filter(|c| !drop.contains(c)).collect::<String>();
    for line in rs_lines(text) {
        if RS_RE_ON.is_match(line) {
            in_on = true;
            let value = line.split_once(':').map_or("", |(_, v)| v);
            let value = value.replace(['[', ']', ',', '"', '\''], " ");
            bad |= value
                .split_whitespace()
                .any(|t| !rs_is_nightly_key(t) && !t.starts_with('#'));
            continue;
        }
        in_on &= !rs_ends_block(line);
        if in_on && rs_is_top_trigger_key(line) {
            let key = strip(
                line.split(':').next().unwrap_or(""),
                &[' ', '\t', '"', '\''],
            );
            bad |= !rs_is_nightly_key(&key);
        }
        if in_on && line.starts_with("  - ") {
            let key = strip(line, &[' ', '\t', '"', '\'', '-']);
            bad |= !rs_is_nightly_key(&key);
        }
    }
    bad
}

/// `^  ["']?[a-z_]+["']?[ \t]*:` — a trigger key at two-space indent.
fn rs_is_top_trigger_key(line: &str) -> bool {
    static RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| rs_re(r#"^  ["']?[a-z_]+["']?[ \t]*:"#));
    RE.is_match(line)
}

/// One job as `jobs_tsv` sees it.
struct RsJob {
    line: usize,
    id: String,
    has_timeout: bool,
    reusable: bool,
    runs_on: String,
}

/// awk `/^[^ \t#]/`: a line that starts at column 0 with something other than a comment.
fn rs_ends_block(raw: &str) -> bool {
    raw.chars()
        .next()
        .is_some_and(|c| !matches!(c, ' ' | '\t' | '#'))
}

/// awk `/^jobs:[ \t]*$/`.
fn rs_opens_jobs(raw: &str) -> bool {
    raw.strip_prefix("jobs:")
        .is_some_and(|r| r.chars().all(|c| matches!(c, ' ' | '\t')))
}

/// The awk statements that read one line inside a job body.
fn rs_job_line(job: &mut RsJob, s: &str, in_runs_on: &mut bool) {
    job.has_timeout |= s.starts_with("    timeout-minutes:");
    job.reusable |= s.starts_with("    uses:");
    if RS_RE_KEY4.is_match(s) {
        *in_runs_on = false;
    }
    if let Some(value) = s.strip_prefix("    runs-on:") {
        *in_runs_on = true;
        job.runs_on = value.trim_start_matches([' ', '\t']).to_string();
    } else if *in_runs_on {
        job.runs_on = format!("{} {}", job.runs_on, s);
    }
}

/// `jobs_tsv`: the awk job splitter, ported statement for statement.
fn rs_jobs(text: &str) -> Vec<RsJob> {
    let mut jobs: Vec<RsJob> = Vec::new();
    let (mut in_jobs, mut in_runs_on) = (false, false);
    for (idx, raw) in rs_lines(text).enumerate() {
        if rs_opens_jobs(raw) {
            in_jobs = true;
            continue;
        }
        in_jobs &= !rs_ends_block(raw);
        if !in_jobs {
            continue;
        }
        let s = rs_strip_ws_comment(raw);
        if RS_RE_JOB.is_match(&s) {
            let id: String = s
                .chars()
                .filter(|c| !matches!(c, ' ' | ':' | '\t'))
                .collect();
            jobs.push(RsJob {
                line: idx + 1,
                id,
                has_timeout: false,
                reusable: false,
                runs_on: String::new(),
            });
            in_runs_on = false;
        } else if let Some(job) = jobs.last_mut() {
            rs_job_line(job, &s, &mut in_runs_on);
        }
    }
    jobs
}

/// awk `sub(/[ \t]#.*$/, "", s)`: drop from the first whitespace-then-`#`.
fn rs_strip_ws_comment(line: &str) -> String {
    let bytes = line.as_bytes();
    for i in 1..bytes.len() {
        if bytes[i] == b'#' && matches!(bytes[i - 1], b' ' | b'\t') {
            return line[..i - 1].to_string();
        }
    }
    line.to_string()
}

fn rs_first_line(lines: &[String], matches: impl Fn(&str) -> bool) -> Option<usize> {
    lines.iter().position(|l| matches(l)).map(|i| i + 1)
}

fn rs01(file: &str, text: &str) -> Vec<RsFinding> {
    static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        rs_re(r"^[[:space:]]*cancel-in-progress:[[:space:]]*[^f[:space:]]")
    });
    if !rs_is_pr(text) || rs_nocomment(text).iter().any(|l| RE.is_match(l)) {
        return vec![];
    }
    vec![RsFinding::at("RS-01", file, 1, "pull_request workflow has no concurrency cancel-in-progress: a new push leaves the old run holding runners")]
}

fn rs02(file: &str, text: &str) -> Vec<RsFinding> {
    rs_jobs(text)
        .into_iter()
        .filter(|j| !j.has_timeout && !j.reusable)
        .map(|j| {
            RsFinding::at(
                "RS-02",
                file,
                j.line,
                format!(
                    "job '{}' has no timeout-minutes: a hung job holds its runner for 6 h",
                    j.id
                ),
            )
        })
        .collect()
}

fn rs03(file: &str, text: &str) -> Vec<RsFinding> {
    if !rs_is_pr(text) {
        return vec![];
    }
    rs_jobs(text)
        .into_iter()
        .filter(|j| RS_RE_HOSTED.is_match(&j.runs_on))
        .map(|j| RsFinding::at("RS-03", file, j.line, format!("job '{}' runs on a GitHub-hosted runner on every PR push (paid minutes, no fleet cache)", j.id)))
        .collect()
}

fn rs04(file: &str, text: &str) -> Vec<RsFinding> {
    if !rs_is_pr(text) {
        return vec![];
    }
    let base = file.rsplit('/').next().unwrap_or(file);
    let heavy =
        RS_RE_HEAVY.is_match(base) || rs_jobs(text).iter().any(|j| RS_RE_HEAVY.is_match(&j.id));
    let train = rs_nocomment(text)
        .iter()
        .any(|l| l.contains("train/") || l.contains("heavy_on") || l.contains("merge_group"));
    if !heavy || train {
        return vec![];
    }
    vec![RsFinding::at("RS-04", file, 1, "heavy suite runs on every pull_request push; run it on train/* and the default branch only")]
}

fn rs05(file: &str, text: &str) -> Vec<RsFinding> {
    static RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| rs_re(r"(^|[^a-z_])pull_request_target([^a-z_]|$)"));
    rs_first_line(&rs_nocomment(text), |l| RE.is_match(l))
        .map(|l| {
            vec![RsFinding::at(
                "RS-05",
                file,
                l,
                "pull_request_target runs fork code with repository secrets",
            )]
        })
        .unwrap_or_default()
}

fn rs06(file: &str, text: &str) -> Vec<RsFinding> {
    static RE_PROFILE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| rs_re(r"CARGO_PROFILE_(DEV|TEST)_DEBUG"));
    if !rs_is_pr(text) {
        return vec![];
    }
    let Some(l) = rs_first_line(&rs_nocomment(text), |l| RS_RE_CARGO_RUN.is_match(l)) else {
        return vec![];
    };
    if RE_PROFILE.is_match(text) && text.contains("CARGO_INCREMENTAL") {
        return vec![];
    }
    vec![RsFinding::at("RS-06", file, l, "cargo runs without CARGO_PROFILE_DEV_DEBUG=0 / CARGO_PROFILE_TEST_DEBUG=0 / CARGO_INCREMENTAL=0 (measured: 66G build tree vs 5.5G)")]
}

fn rs07(file: &str, text: &str) -> Vec<RsFinding> {
    static RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| rs_re(r"gate-light|inputs\.|matrix\.|vars\."));
    rs_jobs(text)
        .into_iter()
        .filter(|j| j.id == "gate" && !j.reusable && !RE.is_match(&j.runs_on))
        .map(|j| {
            RsFinding::at(
                "RS-07",
                file,
                j.line,
                "roll-up job 'gate' asks for a heavy runner; it only reads other jobs' results",
            )
        })
        .collect()
}

fn rs08(file: &str, text: &str) -> Vec<RsFinding> {
    rs_nocomment(text)
        .iter()
        .enumerate()
        .filter(|(_, l)| RS_RE_USES.is_match(l) && !RS_RE_SHA.is_match(l))
        .filter(|(_, l)| !l.contains("uses:") || !rs_is_docker_uses(l))
        .map(|(i, l)| {
            let what = RS_RE_USES_PREFIX.replace(l, "");
            RsFinding::at(
                "RS-08",
                file,
                i + 1,
                format!("not pinned to a commit sha: {what}"),
            )
        })
        .collect()
}

/// `uses:[[:space:]]*docker://`
fn rs_is_docker_uses(line: &str) -> bool {
    line.split_once("uses:")
        .is_some_and(|(_, rest)| rest.trim_start().starts_with("docker://"))
}

fn rs09(file: &str, text: &str) -> Vec<RsFinding> {
    if !rs_is_pr(text)
        || rs_nocomment(text)
            .iter()
            .any(|l| l.contains("draft") || l.contains("ready_for_review"))
    {
        return vec![];
    }
    vec![RsFinding::at("RS-09", file, 1, "draft pull requests start this workflow (no draft == false condition, no ready_for_review type)")]
}

fn rs11(file: &str, text: &str) -> Vec<RsFinding> {
    if !rs_is_pr(text) {
        return vec![];
    }
    rs_first_line(&rs_nocomment(text), |l| {
        RS_RE_ALL_TARGETS.is_match(l) && !RS_RE_CLIPPY_CHECK.is_match(l)
    })
    .map(|l| {
        vec![RsFinding::at(
            "RS-11",
            file,
            l,
            "examples/all targets are built on the PR path (measured: 13-24G per runner)",
        )]
    })
    .unwrap_or_default()
}

fn rs12(file: &str, text: &str) -> Vec<RsFinding> {
    if !rs_non_nightly(text) {
        return vec![];
    }
    rs_first_line(&rs_nocomment(text), |l| RS_RE_MUTANTS.is_match(l))
        .map(|l| vec![RsFinding::at("RS-12", file, l, "mutation testing runs outside a scheduled nightly (operator ruling 2026-11-10: nightly only, forever)")])
        .unwrap_or_default()
}

/// The workflow files of `dir/.github/workflows` as `(printed path, text)`: `*.yml` then
/// `*.yaml`, each in byte order, dotfiles skipped (the shell glob). Text is decoded lossily,
/// so a file that is not UTF-8 is scanned, not dropped. `Err` says why nothing was measured.
fn rs_workflow_files(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let none = || "decline: no .github/workflows/*.yml".to_string();
    let root = dir.join(".github/workflows");
    let entries = std::fs::read_dir(&root).map_err(|_| none())?;
    let mut names: Vec<(String, std::ffi::OsString)> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.file_name()))
        .filter(|(n, _)| !n.starts_with('.') && (n.ends_with(".yml") || n.ends_with(".yaml")))
        .collect();
    if names.is_empty() {
        return Err(none());
    }
    names.sort_by(|a, b| (a.0.ends_with(".yaml"), &a.0).cmp(&(b.0.ends_with(".yaml"), &b.0)));
    names
        .into_iter()
        .map(|(shown, raw)| {
            let bytes = std::fs::read(root.join(&raw))
                .map_err(|e| format!("not_measured: cannot read {RS_WORKFLOWS}/{shown}: {e}"))?;
            Ok((
                format!("{RS_WORKFLOWS}/{shown}"),
                String::from_utf8_lossy(&bytes).into_owned(),
            ))
        })
        .collect()
}

/// `scan`: every rule over every workflow file, then the repo-wide RS-10.
/// `Err` carries the reason nothing was measured (no workflows, or an unreadable one).
pub(crate) fn rs_scan(dir: &Path) -> Result<Vec<RsFinding>, String> {
    let files = rs_workflow_files(dir)?;
    let rules: [fn(&str, &str) -> Vec<RsFinding>; 11] = [
        rs01, rs02, rs03, rs04, rs05, rs06, rs07, rs08, rs09, rs11, rs12,
    ];
    let mut found: Vec<RsFinding> = Vec::new();
    for (name, text) in &files {
        for rule in rules {
            found.extend(rule(name, text));
        }
    }
    let pr = files.iter().filter(|(_, t)| rs_is_pr(t)).count();
    if pr > RS_MAX_PR_WORKFLOWS {
        found.push(RsFinding::at(
            "RS-10",
            RS_WORKFLOWS,
            0,
            format!("{pr} workflow files start on pull_request (limit {RS_MAX_PR_WORKFLOWS}): one push fans out into {pr} runs"),
        ));
    }
    Ok(found)
}

fn rs_verdict(id: &str, title: &str, hits: &[&RsFinding]) -> ComplianceCheck {
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
        .take(RS_MESSAGE_HITS)
        .map(|h| format!("{}:{}: {}", h.file, h.line, h.message))
        .collect::<Vec<_>>()
        .join("; ");
    let more = hits.len().saturating_sub(RS_MESSAGE_HITS);
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

/// RS-01..RS-12 for one repository, in id order.
pub(crate) fn check_runner_smells(project_path: &Path) -> Vec<ComplianceCheck> {
    let found = match rs_scan(project_path) {
        Ok(found) => found,
        Err(why) => {
            return RS_TITLES
                .iter()
                .map(|(id, title)| ComplianceCheck {
                    name: format!("{id}: {title}"),
                    status: CheckStatus::Skip,
                    message: why.clone(),
                    severity: Severity::Info,
                })
                .collect();
        }
    };
    RS_TITLES
        .iter()
        .map(|(id, title)| {
            let hits: Vec<&RsFinding> = found.iter().filter(|f| f.rule == *id).collect();
            rs_verdict(id, title, &hits)
        })
        .collect()
}
