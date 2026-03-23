//! PMAT Unified Score Example
//!
//! Demonstrates the `pmat score` command — a single geometric composite
//! of 7 sub-scores that replaces running comply + rust-project-score +
//! repo-score independently.
//!
//! Run with: `cargo run --example score_demo`
//!
//! # Features Demonstrated
//!
//! 1. Composite score computation (geometric mean of 7 sub-scores)
//! 2. Gate checking (exit 1 if below threshold)
//! 3. Score trend tracking (sparkline over commit history)
//! 4. Regression detection (block if quality drops >5 pts)
//! 5. Cross-validation invariants (detect sub-score contradictions)
//! 6. Stack quality (sovereign dep scores)
//! 7. Score diagnosis via `pmat query --score-diagnosis`
//!
//! # Sub-Scores
//!
//! | Sub-Score   | Source                     | Scale |
//! |-------------|----------------------------|-------|
//! | RPS         | pmat rust-project-score    | 0-100 |
//! | Comply      | pmat comply check          | 0-100 |
//! | Coverage    | cargo llvm-cov cache       | 0-100 |
//! | Muda (inv)  | 100 - muda waste score     | 0-100 |
//! | EvoScore    | CB-142 test trajectory     | 0-100 |
//! | DBC         | pmat work codebase-score   | 0-100 |
//! | File Health | CB-040 file line counts    | 0-100 |
//!
//! # CLI Usage
//!
//! ```bash
//! # Full composite score
//! pmat score
//!
//! # Gate for CI (exit 1 if composite < 70)
//! pmat score --gate 70
//!
//! # JSON output for CI artifact
//! pmat score --format json -o score.json
//!
//! # Show score trend over recent commits
//! pmat score --trend
//!
//! # Check for regression (exit 1 if >5 pt drop)
//! pmat score --regression-check
//!
//! # Include sovereign stack dep scores
//! pmat score --stack
//!
//! # Diagnose which code drags each sub-score down
//! pmat query --score-diagnosis --limit 5
//! ```

fn main() {
    println!("PMAT Unified Score Demo");
    println!("{}", "=".repeat(60));

    // Example 1: Sub-score breakdown
    println!("\nExample 1: Sub-Score Sources");
    println!("{}", "-".repeat(40));
    demonstrate_sub_scores();

    // Example 2: Geometric mean formula
    println!("\nExample 2: Geometric Mean Computation");
    println!("{}", "-".repeat(40));
    demonstrate_geometric_mean();

    // Example 3: Gate usage
    println!("\nExample 3: CI Gate Integration");
    println!("{}", "-".repeat(40));
    demonstrate_gate();

    // Example 4: Cross-validation invariants
    println!("\nExample 4: Cross-Validation Invariants");
    println!("{}", "-".repeat(40));
    demonstrate_cross_validation();

    println!("\n{}", "=".repeat(60));
    println!("Run `pmat score` to compute your project's composite score.");
}

fn demonstrate_sub_scores() {
    let sub_scores = [
        ("RPS", 69.4, "Rust Project Score (11 category scorers)"),
        ("Comply", 66.0, "100 - (errors*10 + warnings*3)"),
        ("Coverage", 81.0, "LLVM line coverage percentage"),
        (
            "Muda (inv)",
            63.7,
            "100 - waste score (lower waste = higher)",
        ),
        ("EvoScore", 50.0, "clamp(evoscore, 0, 1) * 100"),
        ("DBC", 71.8, "Work contract 4-factor composite * 100"),
        ("File Health", 71.0, "Based on files exceeding line limits"),
    ];

    for (name, value, source) in &sub_scores {
        println!("  {:<14} {:>5.1}  {}", name, value, source);
    }
}

fn demonstrate_geometric_mean() {
    // Geometric mean: (a * b * c * ...)^(1/n)
    // One zero kills the composite — you can't ace testing while ignoring code quality.
    let values: [f64; 7] = [69.4, 66.0, 81.0, 63.7, 50.0, 71.8, 71.0];
    let n = values.len() as f64;
    let log_sum: f64 = values.iter().map(|v| (*v).ln()).sum();
    let composite = (log_sum / n).exp();

    println!("  Values: {:?}", values);
    println!("  Geometric mean: {:.1}", composite);
    println!(
        "  (Arithmetic mean would be: {:.1})",
        values.iter().sum::<f64>() / n
    );
    println!();
    println!("  Key property: geometric mean penalizes outliers more heavily.");
    println!("  A single low score drags the composite down significantly.");

    // Show what happens if one score is 0
    let with_zero = [69.4, 66.0, 81.0, 0.0, 50.0, 71.8, 71.0];
    println!();
    println!("  With one zero: {:?}", with_zero);
    println!("  Geometric mean: 0.0 (any zero kills the composite)");
}

fn demonstrate_gate() {
    println!("  CI workflow (quality-gate.yml):");
    println!();
    println!("    - name: Quality Gate");
    println!("      run: pmat score --gate 70 --format json > score.json");
    println!();
    println!("  Pre-push hook:");
    println!("    pmat score --gate 60  # advisory check before push");
    println!();
    println!("  Exit codes:");
    println!("    0 = composite >= gate threshold (pass)");
    println!("    1 = composite < gate threshold (fail)");
}

fn demonstrate_cross_validation() {
    let invariants = [
        ("XV-001", "Comply 0 errors => RPS Code Quality >= 40%"),
        ("XV-003", "Coverage >= 90% => RPS Testing >= 60%"),
        ("XV-007", "RPS Grade A => composite >= 75"),
        ("XV-008", "Comply 0 errors => RPS >= 60%"),
        ("XV-009", "File health A => Muda Over-processing < 15"),
        ("XV-010", "Coverage < 50% => composite < 80"),
    ];

    println!("  6 invariant rules detect contradictions between sub-scores:");
    println!();
    for (id, rule) in &invariants {
        println!("    {}: {}", id, rule);
    }
    println!();
    println!("  If 3+ invariants fail: systemic inconsistency warning.");
}
