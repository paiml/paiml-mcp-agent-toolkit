//! CB-16xx Drift Detection Demo — CB-1621 + CB-1615 flipping Skip → Fail.
//!
//! Component 27 of the CB-16xx spec introduces *bind-time snapshots* so that
//! contract artifacts cannot drift silently after a ticket has been bound.
//! This example exercises two of those drift detectors:
//!
//!   * CB-1621 — `ProvableContract{expected}` snapshot vs. current YAML
//!     scalar `expected:` for the same `test_id`.
//!   * CB-1615 — Kani harness body hash recorded in
//!     `.pmat-work/<ID>/kani-harness-shas.json` vs. the YAML's per-harness
//!     `sha:` field.
//!
//! The run has two phases:
//!
//!   1. **Clean phase** — write a L4 ticket whose `ProvableContract.expected`
//!      and `kani-harness-shas.json` both match the YAML. Run `pmat comply
//!      check` and observe CB-1615/CB-1621 flip from Skip to **Pass**.
//!   2. **Drift phase** — mutate the YAML's `expected:` value AND the harness
//!      body (so its sha differs from the bind-time snapshot). Re-run comply
//!      and observe CB-1615/CB-1621 flip to **Fail**.
//!
//! Run with: `cargo run --example cb16xx_drift_detection`

use std::fs;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

const HARNESS_SHA_ORIGINAL: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const HARNESS_SHA_MUTATED: &str =
    "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

fn main() {
    println!("=== PMAT Comply — CB-16xx Drift Detection Demo ===\n");

    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    println!("Synthesizing project tree at {}\n", root.display());

    write_min_project_files(root);
    let clean_sha = write_clean_yaml(root);
    write_work_contract(root, &clean_sha);
    write_harness_snapshot(root, HARNESS_SHA_ORIGINAL);

    println!("── Phase 1: Clean (snapshots match) ──");
    run_and_filter(root, &[1615, 1621]);

    println!("\n── Phase 2: Inject drift ──");
    println!(
        "  * Mutate contracts/kernel.yaml: expected 'pass' → 'fail', harness sha → fresh value.\n\
         * `ProvableContract.expected` snapshot still says \"\\\"pass\\\"\".\n\
         * `kani-harness-shas.json` still says {}…",
        &HARNESS_SHA_ORIGINAL[..8]
    );
    mutate_yaml(root);
    run_and_filter(root, &[1615, 1621]);

    println!("\n=== Drift Semantics ===");
    print_drift_notes();
}

/// Minimal Cargo.toml + lib.rs so the tempdir registers as a Rust project.
fn write_min_project_files(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"cb16xx-drift\"\nversion = \"0.0.1\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("lib.rs"), "pub fn kernel() -> bool { true }\n").expect("write lib.rs");
}

/// The bind-time YAML: scalar `expected: pass`, harness SHA original. Returns
/// the SHA-256 hex digest so we can put it in the ticket's binding — CB-1601
/// will verify it matches on disk.
fn write_clean_yaml(root: &Path) -> String {
    // `kani_harnesses` and `falsification_tests` MUST be top-level — CB-1615
    // and CB-1621 scan for indentation == 0 sections; nesting under
    // `equations` silently drops them from the drift comparison.
    let yaml = format!(
        "metadata:\n\
         \x20 version: 1.0.0\n\
         equations:\n\
         \x20 kernel:\n\
         \x20   formula: kernel() == true\n\
         kani_harnesses:\n\
         \x20 - name: verify_kernel\n\
         \x20   sha: {}\n\
         falsification_tests:\n\
         \x20 - id: kernel_pass_test\n\
         \x20   test: proptest_kernel_pass\n\
         \x20   expected: pass\n",
        HARNESS_SHA_ORIGINAL
    );
    let dir = root.join("contracts");
    fs::create_dir_all(&dir).expect("mkdir contracts");
    fs::write(dir.join("kernel.yaml"), &yaml).expect("write kernel.yaml");

    let mut h = Sha256::new();
    h.update(yaml.as_bytes());
    hex(&h.finalize())
}

/// Write the L4 ticket with a ProvableContract snapshot expecting `"pass"` and
/// a binding referencing the clean YAML. `thresholds` and `baseline_file_manifest`
/// are fully populated — serde_json refuses short-form `{}` for either.
fn write_work_contract(root: &Path, yaml_sha: &str) {
    let contract = serde_json::json!({
        "version": "5.0",
        "work_item_id": "DRIFT-001",
        "created_at": "2026-04-18T00:00:00Z",
        "baseline_commit": "0".repeat(40),
        "baseline_tdg": 95.0,
        "baseline_coverage": 95.0,
        "baseline_rust_score": null,
        "baseline_file_manifest": manifest_stub(),
        "thresholds": thresholds_stub(),
        "verification_level": "L4",
        "iteration": 1,
        "claims": [{
            "hypothesis": "kernel() always returns true under bind",
            "falsification_method": {
                "ProvableContract": {
                    "yaml_path": "contracts/kernel.yaml",
                    "equation": "kernel",
                    "test_id": "kernel_pass_test",
                    // Canonical JSON for the YAML scalar `pass` — the yaml
                    // bareword decodes to the JSON string "pass".
                    "expected": "\"pass\""
                }
            },
            "evidence_required": { "BooleanCheck": true },
            "result": null,
            "override_info": null
        }],
        "implements": [{
            "contract": "kernel",
            "equation": "kernel",
            "file": "contracts/kernel.yaml",
            "sha": yaml_sha,
            "bound_at": "2026-04-18T00:00:00Z"
        }]
    });
    let dir = root.join(".pmat-work").join("DRIFT-001");
    fs::create_dir_all(&dir).expect("mkdir DRIFT-001");
    fs::write(
        dir.join("contract.json"),
        serde_json::to_vec_pretty(&contract).unwrap(),
    )
    .expect("write contract.json");
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

/// Write `.pmat-work/DRIFT-001/kani-harness-shas.json` with the per-harness
/// bind-time hash. CB-1615 compares this against the harness's current sha
/// in the YAML.
fn write_harness_snapshot(root: &Path, bind_time_sha: &str) {
    let snapshot = serde_json::json!({
        "harnesses": [{
            "name": "verify_kernel",
            "sha": bind_time_sha,
        }]
    });
    let dir = root.join(".pmat-work").join("DRIFT-001");
    fs::write(
        dir.join("kani-harness-shas.json"),
        serde_json::to_vec_pretty(&snapshot).unwrap(),
    )
    .expect("write kani-harness-shas.json");
}

/// Phase 2 mutation: swap the YAML's `expected:` scalar AND replace the
/// harness sha. The bind-time snapshot is left alone so drift shows up.
/// Note: CB-1601 will also flip to Fail here because the YAML bytes changed,
/// which is the canonical "re-bind needed" signal. We leave that alone —
/// the point is to keep the original snapshot on disk.
fn mutate_yaml(root: &Path) {
    let yaml = format!(
        "metadata:\n\
         \x20 version: 1.0.0\n\
         equations:\n\
         \x20 kernel:\n\
         \x20   formula: kernel() == true\n\
         kani_harnesses:\n\
         \x20 - name: verify_kernel\n\
         \x20   sha: {}\n\
         falsification_tests:\n\
         \x20 - id: kernel_pass_test\n\
         \x20   test: proptest_kernel_pass\n\
         \x20   expected: fail\n",
        HARNESS_SHA_MUTATED
    );
    fs::write(root.join("contracts").join("kernel.yaml"), yaml).expect("overwrite kernel.yaml");
}

fn hex(d: &[u8]) -> String {
    use std::fmt::Write;
    d.iter().fold(String::with_capacity(64), |mut acc, b| {
        let _ = write!(acc, "{:02x}", b);
        acc
    })
}

/// Run `pmat comply check` on `root` and print only lines mentioning the
/// given CB numeric ids. Missing pmat is not a fatal — we still print the
/// notes so the example works as documentation.
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

fn print_drift_notes() {
    println!(
        "\
CB-1615 and CB-1621 share the bind-time-snapshot pattern: a ticket's
`.pmat-work/<ID>/` directory records the artifact value at bind time, and the
check fails if the current YAML diverges without a re-bind. The workflow:

  1. `pmat work bind` writes the snapshot (and the `ContractBinding::sha`).
  2. YAML edits thereafter show as Fail until the ticket re-binds.
  3. Re-binding rotates both the snapshot and the YAML sha, clearing drift.

Other drift detectors in the same family:

  CB-1601  YAML body sha vs. bind-time ContractBinding.sha
  CB-1609  YAML file is git-tracked (structural drift-prevention)
  CB-1627  post-bind YAML drift propagated to inherited roster entries

Together they form a ratchet: once an artifact is bound, its shape is frozen
until an explicit re-bind event is recorded. No silent divergence.
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
