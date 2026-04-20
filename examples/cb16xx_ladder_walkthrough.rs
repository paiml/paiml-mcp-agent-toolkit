//! CB-16xx Work Ladder Walkthrough — L0..L5 progressive activation demo.
//!
//! The CB-16xx spec (Components 27-31, PR #302) introduces a five-level work
//! ladder: L0 (agent) → L1 (tests) → L2 (DbC) → L3 (falsification) → L4 (Kani)
//! → L5 (Lean). Each check `CB-1610..CB-1619` activates as its upstream writer
//! deposits evidence into the ticket's `.pmat-work/<ID>/` directory:
//!
//!   * L1 → `verification-report.json` with `l1_test_evidence`
//!   * L3 → `falsification.log` with all `status: "pass"` lines
//!   * L4 → `kani-report.json` + `kani-harness-shas.json` snapshot
//!   * L5 → `lean-proof.json` with `sorry_count: 0`
//!
//! This example stands up six fake tickets (one per level) and stages matching
//! evidence for each, then runs `pmat comply check` and prints only the
//! CB-1610..CB-1619 rows so the reader can watch checks flip Skip → Pass as
//! tickets climb the ladder.
//!
//! Run with: `cargo run --example cb16xx_ladder_walkthrough`

use std::fs;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

fn main() {
    println!("=== PMAT Comply — CB-16xx Work Ladder (L0..L5) Demo ===\n");

    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    println!("Synthesizing project tree at {}\n", root.display());

    write_min_project_files(root);
    let yaml_sha = write_contract_yaml(root);

    let tickets = [
        ("LADDER-L0", "L0"),
        ("LADDER-L1", "L1"),
        ("LADDER-L2", "L2"),
        ("LADDER-L3", "L3"),
        ("LADDER-L4", "L4"),
        ("LADDER-L5", "L5"),
    ];
    for (ticket_id, level) in tickets {
        write_work_contract(root, ticket_id, level, &yaml_sha);
        stage_level_evidence(root, ticket_id, level);
    }

    println!("Tickets staged:");
    for (t, l) in &tickets {
        println!("  {t:<12}  target={l}");
    }
    println!();

    run_pmat_comply(root);

    println!("\n=== Ladder Semantics ===");
    print_ladder_notes();
}

/// Write `contracts/ladder-v1.yaml` used by every ticket's binding. The YAML
/// declares one equation, one Kani harness (with a `sha:` sibling so CB-1615
/// has something to diff), and one falsification test.
///
/// CRITICAL: `kani_harnesses`, `falsification_tests`, and `lean_theorem` MUST
/// be top-level — `max_attainable_from_yaml` and CB-1615's YAML scanner both
/// scan for top-level keys (indentation == 0). Nesting them under `equations`
/// silently caps the max-attainable level at L2.
fn write_contract_yaml(root: &Path) -> String {
    let yaml = r#"metadata:
  version: 1.0.0
  author: CB-16xx walkthrough demo
equations:
  climb:
    formula: level_{n+1} >= level_n
    domain: n in Nat
    codomain: Bool
falsification_tests:
  - id: climb_monotone_test
    test: proptest_climb_monotone
    expected: true
kani_harnesses:
  - name: verify_climb_monotone
    sha: b8f4c16a4f8c16a4b8f4c16a4f8c16a4b8f4c16a4f8c16a4b8f4c16a4f8c16a4
lean_theorem:
  name: Climb_Monotone
  status: proved
"#;
    let dir = root.join("contracts");
    fs::create_dir_all(&dir).expect("mkdir contracts");
    let path = dir.join("ladder-v1.yaml");
    fs::write(&path, yaml).expect("write ladder yaml");

    let mut h = Sha256::new();
    h.update(yaml.as_bytes());
    hex(&h.finalize())
}

/// Synthesize a `.pmat-work/<TICKET>/contract.json` for one ticket with a
/// fixed `verification_level` (L0..L5). The binding SHA is shared across
/// tickets because every ticket points at the same `ladder-v1.yaml`.
///
/// The contract shape here matches `WorkContract`'s non-optional fields
/// exactly — `thresholds` and `baseline_file_manifest` must both be fully
/// populated for serde_json to deserialize. `claims`, `implements` and
/// `verification_level` are the levers CB-1610..CB-1619 actually read.
fn write_work_contract(root: &Path, ticket: &str, level: &str, yaml_sha: &str) {
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
        "claims": [{
            "hypothesis": format!("{} climbs monotonically", ticket),
            "falsification_method": {
                "ProvableContract": {
                    "yaml_path": "contracts/ladder-v1.yaml",
                    "equation": "climb",
                    "test_id": "climb_monotone_test",
                    "expected": "true"
                }
            },
            "evidence_required": { "BooleanCheck": true },
            "result": null,
            "override_info": null
        }],
        "implements": [{
            "contract": "ladder-v1",
            "equation": "climb",
            "file": "contracts/ladder-v1.yaml",
            "sha": yaml_sha,
            "bound_at": "2026-04-18T00:00:00Z"
        }]
    });
    let dir = root.join(".pmat-work").join(ticket);
    fs::create_dir_all(&dir).expect("mkdir ticket");
    fs::write(
        dir.join("contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .expect("write contract.json");
}

/// Fully-populated `ContractThresholds` JSON — required fields only.
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

/// `FileManifest` stub — serde requires all three fields.
fn manifest_stub() -> serde_json::Value {
    serde_json::json!({
        "files": {},
        "coverage_required": [],
        "manifest_hash": "stub"
    })
}

/// Layer the evidence files matching each level so the corresponding CB check
/// can flip from Skip to Pass. Lower-level evidence is required by higher
/// levels (L3 requires L1 evidence too), so we stack files as we ascend.
fn stage_level_evidence(root: &Path, ticket: &str, level: &str) {
    let ticket_dir = root.join(".pmat-work").join(ticket);
    let lvl = level_as_num(level);

    // L1+ : verification-report.json carrying green l1_test_evidence.
    if lvl >= 1 {
        let vr = serde_json::json!({
            "l1_test_evidence": { "success": true, "exit_code": 0 },
        });
        fs::write(
            ticket_dir.join("verification-report.json"),
            serde_json::to_vec_pretty(&vr).unwrap(),
        )
        .expect("write verification-report");
    }

    // L3+ : falsification.log with every line status=pass.
    if lvl >= 3 {
        let line = serde_json::json!({
            "test_id": "climb_monotone_test",
            "method": "ProvableContract",
            "status": "pass",
            "duration_ms": 12,
        });
        let body = format!("{}\n", serde_json::to_string(&line).unwrap());
        fs::write(ticket_dir.join("falsification.log"), body).expect("write falsification.log");
    }

    // L4+ : kani-report.json + kani-harness-shas.json snapshot.
    if lvl >= 4 {
        let kr = serde_json::json!({ "success": true, "harnesses": ["verify_climb_monotone"] });
        fs::write(
            ticket_dir.join("kani-report.json"),
            serde_json::to_vec_pretty(&kr).unwrap(),
        )
        .expect("write kani-report");
        // Snapshot must match the YAML's per-harness sha to satisfy CB-1615.
        let snapshot = serde_json::json!({
            "harnesses": [{
                "name": "verify_climb_monotone",
                "sha": "b8f4c16a4f8c16a4b8f4c16a4f8c16a4b8f4c16a4f8c16a4b8f4c16a4f8c16a4",
            }]
        });
        fs::write(
            ticket_dir.join("kani-harness-shas.json"),
            serde_json::to_vec_pretty(&snapshot).unwrap(),
        )
        .expect("write kani-harness-shas");
    }

    // L5 : lean-proof.json with sorry_count: 0.
    if lvl >= 5 {
        let lp = serde_json::json!({ "sorry_count": 0, "theorem": "Climb_Monotone" });
        fs::write(
            ticket_dir.join("lean-proof.json"),
            serde_json::to_vec_pretty(&lp).unwrap(),
        )
        .expect("write lean-proof");
    }
}

fn level_as_num(level: &str) -> u8 {
    match level.chars().nth(1) {
        Some(d) if d.is_ascii_digit() => d.to_digit(10).unwrap_or(0) as u8,
        _ => 0,
    }
}

fn write_min_project_files(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"cb16xx-ladder\"\nversion = \"0.0.1\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(
        src.join("lib.rs"),
        "pub fn climb(n: u32) -> u32 { n + 1 }\n",
    )
    .expect("write lib.rs");
}

fn hex(d: &[u8]) -> String {
    use std::fmt::Write;
    d.iter().fold(String::with_capacity(64), |mut acc, b| {
        let _ = write!(acc, "{:02x}", b);
        acc
    })
}

/// Invoke `pmat comply check` on the synthesized project and echo CB-1610..
/// CB-1619 lines. If `pmat` is missing we print a hint and keep going — the
/// walkthrough text is the primary payload.
fn run_pmat_comply(root: &Path) {
    println!("Invoking: pmat comply check -p <tmpdir> -f text");
    let binary = option_env!("CARGO_BIN_EXE_pmat").unwrap_or("pmat");
    let out = Command::new(binary)
        .args(["comply", "check", "-p"])
        .arg(root)
        .args(["-f", "text"])
        .output();

    let output = match out {
        Ok(o) => o,
        Err(e) => {
            println!("  (pmat not found on PATH: {e})");
            println!("  Install with `cargo install --path .` from the pmat checkout.");
            return;
        }
    };
    println!("  exit status: {}", output.status);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let ladder: Vec<&str> = stdout
        .lines()
        .chain(stderr.lines())
        .filter(|l| is_ladder_line(l))
        .collect();
    if ladder.is_empty() {
        println!("  (no CB-1610..CB-1619 lines — likely a pmat build predating PR #302)");
    } else {
        println!("  Ladder check rows (CB-1610..CB-1619):");
        for line in ladder {
            println!("    {}", line.trim_start());
        }
    }
}

fn is_ladder_line(line: &str) -> bool {
    for n in 1610..=1619 {
        let needle = format!("CB-{}:", n);
        if line.contains(&needle) {
            return true;
        }
    }
    false
}

fn print_ladder_notes() {
    println!(
        "\
After staging level-matched evidence for each ticket, the ladder checks
should behave as follows against the synthesized tempdir:

  CB-1610  all six tickets parse their verification_level cleanly     → Pass
  CB-1611  each ticket's claimed level is <= binding's max attainable → Pass
  CB-1612  only LADDER-L1..L5 carry l1_test_evidence (L0 has none)    → Pass
  CB-1613  LADDER-L3..L5 each have a pass-only falsification.log      → Pass
  CB-1614  LADDER-L4..L5 each have kani-report.json with success=true → Pass
  CB-1615  L4+ kani-harness-shas snapshot matches YAML per-harness sha → Pass
  CB-1616  LADDER-L5 has lean-proof.json with sorry_count=0           → Pass

Remove any of the staged evidence files to watch the corresponding check flip
back to Skip (or to Fail if the shape is malformed). See
docs/specifications/components/pmat-work-contract-binding.md for the migration
order and the semantics of each level.
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
