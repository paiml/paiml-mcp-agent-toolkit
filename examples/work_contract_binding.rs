//! Work Contract Binding Walkthrough (R6)
//!
//! Small programmatic walkthrough of the CB-1400 work-contract pattern:
//!
//!   1. Call `pmat work list` to surface the current work items (if any).
//!   2. Pick the first ticket ID from stdout (or use a placeholder).
//!   3. Print a skeleton of `.pmat-work/<TICKET>/contract.json` showing
//!      where FALSIFY tokens, falsification receipts, and quality claims
//!      are wired in per the CB-1400 specification.
//!
//! Run with: `cargo run --example work_contract_binding`
//!
//! # CB-1400 Contract Schema (abbreviated)
//!
//! Each ticket gets a `contract.json` under `.pmat-work/<TICKET>/`. The
//! contract is the ticket's falsifiable promise: every listed claim must
//! have a corresponding FALSIFY receipt, otherwise the claim is vacuous
//! and `pmat comply check` rejects it.
//!
//! This example does NOT write the file — it only prints the skeleton so
//! developers understand what `pmat work ship` would generate.

use std::process::Command;

fn pmat_bin() -> &'static str {
    option_env!("CARGO_BIN_EXE_pmat").unwrap_or("pmat")
}

/// Shell out to `pmat work list` and capture stdout. Returns `None` if
/// the command is not available or exits non-zero — this example is
/// purely illustrative and must not hard-fail on missing state.
fn run_work_list(bin: &str) -> Option<String> {
    let out = Command::new(bin).args(["work", "list"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

/// Extract the first plausible ticket id (e.g. `PMAT-1234`, `CB-1401`,
/// `R6-001`) from `pmat work list` output. This is heuristic — the real
/// pmat APIs expose structured JSON, but the point here is to show the
/// shape, not to parse every output format.
fn extract_first_ticket(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        for token in line.split_whitespace() {
            let trimmed = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if is_ticket_id(trimmed) {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Matches `<UPPERCASE_PREFIX>-<digits>`, e.g. `PMAT-1204`, `CB-1400`, `R6-001`.
fn is_ticket_id(s: &str) -> bool {
    let (prefix, number) = match s.split_once('-') {
        Some(parts) => parts,
        None => return false,
    };
    if prefix.is_empty() || number.is_empty() {
        return false;
    }
    prefix
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        && number.chars().all(|c| c.is_ascii_digit())
}

fn main() {
    println!("R6 — Work Contract Binding Walkthrough");
    println!("{}", "=".repeat(60));

    let bin = pmat_bin();
    println!("Using pmat binary: {}", bin);

    println!("\nStep 1: invoke `pmat work list`");
    println!("{}", "-".repeat(40));
    let list = run_work_list(bin);
    let ticket = match list.as_deref() {
        Some(stdout) => {
            let preview: String = stdout.lines().take(5).collect::<Vec<_>>().join("\n");
            if preview.is_empty() {
                println!("  (empty work list)");
            } else {
                println!("  first 5 lines of output:");
                for line in preview.lines() {
                    println!("    | {}", line);
                }
            }
            extract_first_ticket(stdout).unwrap_or_else(|| "PMAT-0000".to_string())
        }
        None => {
            println!("  `pmat work list` unavailable or failed — using placeholder ticket");
            "PMAT-0000".to_string()
        }
    };
    println!("  selected ticket: {}", ticket);

    println!("\nStep 2: contract path");
    println!("{}", "-".repeat(40));
    let contract_path = format!(".pmat-work/{}/contract.json", ticket);
    println!("  path: {}", contract_path);

    println!("\nStep 3: CB-1400 contract.json skeleton");
    println!("{}", "-".repeat(40));
    let skeleton = serde_json::json!({
        "schema": "pmat-work-contract/v1",
        "ticket": ticket,
        "status": "in_progress",
        "claims": [
            {
                "id": "CLAIM-001",
                "statement": "Function `foo::bar` exits non-zero on invalid input",
                "falsify_token": "FALSIFY-FOO-BAR-001",
                "evidence": {
                    "kind": "exit_code",
                    "command": "pmat foo bar",
                    "expected_nonzero": true
                }
            },
            {
                "id": "CLAIM-002",
                "statement": "Cyclomatic complexity of `foo::bar` ≤ 15",
                "falsify_token": "FALSIFY-FOO-BAR-002",
                "evidence": {
                    "kind": "metric",
                    "metric": "cyclomatic_complexity",
                    "op": "<=",
                    "threshold": 15
                }
            }
        ],
        "falsify_receipts": [
            {
                "id": "FALSIFY-FOO-BAR-001",
                "observed": "PANIC: pre: x != 0.0",
                "caught": true,
                "commit": "<git-sha>",
                "schema": "FALSIFY-v1"
            }
        ],
        "quality_gates": {
            "tdg_grade": "A",
            "mutation_coverage": 0.80,
            "line_coverage": 0.95
        },
        "cb_checks": ["CB-1400", "CB-1401", "CB-1402"],
        "notes": "Every claim MUST have a matching FALSIFY receipt. `pmat comply check` rejects vacuous contracts."
    });
    let pretty = serde_json::to_string_pretty(&skeleton).unwrap_or_default();
    println!("{}", pretty);

    println!("\n{}", "=".repeat(60));
    println!("The real `pmat work ship` writes this file and `pmat comply check`");
    println!("verifies every `claims[].falsify_token` resolves to a receipt.");
}

#[cfg(test)]
mod tests {
    use super::is_ticket_id;

    #[test]
    fn ticket_id_matches() {
        assert!(is_ticket_id("PMAT-1204"));
        assert!(is_ticket_id("CB-1400"));
        assert!(is_ticket_id("R6-001"));
    }

    #[test]
    fn ticket_id_rejects() {
        assert!(!is_ticket_id(""));
        assert!(!is_ticket_id("foo"));
        assert!(!is_ticket_id("PMAT-"));
        assert!(!is_ticket_id("-1234"));
        assert!(!is_ticket_id("pmat-1204"));
    }
}
