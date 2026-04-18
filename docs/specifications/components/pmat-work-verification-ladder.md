# Work Verification Ladder

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 28

## Problem

`WorkContract { verification_level: String, ... }` today stores an opaque
string. Nothing in the codebase parses it, compares it, or enforces a
completion gate against it. Compare with
[work-management.md §Verification Level Tracking](work-management.md) which
specifies L0–L5 with strict per-level requirements (L4 Kani-verified, L5
Lean-proved zero-`sorry`). The spec is provable-contracts-aware; the
implementation is not.

Concrete evidence:
- `src/cli/handlers/work_contract_core.rs:~38` — `verification_level: String`
- No code path reads `verification_level` to decide whether a ticket can close.
- `pmat work status <ID>` prints the string verbatim; does not parse it.
- The CLI accepts any value, including misspellings (`"L3 "`, `"l4"`,
  `"strong"`) that silently downgrade enforcement.
- Phase 1 of [commit-level-contract-enforcement.md](commit-level-contract-enforcement.md)
  generates YAML that *assumes* a concrete level; drift is invisible.

A ticket today can claim "verification_level: L5" without linking a Lean
theorem, without a Kani harness, and without any falsification evidence. The
field is decorative — the exact opposite of a load-bearing invariant.

## Goal

Promote `verification_level` from string decoration to a typed, gated invariant.
Each level carries concrete machine-checkable requirements; ticket completion
cannot exceed what evidence supports.

- L0: Documentation review only (no executable check).
- L1: `debug_assert!` contract macros compile and run during `cargo test`.
- L2: `#[contract]` attribute bound; trait-based equations instantiated.
- L3: Bound equation's `falsification_tests[]` execute and pass in the
  ticket's working tree.
- L4: Bound equation's `kani_harnesses[]` verified (Kani exits 0, no counterexamples).
- L5: Bound equation's `lean_theorem` status = `proved`, zero `sorry` in proof.

Completion gates are hard: claim L4 without a passing Kani harness? Ticket
cannot close.

## Schema

Replace the string with a typed enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VerificationLevel {
    L0,  // Review
    L1,  // Runtime asserts
    L2,  // Trait/compile-time
    L3,  // Falsifiable claims
    L4,  // Kani bounded proof
    L5,  // Lean theorem, zero sorry
}

pub struct WorkContract {
    // ...
    pub verification_level: VerificationLevel,  // was String
    pub target_level: VerificationLevel,         // NEW — what the ticket aims at
}
```

`Ord` matters: `L3 > L1`, so the completion gate can compare "evidence
supports level N" vs. "ticket claims level M" and reject M > N.

Migration: legacy `verification_level: "L3"` strings map via
`FromStr`; unknown strings fail-fast at load time (not silent downgrade).

## Per-Level Completion Gates

| Level | Gate | Evidence Source |
|-------|------|-----------------|
| L0 | `--reason` recorded, reviewer SHA logged | `.pmat-work/<ID>/review.json` |
| L1 | `cargo test --lib` green, contract macro runs | Test process exit 0; `contract_pre_*!` / `contract_post_*!` expanded in at least one test |
| L2 | `cargo check` green with `#[contract(...)]` attribute resolved | Compilation includes the bound trait impl |
| L3 | All `falsification_tests[]` from bound equation pass | Per-test JSON lines in `.pmat-work/<ID>/falsification.log` |
| L4 | Kani returns `VERIFICATION:- SUCCESSFUL` for each `kani_harnesses[]` | `kani --output-format json` artifact archived |
| L5 | Lean proof with `status: proved`, `#check` / `#print` shows zero `sorry` | Lean artifact + SHA of the `.lean` file |

## CB-1610..1619: Ladder Enforcement

| Check | Level | Enforces | Catches |
|-------|-------|----------|---------|
| CB-1610 | L1 | `target_level` parses to a known variant | Typo'd levels (`"L3 "`, `"strong"`) |
| CB-1611 | L1 | Target level cannot exceed bound equation's max attainable level | Claiming L4 without `kani_harnesses:` in YAML |
| CB-1612 | L3 | L1 completion: `cargo test --lib` passes in ticket workspace | False-green claims |
| CB-1613 | L3 | L3 completion: falsification log present and all pass | Stub/missing falsification suite |
| CB-1614 | L4 | L4 completion: Kani artifact present, exit 0 | Skipped formal verification |
| CB-1615 | L4 | Kani harness hash in ticket == harness hash in YAML at bind time | Harness drift post-bind |
| CB-1616 | L5 | L5 completion: Lean artifact `sorry` count = 0 | Stub theorems |
| CB-1617 | L3 | No downgrade without `--reason` | Silent scope reduction |
| CB-1618 | L1 | Level monotonicity across checkpoints — ticket cannot go L3 → L1 → L3 without audit | Scope laundering |
| CB-1619 | L3 | On completion, achieved level == target level (no "close as L2 when targeting L4") | Quiet giveups |

CB-1614 and CB-1616 rely on Component 24 verification-backends producing
machine-readable artifacts; this spec does not re-implement Kani/Lean
invocation, it only *gates on their output*.

## CLI Surface

### `pmat work start <ID> --target-level <L0..L5>`

Sets `target_level`. Must be ≤ max attainable level of all bound equations
(CB-1611). Defaults: `L3` for bound tickets, `L1` for unbound.

### `pmat work verify <ID>`

Runs the ladder: L1 test → L3 falsification → L4 Kani → L5 Lean. Stops at
the target level. Writes `.pmat-work/<ID>/verification-report.json` with
the achieved level.

```bash
pmat work verify PMAT-530
# L1: cargo test ............ PASS (42 tests, 0 failed)
# L2: cargo check .......... PASS
# L3: falsification ........ PASS (7/7 tests)
# L4: kani ................. PASS (3/3 harnesses)
# Achieved: L4 (target was L4) — ticket may close
```

### `pmat work complete <ID>`

Refuses to close if `achieved_level < target_level`. Override:
`--force-downgrade --reason <text>` downgrades `target_level` with audit
log entry in `.pmat-work/ledger/downgrades.json` (CB-1617).

## Max-Attainable Level Computation

For each `ContractBinding` (Component 27), the max attainable level is:

```text
max_level(equation) = L5 if lean_theorem.status == "proved"
                      L4 elif kani_harnesses is non-empty
                      L3 elif falsification_tests is non-empty
                      L2 elif the generated contract macros exist
                      L1 always  (debug_assert is always possible)
                      L0 fallback
```

`max_level(ticket) = min(max_level(e) for e in implements)`. A ticket
cannot claim higher than its weakest binding — the weakest link dominates,
matching Liskov-Wing precondition-weakening rules.

## Storage

```
.pmat-work/
└── PMAT-530/
    ├── contract.json                  # target_level, verification_level
    ├── verification-report.json       # per-level results
    ├── falsification.log              # L3 evidence
    ├── kani-report.json               # L4 evidence
    └── lean-proof.status.json         # L5 evidence (by ref to .lean file)
```

Evidence files are immutable once the level is locked; a rerun creates a
new file with timestamp suffix. The report points at the latest.

## Interaction with Other Specs

- [Component 27 pmat-work-contract-binding.md](pmat-work-contract-binding.md)
  supplies `implements[]` — without a binding, max attainable level is L1.
- [Component 24 verification-backends.md](verification-backends.md) owns the
  Kani/Lean invocation contracts. This spec only consumes their output.
- [Component 25 commit-level-contract-enforcement.md](commit-level-contract-enforcement.md)
  §O(1) Firm Requirement caches verification status; CB-1612 reads this
  cache at pre-commit time.
- [Component 22 provable-contracts.md](provable-contracts.md) defines what
  lives in `falsification_tests`, `kani_harnesses`, `lean_theorem`.

## Migration

1. `pmat work migrate --coerce-level-strings` parses existing
   `verification_level` strings; stores unparseable as L0 with a
   `MIGRATION-LEVEL-UNKNOWN` note in the ticket history.
2. `pmat work verify --all --in-progress` re-runs the ladder across all
   open tickets; tickets whose claimed level exceeds evidence are flagged
   for review but not auto-downgraded.
3. After grace period (30 days), CB-1612..1619 move from warn → error.

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/work_contract_core.rs` | Replace `verification_level: String` with `VerificationLevel` enum |
| `src/cli/handlers/work_verification_ladder.rs` | NEW — per-level gate implementations |
| `src/cli/handlers/work_handlers/core_handlers/handlers.rs` | Wire `verify` / `complete` dispatch |
| `src/cli/handlers/comply_handlers/check_handlers/check_verification_ladder.rs` | NEW — CB-1610..1619 |
| `src/services/verification_backends.rs` | Kani/Lean artifact consumers (from Component 24) |

## References

### Internal
- [work-management.md](work-management.md) §Verification Level Tracking
- [provable-contracts.md](provable-contracts.md) CB-1201..1206 verification levels
- [verification-backends.md](verification-backends.md) Kani/Lean artifact formats
- [pmat-work-contract-binding.md](pmat-work-contract-binding.md) for `implements[]` source

### Foundational
- Popper, K. (1959). *The Logic of Scientific Discovery*. Falsifiability principle.
- Liskov, B. & Wing, J. (1994). *A Behavioral Notion of Subtyping*. Precondition weakening dominates in multi-binding tickets.
- Leino, K.R.M. (2010). *Dafny: An Automatic Program Verifier for Functional Correctness*. Level-stratified verification as a model.

### arXiv
- Kirchner, F. et al. (2015). Frama-C and modular formal verification — staged proof obligations as the basis for ladder climbing.
- Clarke, E. et al. (2018). *Model Checking* (2nd ed). CEGAR loop motivates L4 gate: evidence is binary, not approximate.
