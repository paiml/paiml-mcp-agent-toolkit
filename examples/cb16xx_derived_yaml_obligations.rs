//! CB-16xx Derived YAML Obligations Demo — CB-1645.
//!
//! Component 31 of the CB-16xx spec (Chain-of-Thought proof + evidence)
//! requires that every ticket's `.pmat-work/<ID>/contract.json` have a
//! matching *derived* YAML at `contracts/work/<sanitized-id>.yaml` whose
//! preconditions/postconditions mirror the contract's `require`/`ensure`
//! clauses.
//!
//! The derived YAML is the machine-readable obligation surface that other
//! tools (Kani harness scaffolders, mutation generators) consume. If it
//! drifts from the contract.json — because someone edited the contract
//! without running `pmat comply refresh-bindings` — downstream artifacts
//! silently bind to yesterday's obligations. CB-1645 catches exactly that.
//!
//! This example walks through five phases on one ticket `DERIVE-001`:
//!
//!   Phase 1 — No `.pmat-work/` tickets at all: CB-1645 Skip.
//!   Phase 2 — Ticket exists but contract.json has no `require`/`ensure`
//!             (nothing to derive): CB-1645 Skip.
//!   Phase 3 — contract.json has clauses but `contracts/work/` is empty:
//!             CB-1645 Fail ("missing derived").
//!   Phase 4 — Derived YAML exists and mentions every clause: CB-1645 Pass.
//!   Phase 5 — Contract adds a new postcondition without rederiving the
//!             YAML: CB-1645 Fail ("stale").
//!
//! Run with: `cargo run --example cb16xx_derived_yaml_obligations`

use std::fs;
use std::path::Path;
use std::process::Command;

const TICKET: &str = "DERIVE-001";

fn main() {
    println!("=== PMAT Comply — CB-1645 Derived YAML Obligations Demo ===\n");

    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    println!("Synthesizing project tree at {}\n", root.display());

    write_min_project_files(root);

    // ── Phase 1: no tickets — Skip ───────────────────────────────────────────
    println!("── Phase 1: no tickets (expect Skip) ──");
    run_and_filter(root, 1645);

    // ── Phase 2: ticket exists but no derivable clauses — Skip ───────────────
    println!("\n── Phase 2: contract.json without require/ensure (Skip) ──");
    write_contract(root, TICKET, serde_json::json!({ "work_item_id": TICKET }));
    run_and_filter(root, 1645);

    // ── Phase 3: contract has clauses but no derived YAML — Fail ─────────────
    println!("\n── Phase 3: add require/ensure but no derived YAML (Fail) ──");
    write_contract(
        root,
        TICKET,
        serde_json::json!({
            "work_item_id": TICKET,
            "require": ["input is a non-empty slice of u32"],
            "ensure":  ["result is sorted ascending", "result preserves length"],
        }),
    );
    run_and_filter(root, 1645);

    // ── Phase 4: write derived YAML that mentions every clause — Pass ────────
    println!("\n── Phase 4: write matching contracts/work/DERIVE-001.yaml (Pass) ──");
    write_derived_yaml(
        root,
        TICKET,
        "name: \"DERIVE-001\"\n\
         preconditions:\n\
         \x20 - \"input is a non-empty slice of u32\"\n\
         postconditions:\n\
         \x20 - \"result is sorted ascending\"\n\
         \x20 - \"result preserves length\"\n",
    );
    run_and_filter(root, 1645);

    // ── Phase 5: add a new postcondition to contract.json, YAML stale — Fail ─
    println!("\n── Phase 5: contract gains a new postcondition, YAML goes stale (Fail) ──");
    write_contract(
        root,
        TICKET,
        serde_json::json!({
            "work_item_id": TICKET,
            "require": ["input is a non-empty slice of u32"],
            "ensure":  [
                "result is sorted ascending",
                "result preserves length",
                "result is a permutation of input",
            ],
        }),
    );
    run_and_filter(root, 1645);

    println!("\n=== Derivation Semantics ===");
    print_notes();
}

fn write_min_project_files(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"cb16xx-derive\"\nversion = \"0.0.1\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(
        src.join("lib.rs"),
        "pub fn sort(v: &mut [u32]) { v.sort(); }\n",
    )
    .expect("write lib.rs");
    // contracts/work/ is the directory CB-1645 scans — we pre-create it so
    // Phase 3's Fail message says "missing derived <ID>.yaml" rather than
    // "missing contracts/work/ directory" (both Fail, but the message
    // differs).
    fs::create_dir_all(root.join("contracts").join("work")).expect("mkdir contracts/work");
}

/// Write `.pmat-work/<ticket>/contract.json` with the given JSON value. The
/// check only reads `require` (array of strings) and `ensure` (array of
/// strings); the other fields are tolerated but ignored.
fn write_contract(root: &Path, ticket: &str, contract: serde_json::Value) {
    let dir = root.join(".pmat-work").join(ticket);
    fs::create_dir_all(&dir).expect("mkdir ticket");
    fs::write(
        dir.join("contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .expect("write contract.json");
}

/// Write `contracts/work/<sanitized_id>.yaml`. The sanitizer replaces any
/// non-alphanumeric/underscore/dash characters with '_', matching the
/// generator in `check_commit_enforcement_p8::generate_work_contract_yamls`
/// and therefore CB-1645's lookup. `DERIVE-001` stays literal because '-'
/// is preserved.
fn write_derived_yaml(root: &Path, ticket: &str, body: &str) {
    let safe = sanitize_work_id(ticket);
    let path = root
        .join("contracts")
        .join("work")
        .join(format!("{}.yaml", safe));
    fs::write(path, body).expect("write derived yaml");
}

/// Same sanitization rule CB-1645 uses: alnum, `-`, `_` survive; everything
/// else becomes `_`. Kept here so the example's filenames match the check's
/// lookup without depending on pmat internals.
fn sanitize_work_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Run `pmat comply check` on `root` and echo lines mentioning `CB-<cb_id>:`.
/// Missing `pmat` binary is non-fatal — the walkthrough text is the primary
/// payload, the check output is validation.
fn run_and_filter(root: &Path, cb_id: u32) {
    let binary = option_env!("CARGO_BIN_EXE_pmat").unwrap_or("pmat");
    let out = Command::new(binary)
        .args(["comply", "check", "-p"])
        .arg(root)
        .args(["-f", "text"])
        .output();

    let output = match out {
        Ok(o) => o,
        Err(e) => {
            println!("  (pmat not found on PATH: {e}; skipping run)");
            return;
        }
    };
    println!("  exit status: {}", output.status);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let needle = format!("CB-{}:", cb_id);
    let rows: Vec<&str> = stdout
        .lines()
        .chain(stderr.lines())
        .filter(|l| l.contains(&needle))
        .collect();
    if rows.is_empty() {
        println!(
            "  (no CB-{} row — likely a pmat build predating PR #302)",
            cb_id
        );
    } else {
        for line in rows {
            println!("    {}", line.trim_start());
        }
    }
}

fn print_notes() {
    println!(
        "\
CB-1645 treats `contracts/work/<ID>.yaml` as a materialized view over
`.pmat-work/<ID>/contract.json`. The two must stay coherent:

  contract.json `require` → YAML `preconditions:`
  contract.json `ensure`  → YAML `postconditions:`

Neither file is the ground truth alone — the ticket's contract.json is what
reviewers edit, but downstream tools (Kani harness scaffolder, mutation
roster generator) read the YAML. Drift between them is silent scope
reduction of the obligation surface.

Fix pattern when CB-1645 fails:
  pmat comply refresh-bindings --ticket DERIVE-001
which regenerates the YAML from the current contract.json. For bulk repair,
drop `--ticket` and let it rebuild every ticket in `.pmat-work/`.

Related checks in the Component 31 family:

  CB-1640  ChainOfThought assumption references resolve to a prior step
  CB-1641  ChainOfThought evidence references exist on disk
  CB-1643  L3 ProvableContract expressions have structured `expr:` field
  CB-1647  Orphan derived YAMLs (YAML without matching contract.json)

Together these form the chain-of-thought coherence ring: every claim links
back to ground truth and every derived artifact stays synchronised.
"
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn main_runs() {
        super::main();
    }
}
