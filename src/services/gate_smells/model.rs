//! Minimal, read-only view of a GitHub Actions workflow file.

use serde_yaml_ng::Value;

/// One detected smell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub id: &'static str,
    pub file: String,
    pub job: String,
    pub message: String,
}

/// What a detector measured. `not_measured` is never a pass (L23).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub not_measured: Vec<String>,
}

impl Report {
    pub fn extend(&mut self, other: Report) {
        self.findings.extend(other.findings);
        self.not_measured.extend(other.not_measured);
    }
}

/// A parsed workflow file.
pub struct Workflow {
    pub path: String,
    doc: Value,
}

/// A job as the detectors see it.
pub(crate) struct Job<'a> {
    pub id: String,
    pub node: &'a Value,
}

fn as_strings(v: &Value) -> Vec<String> {
    match v {
        Value::String(s) => vec![s.clone()],
        Value::Sequence(seq) => seq
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        Value::Mapping(map) => map
            .keys()
            .filter_map(|k| k.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

impl Workflow {
    pub fn parse(path: &str, text: &str) -> Result<Self, String> {
        let doc: Value = serde_yaml_ng::from_str(text).map_err(|e| format!("{path}: {e}"))?;
        Ok(Self {
            path: path.to_string(),
            doc,
        })
    }

    /// The `on:` node. A YAML 1.1 reader turns the key `on` into boolean `true`.
    fn on_node(&self) -> Option<&Value> {
        self.doc
            .get("on")
            .or_else(|| self.doc.get(Value::Bool(true)))
    }

    pub(crate) fn triggers(&self) -> Vec<String> {
        self.on_node().map(as_strings).unwrap_or_default()
    }

    /// True when a PR (or a merge queue) starts this workflow.
    pub(crate) fn on_pr_path(&self) -> bool {
        self.triggers().iter().any(|t| {
            matches!(
                t.as_str(),
                "pull_request" | "pull_request_target" | "merge_group"
            )
        })
    }

    /// `on.pull_request.paths`, or None when the workflow is not path-filtered.
    pub(crate) fn pr_paths(&self) -> Option<Vec<String>> {
        let pr = self.on_node()?.get("pull_request")?;
        pr.get("paths").map(as_strings)
    }

    pub(crate) fn jobs(&self) -> Vec<Job<'_>> {
        let Some(map) = self.doc.get("jobs").and_then(Value::as_mapping) else {
            return Vec::new();
        };
        map.iter()
            .filter_map(|(k, node)| {
                k.as_str().map(|id| Job {
                    id: id.to_string(),
                    node,
                })
            })
            .collect()
    }

    /// A job runs on the PR path when the workflow does and the job's own `if:` does not
    /// confine it to schedule or manual runs.
    pub(crate) fn job_on_pr_path(&self, job: &Job<'_>) -> bool {
        if !self.on_pr_path() {
            return false;
        }
        let cond = job.node.get("if").and_then(Value::as_str).unwrap_or("");
        !confined_off_pr(cond)
    }
}

impl Job<'_> {
    pub(crate) fn steps(&self) -> Vec<&Value> {
        self.node
            .get("steps")
            .and_then(Value::as_sequence)
            .map(|s| s.iter().collect())
            .unwrap_or_default()
    }

    /// Every non-comment, non-blank line of every `run:` script, trimmed.
    pub(crate) fn run_lines(&self) -> Vec<String> {
        self.steps()
            .iter()
            .filter_map(|s| s.get("run").and_then(Value::as_str))
            .flat_map(str::lines)
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect()
    }

    /// `uses:` of the job itself (reusable workflow) and of every step.
    pub(crate) fn uses(&self) -> Vec<String> {
        let own = self.node.get("uses").and_then(Value::as_str);
        let steps = self
            .steps()
            .into_iter()
            .filter_map(|s| s.get("uses").and_then(Value::as_str));
        own.into_iter().chain(steps).map(str::to_string).collect()
    }

    pub(crate) fn needs(&self) -> Vec<String> {
        self.node.get("needs").map(as_strings).unwrap_or_default()
    }

    /// `runs-on` labels, lower-cased. A `${{ }}` expression yields no label.
    pub(crate) fn labels(&self) -> Vec<String> {
        let Some(ro) = self.node.get("runs-on") else {
            return Vec::new();
        };
        let mut out = match ro {
            Value::Mapping(_) => ro.get("labels").map(as_strings).unwrap_or_default(),
            other => as_strings(other),
        };
        out.retain(|l| !l.contains("${{"));
        out.iter().map(|l| l.to_lowercase()).collect()
    }
}

/// True when an `if:` confines a job to schedule or manual runs. A negated test
/// (`event_name != 'schedule'`) keeps the job on the PR path, so those are removed
/// before looking for the event names.
fn confined_off_pr(cond: &str) -> bool {
    let mut c: String = cond.split_whitespace().collect::<Vec<_>>().join("");
    for kw in ["schedule", "workflow_dispatch"] {
        for q in ["'", "\""] {
            c = c.replace(&format!("!={q}{kw}{q}"), "");
        }
    }
    (c.contains("schedule") || c.contains("workflow_dispatch")) && !c.contains("pull_request")
}
