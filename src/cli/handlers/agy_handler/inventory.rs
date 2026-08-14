//! What `pmat agy sync` can establish today: an inventory of the PMAT work
//! contracts it is asked to transpile.
//!
//! MACS-017 asks for a transpiler from these contracts into Google
//! Anti-Gravity's customization files. Half of that is knowable and half is
//! not: the SOURCE is a concrete, on-disk schema (`WorkContract`, written by
//! `pmat work start`), while the TARGET has no schema anywhere in this
//! repository — see the refusal in `agy_handler.rs`. So this module does the
//! half that is real, and the command still refuses the half that is not.
//!
//! ## Why a projection instead of `WorkContract`
//!
//! Two reasons, both measured on this repo's own `.pmat-work`:
//!
//! 1. **Size.** The 125 contracts total 876 MB, dominated by
//!    `baseline_file_manifest` (up to 254,649 entries in one file). The
//!    inventory needs none of it, and `IgnoredAny` skips it without building
//!    the map.
//! 2. **Reach.** Contracts on disk predate v5.0: some carry no `version`, one
//!    (`PMAT-478`) carries no `baseline_file_manifest` at all and spells its
//!    claims `falsifiable_claims`. Deserializing the strict struct would
//!    refuse contracts that exist, and a reader that cannot see a contract
//!    cannot report it as a transpile input.
//!
//! The projection is therefore deliberately lenient, and every field it fails
//! to find is reported rather than defaulted away.

use anyhow::{Context, Result};
use serde::de::IgnoredAny;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Header projection of `<work-dir>/<id>/contract.json`.
#[derive(Deserialize)]
struct ContractHeader {
    work_item_id: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    verification_level: Option<String>,
    #[serde(default)]
    claims: Vec<IgnoredAny>,
    /// The pre-v4 spelling of `claims`, still on disk (`PMAT-478`).
    #[serde(default)]
    falsifiable_claims: Vec<IgnoredAny>,
    #[serde(default)]
    require: Vec<Clause>,
    #[serde(default)]
    ensure: Vec<Clause>,
    #[serde(default)]
    invariant: Vec<Clause>,
    #[serde(default)]
    implements: Vec<Binding>,
    #[serde(default)]
    agent: Option<Agent>,
    // There is deliberately no `title`/`description` here: the schema has
    // none. That absence is the finding reported by `missing_description`.
}

#[derive(Deserialize)]
struct Clause {
    #[serde(default)]
    description: Option<String>,
}

#[derive(Deserialize)]
struct Binding {
    #[serde(default)]
    contract: Option<String>,
    #[serde(default)]
    equation: Option<String>,
}

#[derive(Deserialize)]
struct Agent {
    #[serde(default)]
    harness: Option<String>,
}

/// One contract, as far as a transpiler would need to see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractSummary {
    pub dir: String,
    pub work_item_id: String,
    /// `"5.0"`, or `"(none)"` for the v4 contracts that omit the field.
    pub version: String,
    pub verification_level: String,
    pub require: usize,
    pub ensure: usize,
    pub invariant: usize,
    pub claims: usize,
    /// Claims found under the legacy `falsifiable_claims` key.
    pub legacy_claims_key: bool,
    pub harness: Option<String>,
    pub bindings: Vec<String>,
    /// Clause descriptions: the only rule text a skill file could carry.
    pub rules: Vec<String>,
    pub bytes: u64,
}

impl ContractSummary {
    /// Rule text exists — the body of a skill file is sourceable.
    pub fn has_rules(&self) -> bool {
        !self.rules.is_empty()
    }
}

/// What `--work-dir` actually contains.
#[derive(Debug, Clone, Default)]
pub struct ContractInventory {
    pub root: PathBuf,
    pub contracts: Vec<ContractSummary>,
    /// `(path, why)` for a `contract.json` that exists but could not be read.
    pub unreadable: Vec<(String, String)>,
    /// Directories under `--work-dir` carrying no `contract.json`.
    pub dirs_without_contract: Vec<String>,
    pub bytes_read: u64,
}

impl ContractInventory {
    /// Read every `<work-dir>/<id>/contract.json`. Never partial: a contract
    /// that cannot be parsed is recorded in `unreadable`, not dropped and not
    /// fatal, because "0 contracts" and "125 unreadable contracts" must not
    /// render the same.
    pub fn scan(root: &Path) -> Result<Self> {
        let mut inv = Self {
            root: root.to_path_buf(),
            ..Default::default()
        };

        let mut entries: Vec<PathBuf> = std::fs::read_dir(root)
            .with_context(|| format!("cannot read contract directory {}", root.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();
        entries.sort();

        for dir in entries {
            let name = file_name_of(&dir);
            let contract = dir.join("contract.json");
            if !contract.exists() {
                inv.dirs_without_contract.push(name);
                continue;
            }
            let bytes = std::fs::metadata(&contract).map(|m| m.len()).unwrap_or(0);
            inv.bytes_read += bytes;
            match read_header(&contract) {
                Ok(header) => inv.contracts.push(summarize(name, header, bytes)),
                Err(why) => inv
                    .unreadable
                    .push((contract.display().to_string(), why.to_string())),
            }
        }
        Ok(inv)
    }

    /// Contracts whose rules could populate a skill body.
    pub fn with_rules(&self) -> usize {
        self.contracts.iter().filter(|c| c.has_rules()).count()
    }

    /// Every distinct clause description across the corpus: the candidate
    /// rule set a transpiler would have to emit, once the target shape for a
    /// rule is known.
    pub fn distinct_rules(&self) -> BTreeSet<&str> {
        self.contracts
            .iter()
            .flat_map(|c| c.rules.iter().map(String::as_str))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty() && self.unreadable.is_empty()
    }
}

fn file_name_of(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.display().to_string())
}

fn read_header(path: &Path) -> Result<ContractHeader> {
    let file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = std::io::BufReader::with_capacity(1 << 20, file);
    let header: ContractHeader =
        serde_json::from_reader(reader).with_context(|| format!("parse {}", path.display()))?;
    Ok(header)
}

fn summarize(dir: String, h: ContractHeader, bytes: u64) -> ContractSummary {
    let mut rules: Vec<String> = Vec::new();
    for clause in h.require.iter().chain(&h.ensure).chain(&h.invariant) {
        if let Some(d) = clause.description.as_ref() {
            if !d.trim().is_empty() {
                rules.push(d.clone());
            }
        }
    }
    let bindings = h
        .implements
        .iter()
        .map(|b| {
            format!(
                "{}::{}",
                b.contract.as_deref().unwrap_or("?"),
                b.equation.as_deref().unwrap_or("?")
            )
        })
        .collect();

    ContractSummary {
        dir,
        work_item_id: h.work_item_id,
        version: h.version.unwrap_or_else(|| "(none)".to_string()),
        verification_level: h.verification_level.unwrap_or_else(|| "-".to_string()),
        require: h.require.len(),
        ensure: h.ensure.len(),
        invariant: h.invariant.len(),
        claims: h.claims.len() + h.falsifiable_claims.len(),
        legacy_claims_key: h.claims.is_empty() && !h.falsifiable_claims.is_empty(),
        harness: h.agent.and_then(|a| a.harness),
        bindings,
        rules,
        bytes,
    }
}

/// Render the inventory. The caller prints this and then refuses: the report
/// is what `agy sync` measured, not a claim that anything was transpiled.
pub(super) fn render(inv: &ContractInventory) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "PMAT work-contract inventory (agy sync, MACS-017)");
    let _ = writeln!(out, "  source: {}", inv.root.display());
    let _ = writeln!(
        out,
        "  contracts read: {} ({:.1} MB of contract JSON)",
        inv.contracts.len(),
        inv.bytes_read as f64 / 1_000_000.0
    );
    if !inv.dirs_without_contract.is_empty() {
        let _ = writeln!(
            out,
            "  directories with no contract.json: {} ({})",
            inv.dirs_without_contract.len(),
            preview(&inv.dirs_without_contract, 5)
        );
    }
    render_table(&mut out, inv);
    render_unreadable(&mut out, inv);
    render_readiness(&mut out, inv);
    out
}

fn render_table(out: &mut String, inv: &ContractInventory) {
    if inv.contracts.is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {:<28} {:>5} {:>5} {:>14} {:>7}  harness",
        "work item", "ver", "level", "req/ens/inv", "claims"
    );
    for c in &inv.contracts {
        let _ = writeln!(
            out,
            "  {:<28} {:>5} {:>5} {:>14} {:>7}  {}",
            truncate(&c.work_item_id, 28),
            c.version,
            c.verification_level,
            format!("{}/{}/{}", c.require, c.ensure, c.invariant),
            c.claims,
            c.harness.as_deref().unwrap_or("-")
        );
    }
}

fn render_unreadable(out: &mut String, inv: &ContractInventory) {
    if inv.unreadable.is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  unreadable contracts: {} (these are inputs a transpiler would silently skip)",
        inv.unreadable.len()
    );
    for (path, why) in &inv.unreadable {
        let _ = writeln!(out, "    {path}: {why}");
    }
}

fn render_readiness(out: &mut String, inv: &ContractInventory) {
    let total = inv.contracts.len();
    if total == 0 {
        // A readiness table of 0/0 rows reads as "everything checked out".
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "  no contract.json found under {}: there is nothing to transpile even \
             once the target format is defined",
            inv.root.display()
        );
        return;
    }
    let with_rules = inv.with_rules();
    let distinct = inv.distinct_rules();
    let legacy = inv.contracts.iter().filter(|c| c.legacy_claims_key).count();
    let bound: usize = inv
        .contracts
        .iter()
        .filter(|c| !c.bindings.is_empty())
        .count();

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  transpile readiness (against the fields ANY skill/rules format needs):"
    );
    let _ = writeln!(
        out,
        "    name:        {total}/{total} have a stable id (work_item_id)"
    );
    let _ = writeln!(
        out,
        "    description: 0/{total} — contract.json has no title or description field,"
    );
    let _ = writeln!(
        out,
        "                 so no skill `description:` can be sourced from a contract alone"
    );
    let _ = writeln!(
        out,
        "    rules:       {with_rules}/{total} carry clause text ({} distinct rules corpus-wide)",
        distinct.len()
    );
    if legacy > 0 {
        let _ = writeln!(
            out,
            "    {legacy} contract(s) store claims under the legacy `falsifiable_claims` key"
        );
    }
    let _ = writeln!(
        out,
        "    {bound}/{total} declare provable-contract bindings (implements[])"
    );
    render_candidate_rules(out, &distinct);
}

fn render_candidate_rules(out: &mut String, distinct: &BTreeSet<&str>) {
    if distinct.is_empty() {
        return;
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "  candidate rules (distinct clause text):");
    for (i, rule) in distinct.iter().enumerate() {
        if i == 40 {
            let _ = writeln!(out, "    ... and {} more", distinct.len() - 40);
            break;
        }
        let _ = writeln!(out, "    - {rule}");
    }
}

fn preview(items: &[String], n: usize) -> String {
    let shown: Vec<&str> = items.iter().take(n).map(String::as_str).collect();
    if items.len() > n {
        format!("{}, ...", shown.join(", "))
    } else {
        shown.join(", ")
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n - 1).chain(std::iter::once('…')).collect()
    }
}

#[cfg(test)]
#[path = "inventory_tests.rs"]
mod inventory_tests;
