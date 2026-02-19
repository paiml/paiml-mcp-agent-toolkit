# Unified Contract-by-Design: Provable Contracts Extension

**Status**: Active
**Version**: 1.0
**Created**: 2026-02-19

## Overview

Extends the Popperian falsification contract system with proof-aware contracts for formal verification languages (Lean 4, Verus, Kani). Adds `FormalProofVerification` as a new falsification method.

## New FalsificationMethod

```
FormalProofVerification
```

- Walks `.lean` files in the project
- Counts `sorry` occurrences (incomplete proofs)
- Compares against `max_sorry_count` threshold

## New ContractThresholds

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `require_proof_verification` | `bool` | `false` | Opt-in for proof checking |
| `max_sorry_count` | `usize` | `0` | Maximum allowed `sorry` markers |
| `min_theorem_coverage` | `f64` | `0.0` | Minimum ratio of theorems to total defs |

## Behavior

- **Off by default**: `require_proof_verification: false`
- When enabled, adds a claim: "No incomplete proofs (sorry) introduced"
- Blocking when `require_proof_verification` is `true`
- Non-proof projects are unaffected

## FileCategory Extension

- New variant: `LeanSource`
- Recognized by `.lean` extension in `FileEntry::from_path()`

## Formal Verification Scorer Extension

The `FormalVerificationScorer` gains Lean 4 awareness:
- Detects `lakefile.lean` / `lean-toolchain` project indicators
- Counts `theorem`/`lemma`/`sorry` in `.lean` files
- Scores: `(theorems - sorrys) / theorems` ratio
- Additive to existing Miri/Kani/Verus scoring (new `LEAN_POINTS: 3.0`)
