// CB-2102: `pmat comply ratchet` — check the baselines, or lower them.
//
// Included from comply_handlers/mod.rs, which owns the imports.

/// Judge the ratchet, or run the lowering pass.
///
/// Fails closed. A missing or unparsable `.pmat-ratchet.toml`, a metric whose
/// command did not run, an empty metric set, or an unreadable git history are
/// all errors — never a ratchet that quietly reports everything is fine.
fn handle_ratchet(project_path: &Path, lower: bool) -> Result<()> {
    use crate::services::metrics_ratchet::{
        self,
        config::{Outcome, RATCHET_FILE},
        RatchetStatus,
    };

    match metrics_ratchet::status(project_path) {
        RatchetStatus::Absent => anyhow::bail!(
            "{RATCHET_FILE} does not exist, so this project declares no ratcheted baselines"
        ),
        RatchetStatus::Deleted => anyhow::bail!(
            "{RATCHET_FILE} was committed and is now gone — deleting a gate's input is not a \
             way of passing it"
        ),
        RatchetStatus::Present => {}
    }

    if lower {
        let changes = metrics_ratchet::lower(project_path).map_err(|e| anyhow::anyhow!(e))?;
        if changes.is_empty() {
            println!("{RATCHET_FILE}: nothing to lower");
        } else {
            println!("{RATCHET_FILE}: lowered {} baseline(s)", changes.len());
            for c in &changes {
                println!("  {c}");
            }
        }
        return Ok(());
    }

    let report = metrics_ratchet::run(project_path).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    for m in &report.metrics {
        println!(
            "{:<40} {:>12} / {:<12} {}",
            m.metric,
            m.observed
                .map_or_else(|| "unmeasured".to_string(), |v| v.to_string()),
            m.baseline,
            m.detail
        );
    }
    for h in &report.holes {
        println!("HOLE: {h}");
    }
    for r in &report.unjustified_raises {
        println!("RAISE: {r}");
    }
    if report.outcome == Outcome::Ok {
        println!("{RATCHET_FILE}: all {} baseline(s) held", report.metrics.len());
        return Ok(());
    }
    anyhow::bail!("{RATCHET_FILE}: the ratchet is red")
}
