# Verification Backends: Lean & Kani

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 24

## Lean Verification Backend (L5)

Lean is the **mandatory** verification backend for L5 contracts.
Every contract YAML with `equations:` MUST have `lean_theorem:`.

### Integration Model

```
Contract YAML                 Lean Theorem
─────────────                ─────────────
equations:                    theorem softmax_sum_one
  softmax:                      (x : Array Float)
    preconditions:               (h_finite : ∀ i, x[i].isFinite)
      - x.iter().all(finite)     : (softmax x).sum = 1.0 := by
    postconditions:              ...proof...
      - result.sum() ≈ 1.0
    lean_theorem: Theorems.Softmax_Sum_One
```

### Lean Contract Schema

```yaml
lean_verification:
  project: "provable-contracts/lean"
  lake_build: "lake build"
  theorems:
    - name: Theorems.Softmax_Sum_One
      file: Theorems/Softmax.lean
      status: proved  # proved | sorry | admitted | wip
      depends_on: [Mathlib.Analysis.SpecificLimits.Basic]
    - name: Theorems.Matmul_Associativity
      file: Theorems/Matmul.lean
      status: proved
      depends_on: [Mathlib.LinearAlgebra.Matrix.Basic]
  sorry_budget: 0  # ZERO sorry allowed for L5 (CB-1308)
  mathlib_version: ">=4.12.0"
```

### Lean-Specific Checks

| Check | What | Severity |
|-------|------|----------|
| CB-1205 | `lean_theorem:` exists for each equation | FAIL |
| CB-1308 | `status: proved` (no `sorry`) | FAIL |
| CB-1500 | `lake build` succeeds | FAIL |
| CB-1501 | Theorem references resolve in Lean project | FAIL |
| CB-1502 | `sorry` count = 0 in compiled Lean files | FAIL |
| CB-1503 | Lean version >= 4.12.0 (Mathlib compat) | WARN |

### Lean Expression Language in Contracts

Contracts can embed Lean expressions directly alongside Rust:

```yaml
postconditions:
  - rust: '(result.iter().sum::<f32>() - 1.0).abs() < 1e-5'
    lean: '(Array.sum (softmax x) - 1.0).abs < 1e-5'
    description: Softmax output sums to 1
```

Both `rust:` and `lean:` are verified independently — the Lean proof
covers all inputs, the Rust assertion catches regressions at runtime.

### Lean for Regex Contracts

Lean can prove regex language containment — that ALL valid inputs
produce outputs matching a given regex pattern:

```lean
-- Prove: ∀ valid input, parse_ticket_id produces PMAT-\d+ or GH-\d+
theorem ticket_id_format_correct
    (input : String)
    (h_valid : IsValidTicketRef input)
    : RegexMatch "^(PMAT|GH|EPIC)-\\d+$" (parseTicketId input) := by
    ...proof...
```

This is **strictly stronger** than Kani bounded checking — Lean proves
the property for ALL inputs, not just inputs up to a bound.

### Lean for Refinement Type Contracts

Lean can prove type class laws (algebraic properties) for refinement types:

```lean
-- Prove: Tokenizer is invertible (encode then decode = identity)
instance : Invertible Tokenizer where
  forward := Tokenizer.encode
  inverse := Tokenizer.decode
  roundtrip := by
    intro x
    simp [Tokenizer.encode, Tokenizer.decode]
    ...proof...
```

---

## Kani Verification Backend (L4)

Kani provides **bounded model checking** for Rust code. It exhaustively
verifies properties up to a configurable bound using SAT/SMT solvers.

### Integration Model

```
Contract YAML                 Kani Harness
─────────────                ─────────────
equations:                    #[kani::proof]
  matmul:                     fn verify_matmul() {
    preconditions:                let m: usize = kani::any();
      - a.len() == m * k          let k: usize = kani::any();
    postconditions:               let n: usize = kani::any();
      - result.len() == m * n     kani::assume(m <= 4 && k <= 4 && n <= 4);
    kani_harness:                 let a = kani::vec::any_vec::<f32>(m * k);
      Verify_Matmul                let b = kani::vec::any_vec::<f32>(k * n);
                                   let result = matmul(&a, &b, m, k, n);
                                   assert!(result.len() == m * n);
                               }
```

### Kani Contract Schema

```yaml
kani_verification:
  harness_dir: "proofs/kani"
  bound: 4  # Default symbolic execution bound
  harnesses:
    - name: Verify_Matmul
      file: proofs/kani/matmul.rs
      bound: 4
      status: verified  # verified | timeout | failed | wip
      unwind: 16
    - name: Verify_Softmax_Range
      file: proofs/kani/softmax.rs
      bound: 8
      status: verified
  timeout_seconds: 300
  solver: cadical  # or: minisat, kissat
```

### Kani-Specific Checks

| Check | What | Severity |
|-------|------|----------|
| CB-1205 | Proof obligations → kani harnesses exist | FAIL |
| CB-1510 | `cargo kani --harness <name>` succeeds | FAIL |
| CB-1511 | No `kani::assume` without matching precondition | WARN |
| CB-1512 | Harness covers all proof obligations | FAIL |
| CB-1513 | Bound >= 4 (meaningful verification) | WARN |

### Kani for Regex Contracts

Kani can bounded-verify regex postconditions — exhaustively checking
that all inputs up to a bound produce regex-matching output:

```rust
#[kani::proof]
fn verify_ticket_id_format() {
    let input: String = kani::any();
    kani::assume(input.len() <= 20);
    kani::assume(input.chars().all(|c| c.is_ascii()));
    if let Ok(result) = parse_ticket_id(&input) {
        let re = Regex::new(r"^(PMAT|GH|EPIC)-\d+$").unwrap();
        assert!(re.is_match(&result));
    }
}
```

### Kani for Refinement Type Contracts

Kani can verify that validated type constructors reject all invalid inputs:

```rust
#[kani::proof]
fn verify_non_empty_vec_constructor() {
    let len: usize = kani::any();
    kani::assume(len <= 100);
    let data: Vec<f32> = kani::vec::any_vec(len);
    match NonEmptyVec::new(data.clone()) {
        Ok(v) => assert!(!v.is_empty()),
        Err(_) => assert!(data.is_empty()),
    }
}
```

And verify type class laws (idempotency, commutativity) by bounded check:

```rust
#[kani::proof]
fn verify_normalize_idempotent() {
    let x: f32 = kani::any();
    kani::assume(x.is_finite());
    let once = normalize(x);
    let twice = normalize(normalize(x));
    assert!((once - twice).abs() < 1e-6);
}
```

---

## Verification Level → Backend Mapping

| Level | Backend | What It Proves | Strength |
|-------|---------|----------------|----------|
| L1 | build.rs | Binding exists, compiles | Weakest |
| L2 | Rust compiler | Type safety, trait impls | Compile-time |
| L3 | `debug_assert!` / `Regex::is_match` | Runtime assertion | Sample inputs |
| L4 | Kani | Exhaustive up to bound | All inputs ≤ bound |
| L5 | Lean | Universal theorem | **All inputs** |

For **regex contracts**: L3 tests the regex at runtime on real inputs,
L4 bounded-verifies it for all inputs ≤ bound, L5 proves language
containment for all inputs universally.

For **refinement types**: L2 ensures the type system enforces the
constructor, L4 bounded-verifies the constructor rejects all invalid
values, L5 proves the refinement predicate is sound.

For **type class laws**: L3 spot-checks with proptest, L4 bounded-verifies
with Kani, L5 proves the laws in Lean for all inputs.

## References

### Lean / Theorem Proving
- Shen et al. (2026). arXiv:2603.18334. VCoT-Bench: LLMs as Theorem Provers for Rust.
- Agarwal et al. (2026). arXiv:2601.18944. NTP4VC: Neural Theorem Proving for Verification.
- SorryDB (2026). arXiv:2603.02668. LLM-assisted Lean sorry elimination.

### Kani / Bounded Model Checking
- Zhu et al. (2025). arXiv:2510.01072. Lessons Learned Verifying the Rust Standard Library.
- Le Blanc & Lam (2024). arXiv:2410.01981. Surveying the Rust Verification Landscape.

### Refinement Types
- Vazou et al. (2014). Liquid Haskell: Refinement types via SMT.
- Swamy et al. (2016). F*: Dependent types for program verification.
- Brady (2013). Idris: Dependent types for systems programming.

### Regex Verification
- Gao et al. (2026). arXiv:2410.15756. Automated Proof Generation for Rust (SAFE).
- Nong et al. (2025). arXiv:2504.21312. Annotating Safety Properties of Unsafe Rust.
