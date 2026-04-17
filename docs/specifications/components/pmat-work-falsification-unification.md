# Work Falsification Unification

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 29

## Problem

`pmat work` uses a bespoke `FalsificationMethod` enum with 24+ variants
(`TdgRegression`, `DifferentialCoverage`, `ManifestIntegrity`, ...) defined
in `src/cli/handlers/work_contract_falsification.rs:1-100`. The
provable-contracts YAML has its own `falsification_tests[]` shape keyed by
equation. Two parallel falsification languages exist; neither can run the
other's tests; neither inherits the other's claims.

Concrete evidence:
- `FalsificationMethod::TdgRegression { threshold: f64 }` runs a pmat-native
  check; cannot reference a YAML `test_id`.
- `falsification_tests:` in YAML describes `{ id, method, expected, tolerance }`
  tuples; pmat cannot invoke them.
- A ticket that binds `rope-kernel-v1/rope` (per Component 27) cannot consume
  the YAML's `falsification_tests[]` because no variant maps to "run test X
  from contract Y."
- `pmat work complete` today checks only `FalsificationMethod` variants,
  skipping YAML-resident tests even for bound tickets.

Result: two sibling falsification ecosystems, neither authoritative. The
ticket's "evidence of falsifiability" is incomplete by construction.

## Goal

One falsification roster per ticket. The enum grows **one** new variant that
routes to provable-contracts YAML. All existing variants stay — this is
additive, not destructive. A ticket bound to `rope-kernel-v1/rope`
automatically has its roster seeded with
`FalsificationMethod::ProvableContract { yaml_path, test_id }` entries, one
per test in the YAML's `falsification_tests[]`.

Completion requires every roster entry to execute, regardless of origin.

## Schema Extension

```rust
pub enum FalsificationMethod {
    // existing 24 variants retained...

    /// Execute a specific falsification test from a provable-contracts YAML.
    /// Seeded automatically by Component 27 binding; ticket owner cannot
    /// manually add or remove entries without unbinding.
    ProvableContract {
        yaml_path: PathBuf,      // "contracts/rope-kernel-v1.yaml"
        equation: String,         // "rope"
        test_id: String,          // id field from falsification_tests[]
        expected: Value,          // structural snapshot for drift detection
    },
}
```

`expected: Value` is a snapshot of the YAML test's `expected` field at bind
time. Runtime compares the current YAML value; divergence raises CB-1621
(mirrors Component 27 CB-1601 SHA drift at the per-test granularity).

## Execution Pipeline

```text
1. ticket.load()
2. for each FalsifiableClaim c:
3.   for each FalsificationMethod m in c.methods:
4.     match m {
5.       ProvableContract { yaml_path, equation, test_id, .. } =>
6.         pv_yaml_loader::run_test(yaml_path, equation, test_id)
7.       TdgRegression { .. } | ...  =>  existing handlers
8.     }
9.   aggregate results → FalsificationLog entry
```

Step 6 invokes the shared YAML loader (`src/services/pv_yaml_loader.rs`,
new in Component 27) and executes the test using
[Component 22 provable-contracts](provable-contracts.md) machinery.
Results normalize to the same `FalsificationResult` struct pmat already
emits.

## Roster Composition

A ticket's roster = manually-authored entries ∪ inherited from bindings:

```rust
pub struct FalsifiableClaim {
    pub id: String,
    pub description: String,
    pub methods: Vec<FalsificationMethod>,  // may include ProvableContract{}
    pub source: ClaimSource,                // NEW — audit who added this
}

pub enum ClaimSource {
    Manual,
    InheritedFromBinding { contract: String, equation: String },
    InheritedFromStack { parent_ticket: String },
}
```

`InheritedFromBinding` entries are **immutable**; `pmat work checkpoint`
regenerates them from the current YAML. Manual entries persist across
regeneration.

## CB-1620..1629: Unification Checks

| Check | Level | Enforces | Catches |
|-------|-------|----------|---------|
| CB-1620 | L1 | For every `ContractBinding`, corresponding `ProvableContract{}` entries exist per YAML test | Binding without inherited falsification |
| CB-1621 | L1 | `expected` snapshot matches current YAML `expected` field | Silent test-expectation drift |
| CB-1622 | L3 | All roster entries executed before completion (no skipped tests) | Partial verification |
| CB-1623 | L3 | No duplicate `(yaml_path, test_id)` across variants (dedup) | Double-counted coverage |
| CB-1624 | L1 | `ProvableContract{}` entries cannot be deleted without `unbind` | Manual scope erosion |
| CB-1625 | L3 | Inherited test failure is fatal; manual-test failure respects `FalsifiableClaim.is_blocker` | Inherited claims can't be weakened |
| CB-1626 | L1 | Referenced `test_id` exists in YAML at completion time | Stale inherited reference |
| CB-1627 | L3 | If bound equation adds a new `falsification_tests[]` entry post-bind, warn (not fail) | Missed coverage on contract evolution |
| CB-1628 | L3 | Every `ProvableContract{}` run emits a log line `{ yaml, test_id, status, duration_ms }` | Silent skips |
| CB-1629 | L4 | L4 tickets cannot complete if any `ProvableContract{}` test timed out | Flaky Kani-adjacent tests |

## CLI Surface

### `pmat work falsify <ID>`

Unified runner. Iterates the roster, calls the right backend per variant,
writes `.pmat-work/<ID>/falsification.log` (JSONL, one line per test).

```bash
pmat work falsify PMAT-530
# [manual] differential_coverage .............. PASS (0.8s)
# [rope-kernel-v1/rope] rope_periodicity_test . PASS (1.2s)
# [rope-kernel-v1/rope] rope_linearity_test ... FAIL (0.3s) — see log
# 2/3 passed; ticket cannot complete (CB-1622)
```

### `pmat work falsify <ID> --refresh`

Re-reads bound YAMLs, re-seeds `ProvableContract{}` entries. Warns on
divergence before overwriting. Writes audit entry.

### `pmat work complete <ID>`

Pre-flight invokes `falsify` internally; completion fails fast if any
entry failed or was skipped.

## Storage

```
.pmat-work/
└── PMAT-530/
    ├── contract.json              # FalsifiableClaim[] with expanded roster
    ├── falsification.log          # JSONL, one line per test run
    └── falsification-digest.json  # summary: total, passed, failed, duration
```

`falsification.log` format (JSONL):

```json
{"ts":"2026-04-17T12:00:00Z","source":"manual","method":"TdgRegression","status":"pass","duration_ms":820}
{"ts":"2026-04-17T12:00:01Z","source":"inherited","yaml":"rope-kernel-v1.yaml","equation":"rope","test_id":"rope_periodicity_test","status":"pass","duration_ms":1200}
```

Human-grep-friendly; machine-parseable; CB-1628 emits per line.

## Handling Claim Source Priority

When a manual claim and an inherited claim overlap (same underlying
assertion), the inherited one takes precedence — per Liskov-Wing,
inherited postconditions cannot be weakened. `pmat work falsify` detects
overlap by test fingerprint (test name + expected value hash) and emits
a warning: "manual claim X is subsumed by inherited Y, consider removing."

## Interaction with Other Specs

- [Component 22 provable-contracts.md](provable-contracts.md) supplies the
  YAML execution backend.
- [Component 27 pmat-work-contract-binding.md](pmat-work-contract-binding.md)
  seeds the inherited entries.
- [Component 28 pmat-work-verification-ladder.md](pmat-work-verification-ladder.md)
  L3 gate consumes the digest; L4/L5 gates cannot pass with roster
  failures.
- [Component 14 work-management.md](work-management.md)
  §Chain-of-Thought Audit Trail logs claim additions with
  `ClaimSource`; Component 31 extends this further.

## Migration

- Existing tickets keep their manual rosters untouched.
- `pmat work migrate --seed-inherited-falsification` reads each ticket's
  `implements[]` and appends `ProvableContract{}` entries for any that are
  missing. Writes `<ID>/contract.json.new` for review.
- CB-1620 enters warn mode for 30 days, then fail.

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/work_contract_falsification.rs` | Add `FalsificationMethod::ProvableContract` variant |
| `src/cli/handlers/work_falsify_runner.rs` | NEW — unified iterator dispatch |
| `src/services/pv_yaml_loader.rs` | Shared YAML test runner (introduced in Component 27) |
| `src/cli/handlers/comply_handlers/check_handlers/check_falsification_unification.rs` | NEW — CB-1620..1629 |
| `src/cli/handlers/work_handlers/core_handlers/handlers.rs` | Wire `falsify` subcommand |

## References

### Internal
- [work-management.md](work-management.md) §Falsifiable Claims, §Chain-of-Thought Audit Trail
- [provable-contracts.md](provable-contracts.md) §Falsification Test Format
- [pmat-work-contract-binding.md](pmat-work-contract-binding.md) inheritance source
- [pmat-work-verification-ladder.md](pmat-work-verification-ladder.md) L3 consumer

### Foundational
- Popper, K. (1959). *The Logic of Scientific Discovery*. Falsifiability as demarcation — every claim must be refutable.
- Meyer, B. (1988). *Object-Oriented Software Construction*. §11.10 inheritance of assertions.

### arXiv
- Li et al. (2025). arXiv:2510.12047. LLMs and formal contracts — empirical evidence that imported test suites (vs. LLM-generated) produce substantially fewer hallucinated claims.
- Meng et al. (2024). arXiv:2411.04937. Differential testing under contract drift — motivates CB-1621 expected-snapshot.
