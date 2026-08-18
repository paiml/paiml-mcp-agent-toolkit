//! Find where a job actually invokes a command — directly in a `run:` block, or
//! one hop away through `make <target>` or a shell script.
//!
//! Indirection is where gates go to die: a workflow step reads
//! `make quality-gate`, the Makefile recipe reads `pmat quality-gate || exit 1`,
//! and nobody notices that the tool exited 0 while printing FAILED. Resolving
//! the hop is the difference between "a job mentions the gate" and "the gate
//! can fail this job".

use super::effect;
use super::workflow::Job;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One resolved invocation site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub workflow: PathBuf,
    pub job_id: String,
    /// Step name, or `step #N` when the step is unnamed.
    pub step: String,
    /// How the command was reached: `run`, `make <target>`, `scripts/x.sh`.
    pub via: String,
    /// Reasons the exit code cannot reach the required check. Empty ⇒ enforcing.
    pub suppressions: Vec<String>,
}

impl Invocation {
    pub fn is_enforcing(&self) -> bool {
        self.suppressions.is_empty()
    }
}

/// Every invocation of `needle` reachable from this job's own steps.
///
/// `job_suppressions` are folded into every invocation found — a job-level
/// `continue-on-error` neuters everything inside it, however carefully the
/// individual step handles its own exit code.
pub fn find_in_job(
    project_path: &Path,
    job: &Job,
    needles: &[&str],
    job_suppressions: &[String],
) -> Vec<Invocation> {
    let mut out = Vec::new();
    for (i, step) in job.steps.iter().enumerate() {
        let Some(script) = step.run.as_deref() else {
            continue;
        };
        let label = step
            .name
            .clone()
            .unwrap_or_else(|| format!("step #{}", i + 1));
        let mut suppressions = job_suppressions.to_vec();
        if step.continue_on_error.suppresses() {
            suppressions.push(format!(
                "step `{label}` carries continue-on-error, so its failure never fails the job"
            ));
        }
        collect_from_script(
            project_path,
            job,
            &label,
            script,
            needles,
            &suppressions,
            &mut out,
            0,
        );
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn collect_from_script(
    project_path: &Path,
    job: &Job,
    label: &str,
    script: &str,
    needles: &[&str],
    inherited: &[String],
    out: &mut Vec<Invocation>,
    depth: usize,
) {
    if depth > 3 {
        return;
    }
    let via = if depth == 0 { "run" } else { "indirect" };
    for needle in needles {
        let Some(idx) = effect::find_line(script, needle) else {
            continue;
        };
        let mut suppressions = inherited.to_vec();
        suppressions.extend(effect::assess(script, idx));
        if selects_a_rule_subset(script.lines().nth(idx).unwrap_or("")) {
            suppressions.push(
                "the invocation restricts which rules run, so it cannot stand in for the \
                 whole error-severity roster"
                    .into(),
            );
        }
        out.push(Invocation {
            workflow: job.workflow.clone(),
            job_id: job.id.clone(),
            step: label.to_string(),
            via: via.to_string(),
            suppressions,
        });
    }
    for (label2, body) in indirect_targets(project_path, script) {
        collect_from_script(
            project_path,
            job,
            &format!("{label} -> {label2}"),
            &body,
            needles,
            inherited,
            out,
            depth + 1,
        );
    }
}

/// Flags that would make one `pmat comply check` run a subset of the rules.
/// None exist today; the list is here so that adding one cannot silently turn a
/// partial run into evidence for the whole roster.
const RULE_SUBSET_FLAGS: &[&str] = &["--checks", "--only", "--rules", "--skip", "--check-id"];

fn selects_a_rule_subset(line: &str) -> bool {
    RULE_SUBSET_FLAGS.iter().any(|f| line.contains(f))
}

/// Bodies reachable one hop from `script`: Makefile recipes and shell scripts.
fn indirect_targets(project_path: &Path, script: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let makefile = load_makefile(project_path);
    for line in script.lines() {
        let code = line.split('#').next().unwrap_or("");
        for target in make_targets(code) {
            if let Some(recipe) = makefile.get(&target) {
                out.push((format!("make {target}"), recipe.clone()));
            }
        }
        if let Some(path) = script_path(code) {
            if let Ok(body) = std::fs::read_to_string(project_path.join(&path)) {
                out.push((path, body));
            }
        }
    }
    out
}

fn make_targets(code: &str) -> Vec<String> {
    let mut out = Vec::new();
    let toks: Vec<&str> = code.split_whitespace().collect();
    for (i, t) in toks.iter().enumerate() {
        if *t == "make" || *t == "$(MAKE)" || *t == "${MAKE}" {
            for cand in toks.iter().skip(i + 1) {
                if cand.starts_with('-') || cand.contains('=') {
                    continue;
                }
                out.push((*cand).to_string());
                break;
            }
        }
    }
    out
}

fn script_path(code: &str) -> Option<String> {
    code.split_whitespace()
        .map(|t| t.trim_start_matches("./"))
        .find(|t| t.ends_with(".sh") && !t.starts_with('-'))
        .map(str::to_string)
}

/// Very small Makefile reader: target name → recipe body. Enough to follow a
/// workflow step into a recipe; not a Make implementation.
fn load_makefile(project_path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(text) = std::fs::read_to_string(project_path.join("Makefile")) else {
        return map;
    };
    let mut current: Option<String> = None;
    let mut body = String::new();
    for line in text.lines() {
        if line.starts_with('\t') {
            if current.is_some() {
                body.push_str(line.trim_start_matches('\t').trim_start_matches('@'));
                body.push('\n');
            }
            continue;
        }
        if let Some(name) = current.take() {
            map.insert(name, std::mem::take(&mut body));
        }
        current = target_name(line);
    }
    if let Some(name) = current {
        map.insert(name, body);
    }
    map
}

fn target_name(line: &str) -> Option<String> {
    let colon = line.find(':')?;
    if line.starts_with(' ') || line.starts_with('#') {
        return None;
    }
    let name = line[..colon].trim();
    let after = line[colon..].chars().nth(1);
    if after == Some('=') || name.is_empty() || name.contains(' ') {
        return None;
    }
    Some(name.to_string())
}
