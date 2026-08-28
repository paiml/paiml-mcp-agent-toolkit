// CB-2101: `pmat comply coherence` — classify every threshold, with reasons.
//
// Included from comply_handlers/mod.rs, which owns the imports.

/// Audit `.pmat-metrics.toml` and print one row per threshold.
///
/// Same judgement CB-2101 makes inside `pmat comply check`; this is the form
/// that fits on a screen and, with `--format json`, the form a CI job can act
/// on. A single-line comply message can say WHAT is wrong with seventeen
/// numbers; only the structured report can say what each one should become.
///
/// Fails closed: an absent or unparsable config on either side is an error,
/// never a quiet "nothing to report".
fn handle_coherence(project_path: &Path, format: ComplyOutputFormat) -> Result<()> {
    use crate::services::metrics_ratchet::{
        self,
        config::{Outcome, METRICS_FILE, RATCHET_FILE},
        RatchetStatus,
    };

    match metrics_ratchet::status(project_path) {
        RatchetStatus::Absent => anyhow::bail!(
            "{RATCHET_FILE} does not exist, so no threshold in {METRICS_FILE} is bound to \
             anything and none of them can be classified"
        ),
        RatchetStatus::Deleted => anyhow::bail!(
            "{RATCHET_FILE} was committed and is now gone — deleting the file that binds \
             every threshold is not a way of passing the audit"
        ),
        RatchetStatus::Present => {}
    }

    let report =
        metrics_ratchet::run_coherence(project_path).map_err(|e| anyhow::anyhow!(e.to_string()))?;

    match format {
        ComplyOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        _ => {
            for t in &report.thresholds {
                println!(
                    "{:<44} {:<9} {:<9} configured={:<12} {}",
                    t.key,
                    t.classification.as_str(),
                    t.kind,
                    t.configured,
                    t.detail
                );
            }
            for s in &report.undeclared_sections {
                println!("UNDECLARED SECTION: [{s}]");
            }
            println!(
                "{METRICS_FILE}: {} threshold(s) classified",
                report.thresholds.len()
            );
        }
    }

    if report.outcome == Outcome::Fail {
        anyhow::bail!("{METRICS_FILE}: the threshold audit is red");
    }
    Ok(())
}
