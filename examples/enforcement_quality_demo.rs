//! Enforcement Quality Demo (CB-1214)
//!
//! Demonstrates how pmat measures contract enforcement quality at call sites.
//!
//! Run with: `cargo run --example enforcement_quality_demo`
//!
//! # E-Level Classification
//!
//! pmat classifies contract call sites into three enforcement levels:
//!
//! - **E0 (0.1)**: Generic `!is_empty()` — placeholder assertion
//! - **E1 (0.5)**: Domain-specific precondition check
//! - **E2 (1.0)**: Both precondition and postcondition checks
//!
//! Quality = weighted average of E-levels across all call sites.
//! Enforcement = penetration (call sites / bindings) × quality.

fn main() {
    println!("=== Enforcement Quality Demo ===\n");

    // Simulate E-level classification
    let scenarios = vec![
        ("Early adoption", vec![("E0", 10), ("E1", 0), ("E2", 0)]),
        ("Transitioning", vec![("E0", 5), ("E1", 15), ("E2", 3)]),
        ("Mature", vec![("E0", 2), ("E1", 20), ("E2", 30)]),
        ("Full enforcement", vec![("E0", 0), ("E1", 5), ("E2", 45)]),
    ];

    for (name, levels) in &scenarios {
        let e0 = levels
            .iter()
            .find(|(l, _)| *l == "E0")
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let e1 = levels
            .iter()
            .find(|(l, _)| *l == "E1")
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let e2 = levels
            .iter()
            .find(|(l, _)| *l == "E2")
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let total = e0 + e1 + e2;

        let quality = if total > 0 {
            (e0 as f64 * 0.1 + e1 as f64 * 0.5 + e2 as f64 * 1.0) / total as f64
        } else {
            0.0
        };

        let status = if quality >= 0.5 {
            "PASS"
        } else if e1 == 0 && e2 == 0 {
            "WARN (E0-only)"
        } else {
            "FAIL"
        };

        println!("{name}:");
        println!("  E0={e0} E1={e1} E2={e2} total={total} quality={quality:.2} → {status}");
    }

    println!("\n=== Call Site Patterns ===\n");

    println!("E0 — Generic (lowest quality):");
    println!("  contract_pre_softmax!();  // zero-arg no-op form");
    println!();

    println!("E1 — Domain precondition:");
    println!("  contract_pre_softmax!(input);  // checks x.is_finite(), x.len() > 0");
    println!();

    println!("E2 — Full pre+post:");
    println!("  let result = contract_softmax!(input, {{ softmax_impl(input) }});");
    println!("  // checks preconditions AND postconditions");
    println!();

    println!("=== CB-1214 Decision Logic ===\n");
    println!("FAIL:  quality < 0.3 AND >30 sites AND has E1/E2 mix (regression)");
    println!("WARN:  quality < 0.3 with E0-only (legitimate transition)");
    println!("WARN:  0 call sites (contracts exist but never invoked)");
    println!("SKIP:  pv CLI not available");
    println!("PASS:  quality >= 0.3");
}
