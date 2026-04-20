//! Exit-Code Contract Harness (R6)
//!
//! Programmatic test that asserts `pmat <subcmd>` exits with the CORRECT
//! exit code on invalid input. Covers three known bugs from the R-round
//! issue triage:
//!
//!   - D25 (issue #333): `pmat split` with no arg must exit non-zero
//!   - D26 (issue #336): `pmat predict-quality` with no arg must exit non-zero
//!   - D35 (issue #339): `pmat roadmap list` unrecognized subcommand must
//!     exit non-zero
//!
//! The CLI contract for clap-based binaries is: missing required args or
//! unrecognized subcommands must exit with code 2 (standard clap behavior).
//! Any exit code == 0 on these invalid inputs is a FALSIFY target.
//!
//! Run with: `cargo run --example exit_code_contract`
//!
//! # Exit-Code Convention
//!
//! - `0` : success — NEVER acceptable here; every case is invalid input
//! - `1` : generic runtime error
//! - `2` : clap usage error (missing arg, unknown subcommand) — EXPECTED
//!
//! Cases printed `FAIL` when the observed exit code is 0 on invalid input.
//! Otherwise `PASS` — any non-zero exit is treated as contract-respecting.
//!
//! # Reproduction
//!
//! This harness uses `option_env!("CARGO_BIN_EXE_pmat")` so the example
//! runs against the freshly built workspace binary when executed via
//! `cargo run --example`. It falls back to the `pmat` on `PATH` when the
//! env var is unset (e.g. when the example is compiled outside of cargo).
//!
//! # Isolation
//!
//! Each invocation runs inside a throwaway directory from `tempfile::tempdir()`
//! so local `.pmat/` state or project context does not perturb the result.

use std::process::Command;

/// Resolve the pmat binary. Cargo sets `CARGO_BIN_EXE_pmat` for the main
/// binary at example compile time. If unset we fall back to the PATH.
fn pmat_bin() -> &'static str {
    option_env!("CARGO_BIN_EXE_pmat").unwrap_or("pmat")
}

/// A single exit-code contract case.
struct Case {
    id: &'static str,
    issue: &'static str,
    args: &'static [&'static str],
    /// Acceptable exit codes (non-empty). Typically `&[2]` for clap usage errors.
    /// We also accept anything non-zero to stay robust across clap versions.
    expected_nonzero: bool,
}

impl Case {
    fn run(&self, bin: &str, cwd: &std::path::Path) -> (i32, bool) {
        let out = Command::new(bin)
            .args(self.args)
            .current_dir(cwd)
            .output()
            .expect("failed to spawn pmat");
        let code = out.status.code().unwrap_or(-1);
        let pass = if self.expected_nonzero {
            code != 0
        } else {
            code == 0
        };
        (code, pass)
    }
}

fn main() {
    println!("R6 — Exit-Code Contract Harness");
    println!("{}", "=".repeat(60));

    let bin = pmat_bin();
    println!("Using pmat binary: {}", bin);

    // Isolate each case in a throwaway directory so local state
    // (.pmat/, .git/, .pmat-work/, etc.) does not contaminate results.
    let td = tempfile::tempdir().expect("failed to create tempdir");
    let cwd = td.path();
    println!("Temp cwd: {}", cwd.display());
    println!();

    let cases: &[Case] = &[
        Case {
            id: "D25",
            issue: "#333",
            args: &["split"],
            expected_nonzero: true,
        },
        Case {
            id: "D26",
            issue: "#336",
            args: &["predict-quality"],
            expected_nonzero: true,
        },
        Case {
            id: "D35",
            issue: "#339",
            args: &["roadmap", "list"],
            expected_nonzero: true,
        },
    ];

    let mut fails = 0;
    for (idx, case) in cases.iter().enumerate() {
        let (code, pass) = case.run(bin, cwd);
        let verdict = if pass { "PASS" } else { "FAIL" };
        println!(
            "CASE {} [{} / {}]: pmat {}  expected exit=nonzero, got exit={}  {}",
            idx + 1,
            case.id,
            case.issue,
            case.args.join(" "),
            code,
            verdict,
        );
        if !pass {
            fails += 1;
        }
    }

    println!();
    println!("{}", "-".repeat(60));
    println!(
        "Summary: {} passed, {} failed (total: {})",
        cases.len() - fails,
        fails,
        cases.len()
    );

    if fails > 0 {
        println!();
        println!("NOTE: A FAIL here means the contract was FALSIFIED — pmat");
        println!("exited 0 on invalid input. This is the bug the issue tracks.");
        println!("Once the fix lands, this example should print all PASS.");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example_compiles() {
        // Smoke test: building the example is the contract.
    }
}
