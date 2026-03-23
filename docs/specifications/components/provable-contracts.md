# Provable Contracts Integration

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 22

## Problem Statement

Provable contracts (YAML declarations) are Generation 1: separate from code,
drift-prone, easily gamed. `pv lint` Gate 4 catches missing test references,
but cannot verify that contracts match the actual implementation semantics.

The sovereign stack has 98 contracts across 4 repos (trueno 27, entrenar 11,
realizar 11, aprender 49) with 175 test references. But the contracts live
in YAML files separate from the functions they specify — a fundamental
architectural weakness.

## Contract Standard: Escape-Proof Pipeline

ONE type of contract. Six stages. Skip one → compile error.

```
Equation (YAML) → Lean 4 proof → pv lint → build.rs codegen → #[contract] macro → tests
```

- **YAML**: equation + preconditions + postconditions + lean_theorem
- **Lean 4**: proves mathematical properties (no sorry allowed)
- **build.rs**: generates `debug_assert!()` from YAML preconditions
- **#[contract] macro**: inserts assertions, checks binding env var
- **Zero cost in release** — all `debug_assert!()`, stripped by compiler
- **Cannot escape** — missing stage = compile_error!

Full spec: `../provable-contracts/docs/specifications/sub/escape-proof-enforcement.md`

## Generation 1: Current State (YAML)

```yaml
# contracts/gemv-kernel-v1.yaml
equations:
  gemv:
    formula: "c[j] += Σ a[k] * B[k*N + j]"
falsification_tests:
  - id: F-GEMV-001
    test: "test_gemv_basic"
    if_fails: "SIMD and scalar paths diverge"
```

Enforced by:
- `pv lint` Gate 1-3: YAML structure, audit, score
- `pv lint` Gate 4: test references resolve to `fn test_*` in src/
- `pmat comply` CB-1201: PV Lint pass rate
- `pmat comply` CB-1202: contract coverage (critical keywords)
- `pmat score` PV Lint sub-score: weighted gates + fulfillment

### Limitations
- Contract file can be edited without touching the implementation
- No compile-time guarantee that preconditions are checked
- `test:` references are string matches, not compiler-verified
- `old()` pre-state capture requires manual test setup

## Integration with pv lint

**Gate 6: Annotation Verification** — verify that functions with YAML
contracts also have `#[core::contracts::requires]`/`#[ensures]` in source.

```
pv lint Gate 6: annotate
  For each equation in contracts/*.yaml:
    1. Find the implementing function via .pv.toml binding
    2. Check source for #[core::contracts::requires/ensures]
    3. ERROR if contracted function has no annotation
```

### pmat comply CB-1203: Annotation Coverage

Checks that source files with contract-related code have
`core::contracts` annotations. Currently advisory (Gen 2 adoption).

## Implementation Plan

### Phase 1: Add `#![feature(contracts)]` to sovereign stack crates

Annotate the top 10 critical functions per repo with
`#[core::contracts::requires]`/`#[core::contracts::ensures]`.
No external dependencies needed.

### Phase 2: `pv lint` Gate 6

Verify YAML equations have matching `core::contracts` annotations.

### Phase 3: CI enforcement

`RUSTFLAGS="-Z contract-checks=yes" cargo test` in CI to run
contracts as runtime checks during testing only.

## Key Files

| File | Status | Purpose |
|------|--------|---------|
| provable-contracts `gates.rs` | Exists | Gates 1-4 implemented |
| provable-contracts `gates.rs` | Planned | Gate 6 annotation check |
| pmat `check.rs` | Exists | CB-1201, CB-1202 |
| pmat `check.rs` | Planned | CB-1203 annotation coverage |
| pmat `score_handler.rs` | Exists | PV Lint sub-score |

## References

- `contracts` crate: https://crates.io/crates/contracts
- Flux (PLDI 2023): https://github.com/flux-rs/flux
- Creusot: https://github.com/creusot-rs/creusot
- `core::contracts` RFC: rust-lang/rust#128045
- Eiffel DbC: Meyer, B. (1992) "Applying Design by Contract"
- Scoring convergence: [scoring-convergence.md](scoring-convergence.md) §10
