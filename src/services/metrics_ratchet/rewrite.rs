//! Writing a lowered baseline back into `.pmat-ratchet.toml`.
//!
//! Surgical, line-level editing rather than serialise-the-parsed-struct. The
//! file's comments are the reason anyone can audit it — what each predicate
//! means, why a filename heuristic was rejected, which commit the numbers were
//! captured at — and a round-trip through a TOML serialiser deletes every one
//! of them. A scheduled job that quietly strips a file's documentation each
//! time it runs is a worse outcome than not having the job.

use super::config::{MetricBaseline, RatchetConfig, RATCHET_FILE};
use super::kernel::next_baseline;
use std::collections::BTreeMap;

/// What the lowering job would write: `min(baseline, observed)` per metric.
///
/// Never raises (`INV-2102-2`), and never lowers a metric that was not
/// measured (`FALSIFY-2102-4`) — an absent measurement is a failure, and
/// baking a failure into the baseline would convert it into the new truth.
pub fn lowered_baselines(
    metrics: &BTreeMap<String, MetricBaseline>,
    measurements: &super::config::Measurements,
) -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    for (id, m) in metrics {
        let Some(super::config::Measurement::Value(v)) = measurements.get(id) else {
            continue;
        };
        let next = next_baseline(m.baseline, *v);
        if next < m.baseline {
            out.insert(id.clone(), next);
        }
    }
    out
}

/// Apply `new_baselines` to `text`, in place.
///
/// For each named metric: replace its `baseline = N` line and delete its
/// `justification` line, which no longer justifies anything once the baseline
/// has moved down. Everything else — comments, ordering, descriptions,
/// whitespace — is preserved byte for byte.
///
/// Fails closed: a metric whose `baseline` line cannot be found, a multi-line
/// `justification` this editor cannot safely delete, or a result that does not
/// re-parse into exactly the requested baselines, is an error and nothing is
/// written.
pub fn apply(text: &str, new_baselines: &BTreeMap<String, i64>) -> Result<String, String> {
    let mut out: Vec<String> = Vec::new();
    let mut section: Option<String> = None;
    let mut rewritten: BTreeMap<&String, bool> = new_baselines.keys().map(|k| (k, false)).collect();

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("[metric.")
            .and_then(|r| r.strip_suffix(']'))
        {
            section = Some(name.to_string());
            out.push(line.to_string());
            continue;
        }
        if trimmed.starts_with('[') {
            section = None;
            out.push(line.to_string());
            continue;
        }

        let Some(name) = section
            .as_deref()
            .filter(|s| new_baselines.contains_key(*s))
        else {
            out.push(line.to_string());
            continue;
        };

        if trimmed.starts_with("baseline") && trimmed.contains('=') {
            let indent = &line[..line.len() - line.trim_start().len()];
            out.push(format!("{indent}baseline = {}", new_baselines[name]));
            if let Some(done) = rewritten.iter_mut().find(|(k, _)| k.as_str() == name) {
                *done.1 = true;
            }
            continue;
        }
        if trimmed.starts_with("justification") && trimmed.contains('=') {
            if trimmed.contains("\"\"\"") || trimmed.contains("'''") {
                return Err(format!(
                    "metric `{name}` has a multi-line `justification`; this editor will not \
                     guess where it ends. Lower it by hand."
                ));
            }
            continue;
        }
        out.push(line.to_string());
    }

    if let Some((name, _)) = rewritten.iter().find(|(_, done)| !**done) {
        return Err(format!(
            "metric `{name}` has no `baseline =` line in {RATCHET_FILE}"
        ));
    }

    let mut result = out.join("\n");
    if text.ends_with('\n') {
        result.push('\n');
    }
    verify(&result, new_baselines)?;
    Ok(result)
}

/// Re-parse the rewritten text and confirm it says exactly what was asked.
/// An editor that produced something plausible-looking but wrong is the one
/// failure mode a ratchet cannot survive, because the wrong number silently
/// becomes the new truth.
fn verify(result: &str, new_baselines: &BTreeMap<String, i64>) -> Result<(), String> {
    let parsed = RatchetConfig::parse(result)
        .map_err(|e| format!("the rewritten {RATCHET_FILE} does not parse: {e}"))?;
    for (name, want) in new_baselines {
        let spec = parsed
            .metric
            .get(name)
            .ok_or_else(|| format!("the rewritten {RATCHET_FILE} lost metric `{name}`"))?;
        if spec.baseline != *want {
            return Err(format!(
                "the rewritten {RATCHET_FILE} says metric `{name}` is {}, not {want}",
                spec.baseline
            ));
        }
        if spec.justification.is_some() {
            return Err(format!(
                "the rewritten {RATCHET_FILE} still carries a `justification` on `{name}`"
            ));
        }
    }
    Ok(())
}
