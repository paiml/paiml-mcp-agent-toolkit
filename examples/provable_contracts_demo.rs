//! Provable Contracts Compliance Demo
//!
//! Demonstrates the CB-1200..CB-1214 provable-contracts enforcement chain
//! and the PV-01..PV-05 infra-score bonus scoring.
//!
//! Run with: `cargo run --example provable_contracts_demo`
//!
//! # Checks Demonstrated
//!
//! ## Compliance (pmat comply check)
//! - CB-1200: Contract existence + pv lint + binding coverage
//! - CB-1201: pv lint pass/fail with error detail
//! - CB-1202: Critical keyword coverage (forward, backward, kernel, etc.)
//! - CB-1203: Contract annotation coverage on bound functions
//! - CB-1204: build.rs pipeline enforcement
//! - CB-1205: Provability invariant (obligations → kani harnesses)
//! - CB-1206: Verification level distribution (L1-L5)
//! - CB-1207: Contract drift detection (90-day staleness)
//! - CB-1208: Binding existence + enforcement level (L0-L3)
//! - CB-1209: Contract trait enforcement (13 kernel traits)
//! - CB-1210: YAML precondition diversity
//! - CB-1211: Codegen fidelity (placeholder ratio check)
//! - CB-1214: Enforcement quality (call-site penetration × quality)
//!
//! ## Infra-Score (pmat infra-score)
//! - PV-01: pv lint passes (3 pts)
//! - PV-02: pv score >= 0.5 (3 pts)
//! - PV-03: At least one contract at proof level L2+ (2 pts)
//! - PV-04: contracts/ directory exists with schema-valid YAML (2 pts)
//! - PV-05: Enforcement quality — call-site penetration (2 pts)
//!
//! # Enforcement Levels
//!
//! | Level | Mechanism | What it proves |
//! |-------|-----------|----------------|
//! | L3 | build.rs + traits | Function exists + right signature + build enforced |
//! | L2 | traits only | Function exists + right signature |
//! | L1 | build.rs only | YAML says "implemented" |
//! | L0 | paper-only | Nothing — ghost bindings (FAIL) |
//!
//! # Enforcement Quality (E-levels)
//!
//! | Level | Score | Meaning |
//! |-------|-------|---------|
//! | E0 | 0.1 | Generic `!is_empty()` assertion at call site |
//! | E1 | 0.5 | Domain-specific precondition check only |
//! | E2 | 1.0 | Both precondition and postcondition checks |

use std::path::Path;

fn main() {
    println!("=== Provable Contracts Compliance Demo ===\n");

    // Demonstrate contract YAML structure
    let example_contract = r#"
metadata:
  version: "1.0.0"
  created: "2026-03-29"
  author: "PAIML Engineering"
  description: "Softmax kernel contract"

equations:
  softmax:
    formula: "softmax(x)_i = exp(x_i) / sum(exp(x_j))"
    preconditions:
      - 'x.iter().all(|v| v.is_finite())'
      - 'x.len() > 0'
    postconditions:
      - '(result.iter().sum::<f32>() - 1.0).abs() < 1e-5'

proof_obligations:
  - id: softmax-sums-to-one
    description: "Softmax output sums to 1.0"
    strategy: algebraic

falsification_tests:
  - name: test_softmax_sums_to_one
    test: "tests::test_softmax_sums_to_one"
"#;

    println!("Example Contract YAML:");
    println!("{example_contract}");

    // Demonstrate binding YAML structure
    let example_binding = r#"
version: 1.0.0
target_crate: my-project
bindings:
  - contract: softmax-kernel-v1.yaml
    equation: softmax
    function: softmax
    module_path: my_project::nn::softmax
    status: implemented
"#;

    println!("Example Binding YAML:");
    println!("{example_binding}");

    // Demonstrate codegen output
    let example_codegen = r#"
// Auto-generated from contracts/softmax-kernel-v1.yaml — DO NOT EDIT
// Contract: softmax-kernel-v1

/// Preconditions for equation `softmax`.
/// Domain-specific. Call: `contract_pre_softmax!(slice_expr)`
macro_rules! contract_pre_softmax {
    () => {{}};
    ($input:expr) => {{
        let x = &$input;
        debug_assert!(x.iter().all(|v| v.is_finite()),
            "Contract softmax: precondition violated — x.iter().all(|v| v.is_finite())");
        debug_assert!(x.len() > 0,
            "Contract softmax: precondition violated — x.len() > 0");
    }};
}

/// Postconditions for equation `softmax`.
/// Call before return: `contract_post_softmax!(result_expr)`
macro_rules! contract_post_softmax {
    ($result:expr) => {{
        let _contract_result = &$result;
        debug_assert!((_contract_result.iter().sum::<f32>() - 1.0).abs() < 1e-5,
            "Contract softmax: postcondition violated");
    }};
}

/// Combined pre+post contract for equation `softmax`.
macro_rules! contract_softmax {
    ($input:expr, $body:expr) => {{
        contract_pre_softmax!($input);
        let _contract_result = $body;
        contract_post_softmax!(_contract_result);
        _contract_result
    }};
}
"#;

    println!("Example Codegen Output:");
    println!("{example_codegen}");

    // Check for contracts/ directory
    let contracts_dir = Path::new("contracts");
    if contracts_dir.exists() {
        println!("contracts/ directory found — running compliance checks:");
        println!("  cargo run -- comply check");
        println!("  cargo run -- infra-score");
    } else {
        println!("No contracts/ directory — PV checks will skip.");
        println!("To add provable contracts:");
        println!("  1. Create contracts/ directory with YAML files");
        println!("  2. Add binding.yaml mapping equations to functions");
        println!("  3. Run: pv codegen contracts/ -o src/generated_contracts.rs");
        println!("  4. Add contract_pre_*/contract_post_* macro invocations");
    }

    println!("\n=== CLI Commands ===\n");
    println!("# Full compliance audit");
    println!("pmat comply check\n");
    println!("# Check specific PV enforcement");
    println!("pmat comply check 2>&1 | grep 'CB-12'\n");
    println!("# Infra-score with PV bonus");
    println!("pmat infra-score\n");
    println!("# Enforcement quality report");
    println!("pv coverage --enforcement . --binding ../provable-contracts/contracts/<project>/binding.yaml\n");

    println!("=== Done ===");
}
