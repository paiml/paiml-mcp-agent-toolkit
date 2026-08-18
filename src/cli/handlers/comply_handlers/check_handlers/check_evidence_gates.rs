//! Evidence-derived comply gates (CB-1700 through CB-1703).
//!
//! These four rules come from the evidence table in
//! `SE_Best_Practices_vs__pmat_comply__Evidence__Enforceability__and_Theater`
//! (tracking issue #1034). Each one gates a practice that has large-N
//! observational support and that `pmat comply` previously did not look at:
//!
//! | rule    | practice                     | evidence                       |
//! |---------|------------------------------|--------------------------------|
//! | CB-1700 | modern code review mechanics | McIntosh et al., MSR 2014      |
//! | CB-1701 | dependency supply chain      | cargo-deny / RustSec, direct   |
//! | CB-1702 | review changeset size        | Rigby & Bird, FSE 2013 (info)  |
//! | CB-1703 | documented rule count        | this repo's own README drift   |
//!
//! # Why the 1700 block and not 1300
//!
//! The source backlog assigns these rules CB-1300 (branch protection),
//! CB-1302 (supply chain) and CB-1305 (diff size). **All three ids are
//! already live in this tree** and have been since Component 23:
//! `build_contract_surface_checks` registers `cb-1300` (CLI arg contracts),
//! `cb-1302` (MCP schema contracts) and `cb-1305` (contract surface
//! classification). A clause id is not a label — it is the key
//! `.pmat.yaml` addresses a rule by, via `ComplyConfig::is_check_enabled`
//! and `get_severity`. Registering a second rule under a live id would mean
//! one config entry silently governing two unrelated checks: disabling
//! `cb-1300` to skip CLI-arg contracts would also disable branch protection,
//! and CB-1702's advisory-only invariant (a rule that may never be escalated
//! to `error`) would have frozen the severity of the anti-leak gate that
//! shares its id. The rules are therefore allocated a free block; the highest
//! id previously in use was `cb-1666`.
//!
//! # Shape
//!
//! Every rule is split into an impure *gatherer* that collects evidence from
//! the network, the filesystem or git, and a pure *evaluator* over that
//! evidence. Only the evaluators encode policy, which is what makes the
//! falsification tests in `check_evidence_gates_tests.rs` able to drive the
//! "API returned 403", "context is `ci / gate-v2`" and "advisory database is
//! three days old" cases without a network or a fixture repository.
//!
//! # Fail-closed
//!
//! Evidence that cannot be gathered is a **Fail**, never a Pass and never a
//! Skip. An unreachable GitHub API, a missing advisory database and a
//! required status check produced by a workflow this repository cannot read
//! are all reported as failures, because "unknown" and "compliant" are not
//! the same answer. `Skip` is reserved for *inapplicability* — a project with
//! no GitHub remote cannot have GitHub branch protection, so there is no
//! proposition to be wrong about.

use super::types::{CheckStatus, ComplianceCheck, Severity};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

/// How long any subprocess this module shells out to may run.
///
/// `gh api` against an unreachable host, or `git log` in a pathological
/// repository, must not be able to wedge `pmat comply check` forever. A
/// timeout is gathered as `Unavailable`, which is a Fail — not a Pass.
pub(crate) const SUBPROCESS_TIMEOUT: Duration = Duration::from_secs(20);

/// Clause ids whose checks are advisory by construction (CB-1702).
///
/// A rule listed here may never be escalated to `error` or `critical` in
/// `.pmat.yaml`; see `validate_advisory_only_severities`.
pub(crate) const ADVISORY_ONLY_CHECKS: &[&str] = &["cb-1702"];

// ---------------------------------------------------------------------------
// Subprocess plumbing
// ---------------------------------------------------------------------------

/// Run `cmd`, killing it if it outruns `SUBPROCESS_TIMEOUT`.
///
/// Returns `Err(reason)` when the binary is missing, the process is killed on
/// the deadline, or the pipes cannot be drained. Callers turn every `Err` into
/// fail-closed evidence.
fn run_bounded(cmd: &mut Command) -> Result<std::process::Output, String> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start process: {e}"))?;

    let deadline = SystemTime::now() + SUBPROCESS_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if SystemTime::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("timed out after {}s", SUBPROCESS_TIMEOUT.as_secs()));
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(e) => return Err(format!("wait failed: {e}")),
        }
    }
    child
        .wait_with_output()
        .map_err(|e| format!("could not read process output: {e}"))
}

// ---------------------------------------------------------------------------
// CB-1700 — branch protection
// ---------------------------------------------------------------------------

/// What is known about the default branch's protection settings.
#[derive(Debug, Clone)]
pub(crate) enum ProtectionEvidence {
    /// The GitHub API answered; the payload is its JSON body.
    Fetched(Value),
    /// The API could not be consulted (no `gh`, no credentials, 403, 404,
    /// rate limit, timeout, unparseable body). Unknown is not compliant.
    Unavailable(String),
    /// The project has no GitHub remote, so GitHub branch protection is not a
    /// property it can have or fail to have.
    NoGitHubRemote,
}

/// Does `ctx` — a *status check context string* — satisfy the doctrinal gate
/// named `gate`?
///
/// Match is on the context string only. The display name of a job is never
/// consulted, because the two routinely differ: this repository has a
/// top-level job literally named `gate` that reports as context `gate` and is
/// **not** required, while the required context is `ci / gate`, produced by
/// the `gate` job of a reusable workflow invoked from the job `ci`. GitHub
/// namespaces a called workflow's jobs as `<calling-job-id> / <called-job>`,
/// so the trailing segment is compared and a bare context is accepted
/// unqualified. `ci / gate-v2` does **not** satisfy the gate `gate`.
pub(crate) fn context_satisfies_gate(ctx: &str, gate: &str) -> bool {
    ctx == gate || ctx.ends_with(&format!(" / {gate}"))
}

/// Required status check contexts, in API order.
pub(crate) fn required_contexts(body: &Value) -> Vec<String> {
    body.get("required_status_checks")
        .and_then(|c| c.get("contexts"))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Number of approving reviews the default branch requires, when stated.
///
/// `None` means the API did not report the field at all — GitHub omits
/// `required_pull_request_reviews` entirely when review is not required, so
/// `None` is "zero reviews required", not "unknown".
fn required_approving_reviews(body: &Value) -> Option<u64> {
    body.get("required_pull_request_reviews")
        .and_then(|r| r.get("required_approving_review_count"))
        .and_then(Value::as_u64)
}

/// Whether force pushes are permitted. `None` means the API did not say, which
/// is treated as a violation rather than as consent.
fn force_push_allowed(body: &Value) -> Option<bool> {
    body.get("allow_force_pushes")
        .and_then(|f| f.get("enabled"))
        .and_then(Value::as_bool)
}

/// Collect every branch-protection violation in `body`.
fn branch_protection_violations(body: &Value, gates: &[String]) -> Vec<String> {
    let mut violations = Vec::new();
    let contexts = required_contexts(body);

    if contexts.is_empty() {
        violations.push(
            "required_status_checks.contexts is empty: the branch requires no status check at all"
                .to_string(),
        );
    }
    for gate in gates {
        if !contexts.iter().any(|c| context_satisfies_gate(c, gate)) {
            violations.push(format!(
                "doctrinal gate {gate:?} is absent from required_status_checks.contexts {contexts:?} \
                 (a job of that display name is not the same thing as a required context)"
            ));
        }
    }
    match required_approving_reviews(body) {
        Some(n) if n >= 1 => {}
        Some(n) => violations.push(format!(
            "required_approving_review_count is {n}; modern code review needs at least 1"
        )),
        None => violations.push(
            "no required_pull_request_reviews block: the default branch requires zero approving reviews"
                .to_string(),
        ),
    }
    match force_push_allowed(body) {
        Some(false) => {}
        Some(true) => violations.push(
            "allow_force_pushes.enabled is true: history on the default branch is rewritable"
                .to_string(),
        ),
        None => violations.push(
            "allow_force_pushes was not reported, so force-push status is unknown".to_string(),
        ),
    }
    violations
}

/// CB-1700: branch protection on the default branch.
///
/// Asserts, against the GitHub API and never against workflow YAML, that the
/// doctrinal gate is a *required status check context*, that at least one
/// approving review is required, and that force pushes are disabled.
///
/// McIntosh, Kamei, Adams & Hassan (MSR 2014) is the strongest large-N
/// observational entry in the evidence table: components with low review
/// coverage and low review participation had significantly more post-release
/// defects. The three properties above are the mechanical preconditions for
/// review coverage to be non-zero at all.
pub(crate) fn evaluate_branch_protection(
    evidence: &ProtectionEvidence,
    gates: &[String],
) -> ComplianceCheck {
    let name = "CB-1700: Branch Protection".to_string();
    match evidence {
        ProtectionEvidence::NoGitHubRemote => ComplianceCheck {
            name,
            status: CheckStatus::Skip,
            message: "no GitHub remote: GitHub branch protection is not a property of this project"
                .to_string(),
            severity: Severity::Info,
        },
        ProtectionEvidence::Unavailable(reason) => ComplianceCheck {
            name,
            status: CheckStatus::Fail,
            message: format!(
                "branch protection could not be read ({reason}). An unverifiable gate is not a \
                 verified one, so this is a failure, not a pass. Fix the access path (gh auth \
                 login / GH_TOKEN with repo scope) or disable cb-1700 in .pmat.yaml with a reason."
            ),
            severity: Severity::Error,
        },
        ProtectionEvidence::Fetched(body) => {
            let violations = branch_protection_violations(body, gates);
            if violations.is_empty() {
                ComplianceCheck {
                    name,
                    status: CheckStatus::Pass,
                    message: format!(
                        "default branch requires {:?}, >=1 approving review, and forbids force-push",
                        required_contexts(body)
                    ),
                    severity: Severity::Info,
                }
            } else {
                ComplianceCheck {
                    name,
                    status: CheckStatus::Fail,
                    message: format!(
                        "{} branch-protection violation(s): {}",
                        violations.len(),
                        violations.join("; ")
                    ),
                    severity: Severity::Error,
                }
            }
        }
    }
}

/// Extract `owner/repo` from a git remote URL pointing at github.com.
///
/// Handles the three forms git emits: `git@github.com:owner/repo.git`,
/// `https://github.com/owner/repo(.git)` and
/// `ssh://git@github.com/owner/repo(.git)`. Any other host yields `None`,
/// which makes the rule inapplicable rather than failed.
pub(crate) fn parse_github_slug(url: &str) -> Option<String> {
    let url = url.trim();
    let rest = if let Some(r) = url.strip_prefix("git@github.com:") {
        r
    } else if let Some(r) = url.strip_prefix("https://github.com/") {
        r
    } else if let Some(r) = url.strip_prefix("http://github.com/") {
        r
    } else {
        url.strip_prefix("ssh://git@github.com/")?
    };
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let rest = rest.trim_end_matches('/');
    let mut parts = rest.split('/');
    let owner = parts.next().filter(|s| !s.is_empty())?;
    let repo = parts.next().filter(|s| !s.is_empty())?;
    if parts.next().is_some() {
        return None;
    }
    Some(format!("{owner}/{repo}"))
}

/// Resolve the project's GitHub slug from `origin`, then `upstream`.
fn github_slug(project_path: &Path) -> Option<String> {
    for remote in ["origin", "upstream"] {
        let out = run_bounded(
            Command::new("git")
                .arg("-C")
                .arg(project_path)
                .args(["remote", "get-url", remote]),
        )
        .ok()?;
        if out.status.success() {
            if let Some(slug) = parse_github_slug(&String::from_utf8_lossy(&out.stdout)) {
                return Some(slug);
            }
        }
    }
    None
}

/// Ask `gh` for a JSON endpoint, mapping every failure mode onto a reason
/// string. HTTP 403 (rate limit, or a token without `repo` scope) and 404 (no
/// protection configured, or no visibility) both land here as reasons.
fn gh_api_json(project_path: &Path, endpoint: &str) -> Result<Value, String> {
    let out = run_bounded(
        Command::new("gh")
            .args(["api", endpoint])
            .current_dir(project_path)
            .env("GH_PAGER", "cat")
            .env("NO_COLOR", "1"),
    )?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err.lines().next().unwrap_or("no stderr").trim().to_string();
        return Err(format!("gh api {endpoint} exited {}: {first}", out.status));
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("gh api {endpoint} returned unparseable JSON: {e}"))
}

/// Gather CB-1700's evidence: the default branch's protection settings.
pub(crate) fn fetch_branch_protection(project_path: &Path) -> ProtectionEvidence {
    let Some(slug) = github_slug(project_path) else {
        return ProtectionEvidence::NoGitHubRemote;
    };
    let repo = match gh_api_json(project_path, &format!("repos/{slug}")) {
        Ok(v) => v,
        Err(e) => return ProtectionEvidence::Unavailable(e),
    };
    let Some(branch) = repo.get("default_branch").and_then(Value::as_str) else {
        return ProtectionEvidence::Unavailable(format!(
            "repos/{slug} did not report a default_branch"
        ));
    };
    match gh_api_json(
        project_path,
        &format!("repos/{slug}/branches/{branch}/protection"),
    ) {
        Ok(v) => ProtectionEvidence::Fetched(v),
        Err(e) => ProtectionEvidence::Unavailable(e),
    }
}

// ---------------------------------------------------------------------------
// CB-1701 — supply chain
// ---------------------------------------------------------------------------

/// Age of the RustSec advisory database backing `cargo deny`/`cargo audit`.
#[derive(Debug, Clone)]
pub(crate) enum AdvisoryDbAge {
    /// Time since the database was last fetched.
    Known(Duration),
    /// The database could not be located or dated.
    Unknown(String),
}

/// One job discovered in `.github/workflows/*.yml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkflowJob {
    /// Workflow file the job came from, for diagnostics.
    pub file: String,
    /// Status check context this job reports as: its `name:` if it has one,
    /// otherwise its job id.
    pub context: String,
    /// True when the job delegates to a reusable workflow via a job-level
    /// `uses:`. GitHub then reports its leaf jobs as `<job-id> / <leaf>` and
    /// those leaves live in another repository, so their steps cannot be read
    /// from here.
    pub delegates: bool,
    /// Whether the job may fail without failing the workflow.
    pub continue_on_error: bool,
    /// Every `run:` script in the job, concatenated.
    pub run_script: String,
}

/// Whether the required contexts are known.
#[derive(Debug, Clone)]
pub(crate) enum RequiredContexts {
    Known(Vec<String>),
    Unknown(String),
    NotAGitHubProject,
}

/// Evidence for CB-1701.
#[derive(Debug, Clone)]
pub(crate) struct SupplyChainEvidence {
    /// Contents of `deny.toml`, when it exists.
    pub deny_toml: Option<String>,
    /// Contexts branch protection requires.
    pub required: RequiredContexts,
    /// Jobs parsed out of `.github/workflows/`.
    pub jobs: Vec<WorkflowJob>,
    /// Freshness of the advisory database.
    pub advisory_db: AdvisoryDbAge,
}

/// The four cargo-deny check families. A family with no section in `deny.toml`
/// is a family whose policy was never stated.
pub(crate) const DENY_FAMILIES: &[&str] = &["advisories", "bans", "licenses", "sources"];

/// An advisory database older than this is stale ground truth: a `cargo deny`
/// run against it cannot see anything published since the last fetch, so a
/// green result asserts nothing.
pub(crate) const ADVISORY_DB_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// What a job's shell script does about `cargo deny`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DenyCoverage {
    /// Runs all four families, and a non-zero exit propagates.
    Blocking,
    /// Runs `cargo deny`, but the exit status is swallowed (`|| true`,
    /// `|| echo`, `continue-on-error`, `set +e`).
    NonBlocking(String),
    /// Runs `cargo deny check` against only some families.
    Partial(Vec<String>),
    /// Does not run cargo-deny at all.
    Absent,
}

/// Classify a `cargo deny` invocation inside a shell script.
///
/// `cargo deny check` with no family argument runs all four; naming families
/// restricts it. `paiml/.github`'s `sovereign-ci.yml` is the reason the
/// non-blocking case is called out separately: it appends `|| echo ...` to the
/// licenses and sources run, which reports findings and then exits zero, so
/// the check is present and enforces nothing.
pub(crate) fn classify_deny_invocation(script: &str) -> DenyCoverage {
    let mut best = DenyCoverage::Absent;
    for raw in script.lines() {
        let line = raw.trim();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if !invokes_cargo_deny_check(&tokens) {
            continue;
        }
        if let Some(reason) = swallows_exit_status(line) {
            best = merge_coverage(best, DenyCoverage::NonBlocking(reason));
            continue;
        }
        let named: Vec<String> = DENY_FAMILIES
            .iter()
            .filter(|f| tokens.iter().any(|t| t == *f))
            .map(|f| (*f).to_string())
            .collect();
        let covered = if named.is_empty() {
            // Bare `cargo deny check` runs every family.
            DenyCoverage::Blocking
        } else if named.len() == DENY_FAMILIES.len() {
            DenyCoverage::Blocking
        } else {
            DenyCoverage::Partial(
                DENY_FAMILIES
                    .iter()
                    .filter(|f| !named.contains(&(**f).to_string()))
                    .map(|f| (*f).to_string())
                    .collect(),
            )
        };
        best = merge_coverage(best, covered);
    }
    best
}

/// Does this tokenised line invoke `cargo deny check` (or `cargo-deny check`)?
fn invokes_cargo_deny_check(tokens: &[&str]) -> bool {
    tokens.windows(3).any(|w| w == ["cargo", "deny", "check"])
        || tokens.windows(2).any(|w| w == ["cargo-deny", "check"])
}

/// Report why a line's exit status cannot fail the job, if it cannot.
fn swallows_exit_status(line: &str) -> Option<String> {
    for marker in ["|| true", "||true", "|| echo", "|| :", "|| exit 0"] {
        if line.contains(marker) {
            return Some(format!("exit status swallowed by `{marker}`"));
        }
    }
    None
}

/// Keep the strongest coverage seen so far. Blocking beats everything.
fn merge_coverage(a: DenyCoverage, b: DenyCoverage) -> DenyCoverage {
    fn rank(c: &DenyCoverage) -> u8 {
        match c {
            DenyCoverage::Blocking => 3,
            DenyCoverage::Partial(_) => 2,
            DenyCoverage::NonBlocking(_) => 1,
            DenyCoverage::Absent => 0,
        }
    }
    if rank(&b) > rank(&a) {
        b
    } else {
        a
    }
}

/// TOML section headers present in `deny.toml`, e.g. `advisories`.
pub(crate) fn deny_sections(toml: &str) -> BTreeSet<String> {
    toml.lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix('[').and_then(|l| l.strip_suffix(']')))
        .map(|s| s.split('.').next().unwrap_or(s).trim().to_string())
        .collect()
}

/// Diagnose the required-context side of CB-1701: is any *required* status
/// check produced by a job this repository can read, and does that job run a
/// blocking, all-family `cargo deny check`?
fn deny_wiring_violation(ev: &SupplyChainEvidence) -> Option<String> {
    let contexts =
        match &ev.required {
            RequiredContexts::NotAGitHubProject => return None,
            RequiredContexts::Unknown(why) => {
                return Some(format!(
                "the set of required status checks is unknown ({why}), so it cannot be shown that \
                 cargo-deny runs inside one"
            ))
            }
            RequiredContexts::Known(c) if c.is_empty() => return Some(
                "the default branch requires no status check, so no cargo-deny run can be required"
                    .to_string(),
            ),
            RequiredContexts::Known(c) => c,
        };

    let mut notes = Vec::new();
    for ctx in contexts {
        match ev.jobs.iter().find(|j| &j.context == ctx) {
            None => {
                // A required context whose leading segment names a delegating
                // job is produced by a reusable workflow in another repo.
                let delegated = ev
                    .jobs
                    .iter()
                    .find(|j| j.delegates && ctx.starts_with(&format!("{} / ", j.context)));
                match delegated {
                    Some(j) => notes.push(format!(
                        "{ctx:?} comes from the reusable workflow invoked by {} ({}); its steps are \
                         not readable here",
                        j.context, j.file
                    )),
                    None => notes.push(format!(
                        "{ctx:?} is required but no job in .github/workflows/ produces it"
                    )),
                }
            }
            Some(job) => match classify_deny_invocation(&job.run_script) {
                DenyCoverage::Blocking if !job.continue_on_error => return None,
                DenyCoverage::Blocking => notes.push(format!(
                    "{ctx:?} ({}) runs cargo-deny but is continue-on-error",
                    job.file
                )),
                DenyCoverage::NonBlocking(why) => {
                    notes.push(format!("{ctx:?} ({}) runs cargo-deny but {why}", job.file))
                }
                DenyCoverage::Partial(missing) => notes.push(format!(
                    "{ctx:?} ({}) runs cargo-deny without {missing:?}",
                    job.file
                )),
                DenyCoverage::Absent => {
                    notes.push(format!("{ctx:?} ({}) runs no cargo-deny", job.file))
                }
            },
        }
    }
    Some(format!(
        "no required status check is known to run a blocking `cargo deny check` over all of \
         {DENY_FAMILIES:?}: {}",
        notes.join("; ")
    ))
}

/// CB-1701: dependency supply chain.
///
/// Asserts that all four cargo-deny families — advisories, bans, licenses and
/// sources — have a stated policy, that they run inside a *required* status
/// check with a propagating exit status, and that the advisory database they
/// run against is less than 24 hours old.
///
/// The last clause is INV-1302-2 from the backlog and is not decoration: a
/// clean `cargo deny advisories` against a three-day-old database asserts
/// nothing about the three days of advisories it has never seen.
///
/// Scope: cargo-deny reads the **RustSec** database. Dependabot reads GHSA and
/// neither is a superset of the other (paiml/.github#48), so a green CB-1701
/// says the tree is clean against RustSec, not that it is advisory-free.
pub(crate) fn evaluate_supply_chain(ev: &SupplyChainEvidence) -> ComplianceCheck {
    let name = "CB-1701: Supply Chain".to_string();
    let mut violations = Vec::new();

    match &ev.deny_toml {
        None => {
            violations.push("deny.toml is absent: no supply-chain policy is stated".to_string())
        }
        Some(toml) => {
            let sections = deny_sections(toml);
            let missing: Vec<&str> = DENY_FAMILIES
                .iter()
                .filter(|f| !sections.contains(**f))
                .copied()
                .collect();
            if !missing.is_empty() {
                violations.push(format!(
                    "deny.toml states no policy for {missing:?}; an unconfigured family is an \
                     unstated policy, not a passing one"
                ));
            }
        }
    }

    match &ev.advisory_db {
        AdvisoryDbAge::Unknown(why) => violations.push(format!(
            "advisory database age is unknown ({why}); a check against unknown ground truth is a \
             vacuous assertion"
        )),
        AdvisoryDbAge::Known(age) if *age > ADVISORY_DB_MAX_AGE => violations.push(format!(
            "advisory database was last fetched {}h ago (limit {}h): a green result cannot cover \
             advisories published since",
            age.as_secs() / 3600,
            ADVISORY_DB_MAX_AGE.as_secs() / 3600
        )),
        AdvisoryDbAge::Known(_) => {}
    }

    if let Some(v) = deny_wiring_violation(ev) {
        violations.push(v);
    }

    if violations.is_empty() {
        ComplianceCheck {
            name,
            status: CheckStatus::Pass,
            message: format!(
                "all of {DENY_FAMILIES:?} are configured and run blocking inside a required check, \
                 against an advisory database under {}h old",
                ADVISORY_DB_MAX_AGE.as_secs() / 3600
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name,
            status: CheckStatus::Fail,
            message: format!(
                "{} supply-chain violation(s): {}",
                violations.len(),
                violations.join("; ")
            ),
            severity: Severity::Error,
        }
    }
}

/// Parse the jobs of every `.github/workflows/*.yml` under `project_path`.
pub(crate) fn scan_workflow_jobs(project_path: &Path) -> Vec<WorkflowJob> {
    let dir = project_path.join(".github").join("workflows");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "yml" || e == "yaml")
        })
        .collect();
    files.sort();

    let mut jobs = Vec::new();
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let file = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_string();
        jobs.extend(parse_workflow_jobs(&file, &text));
    }
    jobs
}

/// Parse one workflow document into jobs. Pure, so the delegating-job and
/// `continue-on-error` cases are testable without a filesystem.
pub(crate) fn parse_workflow_jobs(file: &str, yaml: &str) -> Vec<WorkflowJob> {
    let Ok(doc) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(yaml) else {
        return Vec::new();
    };
    let Some(jobs) = doc.get("jobs").and_then(|j| j.as_mapping()) else {
        return Vec::new();
    };
    jobs.iter()
        .filter_map(|(id, body)| {
            let id = id.as_str()?;
            Some(WorkflowJob {
                file: file.to_string(),
                context: body
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or(id)
                    .to_string(),
                delegates: body.get("uses").is_some(),
                continue_on_error: body
                    .get("continue-on-error")
                    .and_then(|c| c.as_bool())
                    .unwrap_or(false),
                run_script: job_run_script(body),
            })
        })
        .collect()
}

/// Concatenate every `run:` script in a job body, including the `run:` of any
/// step that also carries `continue-on-error: true` (which is why the marker
/// is appended — a swallowed step must not read as blocking).
fn job_run_script(body: &serde_yaml_ng::Value) -> String {
    let Some(steps) = body.get("steps").and_then(|s| s.as_sequence()) else {
        return String::new();
    };
    let mut out = String::new();
    for step in steps {
        let Some(run) = step.get("run").and_then(|r| r.as_str()) else {
            continue;
        };
        let swallowed = step
            .get("continue-on-error")
            .and_then(|c| c.as_bool())
            .unwrap_or(false);
        for line in run.lines() {
            out.push_str(line);
            if swallowed {
                out.push_str(" || true");
            }
            out.push('\n');
        }
    }
    out
}

/// Date the RustSec advisory database.
///
/// `cargo deny` clones it under `$CARGO_HOME/advisory-dbs/<hash>/`; `cargo
/// audit` uses `$CARGO_HOME/advisory-db`. Both are checked and the newest
/// fetch wins, because either tool refreshing it makes the ground truth fresh.
pub(crate) fn advisory_db_age() -> AdvisoryDbAge {
    let home = std::env::var("CARGO_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| std::path::PathBuf::from(h).join(".cargo")));
    let Ok(home) = home else {
        return AdvisoryDbAge::Unknown("neither CARGO_HOME nor HOME is set".to_string());
    };

    let mut candidates: Vec<std::path::PathBuf> = vec![home.join("advisory-db")];
    if let Ok(entries) = std::fs::read_dir(home.join("advisory-dbs")) {
        candidates.extend(entries.filter_map(Result::ok).map(|e| e.path()));
    }

    let newest = candidates
        .iter()
        .filter_map(|db| newest_fetch_time(db))
        .max();
    match newest {
        Some(t) => match SystemTime::now().duration_since(t) {
            Ok(age) => AdvisoryDbAge::Known(age),
            // A fetch stamped in the future is a broken clock, not freshness.
            Err(_) => AdvisoryDbAge::Known(Duration::ZERO),
        },
        None => AdvisoryDbAge::Unknown(format!(
            "no advisory database found under {} (run `cargo deny check advisories` once)",
            home.display()
        )),
    }
}

/// Most recent fetch stamp inside a cloned advisory database.
fn newest_fetch_time(db: &Path) -> Option<SystemTime> {
    ["FETCH_HEAD", "HEAD"]
        .iter()
        .filter_map(|f| std::fs::metadata(db.join(".git").join(f)).ok())
        .filter_map(|m| m.modified().ok())
        .max()
}

// ---------------------------------------------------------------------------
// CB-1702 — review changeset size (ADVISORY)
// ---------------------------------------------------------------------------

/// One sampled commit's changed-line count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffSizeSample {
    pub commit: String,
    pub lines_changed: u64,
}

/// Changeset size above which Rigby & Bird (FSE 2013) observe review
/// effectiveness falling off. It is a *reference point from an observational
/// study*, not a defensible universal threshold, which is exactly why CB-1702
/// is advisory: the paper supports "bigger reviews find proportionally less",
/// not "N lines is wrong".
pub(crate) const DIFF_SIZE_REFERENCE_LINES: u64 = 400;

/// CB-1702: distribution of changeset sizes over recent history.
///
/// **Advisory by design.** This function returns `Pass`, `Warn` or `Skip` and
/// never `Fail`, for every input. `ComplianceReport::is_compliant` is
/// `failures == 0` over *statuses*, so a rule that never emits `Fail` can
/// never move the verdict, whatever severity a config assigns it — and
/// `validate_advisory_only_severities` additionally refuses a config that
/// tries to escalate it.
pub(crate) fn evaluate_diff_size(samples: &[DiffSizeSample]) -> ComplianceCheck {
    let name = "CB-1702: Review Changeset Size (advisory)".to_string();
    if samples.is_empty() {
        return ComplianceCheck {
            name,
            status: CheckStatus::Skip,
            message: "no commits sampled: nothing to describe".to_string(),
            severity: Severity::Info,
        };
    }
    let mut sizes: Vec<u64> = samples.iter().map(|s| s.lines_changed).collect();
    sizes.sort_unstable();
    let median = sizes[sizes.len() / 2];
    let over = sizes
        .iter()
        .filter(|n| **n > DIFF_SIZE_REFERENCE_LINES)
        .count();
    let message = format!(
        "{} commits sampled; median {median} lines changed; {over} ({:.0}%) exceed the \
         {DIFF_SIZE_REFERENCE_LINES}-line Rigby & Bird reference. Informational only — no \
         defensible absolute threshold exists, so this never fails a run.",
        sizes.len(),
        100.0 * over as f64 / sizes.len() as f64
    );
    ComplianceCheck {
        name,
        status: if over * 2 > sizes.len() {
            CheckStatus::Warn
        } else {
            CheckStatus::Pass
        },
        message,
        severity: Severity::Info,
    }
}

/// Sample changed-line counts for the last `limit` commits.
pub(crate) fn sample_diff_sizes(project_path: &Path, limit: usize) -> Vec<DiffSizeSample> {
    let out = run_bounded(Command::new("git").arg("-C").arg(project_path).args([
        "log",
        &format!("-n{limit}"),
        "--no-merges",
        "--pretty=format:C %H",
        "--numstat",
    ]));
    let Ok(out) = out else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    parse_numstat_log(&String::from_utf8_lossy(&out.stdout))
}

/// Parse `git log --pretty=format:'C %H' --numstat` output. Pure.
pub(crate) fn parse_numstat_log(text: &str) -> Vec<DiffSizeSample> {
    let mut samples: Vec<DiffSizeSample> = Vec::new();
    for line in text.lines() {
        if let Some(sha) = line.strip_prefix("C ") {
            samples.push(DiffSizeSample {
                commit: sha.trim().to_string(),
                lines_changed: 0,
            });
            continue;
        }
        let mut cols = line.split('\t');
        let (Some(add), Some(del)) = (cols.next(), cols.next()) else {
            continue;
        };
        // Binary files are reported as `-`; they contribute no reviewable lines.
        let add: u64 = add.trim().parse().unwrap_or(0);
        let del: u64 = del.trim().parse().unwrap_or(0);
        if let Some(last) = samples.last_mut() {
            last.lines_changed += add + del;
        }
    }
    samples
}

// ---------------------------------------------------------------------------
// CB-1703 — documented rule count
// ---------------------------------------------------------------------------

/// A prose claim about how many comply checks exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuleCountClaim {
    /// No countable claim was found.
    Absent,
    /// `number` checks were claimed; `plus` records a trailing `+`.
    Stated { number: usize, plus: bool },
}

/// Extract a "N automated checks" / "N+ automated checks" claim from prose.
pub(crate) fn extract_rule_count_claim(text: &str) -> RuleCountClaim {
    for line in text.lines() {
        let Some(idx) = line.find("automated checks") else {
            continue;
        };
        let head = line[..idx].trim_end();
        let token = head.split_whitespace().next_back().unwrap_or_default();
        let plus = token.ends_with('+');
        let digits = token.trim_end_matches('+');
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        if let Ok(number) = digits.parse::<usize>() {
            return RuleCountClaim::Stated { number, plus };
        }
    }
    RuleCountClaim::Absent
}

/// CB-1703: the documented check count must equal the enumerated one.
///
/// The README said "30+ automated checks" while the builders registered 150
/// clause ids — an understatement by a factor of five that nothing could
/// detect, because a prose number with no gate behind it is unfalsifiable in
/// either direction. `plus` is deliberately not treated as a lower bound:
/// "30+" is arithmetically true of 150 and still useless, and only an exact
/// binding makes "add a rule without touching the README" go red.
pub(crate) fn evaluate_rule_count_claim(
    claim: &RuleCountClaim,
    mentions_comply: bool,
    registry: Option<usize>,
) -> ComplianceCheck {
    let name = "CB-1703: Documented Rule Count".to_string();
    let skip = |msg: &str| ComplianceCheck {
        name: name.clone(),
        status: CheckStatus::Skip,
        message: msg.to_string(),
        severity: Severity::Info,
    };
    let fail = |msg: String| ComplianceCheck {
        name: name.clone(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    };

    if !mentions_comply {
        return skip("documentation does not describe `pmat comply`, so it claims no check count");
    }
    let Some(actual) = registry else {
        return fail(
            "the comply rule registry could not be enumerated, so the documented count cannot be \
             checked against anything — unknown is not a match"
                .to_string(),
        );
    };
    match claim {
        RuleCountClaim::Absent => fail(format!(
            "documentation describes `pmat comply` but states no check count; {actual} rules are \
             registered and nothing binds the prose to them"
        )),
        RuleCountClaim::Stated { number, plus } if *number == actual => ComplianceCheck {
            name,
            status: CheckStatus::Pass,
            message: format!(
                "documented count {number}{} matches the {actual} registered rules",
                if *plus { "+" } else { "" }
            ),
            severity: Severity::Info,
        },
        RuleCountClaim::Stated { number, plus } => fail(format!(
            "documentation claims {number}{} automated checks; {actual} rules are registered \
             ({} by {}). Update the prose or the registry.",
            if *plus { "+" } else { "" },
            if *number < actual {
                "understated"
            } else {
                "overstated"
            },
            actual.abs_diff(*number)
        )),
    }
}

/// Enumerate the comply rule registry: every clause id the check builders
/// register with `filter_check_by_config` / `run_checks_parallel`.
///
/// This is the set `.pmat.yaml` can address, so it is the honest answer to
/// "how many checks are there". It is deliberately *not* `grep -o 'CB-[0-9]*'
/// src/`, which counts comments, doc tables and test fixtures and reports 260
/// for a tree with 150 rules.
///
/// Returns `None` when no builder file exists, so callers fail closed instead
/// of reporting a registry of size zero.
pub(crate) fn enumerate_comply_rule_ids(project_path: &Path) -> Option<BTreeSet<String>> {
    let dir = project_path
        .join("src")
        .join("cli")
        .join("handlers")
        .join("comply_handlers")
        .join("check_handlers");
    let entries = std::fs::read_dir(&dir).ok()?;
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|f| f.to_str())
                .is_some_and(|f| f.starts_with("check_builders_") && f.ends_with(".rs"))
        })
        .collect();
    if files.is_empty() {
        return None;
    }
    files.sort();
    let mut ids = BTreeSet::new();
    for path in files {
        if let Ok(text) = std::fs::read_to_string(&path) {
            ids.extend(extract_clause_ids(&text));
        }
    }
    Some(ids)
}

/// Pull `"cb-nnnn"` string literals out of builder source. Pure.
pub(crate) fn extract_clause_ids(source: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    while let Some(rel) = source[i..].find("\"cb-") {
        let start = i + rel + 1;
        let Some(len) = bytes[start..].iter().position(|b| *b == b'"') else {
            break;
        };
        let candidate = &source[start..start + len];
        if is_clause_id(candidate) {
            ids.insert(candidate.to_string());
        }
        i = start + len + 1;
    }
    ids
}

/// `cb-` followed by digits, optionally `-` plus one alphanumeric suffix
/// (`cb-081-f`). Anything else is prose, not a clause id.
fn is_clause_id(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("cb-") else {
        return false;
    };
    let mut parts = rest.split('-');
    let Some(num) = parts.next() else {
        return false;
    };
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    match parts.next() {
        None => true,
        Some(suffix) => {
            parts.next().is_none()
                && !suffix.is_empty()
                && suffix.chars().all(|c| c.is_ascii_alphanumeric())
        }
    }
}

// ---------------------------------------------------------------------------
// Advisory-only severity guard (CB-1702's falsifier)
// ---------------------------------------------------------------------------

/// Reject a configuration that escalates an advisory-only rule.
///
/// CB-1702 reports an observational signal with no defensible absolute
/// threshold. Letting a project set `severity: error` on it would relabel that
/// signal as a defect and reintroduce exactly the false precision the rule was
/// written to avoid, so the configuration is refused rather than obeyed. The
/// refusal is loud: `compute_compliance_report` propagates it and the run
/// stops, because silently ignoring the setting would leave the operator
/// believing an escalation was in force.
pub(crate) fn validate_advisory_only_severities(
    config: &crate::models::comply_config::ComplyConfig,
) -> Result<(), String> {
    use crate::models::comply_config::CheckSeverity;
    let mut bad = Vec::new();
    for id in ADVISORY_ONLY_CHECKS {
        if let Some(entry) = config.checks.get(*id) {
            match entry.severity {
                CheckSeverity::Error | CheckSeverity::Critical => bad.push(format!(
                    "{id} is advisory-only and cannot be set to {:?}",
                    entry.severity
                )),
                CheckSeverity::Info | CheckSeverity::Warning => {}
            }
        }
    }
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "invalid .pmat.yaml: {}. These rules report observational signals with no defensible \
             absolute threshold; they may be disabled, but not promoted to a failure.",
            bad.join("; ")
        ))
    }
}
