//! FALSIFY Token Demo (R6)
//!
//! Demonstrates the `FALSIFY-XXX-001` pattern on a minimal contract. A
//! FALSIFY token is a documented, human-readable proof that a specific
//! contract was exercised with a KNOWN-VIOLATING input and observed to
//! fail — i.e. the contract was falsifiable, not vacuous.
//!
//! Run with: `cargo run --example falsify_token_demo`
//!
//! # The Pattern
//!
//! Given a function with a pre-condition:
//!
//! ```rust,ignore
//! fn reciprocal(x: f64) -> f64 {
//!     assert!(x != 0.0, "pre: x must be nonzero");
//!     1.0 / x
//! }
//! ```
//!
//! A FALSIFY token records:
//!
//!   - `id`          — stable ID, e.g. `FALSIFY-RECIPROCAL-001`
//!   - `contract`    — the source-level contract text
//!   - `violation`   — the specific input that violates it
//!   - `observed`    — what happened (panic, error, etc.)
//!   - `artifact`    — optional path to the captured panic/log
//!   - `commit`      — git SHA at which the falsification was observed
//!
//! The token lives as a JSON object so CI can parse it, and as a source
//! comment so reviewers can read it. CB-1400 work-contracts embed a list
//! of these tokens per ticket.
//!
//! # Why Bother?
//!
//! Mutation testing asks "would a random perturbation be caught?" but
//! FALSIFY tokens are the INVERSE: we KNOW the perturbation and assert
//! the contract catches it. Every claim in a contract.json MUST have a
//! corresponding falsification receipt — otherwise the claim is vacuous.

use std::panic;

/// Trivial contract: `x` must be nonzero. Panics on violation.
fn reciprocal(x: f64) -> f64 {
    assert!(x != 0.0, "pre: x must be nonzero");
    1.0 / x
}

/// Attempt to falsify the contract by calling it with a known-violating
/// input (`0.0`). Capture the panic via `catch_unwind` and build a token.
fn falsify_reciprocal() -> serde_json::Value {
    // Suppress the panic hook stderr output for a clean demo.
    let prev = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(|| reciprocal(0.0));
    panic::set_hook(prev);

    let (observed, caught) = match result {
        Ok(v) => (format!("UNEXPECTED_OK value={}", v), false),
        Err(payload) => {
            // Panic payloads are typed-erased; try the two common shapes.
            let msg = payload
                .downcast_ref::<&'static str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic>".to_string());
            (format!("PANIC: {}", msg), true)
        }
    };

    serde_json::json!({
        "id": "FALSIFY-RECIPROCAL-001",
        "contract": "fn reciprocal(x: f64) -> f64  [pre: x != 0.0]",
        "violation": {
            "call": "reciprocal(0.0)",
            "reason": "pre-condition `x != 0.0` violated"
        },
        "observed": observed,
        "caught": caught,
        "artifact": null,
        "commit": option_env!("PMAT_COMMIT_SHA").unwrap_or("<unknown>"),
        "schema": "FALSIFY-v1"
    })
}

fn main() {
    println!("R6 — FALSIFY Token Demo");
    println!("{}", "=".repeat(60));

    println!("\nStep 1: define a contract");
    println!("{}", "-".repeat(40));
    println!("fn reciprocal(x: f64) -> f64 {{");
    println!("    assert!(x != 0.0, \"pre: x must be nonzero\");");
    println!("    1.0 / x");
    println!("}}");

    println!("\nStep 2: exercise with a violating input");
    println!("{}", "-".repeat(40));
    println!("  calling reciprocal(0.0) ...");
    let token = falsify_reciprocal();
    let caught = token["caught"].as_bool().unwrap_or(false);
    println!("  contract caught violation: {}", caught);

    println!("\nStep 3: emit the FALSIFY token");
    println!("{}", "-".repeat(40));
    // Pretty-print so the shape is readable in the example output.
    let pretty = serde_json::to_string_pretty(&token).unwrap_or_default();
    println!("{}", pretty);

    println!("\nStep 4: token shape (type summary)");
    println!("{}", "-".repeat(40));
    println!(
        "\
  id         : string   (FALSIFY-<NAME>-<NNN>)
  contract   : string   (source-level signature + pre/post)
  violation  : object   (input + natural-language reason)
  observed   : string   (PANIC: .. | ERROR: .. | UNEXPECTED_OK ..)
  caught     : bool     (true iff contract rejected the input)
  artifact   : string?  (path to captured stderr/log, optional)
  commit     : string   (git SHA; proof the test ran at this commit)
  schema     : string   (FALSIFY-v1)
"
    );

    println!("{}", "=".repeat(60));
    println!("A token with caught=true is the receipt a reviewer wants to see.");
    println!("A token with caught=false means the contract is vacuous — fix the contract.");

    // Assert we actually falsified. This double-checks the demo is not silently broken.
    assert!(caught, "demo broken: reciprocal(0.0) did not panic");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_falsification_catches_zero() {
        let token = super::falsify_reciprocal();
        assert_eq!(token["caught"], serde_json::Value::Bool(true));
        assert_eq!(token["schema"], "FALSIFY-v1");
    }
}
