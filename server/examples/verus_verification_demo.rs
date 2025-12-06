//! Example demonstrating Verus formal verification detection in Rust Project Score
//!
//! This example shows how PMAT's FormalVerificationScorer detects and scores
//! Verus formal verification specs (#[requires], #[ensures], #[invariant]).
//!
//! Run with: cargo run --example verus_verification_demo

use pmat::services::rust_project_score::formal_verification_scorer::FormalVerificationScorer;
use pmat::services::rust_project_score::models::ScoringMode;
use pmat::services::rust_project_score::scorer::Scorer;
use std::fs;
use tempfile::TempDir;

fn main() {
    println!("Verus Formal Verification Scoring Demo\n");
    println!("================================================\n");

    // Create a temporary project with Verus specs
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src dir");

    // Create a Cargo.toml with vstd dependency (indicates Verus project)
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"[package]
name = "verus-example"
version = "0.1.0"
edition = "2021"

[dependencies]
vstd = { path = "../vstd" }
"#,
    )
    .expect("Failed to write Cargo.toml");

    // Create source file with Verus specifications
    fs::write(
        src_dir.join("lib.rs"),
        r#"
use vstd::prelude::*;

verus! {

/// A verified function that adds one to a positive number
/// Uses Verus formal verification to prove correctness
#[requires(x > 0)]
#[ensures(result > x)]
pub fn add_one(x: u32) -> (result: u32) {
    x + 1
}

/// A recursive function with decreases clause for termination proof
#[requires(n > 0)]
#[ensures(result <= n)]
#[decreases(n)]
pub fn countdown(n: u32) -> (result: u32) {
    if n == 1 {
        1
    } else {
        countdown(n - 1)
    }
}

/// A struct with data invariants
#[invariant(self.value >= 0)]
pub struct Counter {
    value: i32,
}

impl Counter {
    #[requires(initial >= 0)]
    #[ensures(result.value == initial)]
    pub fn new(initial: i32) -> (result: Self) {
        Counter { value: initial }
    }

    #[requires(self.value < i32::MAX)]
    #[ensures(self.value == old(self).value + 1)]
    pub fn increment(&mut self) {
        self.value += 1;
    }
}

/// Binary search with full correctness proof
#[requires(arr.len() > 0)]
#[requires(forall|i: int, j: int| 0 <= i < j < arr.len() ==> arr[i] <= arr[j])]
#[ensures(result.is_some() ==> arr[result.unwrap() as int] == target)]
#[ensures(result.is_none() ==> forall|i: int| 0 <= i < arr.len() ==> arr[i] != target)]
pub fn binary_search(arr: &[i32], target: i32) -> (result: Option<usize>) {
    let mut lo: usize = 0;
    let mut hi: usize = arr.len();

    while lo < hi
        invariant
            0 <= lo <= hi <= arr.len(),
            forall|i: int| 0 <= i < lo ==> arr[i] < target,
            forall|i: int| hi <= i < arr.len() ==> arr[i] > target,
    {
        let mid = lo + (hi - lo) / 2;
        if arr[mid] < target {
            lo = mid + 1;
        } else if arr[mid] > target {
            hi = mid;
        } else {
            return Some(mid);
        }
    }
    None
}

}
"#,
    )
    .expect("Failed to write lib.rs");

    // Create the scorer and analyze
    let scorer = FormalVerificationScorer::new();

    println!("Project: {}\n", temp_dir.path().display());

    // Score in Quick mode (no subprocess calls)
    let quick_result = scorer
        .score_with_mode(temp_dir.path(), ScoringMode::Quick)
        .expect("Scoring failed");

    println!("Scoring Results (Quick Mode):");
    println!("  Earned: {:.1} / {:.1} points", quick_result.earned, quick_result.max);
    println!("  Percentage: {:.0}%", quick_result.percentage());
    println!();

    // Get recommendations
    let recommendations = scorer.recommendations(temp_dir.path());
    if !recommendations.is_empty() {
        println!("Recommendations:");
        for rec in recommendations {
            println!("  - {}", rec);
        }
        println!();
    }

    // Demonstrate detection details
    println!("Detection Details:");
    println!("  - Detects #[requires(...)] preconditions");
    println!("  - Detects #[ensures(...)] postconditions");
    println!("  - Detects #[invariant(...)] data invariants");
    println!("  - Detects #[decreases(...)] termination measures");
    println!("  - Detects vstd dependency in Cargo.toml");
    println!();

    println!("Points Breakdown:");
    println!("  - Miri (no unsafe): 3.0 pts (full credit for safe code)");
    println!("  - Kani (no proofs): 0.0 pts");
    println!("  - Verus (specs found): ~{:.1} pts", quick_result.earned - 3.0);
    println!();

    println!("Integration with Verus:");
    println!("  - Verus project: https://github.com/verus-lang/verus");
    println!("  - Guide: https://verus-lang.github.io/verus/guide/");
    println!("  - PMAT Issue #106: Verus integration tracking");
}
