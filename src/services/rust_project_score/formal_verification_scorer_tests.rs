// Tests for FormalVerificationScorer
// Included by formal_verification_scorer.rs — shares parent module scope

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scorer_name() {
        let scorer = FormalVerificationScorer::new();
        assert_eq!(scorer.name(), "Formal Verification");
    }

    #[test]
    fn test_max_points() {
        let scorer = FormalVerificationScorer::new();
        assert_eq!(scorer.max_points(), 16.0); // Miri (3) + Kani (5) + Verus (5) + Lean (3)
    }

    #[test]
    fn test_no_unsafe_gives_full_miri_credit() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

        // Create a safe Rust file
        std::fs::write(src_dir.join("lib.rs"), "pub fn safe_fn() -> i32 { 42 }\n")
            .expect("internal error");

        // Create Cargo.toml
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .expect("internal error");

        // Should get full Miri credit (3pts) for no unsafe code
        assert!(result.earned >= MIRI_POINTS);
    }

    #[test]
    fn test_count_unsafe_blocks() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

        // Create file with unsafe blocks
        // SAFETY: String literal test fixture -- not an actual unsafe block in this file.
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
pub fn with_unsafe() {
    unsafe {
        // do something
    }

    unsafe { std::ptr::null::<i32>().read() }
}
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let count = scorer.count_unsafe_blocks(temp_dir.path(), None);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_kani_proofs() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

        // Create file with Kani proofs
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
#[kani::proof]
fn check_addition() {
    let a: u8 = kani::any();
    let b: u8 = kani::any();
    kani::assume(a.checked_add(b).is_some());
    assert!(a + b >= a);
}

#[kani::proof]
fn check_subtraction() {
    let a: u8 = kani::any();
    let b: u8 = kani::any();
    kani::assume(a >= b);
    assert!(a - b <= a);
}
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let count = scorer.count_kani_proofs(temp_dir.path(), None);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_parse_test_count() {
        let output = "test result: ok. 5 passed; 2 failed; 0 ignored;";
        assert_eq!(parse_test_count(output, "passed"), 5);
        assert_eq!(parse_test_count(output, "failed"), 2);
        assert_eq!(parse_test_count(output, "ignored"), 0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = FormalVerificationScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }

    #[test]
    fn test_count_verus_specs() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("internal error");

        // Create file with Verus specifications
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
use vstd::prelude::*;

verus! {

#[requires(x > 0)]
#[ensures(result > x)]
pub fn add_one(x: u32) -> u32 {
    x + 1
}

#[requires(len > 0)]
#[ensures(result <= len)]
#[decreases(len)]
pub fn recursive_fn(len: u32) -> u32 {
    if len == 1 { 1 } else { recursive_fn(len - 1) }
}

#[invariant(self.value >= 0)]
pub struct Counter {
    value: i32,
}

}
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let count = scorer.count_verus_specs(temp_dir.path(), None);
        // Only counts Verus-specific: 1x decreases (requires/ensures are provable-contracts macros)
        assert_eq!(count, 1);
    }

    #[test]
    fn test_has_vstd_dependency() {
        let temp_dir = TempDir::new().expect("internal error");

        // Create Cargo.toml with vstd dependency
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "verus-project"
version = "0.1.0"
edition = "2021"

[dependencies]
vstd = { path = "../vstd" }
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        assert!(scorer.has_vstd_dependency(temp_dir.path()));
    }

    #[test]
    fn test_no_vstd_dependency() {
        let temp_dir = TempDir::new().expect("internal error");

        // Create Cargo.toml without vstd
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "regular-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#,
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        assert!(!scorer.has_vstd_dependency(temp_dir.path()));
    }

    #[test]
    fn test_contract_macros_give_points() {
        let temp_dir = TempDir::new().expect("internal error");
        let src_dir = temp_dir.path().join("src");
        let contracts_dir = temp_dir.path().join("contracts");
        std::fs::create_dir_all(&src_dir).expect("internal error");
        std::fs::create_dir_all(&contracts_dir).expect("internal error");

        // Create file with #[contract] macros
        std::fs::write(
            src_dir.join("lib.rs"),
            r#"
#[contract("softmax-v1", equation = "softmax")]
pub fn softmax(x: &[f32]) -> Vec<f32> { vec![1.0] }

#[provable_contracts_macros::contract("relu-v1", equation = "relu")]
pub fn relu(x: &[f32]) -> Vec<f32> { vec![0.0] }
"#,
        )
        .expect("internal error");

        // Create Cargo.toml
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("internal error");

        let scorer = FormalVerificationScorer::new();
        let result = scorer
            .score_with_mode(temp_dir.path(), ScoringMode::Quick)
            .expect("internal error");

        // Should get Miri points (no unsafe) + contract points
        assert!(result.earned > MIRI_POINTS);
    }
}
