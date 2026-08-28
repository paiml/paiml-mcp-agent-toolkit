//! Which CI invocations actually EXECUTE tests, derived from the workflow files.
//!
//! Derived, not hand-listed. A hand-listed set of legs stops covering legs added
//! later, which is the same failure mode `orphan-ledger` exists to prevent one
//! level up. It also cannot notice a leg being *removed*, which would silently
//! widen the unrun set without a single line of this analysis changing.
//!
//! `cargo check` is deliberately not a leg. It neither lints nor runs a test
//! body, and treating it as coverage is the exact hole this repository already
//! found for features: `mcp-integration` was compile-checked for months while
//! 910 of its tests, including three regression tests, had never executed.

use std::collections::BTreeMap;
use std::path::Path;

/// One `cargo test` invocation, resolved.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Leg {
    /// `feature-matrix.yml:feature-tests[full]` — where it came from.
    pub origin: String,
    /// Flags typed on the command line, before closure expansion.
    pub features: Vec<String>,
    pub default_features: bool,
    pub all_features: bool,
    /// Does this invocation build the lib test target?
    pub runs_lib: bool,
}

/// Scan every workflow for `cargo test` invocations.
#[must_use]
pub fn from_workflows(workflows_dir: &Path) -> Vec<Leg> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(workflows_dir) else {
        return out;
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| matches!(p.extension().and_then(|s| s.to_str()), Some("yml" | "yaml")))
        .collect();
    files.sort();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let name = f
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        out.extend(from_workflow(&name, &text));
    }
    out.sort();
    out.dedup();
    out
}

fn from_workflow(file: &str, text: &str) -> Vec<Leg> {
    let mut out = Vec::new();
    for (job, body) in jobs(text) {
        let matrix = matrix_values(&body);
        for cmd in test_commands(&body) {
            for leg in expand(file, &job, &cmd, &matrix) {
                out.push(leg);
            }
        }
    }
    out
}

/// Split a workflow into `(job id, job body)`. Jobs are the 2-space-indented
/// keys under a column-0 `jobs:`.
fn jobs(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut in_jobs = false;
    let mut cur: Option<(String, Vec<&str>)> = None;
    for line in text.lines() {
        if !line.starts_with(' ') && !line.trim().is_empty() {
            if let Some((id, body)) = cur.take() {
                out.push((id, body.join("\n")));
            }
            in_jobs = line.trim_end() == "jobs:";
            continue;
        }
        if !in_jobs {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if indent == 2 && !trimmed.starts_with('#') && trimmed.ends_with(':') {
            if let Some((id, body)) = cur.take() {
                out.push((id, body.join("\n")));
            }
            cur = Some((trimmed.trim_end_matches(':').to_string(), Vec::new()));
            continue;
        }
        if let Some((_, body)) = cur.as_mut() {
            body.push(line);
        }
    }
    if let Some((id, body)) = cur {
        out.push((id, body.join("\n")));
    }
    out
}

/// `strategy.matrix` entries, as `key -> [values]`. Both the `include:` list
/// form (`- features: full`) and the inline list form (`features: [a, b]`).
fn matrix_values(job: &str) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut in_matrix = false;
    let mut matrix_indent = 0usize;
    for line in job.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if trimmed == "matrix:" {
            in_matrix = true;
            matrix_indent = indent;
            continue;
        }
        if in_matrix && indent <= matrix_indent {
            in_matrix = false;
        }
        if !in_matrix {
            continue;
        }
        let entry = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        let Some((k, v)) = entry.split_once(':') else {
            continue;
        };
        let (k, v) = (k.trim(), v.trim().trim_matches(|c| c == '\'' || c == '"'));
        if v.is_empty() || v.contains("${{") {
            continue;
        }
        let vals: Vec<String> =
            if let Some(inner) = v.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                inner
                    .split(',')
                    .map(|s| s.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                vec![v.to_string()]
            };
        out.entry(k.to_string()).or_default().extend(vals);
    }
    out
}

/// Every `cargo test` command line in a job's `run:` scripts, comments removed.
fn test_commands(job: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut acc: Option<String> = None;
    for line in job.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let joined = match acc.take() {
            Some(prev) => format!("{prev} {trimmed}"),
            None => trimmed.to_string(),
        };
        if let Some(stripped) = joined.strip_suffix('\\') {
            acc = Some(stripped.trim_end().to_string());
            continue;
        }
        if joined.contains("cargo test") {
            out.push(joined);
        }
    }
    out
}

fn expand(file: &str, job: &str, cmd: &str, matrix: &BTreeMap<String, Vec<String>>) -> Vec<Leg> {
    let base = Leg {
        origin: format!("{file}:{job}"),
        features: Vec::new(),
        default_features: !cmd.contains("--no-default-features"),
        all_features: cmd.contains("--all-features"),
        // No target selector means every target, lib included.
        runs_lib: cmd.contains("--lib") || !cmd.contains("--test ") && !cmd.contains("--bins"),
    };
    let Some(spec) = feature_spec(cmd) else {
        return vec![base];
    };
    for (key, values) in matrix {
        let needle = format!("${{{{ matrix.{key} }}}}");
        if !spec.contains(&needle) {
            continue;
        }
        return values
            .iter()
            .map(|v| Leg {
                origin: format!("{file}:{job}[{v}]"),
                features: split_features(&spec.replace(&needle, v)),
                ..base.clone()
            })
            .collect();
    }
    if spec.contains("${{") {
        // An expression we cannot resolve. Contributing it as a leg would
        // silently mark tests as run; drop it, so the unrun set can only grow.
        return Vec::new();
    }
    vec![Leg {
        features: split_features(&spec),
        ..base
    }]
}

fn feature_spec(cmd: &str) -> Option<String> {
    let idx = cmd.find("--features")?;
    let rest = cmd[idx + "--features".len()..].trim_start();
    let rest = rest.strip_prefix('=').unwrap_or(rest).trim_start();
    let (quote, rest) = match rest.chars().next() {
        Some(q @ ('\'' | '"')) => (Some(q), &rest[1..]),
        _ => (None, rest),
    };
    let end = match quote {
        Some(q) => rest.find(q)?,
        None => rest.find(char::is_whitespace).unwrap_or(rest.len()),
    };
    Some(rest[..end].to_string())
}

fn split_features(spec: &str) -> Vec<String> {
    spec.split([',', ' '])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}
