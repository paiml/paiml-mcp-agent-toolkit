# Provable Contracts Integration

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 22

## Root-Cause Analysis: Why Generated Assertions Don't Catch Bugs

Five Whys (2026-03-29):

1. **Why are generated assertions useless?** Every generated precondition
   is `!_contract_input.is_empty()`, regardless of what the YAML says.
2. **Why does codegen emit the same assertion for all equations?**
   `codegen.rs` line 60 hardcodes `!_contract_input.is_empty()` instead
   of looping over `equation.preconditions` — the same pattern that
   already works correctly for postconditions on lines 78-84.
3. **Why wasn't this caught?** CB-1210 checks YAML precondition diversity
   (which is fine — the YAML has real expressions), not generated output
   diversity. Nobody validated that codegen output matches YAML input.
4. **Why is there no codegen-vs-YAML validation?** The enforcement tooling
   (CB-1200..1210) measures contract *metadata* — existence, schema
   validity, binding coverage, verification levels — but never diffs
   generated Rust against source YAML.

**The bug (FIXED — provable-contracts commit 013397a):** `codegen.rs`
previously hardcoded `!_contract_input.is_empty()` for all preconditions.
Now fixed: lines 73-79 (multi-arg) and 90-101 (single-input) loop over
`equation.preconditions` and emit each YAML expression as a `debug_assert!`.

**What works:** YAML contracts contain domain-specific Rust expressions
(softmax: `x.iter().all(|v| v.is_finite())`; matmul: `a.len() == m * k`;
rmsnorm: `eps > 0.0`). Both precondition and postcondition codegen now
iterate YAML and emit real assertions.

## Enforcement Chain: CB-1200 through CB-1214

pmat enforces provable-contracts compliance via 15 checks spanning
the full verification ladder (L0-L5):

| Check | Level | What it enforces | Catches |
|-------|-------|-----------------|---------|
| CB-1200 | L0.5 | Contract existence + pv lint + binding coverage | Missing contracts |
| CB-1201 | L0.5 | pv lint pass/fail with error detail | Schema violations |
| CB-1202 | L1 | Critical keyword coverage (forward, backward, kernel, etc.) | Uncontracted critical fns |
| CB-1203 | L3 | `#[contract]` annotation coverage on bound functions | Missing compile-time assertions |
| CB-1204 | L1 | build.rs pipeline (superseded by traits at L2+) | Missing build enforcement |
| CB-1205 | L4 | Provability invariant (obligations → kani harnesses) | Missing Kani coverage |
| CB-1206 | L4/L5 | Verification level distribution per-project | Kani/Lean coverage gaps |
| CB-1207 | — | Contract drift (stale YAML vs source) | Outdated contracts |
| **CB-1208** | **L1-L3** | **Binding existence + enforcement level (L0-L3)** | **Ghost bindings, paper-only repos** |
| **CB-1209** | **L2** | **Contract trait enforcement (13 kernel traits)** | **Missing trait impls** |
| **CB-1210** | **L3** | **Generated assertion diversity** | **Codegen emitting identical placeholders** |
| **CB-1211** | **L3** | **Codegen fidelity — assertion count matches YAML** | **Codegen dropping preconditions** |
| **CB-1212** | **L3** | **Postcondition codegen — wrapper macro pattern** | **Unenforced postconditions** |
| **CB-1213** | **L3** | **Binding-level typed assertions** | **Generic contracts on typed functions** |
| **CB-1214** | **L3** | **Enforcement quality — call-site penetration × quality** | **Contracts exist but never invoked** |

### Enforcement Levels (detected by CB-1208)

| Level | Mechanism | Repos | What it proves |
|-------|-----------|-------|---------------|
| L3 | build.rs + traits | aprender, entrenar, forjar, realizar, ruchy, simular | Function exists + right signature + build enforced |
| L2 | traits only | bashrs | Function exists + right signature |
| L1 | build.rs only | trueno | YAML says "implemented" |
| L0 | paper-only | 18 repos | **Nothing — ghost bindings** |

L0 repos with binding.yaml but no enforcement **FAIL** CB-1208.

## CB-1210..1213: Contract Enforcement Quality (NEW)

### CB-1210: Precondition Quality (STRENGTHENED — WARN → FAIL)

**Previous behavior:** WARN when >90% of YAML preconditions are identical.
**Problem:** The YAML preconditions are already diverse and correct. CB-1210
was checking the wrong thing — it should check *generated output*, not YAML input.

**New behavior:** Check the **generated** `contract_pre_*` macros. FAIL when:

1. **Generated output diversity < 30%** — codegen emits the same `debug_assert!`
   for >70% of equations (symptom of the hardcoded placeholder bug)
2. **Generated output doesn't match YAML** — a generated assertion is a known
   placeholder (`!_contract_input.is_empty()`) but the source YAML has
   domain-specific expressions like `x.iter().all(|v| v.is_finite())`

**Note:** The YAML data is correct. Core kernels already have real Rust
expressions. The fix is in codegen, not in contract authoring.

### CB-1211: Codegen Fidelity (NEW)

**What it checks:** Generated `debug_assert!` count and content must match
the source YAML for each equation.

**Detection:** For each contract YAML, count preconditions. Then count
`debug_assert!` lines in the generated `contract_pre_*` macro. FAIL if:

1. YAML has N preconditions but generated macro has <N assertions
2. Generated macro contains `!_contract_input.is_empty()` but YAML doesn't

**Status: FIXED** (provable-contracts commit `013397a`). Codegen now
loops over `equation.preconditions` at lines 73-79 (multi-arg) and
90-101 (single-input), mirroring the postcondition pattern.

CB-1211 remains as a **regression detector**: if a future change
reintroduces hardcoded placeholders, the assertion-count-vs-YAML check
catches it.

**Severity:** FAIL.

### CB-1212: Combined Wrapper Macro (NEW)

**What it checks:** For equations with both preconditions and postconditions,
codegen should emit a combined `contract_*` wrapper in addition to the
separate `contract_pre_*` / `contract_post_*` macros.

**Why:** The separate `contract_post_*` macro works (lines 70-88 emit real
YAML postconditions), but callers must manually capture the return value
and call the post macro. A combined wrapper makes postcondition enforcement
automatic.

**Combined pattern (generated alongside existing separate macros):**

```rust
macro_rules! contract_softmax {
    ($input:expr, $body:expr) => {{
        contract_pre_softmax!($input);
        let _result = $body;
        contract_post_softmax!(_result);
        _result
    }};
}
```

**Detection:** WARN if a contract has both preconditions and postconditions
but codegen only emits separate macros without a combined wrapper.

**Severity:** WARN. The separate macros already work; the wrapper is ergonomic.

### CB-1213: Binding-Level Typed Assertions (NEW)

**What it checks:** Each entry in `binding.yaml` with `status: implemented`
should have typed preconditions/postconditions that reference the actual
function signature parameters, not the generic equation parameters.

**Problem:** A contract equation uses `x`, `m`, `k`, `n` — but the bound
function might use `logits`, `rows`, `cols`. The binding must translate.

**Schema extension for binding.yaml:**

```yaml
bindings:
  - contract: softmax-kernel-v1
    equation: softmax
    function: softmax
    module_path: trueno::blis::softmax::softmax
    signature: "fn(&[f32]) -> Vec<f32>"
    status: implemented
    # NEW: typed assertions per binding
    preconditions:
      - 'logits.iter().all(|v| v.is_finite())'
      - 'logits.len() <= 131072'  # max context window
    postconditions:
      - '(result.iter().sum::<f32>() - 1.0).abs() < 1e-5'
```

**Detection:** For each binding with `status: implemented`:
1. WARN if no `preconditions` field (inherits generic equation preconditions)
2. FAIL if binding `preconditions` contains known placeholders
3. INFO if binding adds domain constraints beyond the equation
   (e.g., `logits.len() <= 131072` — deployment-specific bound)

**Severity:** WARN. Binding-level assertions are an upgrade path, not blocking.

## Configurable Thresholds

Configure strictness in `.pmat.yaml`:

```yaml
comply:
  thresholds:
    pv_lint_is_error: true        # CB-1201: WARN → FAIL on lint failure
    min_binding_existence: 95     # CB-1208: % threshold for binding verification
    require_all_traits: true      # CB-1209: require 13/13 traits
    min_kani_coverage: 20         # CB-1206: minimum Kani proof %
    # NEW: codegen enforcement
    codegen_fidelity: true          # CB-1211: FAIL if generated assertions don't match YAML
    min_generated_diversity: 30     # CB-1210: % unique assertions in generated output (FAIL below)
    require_wrapper_macros: false   # CB-1212: WARN-only (set true to FAIL)
    require_binding_assertions: false  # CB-1213: WARN-only (set true to FAIL)
    min_enforcement_quality: 0.3      # CB-1214: FAIL if quality below threshold
```

### CB-1214: Enforcement Quality (NEW)

Runs `pv coverage --enforcement <src> --binding <binding.yaml>` and parses
the enforcement score. Classifies contract call sites:

| Level | Score | Meaning |
|-------|-------|---------|
| E0 | 0.1 | Generic `!is_empty()` assertion at call site |
| E1 | 0.5 | Domain-specific precondition check only |
| E2 | 1.0 | Both precondition and postcondition checks |

**Quality** = weighted average of E levels across call sites.
**Enforcement** = penetration (call sites / bindings) × quality.

FAIL if quality < 0.3 (more than 70% of call sites are E0 generic).
WARN if 0 call sites found (contracts exist but are never invoked).

## Infra-Score PV Bonus (PV-01..PV-05)

`pmat infra-score` awards up to 12 bonus points for provable-contracts:

| Check | Points | What it checks |
|-------|--------|---------------|
| PV-04 | 2 | contracts/ directory exists with schema-valid YAML |
| PV-01 | 3 | `pv lint` passes (falls back to YAML structure check) |
| PV-02 | 3 | `pv score >= 0.5` (FAILs without pv CLI) |
| PV-03 | 2 | At least one contract at proof level L2+ |
| PV-05 | 2 | Enforcement quality — `pv coverage --enforcement` finds call sites |

## Finding Missing Contracts

```bash
# Full audit
pmat comply check

# Which bound functions don't exist in source?
pmat comply check 2>&1 | grep 'CB-1208'

# Are contract traits implemented?
pmat comply check 2>&1 | grep 'CB-1209'

# Are preconditions real or mass-generated?
pmat comply check 2>&1 | grep 'CB-1210'

# Which critical functions lack contracts?
pmat comply check 2>&1 | grep 'CB-1202'
```

## Key Files

| File | Purpose |
|------|---------|
| `check_pv_enforcement.rs` | CB-1201..1213 implementation |
| `check_provable_contracts.rs` | CB-1200 detection + pv lint/score |
| `infra_score_handlers.rs` | `pmat infra-score` CLI handler |
| `provable_contracts.rs` (infra_score) | PV-01..PV-04 bonus scorer |
| `comply_config_types.rs` | PV threshold configuration |
| `codegen.rs` (provable-contracts) | **Root cause** — precondition codegen ignores YAML |

## Remediation Roadmap

### Fix 1: Precondition codegen loop (CB-1211) — DONE

Fixed in provable-contracts commit `013397a`. Codegen now loops over
`equation.preconditions` and emits each YAML expression.

### Fix 2: Combined wrapper macro (CB-1212) — ~20 lines

Add a third macro `contract_<eq>!` that composes the existing `contract_pre_*`
and `contract_post_*` macros around a body expression. Ergonomic, not blocking.

**Scope:** `codegen.rs` `generate_from_contract()`.

### Fix 3: Binding-level assertions (CB-1213) — schema + codegen

Extend `binding.yaml` schema with optional per-binding preconditions using
actual function parameter names. Codegen reads these when present, falls
back to equation-level assertions otherwise.

**Scope:** `schema.rs` (Binding struct), `codegen.rs` (binding-aware generation).

### Fix priority

| Fix | Check | Impact | Effort | Status |
|-----|-------|--------|--------|--------|
| 1 | CB-1211 | 516 real assertions | ~10 lines | **DONE** (013397a) |
| 2 | CB-1212 | Ergonomic postconditions | ~20 lines | **TODO** |
| 3 | CB-1213 | Per-binding type safety | Schema + codegen | **TODO** |

## References

### Provable-Contracts Spec
- pv-spec.md §2: Verification Ladder (L0-L5)
- pv-spec.md §23: Contract-Trait Enforcement
- pv-spec.md §27: The One Way (unified enforcement)

### arXiv
- Dardik & Kang (2025). arXiv:2509.06250. Assume-guarantee contracts.
- Li et al. (2025). arXiv:2510.12047. LLMs and formal contracts.
- Le Blanc & Lam (2024). arXiv:2410.01981. Rust verification landscape.

### Foundational
- Meyer (1988). Design by Contract. Preconditions, postconditions, invariants.
- `core::contracts` RFC: rust-lang/rust#128045
