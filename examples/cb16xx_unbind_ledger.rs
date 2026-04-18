//! CB-16xx Unbind Ledger Demo — CB-1624 catching manual deletions.
//!
//! Component 29 of the CB-16xx spec mandates an audit ledger for every
//! mutation of a ticket's inherited ProvableContract roster. The ledger lives
//! at `.pmat-work/ledger/roster-mutations.json` and every **deletion** entry
//! must carry `via_unbind: true` — the flag that `pmat work unbind` sets when
//! it performs a legitimate removal.
//!
//! A deletion entry **without** `via_unbind: true` means someone edited
//! `contract.json` directly, bypassing the unbind audit trail. CB-1624 catches
//! exactly that class of silent scope erosion.
//!
//! This example writes a ledger with three entries:
//!
//!   1. A legitimate `delete` via `pmat work unbind` (`via_unbind: true`).
//!   2. A legitimate `add` (no via_unbind flag required for adds).
//!   3. A *manual* `delete` with `via_unbind: false` — the violation.
//!
//! Running `pmat comply check` flips CB-1624 from Skip → Fail and points at
//! the offending entry.
//!
//! Run with: `cargo run --example cb16xx_unbind_ledger`

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("=== PMAT Comply — CB-16xx Unbind Ledger Audit (CB-1624) ===\n");

    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    println!("Synthesizing project tree at {}\n", root.display());

    write_min_project_files(root);

    // Phase 1: empty ledger — CB-1624 should Pass (nothing to audit).
    println!("── Phase 1: empty ledger (nothing to audit) ──");
    write_ledger(root, &[]);
    run_and_filter(root, 1624);

    // Phase 2: ledger with a clean unbind + a clean add — CB-1624 still Pass.
    println!("\n── Phase 2: clean ledger (delete via_unbind=true + add) ──");
    write_ledger(root, &[clean_delete("UNBIND-001"), clean_add("UNBIND-001")]);
    run_and_filter(root, 1624);

    // Phase 3: inject a manual delete — CB-1624 flips to Fail.
    println!("\n── Phase 3: inject manual delete (via_unbind=false) ──");
    write_ledger(
        root,
        &[
            clean_delete("UNBIND-001"),
            clean_add("UNBIND-001"),
            manual_delete("UNBIND-002"),
        ],
    );
    run_and_filter(root, 1624);

    println!("\n=== Ledger Semantics ===");
    print_ledger_notes();
}

fn write_min_project_files(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"cb16xx-unbind\"\nversion = \"0.0.1\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("lib.rs"), "pub fn stub() {}\n").expect("write lib.rs");
    // `.pmat-work/` must exist for CB-1624's first tier gate to open.
    fs::create_dir_all(root.join(".pmat-work").join("ledger")).expect("mkdir ledger");
}

/// Write `.pmat-work/ledger/roster-mutations.json` as a JSON array. The check
/// requires an array at the top level.
fn write_ledger(root: &Path, entries: &[serde_json::Value]) {
    let path = root
        .join(".pmat-work")
        .join("ledger")
        .join("roster-mutations.json");
    let body = serde_json::Value::Array(entries.to_vec());
    fs::write(path, serde_json::to_vec_pretty(&body).unwrap()).expect("write roster-mutations");
}

/// A delete entry from `pmat work unbind` — the happy path.
fn clean_delete(ticket: &str) -> serde_json::Value {
    serde_json::json!({
        "ticket": ticket,
        "action": "delete",
        "target": {
            "yaml": "contracts/widget.yaml",
            "equation": "widget",
            "test_id": "widget_smoke_test",
        },
        "via_unbind": true,
        "timestamp": "2026-04-18T09:00:00Z",
        "reason": "pmat work unbind widget/widget_smoke_test",
    })
}

/// An add entry — `via_unbind` is irrelevant for adds.
fn clean_add(ticket: &str) -> serde_json::Value {
    serde_json::json!({
        "ticket": ticket,
        "action": "add",
        "target": {
            "yaml": "contracts/widget.yaml",
            "equation": "widget",
            "test_id": "widget_retry_test",
        },
        "timestamp": "2026-04-18T09:30:00Z",
        "reason": "pmat work bind widget/widget_retry_test",
    })
}

/// The violation: a delete entry with `via_unbind: false`. A manual edit that
/// bypasses the unbind audit trail — exactly what CB-1624 catches.
fn manual_delete(ticket: &str) -> serde_json::Value {
    serde_json::json!({
        "ticket": ticket,
        "action": "delete",
        "target": {
            "yaml": "contracts/widget.yaml",
            "equation": "widget",
            "test_id": "widget_retry_test",
        },
        "via_unbind": false,
        "timestamp": "2026-04-18T12:00:00Z",
        "reason": "manual contract.json edit (audit violation)",
    })
}

/// Run `pmat comply check` and echo lines mentioning the given CB id. Keeps
/// the example useful as documentation when `pmat` is missing from PATH.
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

fn print_ledger_notes() {
    println!(
        "\
CB-1624's contract is narrow but load-bearing: every deletion in the roster-
mutations ledger must have been produced by `pmat work unbind`, which always
sets `via_unbind: true`. Any other path into a `delete`/`Delete`/`deletion`
entry (matched case-insensitively on the `delet` prefix) is a red flag:

  * direct JSON editor on `.pmat-work/<ID>/contract.json`
  * a bespoke cleanup script that forgot to stamp the audit flag
  * a migration tool that rewrote the roster without replaying through unbind

Fix pattern: replay the deletion through `pmat work unbind --equation
<contract>/<equation> --test-id <id>`, which rewrites the offending entry with
`via_unbind: true` and a reason string.

Related checks sharing the ledger surface:

  CB-1617  downgrade ledger (.pmat-work/ledger/downgrades.json) audits
           L-level regressions — missing entries fail CB-1618 monotonicity
  CB-1602  unbind ledger at .pmat-work/ledger/unbinds.json tracks every
           `pmat work unbind` invocation for chain-of-custody
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
