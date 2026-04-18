# Work Compile-Time Codegen

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 30

## Problem

`pv codegen` today reads provable-contracts YAML and emits
`contract_pre_<eq>!` / `contract_post_<eq>!` macros backed by
`debug_assert!` plus a `#[contract("name", equation = "eq")]` attribute that
wraps functions. That pipeline is YAML → Rust.

`pmat work` has the inverse direction via
[commit-level-contract-enforcement.md](commit-level-contract-enforcement.md)
Phase 1: ticket → `contracts/work/<ID>.yaml`. But nothing emits Rust
*from the ticket side*. The ticket's contract clauses (`require`, `ensure`,
`invariant`) exist only in `.pmat-work/<ID>/contract.json`; no macro makes
them executable, no attribute links them to their Rust implementation.

Concrete evidence:
- `ContractClause { kind: ClauseKind::Require, body: String, ... }` in
  `work_contract_dbc.rs:4-48` — body is prose, not executable.
- No `#[pmat_work_contract]` attribute exists. The ticket's clauses cannot
  be enforced at call sites.
- `pmat work stack-top` knows the current ticket but has no way to inject
  per-function preconditions from that ticket's contract.

Net: ticket invariants are inert documentation. A future commit that
violates an `ensure` clause compiles cleanly and passes tests.

## Goal

Extend `pv codegen` (or a new `pmat work codegen`) to emit Rust from the
**ticket side**, producing:

1. A `#[pmat_work_contract(id = "PMAT-530")]` attribute macro that binds a
   function to its ticket. Attribute body emits `debug_assert!` wrappers
   around ticket-declared preconditions/postconditions — identical surface
   to `#[contract(...)]` but sourced from `contract.json`, not YAML.
2. A `contracts/work/<ID>.rs` generated module with one `pub mod` per
   contracted function, containing the assertion macros.
3. A build-script integration that ensures every staged function that is
   tagged with `#[pmat_work_contract]` has a corresponding generated
   module entry, else `cargo check` fails.

The ticket's contract clauses graduate from JSON prose to executable
assertions compiled into debug builds.

## Schema

Each `ContractClause` gains an `expr` field (Rust expression body) and a
`binds_to` field (function path):

```rust
pub struct ContractClause {
    pub id: String,
    pub kind: ClauseKind,
    pub body: String,      // prose description
    pub expr: Option<String>,    // NEW — Rust expression, e.g. "x > 0 && x < limit"
    pub binds_to: Option<String>,// NEW — "crate::rope::encode" fully-qualified
    pub source: ClauseSource,
}
```

Clauses without `expr` stay as prose (L0 review only). Clauses with `expr`
get codegen and compile-check treatment.

## Generation Target

```rust
// contracts/work/PMAT-530.rs  — GENERATED, do not hand-edit
// Source: .pmat-work/PMAT-530/contract.json
// Binding: rope-kernel-v1/rope  (via Component 27)

#[cfg(debug_assertions)]
pub mod pmat_530 {
    #[macro_export]
    macro_rules! pmat_work_contract_pre_rope_encode {
        ($x: expr, $limit: expr) => {{
            debug_assert!($x > 0 && $x < $limit,
                "[PMAT-530 require R1] {}", stringify!($x > 0 && $x < $limit));
        }};
    }

    #[macro_export]
    macro_rules! pmat_work_contract_post_rope_encode {
        ($result: expr) => {{
            debug_assert!($result.is_finite(),
                "[PMAT-530 ensure E1] result is finite");
        }};
    }
}
```

The generated module name (`pmat_530`) is derived from the ticket ID with
safe normalization. Macros are `#[macro_export]` so downstream crates in
the workspace pick them up.

## `#[pmat_work_contract]` Attribute

Procedural attribute macro wrapping the target function:

```rust
#[pmat_work_contract(id = "PMAT-530", require = "R1", ensure = "E1")]
pub fn encode(x: i64, limit: i64) -> f64 {
    // body
}
```

Expands to:

```rust
pub fn encode(x: i64, limit: i64) -> f64 {
    pmat_work_contract_pre_rope_encode!(x, limit);
    let __pmat_result = (|| {
        // original body
    })();
    pmat_work_contract_post_rope_encode!(__pmat_result);
    __pmat_result
}
```

Attribute resolution fails compilation if:
- The referenced ticket ID has no generated module (CB-1631).
- The named clause (`R1`, `E1`) doesn't exist in that module (CB-1632).

## CB-1630..1639: Codegen Enforcement

| Check | Level | Enforces | Catches |
|-------|-------|----------|---------|
| CB-1630 | L1 | `pmat work codegen <ID>` succeeds without diagnostic errors | Malformed `expr` fields |
| CB-1631 | L2 | Every `#[pmat_work_contract(id = X, ...)]` has a generated `contracts/work/X.rs` | Attribute referencing a closed/purged ticket |
| CB-1632 | L2 | Every `require = "Y"` / `ensure = "Y"` argument matches a clause in the ticket | Typo'd clause ID |
| CB-1633 | L1 | Generated file SHA matches `contract.json` + binding YAMLs | Hand-edited generated file |
| CB-1634 | L3 | Every clause with `expr` has at least one `binds_to` function wearing the attribute | Clauses that exist but aren't attached anywhere |
| CB-1635 | L3 | Attribute's `binds_to` function is actually modified by the ticket (in git diff) | Drive-by contract additions on unrelated code |
| CB-1636 | L1 | Generated macros compile-check clean in both release and debug profiles | `debug_assert!` side-effect misuse in debug-only eval |
| CB-1637 | L3 | If ticket is `target_level >= L2`, every modified public function must have a bound attribute | Scope creep without contract coverage |
| CB-1638 | L3 | Generated modules under `contracts/work/` are tracked in git | Transient ungenerated state |
| CB-1639 | L4 | For L4+ tickets, `kani_harnesses[]` in the ticket's derived YAML reference the generated contract macros | Orphaned Kani harnesses |

CB-1633 uses the same SHA drift approach as Components 27/29 — a changed
contract.json without a regenerated `.rs` file is a hard error.

## CLI Surface

### `pmat work codegen <ID>`

Reads `.pmat-work/<ID>/contract.json`, resolves bound equations (Component 27),
writes `contracts/work/<ID>.rs`. Idempotent. Exit 0 if file unchanged.

### `pmat work codegen --all`

Regenerates all in-progress tickets. Useful post-`git pull` when upstream
changes merged a binding YAML update.

### `pmat work codegen --check`

Dry run. Exits nonzero if generation would change any file; used by
pre-commit (CB-1633).

## Build Integration

Add a workspace-level `build.rs` (or reuse the existing one if present):

```rust
// contracts/work/build.rs
fn main() {
    println!("cargo:rerun-if-changed=.pmat-work");
    let status = std::process::Command::new("pmat")
        .args(["work", "codegen", "--check"])
        .status()
        .expect("pmat must be installed for build");
    if !status.success() {
        panic!("work codegen out-of-date — run `pmat work codegen --all`");
    }
}
```

This couples the Rust build to ticket state; staged changes without
regeneration block compilation. For non-pmat contributors, `make check`
auto-invokes `pmat work codegen --all` during setup.

## Storage

```
contracts/
└── work/
    ├── PMAT-530.rs              # generated module
    ├── PMAT-530.manifest.json   # SHA snapshots (inputs → output)
    └── build.rs                 # check-mode invoker
```

`manifest.json`:

```json
{
  "ticket": "PMAT-530",
  "contract_sha": "3a2f...",
  "bindings": [{"yaml":"rope-kernel-v1.yaml","sha":"9b1c..."}],
  "generated_sha": "7de4...",
  "generated_at": "2026-04-17T12:00:00Z"
}
```

CB-1633 compares computed SHAs to manifest to detect drift.

## Attribute Macro Implementation

The `#[pmat_work_contract]` proc macro lives in a new crate
`pmat-work-macros/` to keep proc-macro build isolation. Crate is feature-
gated behind `pmat-work-contracts` on consumer crates, default-on in
workspace members that edit ticket-bound code.

Expansion is done at crate compile time; zero runtime cost in release
builds because `debug_assert!` compiles to nothing with
`-C debug-assertions=no`.

## Interaction with Other Specs

- [Component 22 provable-contracts.md](provable-contracts.md) / `pv codegen`
  is the prior art this spec extends. Shared macro-emission style.
- [Component 27 pmat-work-contract-binding.md](pmat-work-contract-binding.md)
  supplies `implements[]`; inherited clauses get codegen too.
- [Component 28 pmat-work-verification-ladder.md](pmat-work-verification-ladder.md)
  L2 gate: `cargo check` must succeed with attribute-wrapped functions.
- [Component 25 commit-level-contract-enforcement.md](commit-level-contract-enforcement.md)
  Phase 1 YAML generation feeds back: emitted YAML references the same
  macros the Rust generator emits.
- [Component 31 pmat-work-cot-proof-derivation.md](pmat-work-cot-proof-derivation.md)
  auto-populates `expr` fields from chain-of-thought assumptions.

## Migration

- Existing tickets with empty `expr` fields stay as L0 review clauses —
  nothing generated, nothing broken.
- `pmat work migrate --derive-expr` attempts to translate common prose
  patterns ("x must be positive", "returns non-empty") into `expr`
  fields; non-trivial ones are flagged for manual review.
- `contracts/work/` directory added to `.gitignore` during migration,
  then removed from ignore once manifests are committed.

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/work_codegen.rs` | NEW — ticket → Rust generator |
| `src/cli/handlers/work_contract_dbc.rs` | Add `expr`/`binds_to` fields |
| `crates/pmat-work-macros/src/lib.rs` | NEW — `#[pmat_work_contract]` proc macro |
| `src/cli/handlers/comply_handlers/check_handlers/check_codegen.rs` | NEW — CB-1630..1639 |
| `contracts/work/build.rs` | NEW — build-time check invoker |
| `src/services/pv_codegen_shared.rs` | NEW — shared generator internals between `pv codegen` and `pmat work codegen` |

## References

### Internal
- [provable-contracts.md](provable-contracts.md) §Code Generation
- [commit-level-contract-enforcement.md](commit-level-contract-enforcement.md) Phase 1 (inverse direction)
- [pmat-work-verification-ladder.md](pmat-work-verification-ladder.md) L2 consumer
- [agent-integration.md](agent-integration.md) provable-contract-first design

### Foundational
- Meyer, B. (1988). *Object-Oriented Software Construction*. §11 Design by Contract — executable assertions as first-class code.
- Findler, R.B. & Felleisen, M. (2002). *Contracts for higher-order functions*. Basis for proc-macro contract composition.

### arXiv
- Rocha et al. (2024). arXiv:2412.00891. Runtime assertion checking with near-zero release overhead — validates `debug_assert!` as the correct wire.
- Bruni et al. (2026). arXiv:2602.22302. Agent Behavioral Contracts — Component 10 pairs with compile-time agent-generated contracts to prevent drift.
