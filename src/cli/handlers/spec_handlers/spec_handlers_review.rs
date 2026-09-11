// Included from mod.rs — do NOT add `use` imports or `#!` attributes here.
//
// PMAT-1299 (goal-mode.md §6.3): `pmat spec review --record <json>` validates
// a spec review artifact and stages it. It does not produce one.

/// Handle `pmat spec review --record <json>`: judge the review against the
/// spec it names exactly as CB-2111 will, then write it to
/// `docs/audits/spec-<slug>-review.json` and stage it. A refusal writes
/// nothing, prints every finding, and exits non-zero.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_spec_review(record: &Path, project: &Path) -> anyhow::Result<()> {
    use crate::services::spec_review::{record as record_review, RecordRefusal};
    match record_review(project, record) {
        Ok(recorded) => {
            println!(
                "recorded {}: the review of {} at sha256 {} is staged — commit it with the change it reviews; editing the spec stales it (CB-2111)",
                recorded.artifact, recorded.spec, recorded.spec_sha256
            );
            Ok(())
        }
        Err(refusal) => {
            if let RecordRefusal::Findings(findings) = &refusal {
                for finding in findings {
                    eprintln!("{}", finding.render());
                }
            }
            anyhow::bail!("{}", refusal.render())
        }
    }
}
