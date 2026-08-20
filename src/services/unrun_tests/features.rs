//! `[features]` from `Cargo.toml`, and the closure a `--features` flag expands to.
//!
//! Parsed by hand rather than with `toml`, because `toml` is an *optional*
//! dependency here (it arrives via `standard-deps`). Depending on it would put
//! this analysis behind a feature flag — which is the defect the analysis
//! exists to find.

use std::collections::{BTreeMap, BTreeSet};

/// name -> the features it enables (dependency activations dropped).
pub type FeatureGraph = BTreeMap<String, Vec<String>>;

/// Parse the `[features]` table.
///
/// Entries of the form `dep:foo` and `some-dep/feat` activate a *dependency*,
/// never a feature of this crate, so they cannot make a `#[cfg(feature = …)]`
/// in this crate true and are dropped.
#[must_use]
pub fn parse(cargo_toml: &str) -> FeatureGraph {
    let mut out = FeatureGraph::new();
    let mut in_features = false;
    let mut pending: Option<(String, String)> = None;
    for raw in cargo_toml.lines() {
        let line = strip_comment(raw);
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_features = trimmed == "[features]";
            continue;
        }
        if !in_features {
            continue;
        }
        match pending.take() {
            Some((name, mut buf)) => {
                buf.push_str(trimmed);
                if buf.contains(']') {
                    out.insert(name, deps(&buf));
                } else {
                    pending = Some((name, buf));
                }
            }
            None => {
                let Some((name, rest)) = trimmed.split_once('=') else {
                    continue;
                };
                let name = name.trim().trim_matches('"').to_string();
                if name.is_empty() {
                    continue;
                }
                let rest = rest.trim().to_string();
                if rest.contains(']') {
                    out.insert(name, deps(&rest));
                } else if rest.starts_with('[') {
                    pending = Some((name, rest));
                }
            }
        }
    }
    out
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        // A `#` inside a string is not a comment; feature names have no `"`
        // before the `#` in this table, so a quote count of zero is the test.
        Some(i) if !line[..i].contains('"') => &line[..i],
        _ => line,
    }
}

fn deps(list: &str) -> Vec<String> {
    list.split('"')
        .skip(1)
        .step_by(2)
        .filter(|d| !d.starts_with("dep:") && !d.contains('/'))
        .map(str::to_string)
        .collect()
}

/// Every feature a `--features a,b` invocation ends up enabling.
///
/// Names not present in the table are still inserted: a `#[cfg(feature = "x")]`
/// guarded on an undeclared feature is dead code, but recording it keeps the
/// closure a faithful record of what was asked for.
#[must_use]
pub fn closure<I, S>(graph: &FeatureGraph, roots: I) -> BTreeSet<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = BTreeSet::new();
    let mut stack: Vec<String> = roots
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    while let Some(f) = stack.pop() {
        if !seen.insert(f.clone()) {
            continue;
        }
        if let Some(children) = graph.get(&f) {
            stack.extend(children.iter().cloned());
        }
    }
    seen
}
