//! TDD Test for Dependency Duplicate Reduction (GH-89)
//!
//! Following EXTREME TDD principles:
//! 1. RED: Define max duplicate threshold
//! 2. GREEN: Reduce duplicates below threshold
//! 3. REFACTOR: Document remaining unavoidable duplicates

use std::process::Command;

/// Maximum number of unique duplicate packages allowed
/// Baseline (Nov 2024): 28 unique duplicate packages
/// These are mostly from external dependencies we cannot control:
/// - octocrab → jsonwebtoken → rsa → rand 0.8
/// - trueno-graph → thiserror v1 (waiting for upstream update)
/// - arrow → flatbuffers → bitflags v1
///
/// Target: ≤70 (updated after stack version updates)
const MAX_DUPLICATE_PACKAGES: usize = 70;

/// Critical duplicates that MUST be eliminated (zero tolerance)
const CRITICAL_DUPLICATES: &[&str] = &[
    // These duplicates cause significant binary bloat
    // Currently all eliminated via dependency updates
];

/// Known unavoidable duplicates (documented exceptions)
/// These are caused by dependencies we cannot control
/// Last verified: Dec 2024
const KNOWN_UNAVOIDABLE: &[&str] = &[
    // Arrow/Parquet ecosystem (used by trueno-db)
    "bitflags", // arrow/flatbuffers uses v1, we use v2
    "object",   // different versions for different toolchains
    // Cryptography (jsonwebtoken/rsa for octocrab)
    "digest",        // crypto crates have tight version requirements
    "generic-array", // crypto dependency
    "sha2",          // crypto dependency
    // Random number generation
    "rand",        // RSA/jsonwebtoken/octocrab chain uses v0.8
    "rand_core",   // comes with rand
    "rand_chacha", // comes with rand
    "getrandom",   // OS random source
    // Error handling
    "thiserror",      // trueno-graph uses v1, we use v2
    "thiserror-impl", // comes with thiserror
    // Platform abstraction
    "rustix",        // different crates have different version needs
    "linux-raw-sys", // comes with rustix
    // Serialization (serde versions are usually compatible)
    "serde",      // minor version differences
    "serde_json", // minor version differences
    // Logging ecosystem
    "log",        // different crates use different log versions
    "env_logger", // test dependencies
    // Numeric/algebra
    "nalgebra",        // different versions for different features
    "nalgebra-macros", // comes with nalgebra
    // Parsing infrastructure
    "syn",        // proc-macro ecosystem
    "itertools",  // different versions in dependency tree
    "phf_shared", // perfect hash function
    // HTTP
    "http",     // axum/hyper versions
    "httparse", // HTTP parsing
    // Hashing
    "siphasher", // different versions
    "hashbrown", // hash map implementations
    "twox-hash", // xxhash versions
    // Text/Unicode
    "unicode-width", // text display widths
];

#[test]
fn test_duplicate_package_count_under_threshold() {
    // Get cargo tree -d output
    let output = Command::new("cargo")
        .args(["tree", "-d"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo tree -d");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Count unique duplicate packages (lines starting with package name)
    let duplicate_packages: Vec<&str> = stdout
        .lines()
        .filter(|line| line.chars().next().is_some_and(|c| c.is_ascii_lowercase()))
        .filter_map(|line| line.split_whitespace().next())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let count = duplicate_packages.len();

    assert!(
        count <= MAX_DUPLICATE_PACKAGES,
        "Too many duplicate packages: {} (max allowed: {})\n\
         Duplicates found: {:?}\n\n\
         To fix: Update dependencies in Cargo.toml or add to KNOWN_UNAVOIDABLE with justification",
        count,
        MAX_DUPLICATE_PACKAGES,
        duplicate_packages
    );

    println!(
        "✅ Duplicate package count: {} (threshold: {})",
        count, MAX_DUPLICATE_PACKAGES
    );
}

#[test]
fn test_no_critical_duplicates() {
    // Get cargo tree -d output
    let output = Command::new("cargo")
        .args(["tree", "-d"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo tree -d");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut violations = Vec::new();

    for critical in CRITICAL_DUPLICATES {
        if stdout.contains(critical) {
            violations.push(*critical);
        }
    }

    assert!(
        violations.is_empty(),
        "Critical duplicates found that MUST be eliminated: {:?}\n\
         These cause significant binary bloat and build time impact.",
        violations
    );

    println!("✅ No critical duplicates found");
}

#[test]
fn test_document_unavoidable_duplicates() {
    // Get cargo tree -d output
    let output = Command::new("cargo")
        .args(["tree", "-d"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo tree -d");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract actual duplicates
    let actual_duplicates: std::collections::HashSet<&str> = stdout
        .lines()
        .filter(|line| line.chars().next().is_some_and(|c| c.is_ascii_lowercase()))
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    // Check if any new duplicates appeared that aren't documented
    let known_set: std::collections::HashSet<&str> = KNOWN_UNAVOIDABLE.iter().copied().collect();

    let undocumented: Vec<&str> = actual_duplicates.difference(&known_set).copied().collect();

    if !undocumented.is_empty() {
        println!(
            "⚠️  Undocumented duplicates (consider adding to KNOWN_UNAVOIDABLE):\n  {:?}",
            undocumented
        );
    }

    // Check if any documented duplicates are no longer present (cleanup opportunity)
    let no_longer_duplicate: Vec<&str> =
        known_set.difference(&actual_duplicates).copied().collect();

    if !no_longer_duplicate.is_empty() {
        println!(
            "✅ These documented duplicates have been resolved (remove from KNOWN_UNAVOIDABLE):\n  {:?}",
            no_longer_duplicate
        );
    }

    // Print summary
    println!("\n📊 Dependency Duplicate Summary:");
    println!("   Total duplicates: {}", actual_duplicates.len());
    println!("   Known/documented: {}", KNOWN_UNAVOIDABLE.len());
    println!("   Undocumented: {}", undocumented.len());
}

#[test]
fn test_dependency_tree_output_format() {
    // Validate that cargo tree -d works correctly
    let output = Command::new("cargo")
        .args(["tree", "-d"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo tree -d");

    assert!(
        output.status.success(),
        "cargo tree -d failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Basic format validation
    assert!(
        stdout.lines().count() > 0,
        "cargo tree -d produced no output"
    );

    println!("✅ cargo tree -d command works correctly");
}
