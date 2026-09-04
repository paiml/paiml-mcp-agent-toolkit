# Work Contract Binding

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 27

## Problem

`pmat work` has lived on an island. Its per-ticket `contract.json` defines
`require`/`ensure`/`invariant` clauses with a bespoke `FalsificationMethod`
enum and its own JSON shape. The provable-contracts ecosystem has a parallel
world: YAML contracts with `equations`, `preconditions`, `postconditions`,
`invariants`, `proof_obligations`, `falsification_tests`, `kani_harnesses`,
and `lean_theorem`. Both exist; neither cites the other.

Concrete evidence today:
- `WorkContract { verification_level: String, references: WorkReferences, ... }`
  in `src/cli/handlers/work_contract_core.rs:1-94` — the fields exist but carry
  no executable link to any YAML contract.
- `references.arxiv: Vec<String>` is informational only; the work contract
  never opens a YAML file or checks the cited spec.
- Contract clause inheritance (`ContractClause::source: ClauseSource::Inherited`)
  stacks only from other work contracts, not from provable-contracts YAML.

The impedance mismatch means a ticket that modifies a provable-contracts-bound
kernel can mark itself "complete" without the YAML contract's obligations
feeding into its falsification checks. Commit-level enforcement
([Component 25](commit-level-contract-enforcement.md)) Phase 1 already proposes
bi-directional generation; this sub-spec describes the **binding** half.

## Goal

Every `pmat work` ticket that touches contracted code MUST declare which
provable-contract equation it implements. The declared binding is load-bearing:
- Ticket preconditions inherit `equation.preconditions`
- Ticket postconditions inherit `equation.postconditions`
- Completion re-runs `falsification_tests` from the YAML
- Contract drift between ticket claims and YAML raises CB-1601

## Schema Extension

Add `implements` to `WorkContract`:

```rust
pub struct ContractBinding {
    pub contract: String,     // "rope-kernel-v1"
    pub equation: String,     // "rope"
    pub file: PathBuf,        // "contracts/rope-kernel-v1.yaml"
    pub sha: String,          // SHA-256 of the YAML at bind time (drift detection)
    pub bound_at: DateTime<Utc>,
}

pub struct WorkContract {
    // existing fields...
    pub implements: Vec<ContractBinding>,  // NEW — zero, one, or many
}
```

`implements` is a `Vec` because a work item may span multiple contracted
equations (e.g., a cross-kernel refactor). Empty vec = the ticket does not
modify contracted code (linting is a sufficient gate).

## CLI Surface

### `pmat work start <ID> --implements <contract>/<equation>`

Repeatable. Each flag appends a `ContractBinding`:

```bash
pmat work start PMAT-530 \
  --implements rope-kernel-v1/rope \
  --implements softmax-v2/softmax
```

Since #1186 (PMAT-670) the same flag is accepted on `pmat work edit`, so a
ticket that is already InProgress can be bound without restarting it; a bound
ticket's verification claim is lifted from L1 to L2 unless `--level` says
otherwise. Resolution now also enforces CB-1607: the named equation must be
declared under `equations:` in the YAML, and the refusal lists the declared ones.

Behavior at start:

1. **Resolve.** Locate `contracts/rope-kernel-v1.yaml`. Search order:
   project root, `provable-contracts/contracts/` sibling, then
   `$PMAT_CONTRACTS_PATH`. Fail-fast if missing.
2. **Parse.** Extract `equations.rope` — refused if absent (CB-1607, #1186).
3. **Inherit.** For each `preconditions[i]`, emit a `require` clause with
   `source: ClauseSource::InheritedFromPv { binding }`. Same for
   `postconditions` → `ensure`, `invariants` → `invariant`.
4. **Freeze.** Record SHA-256 of the YAML bytes in `ContractBinding::sha`.
5. **Persist.** Save `.pmat-work/<ID>/contract.json` with `implements`
   populated, inherited clauses visible but flagged non-editable.

### `pmat work bind <ID> <contract>/<equation>`

For retroactive binding on existing in-progress tickets. Re-evaluates require
clauses immediately and emits a warning if any inherited precondition fails
for the current code state.

### `pmat work unbind <ID> <contract>/<equation>`

Requires `--reason` and writes to
`.pmat-work/ledger/unbind-<timestamp>.json`. Unbinding a ticket drops its
inherited clauses but preserves manually-added clauses. CB-1602 audits all
unbinds and warns if a ticket unbinds without a linked DEBT follow-up.

## CB-1600..1609: Binding Enforcement Checks

| Check | Level | Enforces | Catches |
|-------|-------|----------|---------|
| CB-1600 | L1 | Tickets modifying contracted files MUST have `implements` entry | Orphan work on bound kernels |
| CB-1601 | L1 | `ContractBinding::sha` matches current YAML sha | Contract drift mid-ticket |
| CB-1602 | L1 | `unbind` without DEBT ticket reference | Silent contract abandonment |
| CB-1603 | L3 | Inherited require clauses present in contract.json | Inheritance pipeline bug |
| CB-1604 | L3 | Ticket cannot override inherited postcondition via weaker threshold | Postcondition downgrade |
| CB-1605 | L4 | If YAML has `kani_harnesses[]`, completion checks they pass | Unverified bound completion |
| CB-1606 | L5 | If YAML `lean_theorem.status != "proved"`, ticket must link a `BLOCK-ON-PROOF` follow-up | Stranded completions |
| CB-1607 | L3 | `--implements` target exists in referenced YAML | Fat-fingered equation names |
| CB-1608 | L1 | Cross-binding consistency: multi-bind tickets satisfy all bindings | Partial completion masquerading as done |
| CB-1609 | L1 | `implements[].file` is tracked in git | Transient/scratch YAML |

CB-1600 reads the git diff of staged changes, intersects it with
`binding.yaml` (from [provable-contracts.md](provable-contracts.md) CB-1208
binding map), and blocks commit if the intersection is non-empty and the
active work item has no `implements` entry covering those bindings.

## Detection of Contracted Files

Leveraging [Component 19 pv-compatibility.md](pv-compatibility.md) binding
index:

```
.pmat/binding-index.json          # from CB-1208
  └── function_bindings: [
        { file: "src/rope.rs", line: 42, contract: "rope-kernel-v1",
          equation: "rope", verification_level: "L3" },
        ...
      ]
```

The pre-commit path reads this cache (O(1), <5ms per
[commit-level-contract-enforcement.md §O(1) Firm Requirement]) and answers:
"for the staged files, which bindings are in scope?" If the set is non-empty,
CB-1600 enforces `implements` presence.

## Storage Layout

```
.pmat-work/
└── PMAT-530/
    ├── contract.json              # includes "implements": [...]
    ├── inherited-clauses.json     # derived view for quick audit
    └── binding-manifest.json      # SHA snapshots per binding
```

`inherited-clauses.json` is **generated**; never hand-edited. Regenerated on
every `pmat work start`, `bind`, `unbind`, and `checkpoint`. If
`contract.json` has N `implements` entries, the file records each
(binding, clause_id, source_yaml_line, rust_assertion) tuple for CB-1603
audit.

## Falsification Inheritance

When the ticket declares `implements rope-kernel-v1/rope`, the YAML's
`falsification_tests[]` feed into the ticket's falsification roster as
`FalsificationMethod::ProvableContract { yaml, test_id }` entries (spec
[Component 29](pmat-work-falsification-unification.md)). The ticket cannot
complete without these executing.

## Assume-Guarantee Chain

Dardik & Kang (2025, arXiv:2509.06250): a stage's `ensure` forms the next
stage's `require`. In the binding model, this generalizes to:
a ticket's postcondition on equation E is the precondition consumed by any
downstream ticket that also declares `implements .../E`. Binding chains
compose — and regressions are caught by CB-1601 SHA diff.

## Migration Path

Existing tickets without `implements` are valid. CB-1600 only fires when
staged files hit the binding index. An `pmat work migrate --backfill-bindings`
command infers `implements` for in-progress tickets by git-log-walking their
modified files against `binding.yaml`, writes a candidate `contract.json.new`
for review, and never overwrites without `--yes`.

## Interaction with Other Specs

- [Component 25 commit-level-contract-enforcement.md](commit-level-contract-enforcement.md)
  Phase 1 generates `contracts/work/<ID>.yaml` *from* the bound ticket. This
  spec describes the inverse direction — tickets importing from contracts.
  Both pipelines must agree on the SHA recorded at bind time.
- [Component 28 pmat-work-verification-ladder.md](pmat-work-verification-ladder.md)
  consumes `implements[].verification_level` to set the ticket's target level.
- [Component 22 provable-contracts.md](provable-contracts.md) CB-1208 supplies
  the binding index that CB-1600 depends on.

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/work_contract_core.rs` | Extend `WorkContract` with `implements: Vec<ContractBinding>` |
| `src/cli/handlers/work_contract_binding.rs` | NEW — resolve/parse/inherit pipeline |
| `src/cli/handlers/work_handlers/core_handlers/handlers.rs` | `start`/`bind`/`unbind` dispatch |
| `src/cli/handlers/comply_handlers/check_handlers/check_binding_scope.rs` | NEW — CB-1600..1609 |
| `src/services/pv_yaml_loader.rs` | NEW — shared YAML parser consumed here and by Component 29/30 |

## References

### Internal
- [work-management.md](work-management.md) §Verification Level Tracking, §Chain-of-Thought Audit Trail
- [provable-contracts.md](provable-contracts.md) CB-1208 binding existence
- [pv-compatibility.md](pv-compatibility.md) binding index format
- [commit-level-contract-enforcement.md](commit-level-contract-enforcement.md) Phase 1 — work → YAML direction

### Foundational
- Meyer, B. (1988). *Object-Oriented Software Construction*. Inheritance of contracts (§11.9, §11.10).
- Liskov, B. & Wing, J. (1994). *A Behavioral Notion of Subtyping*. Precondition weakening, postcondition strengthening rules that govern inherited clauses.

### arXiv
- Dardik & Kang (2025). arXiv:2509.06250. Assume-guarantee compositional inference.
- Li et al. (2025). arXiv:2510.12047. LLMs and formal contracts — motivates imported clauses as ground truth.
