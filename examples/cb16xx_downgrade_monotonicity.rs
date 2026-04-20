//! CB-16xx Downgrade + Monotonicity Audit Demo — CB-1617 and CB-1618.
//!
//! Component 28 of the CB-16xx spec introduces a *downgrade audit ledger* at
//! `.pmat-work/ledger/downgrades.json` and requires that every verification-
//! level regression in a ticket's per-checkpoint timeline be matched by an
//! audit entry. Two checks work in tandem to enforce this:
//!
//!   * **CB-1617** — every entry in the downgrade ledger must carry a
//!     non-empty `reason` field. An empty reason is silent scope reduction.
//!   * **CB-1618** — a ticket's `verification_level` across its
//!     `.pmat-work/<ID>/checkpoints/*.json` files must be non-decreasing,
//!     unless the ticket appears in the downgrade ledger. Presence alone
//!     satisfies CB-1618; CB-1617 independently checks the `reason`.
//!
//! This example walks through four phases on a single ticket `AUDIT-001` to
//! show the full Skip → Pass → Fail → Pass lifecycle of both checks.
//!
//!   Phase 1 — No checkpoints, no ledger: both checks Skip.
//!   Phase 2 — Monotonic ascending checkpoints (L1 → L3 → L4) without ledger:
//!             CB-1618 Pass, CB-1617 still Skip (ledger absent).
//!   Phase 3 — Add a regressing checkpoint (L4 → L2) with NO ledger entry:
//!             CB-1618 Fail ("unaudited regression"), CB-1617 still Skip.
//!   Phase 4 — Write a ledger entry that cites the regression AND has a
//!             real reason string: CB-1618 Pass, CB-1617 Pass.
//!   Phase 5 — Mutate that ledger entry to blank the reason: CB-1617 Fail
//!             (CB-1618 still Pass because presence alone is enough for it).
//!
//! Run with: `cargo run --example cb16xx_downgrade_monotonicity`

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("=== PMAT Comply — CB-1617 + CB-1618 Downgrade/Monotonicity Demo ===\n");

    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    println!("Synthesizing project tree at {}\n", root.display());

    write_min_project_files(root);

    // ── Phase 1: no checkpoints, no ledger — both Skip ───────────────────────
    println!("── Phase 1: no checkpoints, no ledger (both Skip) ──");
    run_and_filter(root, &[1617, 1618]);

    // ── Phase 2: monotonic ascending L1 → L3 → L4 — CB-1618 Pass, 1617 Skip ──
    println!("\n── Phase 2: monotonic L1 → L3 → L4 (CB-1618 Pass, CB-1617 Skip) ──");
    write_checkpoint(root, "AUDIT-001", "a.json", "2026-04-18T09:00:00Z", "L1");
    write_checkpoint(root, "AUDIT-001", "b.json", "2026-04-18T10:00:00Z", "L3");
    write_checkpoint(root, "AUDIT-001", "c.json", "2026-04-18T11:00:00Z", "L4");
    run_and_filter(root, &[1617, 1618]);

    // ── Phase 3: regress to L2 without ledger — CB-1618 Fail, 1617 Skip ──────
    println!("\n── Phase 3: regress L4 → L2 without ledger (CB-1618 Fail) ──");
    write_checkpoint(root, "AUDIT-001", "d.json", "2026-04-18T12:00:00Z", "L2");
    run_and_filter(root, &[1617, 1618]);

    // ── Phase 4: ledger entry with real reason — both Pass ───────────────────
    println!("\n── Phase 4: add ledger entry with reason (both Pass) ──");
    write_downgrade_ledger(
        root,
        &[ledger_entry(
            "AUDIT-001",
            "kani runner offline — reviewer agreed to defer L4 to next sprint",
        )],
    );
    run_and_filter(root, &[1617, 1618]);

    // ── Phase 5: blank the reason — CB-1617 Fail ─────────────────────────────
    println!("\n── Phase 5: blank the ledger reason (CB-1617 Fail) ──");
    write_downgrade_ledger(root, &[ledger_entry("AUDIT-001", "")]);
    run_and_filter(root, &[1617, 1618]);

    println!("\n=== Downgrade/Monotonicity Semantics ===");
    print_notes();
}

fn write_min_project_files(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"cb16xx-downgrade\"\nversion = \"0.0.1\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("lib.rs"), "pub fn audit() {}\n").expect("write lib.rs");
    // `.pmat-work/` must exist so CB-1618's first tier gate opens; we leave
    // `ledger/` unmaterialized until Phase 4 so Phase 1-3 exercise the Skip
    // branch in CB-1617.
    fs::create_dir_all(root.join(".pmat-work")).expect("mkdir .pmat-work");
}

/// Append one checkpoint at `.pmat-work/<ticket>/checkpoints/<filename>`.
/// CB-1618 sorts checkpoints by the `timestamp` field, not by filename, so
/// filenames can be anything valid — we use `a.json`..`d.json` for clarity.
fn write_checkpoint(root: &Path, ticket: &str, filename: &str, ts: &str, level: &str) {
    let dir = root.join(".pmat-work").join(ticket).join("checkpoints");
    fs::create_dir_all(&dir).expect("mkdir checkpoints");
    let body = serde_json::json!({
        "timestamp": ts,
        "verification_level": level,
    });
    fs::write(
        dir.join(filename),
        serde_json::to_vec_pretty(&body).unwrap(),
    )
    .expect("write checkpoint");
}

/// Write `.pmat-work/ledger/downgrades.json` from the given entries. The
/// file must be a JSON array at the top level — anything else trips the
/// structural Fail branch of CB-1617.
fn write_downgrade_ledger(root: &Path, entries: &[serde_json::Value]) {
    let dir = root.join(".pmat-work").join("ledger");
    fs::create_dir_all(&dir).expect("mkdir ledger");
    let body = serde_json::Value::Array(entries.to_vec());
    fs::write(
        dir.join("downgrades.json"),
        serde_json::to_vec_pretty(&body).unwrap(),
    )
    .expect("write downgrades.json");
}

/// One downgrade ledger entry. Only `ticket` and `reason` are load-bearing —
/// CB-1617 scans for empty `reason` strings; CB-1618 uses `ticket` membership
/// as its "this regression has been audited" predicate.
fn ledger_entry(ticket: &str, reason: &str) -> serde_json::Value {
    serde_json::json!({
        "ticket": ticket,
        "reason": reason,
        "from_level": "L4",
        "to_level": "L2",
        "timestamp": "2026-04-18T12:30:00Z",
    })
}

/// Run `pmat comply check` on `root` and echo only lines that mention the
/// given CB numeric ids. Missing `pmat` isn't fatal — we still print the
/// notes so the example is useful as documentation even without a build.
fn run_and_filter(root: &Path, cb_ids: &[u32]) {
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

    let targets: Vec<String> = cb_ids.iter().map(|n| format!("CB-{}:", n)).collect();
    let rows: Vec<&str> = stdout
        .lines()
        .chain(stderr.lines())
        .filter(|l| targets.iter().any(|t| l.contains(t)))
        .collect();
    if rows.is_empty() {
        println!("  (no matching CB rows — likely a pmat build predating PR #302)");
    } else {
        for line in rows {
            println!("    {}", line.trim_start());
        }
    }
}

fn print_notes() {
    println!(
        "\
The downgrade ledger is the narrow audit surface for every verification-level
regression. CB-1617 and CB-1618 divide its enforcement:

  CB-1617  ledger entry well-formed       → every `reason` non-empty
  CB-1618  ledger entry present per ticket → regressions are audited

Together they form a ratchet: once a ticket has climbed to L4, any slide back
down to L2 requires a ledger row with a human-readable explanation. Silent
regressions (no entry OR empty reason) fail the gate.

Related surfaces:

  .pmat-work/<ID>/checkpoints/*.json  — the per-ticket timeline CB-1618 reads
  .pmat-work/ledger/downgrades.json   — the audit trail CB-1617 validates
  .pmat-work/ledger/unbinds.json      — CB-1602 chain-of-custody for unbinds
  .pmat-work/ledger/roster-mutations.json — CB-1624 audit for manual deletes

Fix pattern when CB-1618 fires: add a row to downgrades.json with a real
`reason`, then re-run `pmat comply check` — both checks should flip to Pass.
Never blank the reason to appease the gate — CB-1617 will still flag it.
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
