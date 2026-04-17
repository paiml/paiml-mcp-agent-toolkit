# Kani Proof Harnesses

This directory documents the Kani model-checking harnesses that live inline in
the PMAT source tree. Each harness is guarded by `#[cfg(kani)]` so it compiles
only under Kani and is invisible to normal `cargo build` / `cargo check`.

Related issue: [GH-276](https://github.com/paiml/paiml-mcp-agent-toolkit/issues/276)
(Rust Project Score — Formal Verification category).

## Installing Kani

Kani is Amazon's bit-precise model checker for Rust. It is not a runtime
dependency; it installs a separate toolchain and invokes CBMC under the hood.

```bash
cargo install --locked kani-verifier
cargo kani setup
```

`cargo kani setup` downloads the pinned CBMC + Kani runtime the first time it
runs (a few hundred MB). Once installed, `cargo kani --version` should work.

## Running the Proofs

From the repo root:

```bash
# Run every harness in the workspace
cargo kani

# Run a single harness by name (substring match on the function path)
cargo kani --harness from_score_total
cargo kani --harness auto_fail_iff_below_90
cargo kani --harness kani_from_score_total

# With more output
cargo kani --verbose
```

Expected output: `VERIFICATION:- SUCCESSFUL` for every harness. If Kani reports
a failed assertion, it prints a concrete counter-example (inputs that break the
property) — this is the point of the tool.

## Properties Proved

All harnesses target **pure classification / scoring functions** — no I/O, no
heap allocation beyond empty `Vec::new()`, no global state.

### `src/services/file_health_types.rs` — `HealthGrade::from_score(u8)`

Proved by iterating every one of the 256 possible `u8` values:

- **Totality.** `from_score` returns one of `{A, B, C, D, E, F}` for every
  `u8 ∈ 0..=255`. It never panics.
- **`is_passing` ↔ score ≥ 70.** For every `score ∈ 0..=100`, `from_score(score).is_passing()`
  is `true` iff `score ≥ 70`. This locks the passing-threshold contract.
- **High-score band.** Any `score ∈ 90..=100` yields `HealthGrade::A` and is passing.

Harness functions: `kani_from_score_total`, `kani_is_passing_iff_score_ge_70`,
`kani_high_score_yields_high_grade`.

### `src/services/normalized_score.rs` — `Grade::from_score(f64)`

- **Totality (bounded).** For any finite `score ∈ [-1000, 1000]`, `from_score`
  returns one of `{A, B, C, D, F}`. (NaN/Inf excluded by `assume(is_finite)` —
  the public contract is about finite scores.)
- **A-band consistency.** Any finite `score ∈ [90, 100]` classifies as `Grade::A`.
- **F-band consistency.** Any finite `score ∈ [-1000, 60)` classifies as `Grade::F`.

Harness functions (inside `kani_proofs` submodule): `from_score_total_in_bounds`,
`min_score_consistent_at_A_boundary`, `below_D_boundary_is_F`.

### `src/services/infra_score/models.rs` — `InfraGrade` auto-fail semantics

The infra-score spec defines a hard-cutoff invariant: **any score below 90 is
an auto-fail**. This is the single most important safety property of the
infra-score system.

- **Auto-fail cutoff.** For every finite `score ∈ [-1000, 1000]`,
  `InfraGrade::from_score(score).is_auto_fail()` is `true` iff `score < 90.0`.
- **Totality.** `from_score` returns one of `{APlus, A, B, C, D, F}` for every
  finite bounded input.

Harness functions (inside `kani_proofs` submodule): `auto_fail_iff_below_90`,
`from_score_total`.

### `src/services/repo_score/models.rs` — `Grade` + `CategoryScore`

- **Totality of `Grade::from_score`.** Returns one of the 8 grade variants
  for every finite bounded `f64` input.
- **A+ band.** Any finite `score ≥ 95` classifies as `Grade::APlus`.
- **`CategoryScore::new` percentage invariant.** When `max_score > 0`, the
  computed `percentage` equals exactly `score / max_score * 100.0`, and the
  `ScoreStatus` enum correctly reflects the 90 / 70 thresholds.
- **Division-by-zero guard.** When `max_score == 0.0`, `percentage` is `0.0`
  and `status` is `Fail`, with no panic.

Harness functions (inside `kani_proofs` submodule): `grade_from_score_total`,
`score_ge_95_is_a_plus`, `category_score_percentage_invariant`,
`category_score_zero_max_is_safe`.

## Why No CI Job?

Kani verification takes minutes per harness and requires the CBMC toolchain
(~300 MB) plus a pinned nightly Rust. Running it in every CI invocation is not
worth the cost. Instead:

- Harnesses live inline and compile under normal `cargo check` (via
  `#[cfg(kani)]`), so they cannot rot silently.
- Developers run `cargo kani` locally when touching a scoring function.
- A future CI job could run Kani on a schedule (e.g. weekly) once the proof set
  is larger.

## Adding New Proofs

Good Kani candidates in this codebase:

1. Pure classification functions (score → grade, threshold → severity).
2. Integer / bit arithmetic where overflow matters.
3. Small fixed-size parsers (e.g. a `u8` version-byte decoder).

Avoid:

- Anything that reads files or walks directories.
- String parsing beyond a few bytes (Kani is slow on dynamic strings).
- Hashing, regex, or anything that transitively calls `std::collections`
  with non-trivial state.

Bound every symbolic input with `kani::assume(...)` so the proof terminates.
For `f64`, always `assume(x.is_finite())` and a numeric range like
`assume((-1000.0..=1000.0).contains(&x))`.
