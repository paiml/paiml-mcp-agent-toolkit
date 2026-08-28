//! GitHub Actions workflow model: just enough of the schema to answer
//! "which *check-run context string* does this job report as, and what does it run".
//!
//! The context string is the thing branch protection matches on, and it is NOT
//! the job id and NOT always the display name:
//!
//! * a normal job reports as `name:` if present, else its job id;
//! * a job that `uses:` a reusable workflow reports one context per job in the
//!   *callee*, namespaced as `<caller context> / <callee context>`.
//!
//! That second rule is the whole reason this module exists. A repo can have a
//! top-level job whose display name is `gate` (reporting as `gate`) while the
//! required context is `ci / gate` — a different job, in a different file,
//! possibly in a different repository. Matching on display names calls such a
//! repo compliant on the wrong job.

use std::path::{Path, PathBuf};

/// One step of a job. Only the fields that can carry — or silently swallow — a
/// failure are modelled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub name: Option<String>,
    pub run: Option<String>,
    pub uses: Option<String>,
    pub continue_on_error: TriState,
}

/// `continue-on-error:` is a bool *or* an expression. An expression cannot be
/// evaluated statically, and "might be true" is not "provably propagates", so
/// it is kept distinct rather than folded into `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriState {
    No,
    Yes,
    /// `${{ ... }}` — value unknown at analysis time.
    Unknown,
}

impl TriState {
    /// True when the key cannot be shown to leave failure propagation intact.
    pub fn suppresses(self) -> bool {
        !matches!(self, TriState::No)
    }
}

/// One job, with the fields that determine whether its failure can reach the
/// required check.
#[derive(Debug, Clone)]
pub struct Job {
    /// Workflow file this job was declared in, relative to the repo root.
    pub workflow: PathBuf,
    pub id: String,
    pub display_name: Option<String>,
    pub continue_on_error: TriState,
    pub needs: Vec<String>,
    pub if_expr: Option<String>,
    /// `uses:` at job level — a reusable-workflow call.
    pub uses: Option<String>,
    pub steps: Vec<Step>,
}

impl Job {
    /// The check-run context this job reports as when it is a top-level job.
    pub fn context(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.id)
    }

    /// Every `run:` script in the job, in order.
    pub fn run_scripts(&self) -> impl Iterator<Item = &str> {
        self.steps.iter().filter_map(|s| s.run.as_deref())
    }
}

/// A parsed workflow file.
#[derive(Debug, Clone)]
pub struct Workflow {
    pub path: PathBuf,
    pub jobs: Vec<Job>,
}

/// The whole `.github/workflows` directory.
///
/// `unparsable` is carried, never dropped: a workflow that failed to parse is a
/// hole in the reachability graph, and a hole must fail the analysis rather
/// than shrink the job set until everything looks reachable.
#[derive(Debug, Clone, Default)]
pub struct WorkflowSet {
    pub workflows: Vec<Workflow>,
    pub unparsable: Vec<(PathBuf, String)>,
}

impl WorkflowSet {
    pub fn jobs(&self) -> impl Iterator<Item = &Job> {
        self.workflows.iter().flat_map(|w| w.jobs.iter())
    }

    pub fn job(&self, workflow: &Path, id: &str) -> Option<&Job> {
        self.jobs().find(|j| j.workflow == workflow && j.id == id)
    }

    /// Total job count across every parsed workflow.
    pub fn job_count(&self) -> usize {
        self.workflows.iter().map(|w| w.jobs.len()).sum()
    }
}

/// Load and parse every `*.yml` / `*.yaml` under `.github/workflows`.
///
/// Files that do not parse are recorded in [`WorkflowSet::unparsable`] rather
/// than skipped. `.disabled` suffixes are ignored on purpose: GitHub does not
/// run them, so they cannot host a gate.
pub fn load_workflows(project_path: &Path) -> WorkflowSet {
    let dir = project_path.join(".github").join("workflows");
    let mut set = WorkflowSet::default();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return set;
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("yml") | Some("yaml")
            )
        })
        .collect();
    paths.sort();

    for path in paths {
        let rel = relative_to(project_path, &path);
        match std::fs::read_to_string(&path) {
            Ok(text) => match parse_workflow(&rel, &text) {
                Ok(wf) => set.workflows.push(wf),
                Err(e) => set.unparsable.push((rel, e)),
            },
            Err(e) => set.unparsable.push((rel, e.to_string())),
        }
    }
    set
}

fn relative_to(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

/// Parse one workflow's YAML text into a [`Workflow`].
///
/// A workflow with no `jobs:` mapping parses to zero jobs — it is not an error
/// here, but it is also not a gate, and the reachability pass fails on it.
pub fn parse_workflow(path: &Path, text: &str) -> Result<Workflow, String> {
    let doc: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?;
    let jobs_map = doc.get("jobs").and_then(|j| j.as_mapping());
    let mut jobs = Vec::new();
    if let Some(map) = jobs_map {
        for (k, v) in map {
            let Some(id) = k.as_str() else { continue };
            jobs.push(parse_job(path, id, v));
        }
    }
    Ok(Workflow {
        path: path.to_path_buf(),
        jobs,
    })
}

fn parse_job(path: &Path, id: &str, v: &serde_yaml_ng::Value) -> Job {
    Job {
        workflow: path.to_path_buf(),
        id: id.to_string(),
        display_name: v.get("name").and_then(|n| n.as_str()).map(str::to_string),
        continue_on_error: tri_state(v.get("continue-on-error")),
        needs: parse_needs(v.get("needs")),
        if_expr: v.get("if").and_then(scalar_text),
        uses: v.get("uses").and_then(|u| u.as_str()).map(str::to_string),
        steps: parse_steps(v.get("steps")),
    }
}

/// `if:` may be a string, a bool or a number; all of them stringify.
fn scalar_text(v: &serde_yaml_ng::Value) -> Option<String> {
    match v {
        serde_yaml_ng::Value::String(s) => Some(s.clone()),
        serde_yaml_ng::Value::Bool(b) => Some(b.to_string()),
        serde_yaml_ng::Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn parse_needs(v: Option<&serde_yaml_ng::Value>) -> Vec<String> {
    match v {
        Some(serde_yaml_ng::Value::String(s)) => vec![s.clone()],
        Some(serde_yaml_ng::Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|x| x.as_str())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_steps(v: Option<&serde_yaml_ng::Value>) -> Vec<Step> {
    let Some(serde_yaml_ng::Value::Sequence(seq)) = v else {
        return Vec::new();
    };
    seq.iter()
        .map(|s| Step {
            name: s.get("name").and_then(|n| n.as_str()).map(str::to_string),
            run: s.get("run").and_then(scalar_text),
            uses: s.get("uses").and_then(|u| u.as_str()).map(str::to_string),
            continue_on_error: tri_state(s.get("continue-on-error")),
        })
        .collect()
}

fn tri_state(v: Option<&serde_yaml_ng::Value>) -> TriState {
    match v {
        None => TriState::No,
        Some(serde_yaml_ng::Value::Bool(true)) => TriState::Yes,
        Some(serde_yaml_ng::Value::Bool(false)) => TriState::No,
        Some(serde_yaml_ng::Value::String(s)) => {
            let t = s.trim();
            match t {
                "true" => TriState::Yes,
                "false" => TriState::No,
                _ => TriState::Unknown,
            }
        }
        Some(_) => TriState::Unknown,
    }
}
