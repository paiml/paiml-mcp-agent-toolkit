//! CB-16xx Contract-First Binding Lifecycle Demo
//!
//! Demonstrates the 50 new `pmat comply check` gates introduced by PR #302
//! (branch `feat/pmat-work-contract-binding`), spanning Components 27-31:
//!
//!   * CB-1600..1609 — Binding Scope (Component 27)
//!   * CB-1610..1619 — Work Ladder L1..L5 (Component 28)
//!   * CB-1620..1629 — Falsification Unification / ProvableContract (Component 29)
//!   * CB-1630..1639 — Codegen compile + attribute / harness refs (Component 30)
//!   * CB-1640..1649 — Chain-of-Thought proof + evidence (Component 31)
//!
//! Run with: `cargo run --example comply_cb16xx`
//!
//! The example:
//!   1. Synthesizes a minimal project tree under tempdir (`contracts/*.yaml` +
//!      `.pmat-work/<TICKET>/contract.json` with `implements:` + a
//!      `ProvableContract{}` claim).
//!   2. Computes a matching sha256 binding hash so CB-1601 sees no drift.
//!   3. Invokes `pmat comply check --failures-only -f text -p <tmpdir>` via
//!      `std::process::Command` and filters the output for CB-16xx lines.
//!   4. Illustrates the tiered-skip semantics: without real Kani reports,
//!      codegen modules, or git-tracked YAML, most L4+ checks report `Skip`
//!      rather than `Fail` — which is the intended migration-friendly gate.
//!
//! The comply check entry (`handle_check`) is `pub(crate)`, so the example
//! exercises it through the `pmat` CLI. If `pmat` is not on PATH the run is
//! short-circuited with a friendly message — the walkthrough text still
//! prints so the example is useful as documentation.

use std::fs;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

fn main() {
    println!("=== PMAT Comply — CB-16xx Contract-First Binding Demo ===\n");
    print_check_map();

    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    println!("Synthesizing project tree at {}\n", root.display());

    let yaml_sha = write_contract_yaml(root);
    write_work_contract(root, &yaml_sha);
    write_binding_index(root);
    write_min_project_files(root);

    println!("Tree contents:");
    list_tree(root);
    println!();

    run_pmat_comply(root);

    println!("\n=== Tiered-Skip Semantics ===");
    print_tiered_skip_notes();
}

/// Render the 50 new CB-16xx gates organized by component (purely
/// informational — the table mirrors the spec in the PR description).
fn print_check_map() {
    let rows = [
        (
            "Component 27 — Binding Scope (L1..L5)",
            "CB-1600 orphan, CB-1601 SHA drift, CB-1603/1604 inherited pre/post, \
          CB-1605 Kani harness SHA, CB-1607 equation id, CB-1608 cross-binding, \
          CB-1609 YAML git-tracked",
        ),
        (
            "Component 28 — Work Ladder",
            "CB-1610..1614 L1 test evidence + binding, CB-1615 L4 Kani SHA drift, \
          CB-1616/1617 L5 Lean proof, CB-1618 level monotonicity",
        ),
        (
            "Component 29 — Falsification Unification",
            "CB-1620 ProvableContract roster seeded, CB-1621 expected-snapshot drift, \
          CB-1623 duplicate (yaml,test_id), CB-1624 manual deletion audit, \
          CB-1627 post-bind YAML drift, CB-1629 L4+ no timeout",
        ),
        (
            "Component 30 — Codegen",
            "CB-1630 codegen CLI succeeds, CB-1631/1632 debug+release compile, \
          CB-1634 expr→binds_to attribute, CB-1639 Kani harness refs, \
          CB-1636 generated modules tracked",
        ),
        (
            "Component 31 — CoT Proof",
            "CB-1640..1647 chain-of-thought evidence, CB-1648/1649 CoT discharge",
        ),
    ];
    for (head, body) in &rows {
        println!("  {}\n    {}\n", head, body);
    }
}

/// Write a single `contracts/rope-kernel-v1.yaml` with one equation, one
/// falsification test, and a Kani harness — and return its sha256 hex digest
/// so we can embed it in the work contract's `implements[].sha` field.
fn write_contract_yaml(root: &Path) -> String {
    let yaml = r#"metadata:
  version: 1.0.0
  created: '2026-04-18'
  author: PAIML Engineering
  description: Demo rope kernel contract for CB-16xx example
equations:
  rope:
    formula: x' = rotate(x, theta)
    domain: x in R^d, theta in R
    codomain: x' in R^d
    invariants:
      - "|x'| == |x|"
    preconditions:
      - x.len() > 0
      - theta.is_finite()
    postconditions:
      - x_prime.len() == x.len()
    lean_theorem:
      name: Rope_Periodic
      status: assumed
    kani_harnesses:
      - verify_rope_periodic
    falsification_tests:
      - id: rope_periodicity_test
        rule: rope is periodic in theta
        prediction: rope(x, theta + 2pi) == rope(x, theta)
        test: proptest_rope_periodicity
        expected: pass
        if_fails: rope drifts across a full period
"#;
    let contracts_dir = root.join("contracts");
    fs::create_dir_all(&contracts_dir).expect("mkdir contracts");
    let yaml_path = contracts_dir.join("rope-kernel-v1.yaml");
    fs::write(&yaml_path, yaml).expect("write yaml");

    let mut hasher = Sha256::new();
    hasher.update(yaml.as_bytes());
    let digest = hasher.finalize();
    digest.iter().fold(String::with_capacity(64), |mut acc, b| {
        use std::fmt::Write;
        let _ = write!(acc, "{:02x}", b);
        acc
    })
}

/// Write `.pmat-work/ROPE-001/contract.json` with an `implements:` binding
/// plus a `FalsificationMethod::ProvableContract{}` claim — matching the
/// YAML we just emitted. CB-1620/CB-1621/CB-1627 all read this roster.
fn write_work_contract(root: &Path, yaml_sha: &str) {
    let contract = serde_json::json!({
        "version": "5.0",
        "work_item_id": "ROPE-001",
        "created_at": "2026-04-18T00:00:00Z",
        "baseline_commit": "0000000000000000000000000000000000000000",
        "baseline_tdg": 95.0,
        "baseline_coverage": 95.0,
        "baseline_rust_score": null,
        "baseline_file_manifest": { "files": [] },
        "thresholds": {},
        "verification_level": "L3",
        "iteration": 1,
        "claims": [{
            "hypothesis": "rope kernel remains periodic under bind",
            "falsification_method": {
                "ProvableContract": {
                    "yaml_path": "contracts/rope-kernel-v1.yaml",
                    "equation": "rope",
                    "test_id": "rope_periodicity_test",
                    "expected": "\"pass\""
                }
            },
            "evidence_required": "TestRun",
            "result": null,
            "override_info": null
        }],
        "implements": [{
            "contract": "rope-kernel-v1",
            "equation": "rope",
            "file": "contracts/rope-kernel-v1.yaml",
            "sha": yaml_sha,
            "bound_at": "2026-04-18T00:00:00Z"
        }]
    });
    let dir = root.join(".pmat-work/ROPE-001");
    fs::create_dir_all(&dir).expect("mkdir ticket");
    fs::write(
        dir.join("contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .expect("write contract");
}

/// Write `.pmat/binding-index.json` so CB-1600 (orphan detector) has something
/// to intersect against. Maps the synthesized function path to the binding.
fn write_binding_index(root: &Path) {
    let idx = serde_json::json!({
        "src/rope.rs": ["rope-kernel-v1/rope"],
    });
    let dir = root.join(".pmat");
    fs::create_dir_all(&dir).expect("mkdir .pmat");
    fs::write(
        dir.join("binding-index.json"),
        serde_json::to_vec_pretty(&idx).unwrap(),
    )
    .expect("write binding-index");
}

/// A Cargo manifest and a stub source file — just enough for `pmat comply`
/// to recognize the tempdir as a Rust project.
fn write_min_project_files(root: &Path) {
    let cargo_toml = r#"[package]
name = "cb16xx-demo"
version = "0.0.1"
edition = "2021"
"#;
    fs::write(root.join("Cargo.toml"), cargo_toml).expect("write Cargo.toml");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(
        src.join("rope.rs"),
        "pub fn rope(_x: &[f64], _theta: f64) {}\n",
    )
    .expect("write src/rope.rs");
    fs::write(src.join("lib.rs"), "pub mod rope;\n").expect("write lib.rs");
}

/// Shallow listing so a reader can see exactly what got synthesized.
fn list_tree(root: &Path) {
    fn walk(p: &Path, depth: usize) {
        let Ok(entries) = fs::read_dir(p) else { return };
        let mut names: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        names.sort();
        for path in names {
            let rel = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            println!("  {}{}", "  ".repeat(depth), rel);
            if path.is_dir() {
                walk(&path, depth + 1);
            }
        }
    }
    walk(root, 0);
}

/// Invoke `pmat comply check` on the synthesized project and echo any CB-16xx
/// output lines. If `pmat` is missing from PATH we print a hint and move on —
/// the walkthrough text above is the primary documentation payload.
fn run_pmat_comply(root: &Path) {
    println!("Invoking: pmat comply check -p <tmpdir> --failures-only -f text");
    let out = Command::new("pmat")
        .args(["comply", "check", "-p"])
        .arg(root)
        .args(["--failures-only", "-f", "text"])
        .output();

    let output = match out {
        Ok(o) => o,
        Err(e) => {
            println!("  (pmat not found on PATH: {e})");
            println!("  Install with `cargo install --path .` from the pmat checkout.");
            return;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("  exit status: {}", output.status);

    let cb16xx: Vec<&str> = stdout
        .lines()
        .chain(stderr.lines())
        .filter(|l| l.contains("CB-16"))
        .collect();
    if cb16xx.is_empty() {
        println!("  (no CB-16xx lines in output — likely running a pmat build that");
        println!("   predates PR #302; rebuild from `feat/pmat-work-contract-binding`)");
    } else {
        println!("  CB-16xx lines:");
        for line in cb16xx {
            println!("    {}", line);
        }
    }
}

/// The whole point of the skip-if-absent pattern: most CB-16xx checks return
/// Skip when their infrastructure (Kani reports, codegen modules, Lean
/// theorems) isn't yet present, so repositories can opt-in incrementally
/// without getting flooded with Fail rows on day one.
fn print_tiered_skip_notes() {
    println!(
        "\
Most CB-16xx gates follow a three-state pattern:

  Skip : infrastructure absent (no kani-report.json, no codegen module, …)
  Pass : infrastructure present and obligations satisfied
  Fail : infrastructure present but obligations violated

In this synthetic tempdir, you should see Skip for:

  CB-1602  (no .pmat-work/ledger/unbinds.json)
  CB-1605  (no .pmat-work/ROPE-001/kani-report.json)
  CB-1606  (no lean_theorem: blocks with status!='proved')
  CB-1615  (no kani_harnesses[] on YAML + no kani report)
  CB-1624  (no ledger/roster-mutations.json)
  CB-1630  (no contract_traits.rs codegen module)
  CB-1640+ (no chain-of-thought steps on contract.json)

Tighten the ratchet by adding the missing files one component at a time —
see docs/specifications/components/pmat-work-contract-binding.md for the
staged migration path.
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
