use clap::Args;
use std::path::PathBuf;

/// FLOW-011 T1 signer (PMAT-1529). Flags marked PROPOSAL come from
/// T1-signer-shape.md, not from the spec text.
#[derive(Debug, Clone, Args, PartialEq)]
pub struct SignArgs {
    /// Repo as owner/name
    #[arg(long)]
    pub repo: String,
    /// PR number (not needed with --rearm)
    #[arg(long)]
    pub pr: Option<u64>,
    /// Re-sign PR N, keeping the old sign.json as sign.json.disarmed-<ts>
    /// (PROPOSAL). Needs an `ALFREDO <UTC> — …` line naming the PR in --rulings.
    #[arg(long, value_name = "PR")]
    pub rearm: Option<u64>,
    /// noah-rulings.md consulted by --rearm
    #[arg(long, default_value = "noah-rulings.md")]
    pub rulings: PathBuf,
    /// Receipt root (PROPOSAL: /mnt/nvme-raid0/cop-inbox/handoff/receipts)
    #[arg(long, default_value = "/mnt/nvme-raid0/cop-inbox/handoff/receipts")]
    pub root: PathBuf,
    /// Git checkout used to list commits and compute patch-ids
    #[arg(long, default_value = ".")]
    pub repo_dir: PathBuf,
    /// Base ref for the ordered commit list
    #[arg(long, default_value = "origin/main")]
    pub base: String,
    /// Head sha to sign (must equal the PR's current head)
    #[arg(long)]
    pub head: String,
    /// Lane identities allowed to write seat receipts (repeatable)
    #[arg(long = "lane")]
    pub lanes: Vec<String>,
    /// Seat domains the PR requires (repeatable)
    #[arg(long = "seat")]
    pub seats: Vec<String>,
    /// Author model id (repeatable); Co-Authored-By trailers are added too
    #[arg(long = "author-model")]
    pub author_models: Vec<String>,
}
