// CB-2100: `pmat comply ledger` — generate (or check) the enforcement ledger.
//
// Included from comply_handlers/mod.rs, which owns the imports.

/// Generate the ledger, or verify the committed one.
///
/// Without `--write` this is a check: it exits non-zero when the committed
/// ledger is missing or has drifted, so CI can run the generator as its own
/// gate rather than trusting a file nobody regenerates.
///
/// Fails closed. An unresolvable required-check list, an empty rule roster or
/// an unwritable destination are errors — never a ledger that quietly claims
/// everything is fine.
fn handle_ledger(project_path: &Path, write: bool, output: Option<&Path>) -> Result<()> {
    use crate::services::gate_effect::{ledger, roster};

    if !roster::defines_rules(project_path) {
        anyhow::bail!(
            "{} does not exist, so this project declares no CB rules and has no enforcement \
             ledger to generate",
            roster::HANDLER_DIR
        );
    }
    let yaml = crate::models::comply_config::PmatYamlConfig::load(project_path).unwrap_or_default();
    let config = yaml.comply;
    let required = crate::services::gate_effect::required::resolve(project_path)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let report =
        crate::services::gate_effect::analyze_with_contexts(project_path, &config, &required);
    let rendered = ledger::render(project_path, &report, &config).map_err(|e| anyhow::anyhow!(e))?;

    let destination = output.map(Path::to_path_buf);
    if write || destination.is_some() {
        let target = destination.unwrap_or_else(|| project_path.join(ledger::LEDGER_PATH));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, &rendered)?;
        println!("Wrote {}", target.display());
        return Ok(());
    }

    match ledger::committed(project_path) {
        Some(found) if !ledger::drifted(&found, &rendered) => {
            println!("{} is up to date", ledger::LEDGER_PATH);
            Ok(())
        }
        Some(_) => anyhow::bail!(
            "{} has drifted from what CB-2100 computes — regenerate with \
             `pmat comply ledger --write`",
            ledger::LEDGER_PATH
        ),
        None => anyhow::bail!(
            "{} is missing — generate it with `pmat comply ledger --write`",
            ledger::LEDGER_PATH
        ),
    }
}
