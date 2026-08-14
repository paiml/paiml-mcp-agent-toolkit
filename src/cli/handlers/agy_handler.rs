use crate::cli::commands::AgyCommands;
use std::path::Path;

mod inventory;

pub use inventory::{ContractInventory, ContractSummary};

pub async fn handle_agy_command(cmd: &AgyCommands, _base_path: &Path) -> anyhow::Result<()> {
    match cmd {
        AgyCommands::Sync { work_dir, out_dir } => sync_contracts_to_agy(work_dir, out_dir),
    }
}

/// `agy sync` used to be a single `println!` of a ✅ success banner followed by
/// `// TODO(MACS-017)` and `Ok(())`: it never read `--work-dir`, never created
/// `--out-dir`, and reported success even for `--work-dir /does/not/exist`.
///
/// It now does the half of MACS-017 that is knowable, and refuses the half that
/// is not:
///
/// * **Knowable — the source.** `<work-dir>/<id>/contract.json` is a concrete
///   on-disk schema. `agy sync` reads every one of them and reports what a
///   transpiler would be handed: ids, versions, verification levels, the Meyer
///   triad, the distinct rule text, the contracts it could NOT parse, and the
///   one field no contract carries (a human-readable description).
/// * **Not knowable — the target.** Nothing in this repository defines a Google
///   Anti-Gravity skill/rules file. The single sentence that names the format
///   is the MACS-017 acceptance criterion in `docs/roadmaps/roadmap.yaml`
///   ("AGENTS.md rules and skills.json formats"), which agrees with neither of
///   the two directory conventions this command's own flags carry. Emitting a
///   guessed schema would be the ✅-banner defect wearing a JSON hat, so the
///   command still exits non-zero and writes nothing.
fn sync_contracts_to_agy(work_dir: &Path, out_dir: &Path) -> anyhow::Result<()> {
    if !work_dir.exists() {
        anyhow::bail!(
            "agy sync: contract directory not found: {}",
            work_dir.display()
        );
    }

    let inv = inventory::ContractInventory::scan(work_dir)?;
    println!("{}", inventory::render(&inv));

    anyhow::bail!("{}", refusal(&inv, out_dir));
}

/// The refusal names the three facts that are missing, so the reader knows what
/// would unblock it. A refusal that only says "unimplemented" is untriageable.
fn refusal(inv: &ContractInventory, out_dir: &Path) -> String {
    let read = match inv.contracts.len() {
        0 => "no work contracts were found to transpile".to_string(),
        n => format!("the {n} contract(s) above were read"),
    };
    format!(
        "agy sync is not implemented: {}, but no PMAT -> Google \
         Anti-Gravity transpiler exists, so nothing was written to {} and the contracts under {} \
         were left untouched.\n\
         \n\
         The blocker is the TARGET format, not the source. Three facts are undefined anywhere in \
         this repository:\n\
         \x20 1. the schema of an Anti-Gravity skill file — `skills.json` is named once, by the \
         MACS-017 acceptance criterion in docs/roadmaps/roadmap.yaml, with no field list, no \
         required fields and no version;\n\
         \x20 2. which directory is authoritative — three conventions have been named for this \
         one command and no spec picks one: repo-root AGENTS.md + skills.json (that same \
         criterion), .agents/skills (this command's --out-dir default), and .gemini/config/skills \
         (carried in the flag's help text through 3.30.0);\n\
         \x20 3. the unit of translation — one skill per work contract, or one rules file \
         aggregating the {} distinct clause rules above.\n\
         \n\
         A fourth gap is in the source and is visible in the report: contract.json carries no \
         title or description, so the `description` every skill format requires cannot be \
         sourced from a contract alone.\n\
         \n\
         Track this under MACS-017: \
         https://github.com/paiml/paiml-mcp-agent-toolkit/issues/984",
        read,
        out_dir.display(),
        inv.root.display(),
        inv.distinct_rules().len(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::AgyCommands;
    use std::path::PathBuf;

    fn contract_body(id: &str) -> String {
        format!(
            r#"{{"version":"5.0","work_item_id":"{id}","verification_level":"L3",
                "claims":[],
                "require":[{{"id":"require.compiles","description":"Project builds successfully"}}],
                "ensure":[],"invariant":[]}}"#
        )
    }

    fn work_dir_with(id: &str) -> tempfile::TempDir {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let d = temp.path().join(".pmat-work").join(id);
        std::fs::create_dir_all(&d).expect("fixture dir");
        std::fs::write(d.join("contract.json"), contract_body(id)).expect("fixture contract");
        temp
    }

    #[tokio::test]
    async fn test_sync_errors_when_work_dir_is_missing() {
        let cmd = AgyCommands::Sync {
            work_dir: PathBuf::from("/does/not/exist/pmat-work"),
            out_dir: PathBuf::from("/does/not/exist/outdir"),
        };

        let err = handle_agy_command(&cmd, Path::new(""))
            .await
            .expect_err("a nonexistent --work-dir must be an error, not a success banner");
        assert!(
            err.to_string().contains("not found"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_sync_does_not_report_success_without_writing_anything() {
        let temp = tempfile::TempDir::new().unwrap();
        let work_dir = temp.path().join(".pmat-work");
        std::fs::create_dir_all(&work_dir).unwrap();
        std::fs::write(
            work_dir.join("CB-001.yaml"),
            "id: CB-001\ntitle: Example contract\n",
        )
        .unwrap();
        let out_dir = temp.path().join("outdir");

        let cmd = AgyCommands::Sync {
            work_dir: work_dir.clone(),
            out_dir: out_dir.clone(),
        };

        let err = handle_agy_command(&cmd, Path::new(""))
            .await
            .expect_err("an unimplemented transpile must not return Ok");
        assert!(
            err.to_string().contains("not implemented"),
            "unexpected error: {err}"
        );
        assert!(
            !out_dir.exists(),
            "nothing was transpiled, so --out-dir must not be claimed as written"
        );
    }

    /// The refusal must be triageable: it has to name what is missing, and the
    /// count of contracts it actually read. The 3.30.0 refusal named neither.
    #[tokio::test]
    async fn test_refusal_names_the_contracts_read_and_the_missing_target_schema() {
        let temp = work_dir_with("MACS-004");
        let work_dir = temp.path().join(".pmat-work");
        let out_dir = temp.path().join("outdir");

        let cmd = AgyCommands::Sync {
            work_dir: work_dir.clone(),
            out_dir: out_dir.clone(),
        };
        let err = handle_agy_command(&cmd, Path::new(""))
            .await
            .expect_err("still unimplemented")
            .to_string();

        assert!(
            err.contains("1 contract(s) above were read"),
            "the refusal must say what it read: {err}"
        );
        assert!(
            err.contains("skills.json") && err.contains("no field list"),
            "the refusal must name the undefined target schema: {err}"
        );
        assert!(
            err.contains(".agents/skills") && err.contains(".gemini/config/skills"),
            "the refusal must name the conflicting directory conventions: {err}"
        );
        assert!(
            err.contains("issues/984"),
            "the refusal must point at the ticket that would unblock it: {err}"
        );
        assert!(
            !out_dir.exists(),
            "a refusal must not leave a partial output tree behind"
        );
    }

    /// The count in the refusal is measured, not decorative.
    #[test]
    fn test_refusal_count_tracks_the_corpus() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let root = temp.path();
        let empty = ContractInventory::scan(root).expect("scan");
        let empty_msg = refusal(&empty, Path::new("out"));

        for id in ["A-1", "A-2"] {
            let d = root.join(id);
            std::fs::create_dir_all(&d).expect("dir");
            std::fs::write(d.join("contract.json"), contract_body(id)).expect("write");
        }
        let full = ContractInventory::scan(root).expect("scan");
        let full_msg = refusal(&full, Path::new("out"));

        assert!(
            empty_msg.contains("no work contracts were found"),
            "an empty corpus must not be reported as \"contracts above were read\": {empty_msg}"
        );
        assert!(full_msg.contains("2 contract(s)"), "{full_msg}");
        assert!(
            full_msg.contains("1 distinct clause rules"),
            "the rule count is read from the contracts: {full_msg}"
        );
    }
}
