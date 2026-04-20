//! CB-16xx Completion Level vs. Target Demo — CB-1619.
//!
//! Component 28 of the CB-16xx spec enforces that a ticket's recorded
//! `achieved_level` (what was actually verified) matches the `target_level`
//! declared in its contract. The two live together in
//! `.pmat-work/<ID>/verification-report.json`:
//!
//!   {"target_level": "L3", "achieved_level": "L3"}   ← ok
//!   {"target_level": "L4", "achieved_level": "L2"}   ← silent downgrade
//!
//! CB-1619 fails the second shape because a ticket that aimed for L4 Kani
//! verification but shipped with only L2 DbC evidence has drifted from its
//! stated contract. Closing tickets below target is the most common way
//! obligations get silently dropped — CB-1619 catches it at the gate.
//!
//! This example synthesises three tickets side-by-side and marches the same
//! repository through three phases to show how CB-1619 ratchets from Skip
//! to Pass to Fail:
//!
//!   Phase 1 — Tickets have contracts but no verification-report.json yet:
//!             CB-1619 Skip.
//!   Phase 2 — Each ticket reports achieved == target: CB-1619 Pass.
//!   Phase 3 — Mutate COMPLETE-003's report to claim target=L4, achieved=L2
//!             (a typical "kani slipped, we shipped anyway" signal):
//!             CB-1619 Fail listing COMPLETE-003.
//!
//! Run with: `cargo run --example cb16xx_completion_target`

use std::fs;
use std::path::Path;
use std::process::Command;

const TICKETS: &[(&str, &str)] = &[
    ("COMPLETE-001", "L1"),
    ("COMPLETE-002", "L3"),
    ("COMPLETE-003", "L4"),
];

fn main() {
    println!("=== PMAT Comply — CB-1619 Achieved-Level == Target Demo ===\n");

    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    println!("Synthesizing project tree at {}\n", root.display());

    write_min_project_files(root);
    for (id, lvl) in TICKETS {
        write_work_contract(root, id, lvl);
    }

    // ── Phase 1: contracts exist, no reports — CB-1619 Skip ──────────────────
    println!("── Phase 1: contracts only, no verification-report.json (Skip) ──");
    run_and_filter(root, 1619);

    // ── Phase 2: each ticket achieved its target — CB-1619 Pass ──────────────
    println!("\n── Phase 2: write matching reports (achieved == target) (Pass) ──");
    for (id, lvl) in TICKETS {
        write_verification_report(root, id, lvl, lvl);
    }
    run_and_filter(root, 1619);

    // ── Phase 3: COMPLETE-003 downgrades silently — CB-1619 Fail ─────────────
    println!("\n── Phase 3: mutate COMPLETE-003 to achieved=L2 (Fail) ──");
    write_verification_report(root, "COMPLETE-003", "L4", "L2");
    run_and_filter(root, 1619);

    println!("\n=== Completion-Target Semantics ===");
    print_notes();
}

fn write_min_project_files(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"cb16xx-complete\"\nversion = \"0.0.1\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("lib.rs"), "pub fn finish() -> bool { true }\n").expect("write lib.rs");
}

/// Write `.pmat-work/<ticket>/contract.json` at the given target level. The
/// contract is fully serde-compatible: `thresholds` and `baseline_file_manifest`
/// are populated because serde_json rejects short-form `{}` for either.
fn write_work_contract(root: &Path, ticket: &str, level: &str) {
    let contract = serde_json::json!({
        "version": "5.0",
        "work_item_id": ticket,
        "created_at": "2026-04-18T00:00:00Z",
        "baseline_commit": "0".repeat(40),
        "baseline_tdg": 95.0,
        "baseline_coverage": 95.0,
        "baseline_rust_score": null,
        "baseline_file_manifest": manifest_stub(),
        "thresholds": thresholds_stub(),
        "verification_level": level,
        "iteration": 1,
        "claims": [],
        "implements": [],
    });
    let dir = root.join(".pmat-work").join(ticket);
    fs::create_dir_all(&dir).expect("mkdir ticket");
    fs::write(
        dir.join("contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .expect("write contract.json");
}

/// Write `.pmat-work/<ticket>/verification-report.json`. Only `target_level`
/// and `achieved_level` are load-bearing — CB-1619 ignores other fields. The
/// level strings must parse via `VerificationLevel::parse_strict`, i.e. `L0`
/// through `L5` with no trailing whitespace.
fn write_verification_report(root: &Path, ticket: &str, target: &str, achieved: &str) {
    let report = serde_json::json!({
        "target_level":   target,
        "achieved_level": achieved,
        "completed_at":   "2026-04-18T18:00:00Z",
        "notes": if target == achieved {
            format!("{}: achieved target {}", ticket, target)
        } else {
            format!("{}: slipped {} → {}", ticket, target, achieved)
        },
    });
    let path = root
        .join(".pmat-work")
        .join(ticket)
        .join("verification-report.json");
    fs::write(path, serde_json::to_vec_pretty(&report).unwrap())
        .expect("write verification-report.json");
}

fn thresholds_stub() -> serde_json::Value {
    serde_json::json!({
        "min_coverage_pct": 95.0,
        "min_per_file_coverage_pct": 95.0,
        "max_tdg_regression": 0.0,
        "max_function_complexity": 20,
        "max_file_lines": 500,
        "min_spec_score": 95,
        "require_github_sync": false,
        "require_spec_update": false,
        "require_roadmap_update": false,
        "block_on_new_satd": false,
        "block_on_new_dead_code": false,
        "require_lint_pass": false
    })
}

fn manifest_stub() -> serde_json::Value {
    serde_json::json!({
        "files": {},
        "coverage_required": [],
        "manifest_hash": "stub"
    })
}

/// Run `pmat comply check` on `root` and echo CB-<cb_id> rows only. Missing
/// `pmat` is non-fatal — the notes still print as a documentation fallback.
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
CB-1619 is the closing gate of the work-verification ladder. Target level is
declared at bind time (the contract's `verification_level`); achieved level
is declared at completion time (`verification-report.json:achieved_level`).
The two must match — silent downgrade between bind and close is exactly the
scope erosion that the CB-16xx ratchet is designed to prevent.

Decision table:

  target_level missing or unparseable    → this ticket is skipped silently
  achieved_level missing or unparseable  → this ticket is skipped silently
  achieved == target                     → counted as Pass
  achieved <  target                     → counted as Fail row
  achieved >  target  (over-achievement) → counted as Pass (no ratchet)

When every ticket has been skipped (no readable report yet), CB-1619 itself
reports Skip for the whole project, because there's nothing to judge. This
is the intended migration behaviour for repos mid-adoption.

Related ladder checks (all activate under the L3+ profile):

  CB-1610  ticket's verification_level parses as L0..L5
  CB-1611  ticket's verification_level ≤ binding's max attainable level
  CB-1617  downgrade ledger reasons are non-empty
  CB-1618  per-ticket checkpoint level is monotone (or audited)

Fix pattern when CB-1619 fires: either re-run the L4 verification (so
achieved catches up), or explicitly downgrade the contract's target with a
ledger entry — which lifts the bar CB-1619 is comparing against. Never just
blank the `achieved_level` field to silence the check; CB-1619 reads the
report, not the absence of one.
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
